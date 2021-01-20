use crate::Connection;
use async_std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use async_std::pin::Pin;
use futures::task::{Context, Poll};
use futures::Stream;
use futures_lite::stream::StreamExt;
use log::*;

#[allow(dead_code)]
pub struct TcpServer {
    local_addrs: SocketAddr,
    listener: TcpListener,
}

impl TcpServer {
    pub async fn new<A: ToSocketAddrs + std::fmt::Display>(ip_addrs: A) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(&ip_addrs).await?;
        info!("Started TCP server at {}", &ip_addrs);

        Ok(Self {
            local_addrs: listener.local_addr()?,
            listener,
        })
    }
}

impl Stream for TcpServer {
    type Item = Connection;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.listener.incoming().poll_next(cx) {
            Poll::Pending => Poll::Pending,

            Poll::Ready(Some(Ok(conn))) => {
                debug!(
                    "Received connection attempt from {}",
                    conn.peer_addr()
                        .expect("Peer address could not be retrieved")
                );

                Poll::Ready(Some(Connection::from(conn)))
            },

            Poll::Ready(Some(Err(e))) => {
                error!(
                    "Encountered error when accepting connection attempt: {}", e
                );

                Poll::Pending
            },

            Poll::Ready(None) => {
                info!("Shutting TCP server down at {}", self.local_addrs);
                Poll::Ready(None)
            },
        }
    }
}
