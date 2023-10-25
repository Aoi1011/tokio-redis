//! Minimal Redis client implementaion
//!
//! Provides an async connect and methods for issuing the supported commands.

use std::io::{Error, ErrorKind};

use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::debug;

use crate::{
    cmd::{Get, Ping, Set},
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

    pub async fn set(&mut self, key: &str, value: Bytes) -> crate::Result<()> {
        self.set_cmd(Set::new(key, value, None)).await
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

    async fn read_response(&mut self) -> crate::Result<Frame> {
        let res = self.connection.read_frame().await?;

        debug!(?res);

        match res {
            Some(Frame::Error(msg)) => Err(msg.into()),
            Some(frame) => Ok(frame),
            None => {
                let err = Error::new(ErrorKind::ConnectionReset, "connection reset by server");

                Err(err.into())
            }
        }
    }
}
