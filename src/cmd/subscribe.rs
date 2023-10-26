use std::pin::Pin;

use bytes::Bytes;
use tokio_stream::{Stream, StreamMap};

use crate::{db::Db, parse::Parse, shutdown::Shutdown, Connection};

/// Subscribes the client to one or more channels.
///
/// Once the client enters the subscribed state, it is not supposed to issue any
/// other commands, except for additional SUBSCRIBE, PSUBSCRIBE, UNSUBSCRIBE, PUNSUBSCRIBE, PING
/// and QUIT commands.
#[derive(Debug)]
pub struct Subscribe {
    channels: Vec<String>,
}

/// Unsubscribe the client from one or more channels.
///
/// When no channesl are specified, the client is unsubscribed from all the previously subscribed
/// channels.
#[derive(Debug)]
pub struct Unsubscribe {
    channels: Vec<String>,
}

/// Stream of messages. The stream receives messages from the `broadcast::Receiver`. We use
/// `stream!` to create a `Stream` that consumes messages. Because `stream!` values cannot be
/// named, we box the stream using a trait object.
type Message = Pin<Box<dyn Stream<Item = Bytes> + Send>>;

impl Subscribe {
    /// Creates a new `Subscribe` command to listen on the specified channels.
    pub(crate) fn new(channels: Vec<String>) -> Subscribe {
        Subscribe { channels }
    }

    /// Parse a `Subscribe` instance from a received frame.
    ///
    /// The `Parse` arguement provides a cursor-like API to read fields from the
    /// `Frame`. At this point, the entire frame has already been received from the socket.
    ///
    /// The `SUBSCRIBE` string has already been consumed.
    ///
    /// # Returns
    ///
    /// On success, the `Subscribe` value is returned. If the frame is malformed, `Err` is
    /// returned.
    ///
    /// # Format
    ///
    /// Expects an array frame containing two or more entriees.
    ///
    /// ```text
    ///
    /// SUBSCRIBE channel [channel ...]
    ///
    /// ```
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Subscribe> {
        use crate::ParseError::EndOfStream;

        // The `SUBSCRIBE` string has already been consumed. At this point, there is one or more
        // strings remaining in `parse`. These represent the channels to subscribed to.
        //
        // Extract the first string. If there is none, the frame is malformed and the error is
        // bubbled up.
        let mut channels = vec![parse.next_string()?];

        // Now, the remainder of the frame is consumed. Each value must be a string or the frame is
        // malformed. Once all values in the frame have been consumed, the comamnd is fully parsed.
        loop {
            match parse.next_string() {
                // A string has been consumed from the `parse`, push it into the lis of channels to
                // subscribe to.
                Ok(s) => channels.push(s),
                // The `EndOfStream` error indicates theres is no further data to parse.
                Err(EndOfStream) => break,
                // All other errors are bubbled up, resulting in the connection being terminated.
                Err(err) => return Err(err.into()),
            }
        }

        Ok(Subscribe { channels })
    }

    pub(crate) async fn apply(
        mut self,
        db: &Db,
        dst: &mut Connection,
        shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        let mut subscriptions = StreamMap::new();

        loop {
            for channel_name in self.channels.drain(..) {}
        }
        Ok(())
    }
}

async fn subscribe_to_channel(
    channel_name: String,
    subscriptions: &mut StreamMap<String, Message>,
    db: &Db,
    dst: &mut Connection,
) -> crate::Result<()> {
    let mut rx = db.subscribe(channel_name.clone());

    let rx = Box::pin(async_stream);
    Ok(())
}
