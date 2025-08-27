use std::{
    error::Error,
    fmt::{Debug, Display}
};

use bytes::Bytes;
use prost::{DecodeError, Message};
use zeromq::{Endpoint, RepSocket, Socket, SocketRecv, SocketSend, ZmqError};

/// Represents the communication socket with the backend process.
/// 
/// Stores the actual ZeroMQ Socket for concurrency reasons.
pub struct BackendSocket {
    socket: RepSocket,
    pub endpoint: Endpoint
}

impl BackendSocket {
    pub async fn create(endpoint: &str) -> SocketResult<Self> {
        let mut socket = RepSocket::new();

        let endpoint = match socket.bind(endpoint).await {
            Ok(endpoint) => endpoint,
            Err(error) => {
                return Err(SocketError::ZmqBind(error));
            }
        };

        Ok(Self {socket, endpoint})
    }

    /// Sends the contents of a message through the ZeroMQ socket to
    /// the backend.
    /// 
    /// A call to BackendSocket::recv() must have been made successfully
    /// previous to a call to BackendSocket::send(). If not, send() will fail.
    /// 
    /// A call to send() will return when the message has been put on the
    /// ZeroMQ message queue, NOT when the message has actually been received
    /// by the backend. As such, there is no absolute guarantee that the
    /// message has been received when this returns. 
    pub async fn send<S: Message + Debug>(&mut self, msg: &S) -> SocketResult<()> {
        let bytes_send: Bytes = msg.encode_to_vec().into();
        match  self.socket.send(bytes_send.into()).await {
            Ok(_) => Ok(()),
            Err(error) => {
                Err(SocketError::ZmqSend(format!("{:?}", msg), error))
            }
        }
    }

    /// Receives a message from the backend through the ZeroMQ socket.
    /// 
    /// Communication with the backend must be initiated by a call to
    /// this function.
    /// Any subsequent calls to this function must be preceded by a call to
    /// BackendSocket::send(). Otherwise the recv() call will fail.
    /// 
    /// A call to recv() will await until a message is received through the
    /// ZeroMQ socket.
    pub async fn recv<R: Message + Default>(&mut self) -> SocketResult<R> {
        match self.socket.recv().await {
            Ok(buf) => {
                let buf: Bytes = match buf.get(0) {
                    Some(bytes) => bytes,
                    None => {
                        return Err(SocketError::EmptyBuffer)
                    }
                }.to_owned();
                match R::decode(buf.as_ref()) {
                    Ok(msg) => Ok(msg),
                    Err(error) => {
                        Err(SocketError::Decode(error))
                    },
                }
            },
            Err(error) => {
                Err(SocketError::ZmqReceive(error))
            }
        }
    }

    /// Shorthand for a call to BackendSocket::send() followed by a call to
    /// BackendSocket::recv().
    pub async fn send_and_recv<S: Message + Debug, R: Message + Default>(
        &mut self,
        msg: &S,
    ) -> SocketResult<R> {
        self.send(msg).await?;
        self.recv().await
    }
}

type SocketResult<T> = Result<T, SocketError>;

#[derive(Debug)]
pub enum SocketError {
    ZmqBind(ZmqError),
    ZmqSend(String, ZmqError),
    ZmqReceive(ZmqError),
    EmptyBuffer,
    Decode(DecodeError)
}

impl Display for SocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZmqBind(error) => write!(
                f, "couldn't bind to backend socket; {}", error
            ),
            Self::ZmqSend(message, error) => write!(
                f, "failed to send message {}; {}", message, error
            ),
            Self::ZmqReceive(error) => write!(
                f, "receiving on socket failed; {}", error
            ),
            Self::EmptyBuffer => write!(
                f, "no bytes in received message"
            ),
            Self::Decode(error) => write!(
                f, "couldn't decode received message; {}", error
            )
        }
    }
}

impl Error for SocketError {}