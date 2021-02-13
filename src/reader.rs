use crate::protocol::ConnectDatagram;
use async_std::net::SocketAddr;
use async_std::pin::Pin;
use bytes::{Buf, BytesMut};
use futures::task::{Context, Poll};
use futures::{AsyncRead, Stream};
use log::*;
use std::io::Cursor;

pub use futures::SinkExt;
pub use futures::StreamExt;

/// A default buffer size to read in bytes and then deserialize as messages.
const BUFFER_SIZE: usize = 8192;

/// An interface to read messages from the network connection.
///
/// Implements the [`Stream`] trait to asynchronously read messages from the network connection.
///
/// # Example
///
/// Basic usage:
///
/// ```ignore
/// while let Some(msg) = reader.next().await {
///   // handle the received message
/// }
/// ```
///
/// Please see the [tcp-client](https://github.com/sachanganesh/connect-rs/blob/main/examples/tcp-client/)
/// example program or other client example programs for a more thorough showcase.
///

pub struct ConnectionReader {
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    read_stream: Pin<Box<dyn AsyncRead + Send + Sync>>,
    pending_read: Option<BytesMut>,
    closed: bool,
}

impl ConnectionReader {
    /// Creates a new [`ConnectionReader`] from an [`AsyncRead`] trait object and the local and peer
    /// socket metadata.
    pub fn new(
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        read_stream: Pin<Box<dyn AsyncRead + Send + Sync>>,
    ) -> Self {
        Self {
            local_addr,
            peer_addr,
            read_stream,
            pending_read: None,
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

    /// Check if the [`Stream`] of messages from the network is closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub(crate) fn close_stream(&mut self) {
        trace!("Closing the stream for connection with {}", self.peer_addr);
        self.pending_read.take();
        self.closed = true;
    }
}

impl Stream for ConnectionReader {
    type Item = ConnectDatagram;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = BytesMut::new();
        buffer.resize(BUFFER_SIZE, 0);

        trace!("Starting new read loop for {}", self.local_addr);
        loop {
            trace!("Reading from the stream");
            let stream = self.read_stream.as_mut();

            match stream.poll_read(cx, &mut buffer) {
                Poll::Pending => return Poll::Pending,

                Poll::Ready(Ok(mut bytes_read)) => {
                    if bytes_read > 0 {
                        trace!("Read {} bytes from the network stream", bytes_read)
                    } else if self.pending_read.is_none() {
                        self.close_stream();
                        return Poll::Ready(None);
                    }

                    if let Some(mut pending_buf) = self.pending_read.take() {
                        trace!("Prepending broken data ({} bytes) encountered from earlier read of network stream", pending_buf.len());
                        bytes_read += pending_buf.len();

                        pending_buf.unsplit(buffer);
                        buffer = pending_buf;
                    }

                    while bytes_read > 0 {
                        trace!("{} bytes from network stream still unprocessed", bytes_read);

                        buffer.resize(bytes_read, 0);

                        let mut cursor = Cursor::new(buffer.as_mut());
                        match ConnectDatagram::decode(&mut cursor) {
                            Ok(data) => {
                                return match data.version() {
                                    _ => {
                                        let serialized_size = data.size();
                                        trace!(
                                            "Deserialized message of size {} bytes",
                                            serialized_size
                                        );

                                        buffer.advance(serialized_size);
                                        bytes_read -= serialized_size;
                                        trace!("{} bytes still unprocessed", bytes_read);

                                        trace!("Sending deserialized message downstream");
                                        Poll::Ready(Some(data))
                                    }
                                }
                            }

                            Err(err) => {
                                warn!(
                                    "Could not deserialize data from the received bytes: {:#?}",
                                    err
                                );

                                self.pending_read = Some(buffer);
                                buffer = BytesMut::new();
                                break;
                            }
                        }
                    }

                    buffer.resize(BUFFER_SIZE, 0);
                }

                // Close the stream
                Poll::Ready(Err(_e)) => {
                    self.close_stream();
                    return Poll::Ready(None);
                }
            }
        }
    }
}
