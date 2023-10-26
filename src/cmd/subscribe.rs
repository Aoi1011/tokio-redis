use std::pin::Pin;

use bytes::Bytes;
use tokio_stream::Stream;

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

type Message = Pin<Box<dyn Stream<Item = Bytes> + Send>>;
