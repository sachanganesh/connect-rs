use crate::SIZE_PREFIX_BYTE_SIZE;
use crate::{protocol::ConnectDatagram, DATAGRAM_HEADER_BYTE_SIZE};
use async_std::net::SocketAddr;
use async_std::pin::Pin;
use bytes::BytesMut;
use futures::task::{Context, Poll};
use futures::{AsyncRead, Stream};
use log::*;
use std::convert::TryInto;

pub use futures::{SinkExt, StreamExt};

/// A default buffer size to read in bytes and then deserialize as messages.
pub(crate) const BUFFER_SIZE: usize = 8192;

/// An interface to read messages from the network connection.
///
/// Implements the `Stream` trait to asynchronously read messages from the network connection.
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
    buffer: Option<BytesMut>,
    pending_read: Option<BytesMut>,
    pending_datagram: Option<usize>,
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
        let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
        buffer.resize(BUFFER_SIZE, 0);

        Self {
            local_addr,
            peer_addr,
            read_stream,
            buffer: Some(buffer),
            pending_read: None,
            pending_datagram: None,
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

    /// Check if the `Stream` of messages from the network is closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub(crate) fn close_stream(&mut self) {
        debug!("Closing the stream for connection with {}", self.peer_addr);
        self.buffer.take();
        self.pending_datagram.take();
        self.pending_read.take();
        self.closed = true;
    }
}

impl Stream for ConnectionReader {
    type Item = ConnectDatagram;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(size) = self.pending_datagram.take() {
                if let Some(pending_buf) = self.pending_read.take() {
                    if pending_buf.len() >= size {
                        trace!("{} pending bytes is large enough to deserialize datagram of size {} bytes", pending_buf.len(), size);
                        let mut data_buf = pending_buf;
                        let pending_buf = data_buf.split_off(size);

                        let datagram = ConnectDatagram::from_bytes_without_prefix(
                            data_buf.as_ref(),
                        )
                        .expect(
                            "could not construct ConnectDatagram from bytes despite explicit check",
                        );

                        trace!(
                            "deserialized message of size {} bytes",
                            datagram.serialized_size()
                        );
                        return match datagram.version() {
                            // do some special work based on version number if necessary
                            _ => {
                                if pending_buf.len() >= DATAGRAM_HEADER_BYTE_SIZE {
                                    trace!("can deserialize size of next datagram from remaining {} pending bytes", pending_buf.len());

                                    let mut size_buf = pending_buf;
                                    let pending_buf = size_buf.split_off(SIZE_PREFIX_BYTE_SIZE);

                                    let size = u32::from_be_bytes(
                                        size_buf
                                            .to_vec()
                                            .as_slice()
                                            .try_into()
                                            .expect("could not parse bytes into u32"),
                                    ) as usize;

                                    trace!("removed size of next datagram from pending bytes ({}), leaving {} pending bytes remaining", size, pending_buf.len());
                                    self.pending_datagram.replace(size);
                                    self.pending_read.replace(pending_buf);
                                } else {
                                    trace!("cannot deserialize size of next datagram from remaining {} pending bytes", pending_buf.len());
                                    self.pending_read.replace(pending_buf);
                                }

                                trace!("returning deserialized datagram to user");
                                Poll::Ready(Some(datagram))
                            }
                        };
                    } else {
                        trace!("{} pending bytes is not large enough to deserialize datagram of size {} bytes", pending_buf.len(), size);
                        self.pending_datagram.replace(size);
                        self.pending_read.replace(pending_buf);
                    }
                } else {
                    unreachable!()
                }
            }

            let mut buffer = if let Some(buffer) = self.buffer.take() {
                trace!("prepare buffer to read from the network stream");
                buffer
            } else {
                trace!("construct new buffer to read from the network stream");
                let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
                buffer.resize(BUFFER_SIZE, 0);
                buffer
            };

            trace!("reading from the network stream");
            let stream = self.read_stream.as_mut();
            match stream.poll_read(cx, &mut buffer) {
                Poll::Ready(Ok(bytes_read)) => {
                    if bytes_read > 0 {
                        trace!("read {} bytes from the network stream", bytes_read);
                    } else {
                        self.close_stream();
                        return Poll::Ready(None);
                    }

                    let mut pending_buf = if let Some(pending_buf) = self.pending_read.take() {
                        trace!("preparing {} pending bytes", pending_buf.len());
                        pending_buf
                    } else {
                        trace!("constructing new pending bytes");
                        BytesMut::new()
                    };

                    trace!(
                        "prepending incomplete data ({} bytes) from earlier read of network stream",
                        pending_buf.len()
                    );
                    pending_buf.extend_from_slice(&buffer[0..bytes_read]);

                    if self.pending_datagram.is_none() && pending_buf.len() >= SIZE_PREFIX_BYTE_SIZE
                    {
                        trace!(
                            "can deserialize size of next datagram from remaining {} pending bytes",
                            pending_buf.len()
                        );
                        let mut size_buf = pending_buf;
                        let pending_buf = size_buf.split_off(SIZE_PREFIX_BYTE_SIZE);

                        let size = u32::from_be_bytes(
                            size_buf
                                .to_vec()
                                .as_slice()
                                .try_into()
                                .expect("could not parse bytes into u32"),
                        ) as usize;

                        trace!("removed size of next datagram from pending bytes ({}), leaving {} pending bytes remaining", size, pending_buf.len());
                        self.pending_datagram.replace(size);
                        self.pending_read.replace(pending_buf);
                    } else {
                        trace!("size of next datagram already deserialized");
                        self.pending_read.replace(pending_buf);
                    }

                    trace!("finished reading from stream and storing buffer");
                    self.buffer.replace(buffer);
                }

                Poll::Ready(Err(err)) => {
                    error!(
                        "Encountered error when trying to read from network stream {}",
                        err
                    );
                    self.close_stream();
                    return Poll::Ready(None);
                }

                Poll::Pending => {
                    self.buffer.replace(buffer);
                    return Poll::Pending;
                }
            }
        }
    }
}
