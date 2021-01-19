use crate::schema::ConnectionMessage;
use async_channel::RecvError;
use async_std::net::SocketAddr;
use async_std::pin::Pin;
use futures::task::{Context, Poll};
use futures::{AsyncWrite, AsyncWriteExt, Sink};
use log::*;
use protobuf::Message;

pub use futures::SinkExt;
pub use futures::StreamExt;

pub struct ConnectionWriter {
    local_addr:    SocketAddr,
    peer_addr:     SocketAddr,
    write_stream:  Box<dyn AsyncWrite + Send + Sync + Unpin>,
    pending_write: Option<ConnectionMessage>,
    closed:        bool,
}

impl ConnectionWriter {
    pub fn new(
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        write_stream: Box<dyn AsyncWrite + Send + Sync + Unpin>,
    ) -> Self {
        Self {
            local_addr,
            peer_addr,
            write_stream,
            pending_write: None,
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

    fn send_to_conn(&mut self) -> Poll<Result<(), RecvError>> {
        if let Some(pending_msg) = self.pending_write.take() {
            trace!("Send pending message");
            if let Ok(buffer) = pending_msg.write_to_bytes() {
                let msg_size = buffer.len();
                trace!("{} bytes to be sent over network connection", msg_size);

                return if let Ok(_) =
                    futures::executor::block_on(self.write_stream.write_all(buffer.as_slice()))
                {
                    if let Ok(_) = futures::executor::block_on(self.write_stream.flush()) {
                        trace!("Sent message of {} bytes", msg_size);
                        Poll::Ready(Ok(()))
                    } else {
                        trace!("Encountered error while flushing queued bytes to network stream");
                        Poll::Ready(Err(RecvError))
                    }
                } else {
                    error!("Encountered error when writing to network stream");
                    Poll::Ready(Err(RecvError))
                };
            } else {
                error!("Encountered error when serializing message to bytes");
                return Poll::Ready(Err(RecvError));
            }
        } else {
            trace!("No message to send over connection");
        }

        Poll::Ready(Ok(()))
    }
}

impl<M: Message> Sink<M> for ConnectionWriter {
    type Error = RecvError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.pending_write.is_some() {
            trace!("Connection not ready to send message yet, waiting for prior message");
            Poll::Pending
        } else {
            trace!("Connection ready to send message");
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: M) -> Result<(), Self::Error> {
        trace!("Preparing message to be sent next");
        let stitch_msg: ConnectionMessage = ConnectionMessage::from_msg(item);
        self.pending_write.replace(stitch_msg);

        Ok(())
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.send_to_conn()
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let _ = self.send_to_conn();

        self.closed = true;
        if let Ok(_) = futures::executor::block_on(self.write_stream.close()) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(RecvError))
        }
    }
}
