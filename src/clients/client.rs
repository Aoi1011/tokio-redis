use std::io::{Error, ErrorKind};

use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::debug;

use crate::{cmd::Get, Connection, Frame};

pub struct Client {
    connection: Connection,
}

pub struct Subscriber {
    client: Client,
    subscribed_channels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub channel: String,
    pub content: Bytes,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
        let socket = TcpStream::connect(addr).await?;

        let connection = Connection::new(socket);

        Ok(Client { connection })
    }

    pub async fn ping(&mut self, msg: Option<Bytes>) -> crate::Result<Bytes> {
        // let frame =
        //
        Ok(Bytes::new())
    }

    pub async fn get(&mut self, key: &str) -> crate::Result<Option<Bytes>> {
        let frame = Get::new(key).into_frame();

        debug!(request = ?frame);

        self.connection.write_frame(&frame).await?;

        match self.read_response().await? {
            Frame::Simple(val) => Ok(Some(val.into())),
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
