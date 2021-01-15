use crate::Connection;
use async_std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use async_std::pin::Pin;
use async_std::task;
use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};
use log::*;

#[allow(dead_code)]
pub struct TcpServer {
    local_addrs: SocketAddr,
    listener: TcpListener,
}

impl TcpServer {
    pub fn new<A: ToSocketAddrs + std::fmt::Display>(ip_addrs: A) -> anyhow::Result<Self> {
        let listener = task::block_on(TcpListener::bind(&ip_addrs))?;
        info!("Started TCP server at {}", &ip_addrs);

        Ok(Self {
            local_addrs: listener.local_addr()?,
            listener,
        })
    }
}

impl Stream for TcpServer {
    type Item = Connection;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(Ok(conn)) = futures::executor::block_on(self.listener.incoming().next()) {
            Poll::Ready(Some(Connection::from(conn)))
        } else {
            Poll::Ready(None)
        }
    }
}
