use crate::protocol::ConnectDatagram;
use async_std::net::SocketAddr;
use async_std::pin::Pin;
use futures::io::IoSlice;
use futures::task::{Context, Poll};
use futures::{AsyncWrite, Sink};
use log::*;
use std::error::Error;

pub use futures::SinkExt;
pub use futures::StreamExt;
use std::fmt::Debug;

/// Encountered when there is an issue with writing messages on the network stream.
///
#[derive(Debug)]
pub enum ConnectionWriteError {
    /// Encountered when trying to send a message while the connection is closed.
    ConnectionClosed,

    /// Encountered when there is an IO-level error with the connection.
    IoError(std::io::Error),
}

impl Error for ConnectionWriteError {}

impl std::fmt::Display for ConnectionWriteError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConnectionWriteError::ConnectionClosed => formatter.write_str("cannot send message when connection is closed"),
            ConnectionWriteError::IoError(err) => std::fmt::Display::fmt(&err, formatter),
        }
    }
}

/// An interface to write messages to the network connection.
///
/// Implements the [`Sink`] trait to asynchronously write messages to the network connection.
///
/// # Example
///
/// Basic usage:
///
/// ```ignore
/// writer.send(msg).await?;
/// ```
///
/// Please see the [tcp-client](https://github.com/sachanganesh/connect-rs/blob/main/examples/tcp-client/)
/// example program or other client example programs for a more thorough showcase.
///
pub struct ConnectionWriter {
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    write_stream: Pin<Box<dyn AsyncWrite + Send + Sync>>,
    pending_writes: Vec<Vec<u8>>,
    closed: bool,
}

impl ConnectionWriter {
    /// Creates a new [`ConnectionWriter`] from an [`AsyncWrite`] trait object and the local and peer
    /// socket metadata.
    pub fn new(
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        write_stream: Pin<Box<dyn AsyncWrite + Send + Sync>>,
    ) -> Self {
        Self {
            local_addr,
            peer_addr,
            write_stream,
            pending_writes: Vec::new(),
            closed: false,
        }
    }

    /// Get the local IP address and port.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr.clone()
    }

    /// Get the peer IP address and port.
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr.clone()
    }

    /// Check if the [`Sink`] of messages to the network is closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub(crate) fn write_pending_bytes(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), ConnectionWriteError>> {
        if self.pending_writes.len() > 0 {
            let stream = self.write_stream.as_mut();

            match stream.poll_flush(cx) {
                Poll::Pending => Poll::Pending,

                Poll::Ready(Ok(_)) => {
                    trace!("Sending pending bytes");

                    let pending = self.pending_writes.split_off(0);
                    let writeable_vec: Vec<IoSlice> =
                        pending.iter().map(|p| IoSlice::new(p)).collect();

                    let stream = self.write_stream.as_mut();
                    match stream.poll_write_vectored(cx, writeable_vec.as_slice()) {
                        Poll::Pending => Poll::Pending,

                        Poll::Ready(Ok(bytes_written)) => {
                            trace!("Wrote {} bytes to network stream", bytes_written);
                            Poll::Ready(Ok(()))
                        }

                        Poll::Ready(Err(err)) => {
                            error!("Encountered error when writing to network stream");
                            Poll::Ready(Err(ConnectionWriteError::IoError(err)))
                        }
                    }
                }

                Poll::Ready(Err(err)) => {
                    error!("Encountered error when flushing network stream");
                    Poll::Ready(Err(ConnectionWriteError::IoError(err)))
                }
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

impl Sink<ConnectDatagram> for ConnectionWriter {
    type Error = ConnectionWriteError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.is_closed() {
            trace!("Connection is closed, cannot send message");
            Poll::Ready(Err(ConnectionWriteError::ConnectionClosed))
        } else {
            trace!("Connection ready to send message");
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: ConnectDatagram) -> Result<(), Self::Error> {
        trace!("Preparing message to be sent next");

        let buffer = item.encode();
        let msg_size = buffer.len();
        trace!("Serialized pending message into {} bytes", msg_size);

        self.pending_writes.push(buffer);

        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.write_pending_bytes(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.closed = true;

        match self.write_pending_bytes(cx) {
            Poll::Pending => Poll::Pending,

            Poll::Ready(Ok(_)) => {
                let stream = self.write_stream.as_mut();

                match stream.poll_close(cx) {
                    Poll::Pending => Poll::Pending,

                    Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),

                    Poll::Ready(Err(err)) => Poll::Ready(Err(ConnectionWriteError::IoError(err))),
                }
            }

            err => err,
        }
    }
}
