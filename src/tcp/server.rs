use crate::Connection;
use async_std::net::{SocketAddr, TcpListener, ToSocketAddrs};
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
