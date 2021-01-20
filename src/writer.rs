use crate::schema::ConnectionMessage;
use async_channel::RecvError;
use async_std::net::SocketAddr;
use async_std::pin::Pin;
use futures::{AsyncWrite, Sink};
use futures::io::IoSlice;
use futures::task::{Context, Poll};
use log::*;
use protobuf::Message;

pub use futures::SinkExt;
pub use futures::StreamExt;

pub struct ConnectionWriter {
    local_addr:     SocketAddr,
    peer_addr:      SocketAddr,
    write_stream:   Pin<Box<dyn AsyncWrite + Send + Sync>>,
    pending_writes: Vec<Vec<u8>>,
    closed:         bool,
}

impl ConnectionWriter {
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

impl<M: Message> Sink<M> for ConnectionWriter {
    type Error = RecvError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.is_closed() {
            trace!("Connection is closed, cannot send message");
            Poll::Ready(Err(RecvError))
        } else {
            trace!("Connection ready to send message");
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: M) -> Result<(), Self::Error> {
        trace!("Preparing message to be sent next");
        let msg: ConnectionMessage = ConnectionMessage::from_msg(item);

        if let Ok(buffer) = msg.write_to_bytes() {
            let msg_size = buffer.len();
            trace!("Serialized pending message into {} bytes", msg_size);

            self.pending_writes.push(buffer);

            Ok(())
        } else {
            error!("Encountered error when serializing message to bytes");
            Err(RecvError)
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        if self.pending_writes.len() > 0 {
            let stream = self.write_stream.as_mut();

            match stream.poll_flush(cx) {
                Poll::Pending => Poll::Pending,

                Poll::Ready(Ok(_)) => {
                    trace!("Sending pending bytes");

                    let pending = self.pending_writes.split_off(0);
                    let writeable_vec: Vec<IoSlice> = pending.iter().map(|p| {
                        IoSlice::new(p)
                    }).collect();

                    let stream = self.write_stream.as_mut();
                    match stream.poll_write_vectored(cx, writeable_vec.as_slice()) {
                        Poll::Pending => Poll::Pending,

                        Poll::Ready(Ok(bytes_written)) => {
                            trace!("Wrote {} bytes to network stream", bytes_written);
                            Poll::Ready(Ok(()))
                        },

                        Poll::Ready(Err(_e)) => {
                            error!("Encountered error when writing to network stream");
                            Poll::Ready(Err(RecvError))
                        },
                    }
                },

                Poll::Ready(Err(_e)) => {
                    error!("Encountered error when flushing network stream");
                    Poll::Ready(Err(RecvError))
                }
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.closed = true;

        let flush = if self.pending_writes.len() > 0 {
            let stream = self.write_stream.as_mut();

            match stream.poll_flush(cx) {
                Poll::Pending => Poll::Pending,

                Poll::Ready(Ok(_)) => {
                    trace!("Sending pending bytes");

                    let pending = self.pending_writes.split_off(0);
                    let writeable_vec: Vec<IoSlice> = pending.iter().map(|p| {
                        IoSlice::new(p)
                    }).collect();

                    let stream = self.write_stream.as_mut();
                    match stream.poll_write_vectored(cx, writeable_vec.as_slice()) {
                        Poll::Pending => Poll::Pending,

                        Poll::Ready(Ok(bytes_written)) => {
                            trace!("Wrote {} bytes to network stream", bytes_written);
                            Poll::Ready(Ok(()))
                        },

                        Poll::Ready(Err(_e)) => {
                            error!("Encountered error when writing to network stream");
                            Poll::Ready(Err(RecvError))
                        },
                    }
                },

                Poll::Ready(Err(_e)) => {
                    error!("Encountered error when flushing network stream");
                    Poll::Ready(Err(RecvError))
                }
            }
        } else {
            Poll::Ready(Ok(()))
        };

        match flush {
            Poll::Pending => Poll::Pending,

            Poll::Ready(Ok(_)) => {
                let stream = self.write_stream.as_mut();

                match stream.poll_close(cx) {
                    Poll::Pending => Poll::Pending,

                    Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),

                    Poll::Ready(Err(_e)) => Poll::Ready(Err(RecvError)),
                }
            },

            err => err,
        }
    }
}
