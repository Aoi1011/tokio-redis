//! Minimal Redis client implementaion
//!
//! Provides an async connect and methods for issuing the supported commands.

use std::{
    io::{Error, ErrorKind},
    time::Duration,
};

use async_stream::try_stream;
use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_stream::Stream;
use tracing::{debug, instrument};

use crate::{
    cmd::{Get, Ping, Publish, Set, Subscribe, Unsubscribe},
    Connection, Frame,
};

/// Established connection with a Redis server.
///
/// Backed by a single `TcpStream`, `Client` provides basic network client
/// functionally (no pooling, retrying, ...). Connections are established using
/// the [`connect`](fn@connect) function.
///
/// Requests are issued using the various methods of `CLient`.
pub struct Client {
    /// The TCP connection decorated with the redis protocol encoder / decoder
    /// implemented using a bufered `TcpStream`.
    ///
    /// When `Listener` receives an inbound connection, the `TcpStream` is
    /// passed to `Connection::new`, which initializes the associated buffers.
    /// `Connection` allows the handler to operate at the "frame" level and keep
    /// the byte level protocol parsing details encapsulated in `Connection`.
    connection: Connection,
}

/// A client that has entered pub/sub mode.
///
/// Once clients subscribe to a channel, they mey only perform pub/sub related
/// commands. The `Client` type is transitioned to a `Subscriber` type in order
/// to prevent non-pub/sub methods from being called.
pub struct Subscriber {
    /// The subscribed client.
    client: Client,

    /// The set of channels to which the `Subscriber` is currently subscribed.
    subscribed_channels: Vec<String>,
}

/// A message received on a subscribed channel.
#[derive(Debug, Clone)]
pub struct Message {
    pub channel: String,
    pub content: Bytes,
}

impl Client {
    /// Establish a connection with the Redis server located at `addr`.
    ///
    /// `addr` may be any type that can be asynchronously converted to a
    /// `SocketAddr`. This includes `SocketAddr` and strings. The `ToSocketAddrs`
    /// trait is the Tokio version and not the `std` version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_redis::clients::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = match Client::connect("localhost:6379").await {
    ///         Ok(client) => client,
    ///         Err(_) => panic!("failed to establish connection"),
    ///     };
    ///
    ///     // drop(client);
    /// }
    /// ```
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
        // The `addr` argument is passed directly to `TcpStream::connect`. This
        // performs any asynchrnous DNS lookup and attemps to establish the TCP
        // connection. An error at either step returns an error, which is then
        // bubbled up to the caller of `mini_redis` connect.
        let socket = TcpStream::connect(addr).await?;

        // Initialize the connection state. This allocates read/write buffers to
        // perform redis protocol frame parsing.
        let connection = Connection::new(socket);

        Ok(Client { connection })
    }

    /// Ping to the server.
    ///
    /// Returns PONG if no argument is provided, otherwise
    /// return a copy of the argument as a bulk.
    ///
    /// This command is often used to test if a connection
    /// is still alive, or to measure latency.
    ///
    /// # Examples
    ///
    /// Demonstrates basic usage.
    /// ```no_run
    /// use mini_redis::clients::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("127.0.0.1:6379").await.unwrap();
    ///
    ///     let pong = client.ping(None).await.unwrap();
    ///
    ///     assert_eq!(b"PONG", &pong[..]);
    /// }
    /// ```
    pub async fn ping(&mut self, msg: Option<Bytes>) -> crate::Result<Bytes> {
        let frame = Ping::new(msg).into_frame();

        debug!(request = ?frame);

        self.connection.write_frame(&frame).await?;

        match self.read_response().await? {
            Frame::Simple(val) => Ok(val.into()),
            Frame::Bulk(val) => Ok(val),
            frame => Err(frame.to_error()),
        }
    }

    /// Get the value of key.
    ///
    /// If the key does not exist the special value `None` is returned.
    ///
    /// # Examples
    ///
    /// Demonstrates basic usage.
    ///
    /// ```no_run
    /// use mini_redis::clients::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("localhost:6379").await.unwrap();
    ///
    ///     let val = client.get("foo").await.unwrap();
    ///     println!("Got = {:?}", val);
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn get(&mut self, key: &str) -> crate::Result<Option<Bytes>> {
        let frame = Get::new(key).into_frame();

        debug!(request = ?frame);

        self.connection.write_frame(&frame).await?;

        match self.read_response().await? {
            Frame::Simple(val) => Ok(Some(val.into())),
            Frame::Bulk(val) => Ok(Some(val)),
            Frame::Null => Ok(None),
            frame => Err(frame.to_error()),
        }
    }

    /// Set `key` to hold the given `value`.
    ///
    /// The `value` is associated with `key` until it is overwritten by the next
    /// call to `set` or it is removed.
    ///
    /// If key already holds a value, it is overwritten. Any previous time to
    /// live associated with the key is discarded on successful SET operation.
    ///
    /// # Examples
    ///
    /// Demonstrates basic usage.
    ///
    /// ```no_run
    /// use mini_redis::clients::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::connect("localhost:6379").await.unwrap();
    ///
    ///     client.set("foo", "bar".into()).await.unwrap();
    ///
    ///     // Getting the value immediately works
    ///     let val = client.get("foo").await.unwrap().unwrap();
    ///     assert_eq!(val, "bar");
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn set(&mut self, key: &str, value: Bytes) -> crate::Result<()> {
        self.set_cmd(Set::new(key, value, None)).await
    }

    /// Set `key` to hold the given `value`. The value expires after `expiration`
    ///
    /// The `value` is associated with `key` until one of the following:
    /// - it expires.
    /// - it is overwritten by the next call to `set`.
    /// - it is removed.
    ///
    /// If key already holds a value, it is overwritten. Any previous time to
    /// live associated with the key is discarded on a successful SET operation.
    ///
    /// # Examples
    ///
    /// Demonstrates basic usage. This example is not **guaranteed** to always
    /// work as it relies on time based logic and assumes the client and server
    /// stay relatively synchronized in time. The real world tends to not be so
    /// favorable.
    ///
    /// ```no_run
    /// use mini_redis::clients::Client;
    /// use tokio::time;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let ttl = Duration::from_millis(500);
    ///     let mut client = Client::connect("localhost:6379").await.unwrap();
    ///
    ///     client.set_expires("foo", "bar".into(), ttl).await.unwrap();
    ///
    ///     // Getting the value immediately works
    ///     let val = client.get("foo").await.unwrap().unwrap();
    ///     assert_eq!(val, "bar");
    ///
    ///     // Wait for the TTL to expire
    ///     time::sleep(ttl).await;
    ///
    ///     let val = client.get("foo").await.unwrap();
    ///     assert!(val.is_some());
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn set_expires(
        &mut self,
        key: &str,
        value: Bytes,
        expiration: Duration,
    ) -> crate::Result<()> {
        self.set_cmd(Set::new(key, value, Some(expiration))).await
    }

    /// The core `SET` logic, used by both `set` and `set_expires.
    async fn set_cmd(&mut self, cmd: Set) -> crate::Result<()> {
        // Convert the `Set` command into a frame
        let frame = cmd.into_frame();

        debug!(request = ?frame);

        // Write the frame to the socket. This writes the full frame to the
        // socket, waiting if necessary.
        self.connection.write_frame(&frame).await?;

        // Wait for the response from the server. On success, the server
        // responds simply with `OK`. Any other response indicates an error.
        match self.read_response().await? {
            Frame::Simple(res) if res == "OK" => Ok(()),
            frame => Err(frame.to_error()),
        }
    }

    /// Posts `message` to the given `channel`.
    ///
    /// Returns the number of subscribers currently listening on the channel.
    /// There is no gurantee that these subscribers receive the message as they
    /// may diconnect at any time.
    ///
    /// # Examples
    ///
    /// Demonstrate basic usage.
    ///
    /// ```no_run
    /// use mini_redis::clients::Client;
    ///
    /// #[tokio::main]
    /// pub async fn main() {
    ///     let mut client = Client::connect("localhost:6379").await.unwrap();
    ///
    ///     let val = client.publish("foo", "bar".into()).await.unwrap();
    ///     println!("Got = {:?}", val);
    /// }
    /// ```
    #[instrument(skip(self))]
    pub async fn publish(&mut self, channel: &str, message: Bytes) -> crate::Result<u64> {
        // Convert the `Publish` command into a frame
        let frame = Publish::new(channel, message).into_frame();

        debug!(request = ?frame);

        // Write the frame to the socket
        self.connection.write_frame(&frame).await?;

        // Read the response
        match self.read_response().await? {
            Frame::Integer(res) => Ok(res),
            frame => Err(frame.to_error()),
        }
    }

    /// Subscribes the ciient to the specified channels.
    ///
    /// Once a client issues a subscribe command, it may no longer issue any
    /// non-pub/sub commands.  The function consumes `self` and returns a `Subscriber`.
    ///
    /// The `Subscriber` value is used to receive messages as well as manage the
    /// list of channels the client is subscribed to.
    pub async fn subscribe(mut self, channels: Vec<String>) -> crate::Result<Subscriber> {
        // Issue the subscribe command to the server and wait for confirmation.
        // The client will then have been transitioned into the "subscriber"
        // state and may only issue pub/sub comands from that point on.
        self.subscribe_cmd(&channels).await?;

        // Return the `Subscriber` type
        Ok(Subscriber {
            client: self,
            subscribed_channels: channels,
        })
    }

    /// The core `SUBSCRIBE` logic, used by misc subscribe fns
    async fn subscribe_cmd(&mut self, channels: &[String]) -> crate::Result<()> {
        // Convert the `Subscribe` command into a frame
        let frame = Subscribe::new(channels.to_vec()).into_frame();

        debug!(request = ?frame);

        // Write the frame to the socket
        self.connection.write_frame(&frame).await?;

        // For each channel being subscribed to, the server responds with a
        // message confirming subscription to that channel.
        for channel in channels {
            // Read the response
            let response = self.read_response().await?;

            // Verify it is confirmation of subscription
            match response {
                Frame::Array(ref frame) => match frame.as_slice() {
                    // The server responds with an array frame in the form of:
                    //
                    // ```
                    // [ "subscribe", channel, num-subscribed ]
                    // ```
                    //
                    // where channel is the name of the channel and
                    // num-subscribed is the number of channels that the client
                    // is currently subscribed to.
                    [subscribe, schannel, ..]
                        if *subscribe == "subscribe" && *schannel == channel => {}
                    _ => return Err(response.to_error()),
                },
                frame => return Err(frame.to_error()),
            }
        }

        Ok(())
    }

    /// Read a response frame from the socket.
    ///
    /// If an `Err` frame is received, it is converted to `Err`.
    async fn read_response(&mut self) -> crate::Result<Frame> {
        let res = self.connection.read_frame().await?;

        debug!(?res);

        match res {
            Some(Frame::Error(msg)) => Err(msg.into()),
            Some(frame) => Ok(frame),
            None => {
                // Receving `None` here indicates the server has closed the
                // connection without sending a frame. This is unexpected and is represented as a
                // "connection reset by peer" error.
                let err = Error::new(ErrorKind::ConnectionReset, "connection reset by server");

                Err(err.into())
            }
        }
    }
}

impl Subscriber {
    /// Returns the set of channels currently subscribed to.
    pub fn get_subscribed(&self) -> &[String] {
        &self.subscribed_channels
    }

    /// Receive the next message published on a subscribed channel, waiting if necessary.
    ///
    /// `None` indicates the subscription has been terminated.
    pub async fn next_message(&mut self) -> crate::Result<Option<Message>> {
        match self.client.connection.read_frame().await? {
            Some(mframe) => {
                debug!(?mframe);

                match mframe {
                    Frame::Array(ref frame) => match frame.as_slice() {
                        [message, channel, content] if *message == "message" => Ok(Some(Message {
                            channel: channel.to_string(),
                            content: Bytes::from(content.to_string()),
                        })),
                        _ => Err(mframe.to_error()),
                    },
                    _ => Err(mframe.to_error()),
                }
            }
            None => Ok(None),
        }
    }

    /// Convert the subscriber into a `Stream` yielding new messages published on subscribed
    /// channels.
    ///
    /// `Subscriber` does not implement stream itself as doing so with safe code is non trivial.
    /// The usage of async/await would require a manual Stream implementation to use `unsafe` code.
    /// Instead, a conversion function is provided and the returned stream is implemented with the
    /// help of the `async-stream` crate.
    pub fn into_stream(mut self) -> impl Stream<Item = crate::Result<Message>> {
        // Uses the `try_stream` macro from the `async-stream` crate. Generators are not stable in
        // Rust. The crate uses a macro to simulate generators on top of async/await. There are
        // limitations, so read the documentation there.
        try_stream! {
            while let Some(message) = self.next_message().await? {
                yield message;
            }
        }
    }

    /// Subscribe to a list of new channels
    #[instrument(skip(self))]
    pub async fn subscribe(&mut self, channels: &[String]) -> crate::Result<()> {
        // Issue the subscribe comamnd
        self.client.subscribe_cmd(channels).await?;

        // Update the set of subscribed channels.
        self.subscribed_channels
            .extend(channels.iter().map(Clone::clone));

        Ok(())
    }

    /// Unsubscribe to a list of new channels
    pub async fn unsubscribe(&mut self, channels: &[String]) -> crate::Result<()> {
        let frame = Unsubscribe::new(channels).into_frame();

        debug!(request = ?frame);

        // Write the frame to the socket
        self.client.connection.write_frame(&frame).await?;

        // if the input channel list is empty, server acknowledge as unsubscribing from all
        // subscribed channels, so we assert that the unsubscribe list received matches the client
        // subscribed one
        let num = if channels.is_empty() {
            self.subscribed_channels.len()
        } else {
            channels.len()
        };

        // Read the response
        for _ in 0..num {
            let res = self.client.read_response().await?;

            match res {
                Frame::Array(ref frame) => match frame.as_slice() {
                    [unsubscribe, channel, ..] if *unsubscribe == "unsubscribe" => {
                        let len = self.subscribed_channels.len();

                        if len == 0 {
                            // There must be at least one channel
                            return Err(res.to_error());
                        }

                        // unsubscribed channel should exist in the subscribed list at this point
                        self.subscribed_channels.retain(|c| *channel != &c[..]);

                        // Only a single channel should be removed from the list of subscribed
                        // channels.
                        if self.subscribed_channels.len() != len - 1 {
                            return Err(res.to_error());
                        }
                    }
                    _ => return Err(res.to_error()),
                },
                frame => return Err(frame.to_error()),
            }
        }

        Ok(())
    }
}
