use crate::schema::StitchMessage;
use async_channel::RecvError;
use async_std::net::SocketAddr;
use async_std::pin::Pin;
use futures::task::{Context, Poll};
use futures::{AsyncWrite, AsyncWriteExt, Sink};
use log::*;
use protobuf::Message;

pub use futures::SinkExt;
pub use futures::StreamExt;

pub struct StitchConnectionWriter {
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    write_stream: Box<dyn AsyncWrite + Send + Sync + Unpin>,
    pending_write: Option<StitchMessage>,
}

impl StitchConnectionWriter {
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
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr.clone()
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr.clone()
    }
}

impl<T: Message> Sink<T> for StitchConnectionWriter {
    type Error = RecvError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.pending_write.is_some() {
            debug!("Connection not ready to send message yet, waiting for prior message");
            Poll::Pending
        } else {
            debug!("Connection ready to send message");
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        debug!("Preparing message to be sent next");
        let stitch_msg: StitchMessage = StitchMessage::from_msg(item);
        self.pending_write.replace(stitch_msg);

        Ok(())
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        if let Some(pending_msg) = self.pending_write.take() {
            debug!("Send pending message");
            if let Ok(buffer) = pending_msg.write_to_bytes() {
                let msg_size = buffer.len();
                debug!("{} bytes to be sent over network connection", msg_size);

                debug!("{:?}", buffer.as_slice());

                return if let Ok(_) =
                    futures::executor::block_on(self.write_stream.write_all(buffer.as_slice()))
                {
                    if let Ok(_) = futures::executor::block_on(self.write_stream.flush()) {
                        debug!("Sent message of {} bytes", msg_size);
                        Poll::Ready(Ok(()))
                    } else {
                        debug!("Encountered error while flushing queued bytes to network stream");
                        Poll::Ready(Err(RecvError))
                    }
                } else {
                    debug!("Encountered error when writing to network stream");
                    Poll::Ready(Err(RecvError))
                };
            } else {
                debug!("Encountered error when serializing message to bytes");
                return Poll::Ready(Err(RecvError));
            }
        } else {
            debug!("No message to send over connection");
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        if let Ok(_) = futures::executor::block_on(self.write_stream.close()) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(RecvError))
        }
    }
}
