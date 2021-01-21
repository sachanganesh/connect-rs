use crate::Connection;
use async_std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use async_std::pin::Pin;
use async_std::task::{Context, Poll};
use futures::{Stream, StreamExt};
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

    pub async fn accept(&self) -> anyhow::Result<Connection> {
        let (stream, ip_addr) = self.listener.accept().await?;
        debug!("Received connection attempt from {}", ip_addr);

        Ok(Connection::from(stream))
    }
}

impl Stream for TcpServer {
    type Item = Connection;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match futures::executor::block_on(self.listener.incoming().next()) {
            Some(Ok(tcp_stream)) => {
                let peer_addr = tcp_stream
                    .peer_addr()
                    .expect("Could not retrieve peer IP address");
                debug!("Received connection attempt from {}", peer_addr);

                Poll::Ready(Some(Connection::from(tcp_stream)))
            }

            Some(Err(e)) => {
                error!(
                    "Encountered error when trying to accept new connection {}",
                    e
                );
                Poll::Pending
            }

            None => Poll::Ready(None),
        }
    }
}
