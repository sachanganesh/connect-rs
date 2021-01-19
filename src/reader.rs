use crate::schema::ConnectionMessage;
use async_std::net::SocketAddr;
use async_std::pin::Pin;
use bytes::{Buf, BytesMut};
use futures::task::{Context, Poll};
use futures::{AsyncRead, AsyncReadExt, Stream};
use log::*;
use protobuf::Message;
use std::convert::TryInto;

pub use futures::SinkExt;
pub use futures::StreamExt;
use protobuf::well_known_types::Any;

const BUFFER_SIZE: usize = 8192;

pub struct ConnectionReader {
    local_addr:   SocketAddr,
    peer_addr:    SocketAddr,
    read_stream:  Box<dyn AsyncRead + Send + Sync + Unpin>,
    pending_read: Option<BytesMut>,
    closed:       bool,
}

impl ConnectionReader {
    pub fn new(
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        read_stream: Box<dyn AsyncRead + Send + Sync + Unpin>,
    ) -> Self {
        Self {
            local_addr,
            peer_addr,
            read_stream,
            pending_read: None,
            closed: false,
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr.clone()
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr.clone()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

impl Stream for ConnectionReader {
    type Item = Any;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = BytesMut::new();
        buffer.resize(BUFFER_SIZE, 0);

        trace!("Starting new read loop for {}", self.local_addr);
        loop {
            trace!("Reading from the stream");
            match futures::executor::block_on(self.read_stream.read(&mut buffer)) {
                Ok(mut bytes_read) => {
                    if bytes_read > 0 {
                        trace!("Read {} bytes from the network stream", bytes_read)
                    } else if bytes_read == 0 && self.pending_read.is_none() {
                        return Poll::Pending;
                    }

                    if let Some(mut pending_buf) = self.pending_read.take() {
                        trace!("Prepending broken data ({} bytes) encountered from earlier read of network stream", pending_buf.len());
                        bytes_read += pending_buf.len();

                        pending_buf.unsplit(buffer);
                        buffer = pending_buf;
                    }

                    let mut bytes_read_u64: u64 = bytes_read.try_into().expect(
                        format!("Conversion from usize ({}) to u64 failed", bytes_read).as_str(),
                    );
                    while bytes_read_u64 > 0 {
                        trace!(
                            "{} bytes from network stream still unprocessed",
                            bytes_read_u64
                        );

                        buffer.resize(bytes_read, 0);

                        match ConnectionMessage::parse_from_bytes(buffer.as_ref()) {
                            Ok(mut data) => {
                                let serialized_size = data.compute_size();
                                trace!("Deserialized message of size {} bytes", serialized_size);

                                buffer.advance(serialized_size as usize);

                                let serialized_size_u64: u64 = serialized_size.try_into().expect(
                                    format!(
                                        "Conversion from usize ({}) to u64 failed",
                                        serialized_size
                                    )
                                    .as_str(),
                                );
                                bytes_read_u64 -= serialized_size_u64;
                                trace!("{} bytes still unprocessed", bytes_read_u64);

                                trace!("Sending deserialized message downstream");
                                return Poll::Ready(Some(data.take_payload()));
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
                Err(_err) => {
                    trace!("Closing the stream");
                    self.pending_read.take();
                    self.closed = true;
                    return Poll::Ready(None);
                }
            }
        }
    }
}
