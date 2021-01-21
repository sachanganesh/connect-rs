use crate::tls::TlsConnectionMetadata;
use crate::Connection;
use async_std::net::*;
use async_tls::TlsAcceptor;
use log::*;

#[allow(dead_code)]
pub struct TlsServer {
    local_addrs: SocketAddr,
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsServer {
    pub async fn new<A: ToSocketAddrs + std::fmt::Display>(
        ip_addrs: A,
        acceptor: TlsAcceptor,
    ) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(ip_addrs).await?;
        info!("Started TLS server at {}", listener.local_addr()?);

        Ok(Self {
            local_addrs: listener.local_addr()?,
            listener,
            acceptor,
        })
    }

    pub async fn accept(&self) -> anyhow::Result<Option<Connection>> {
        let (tcp_stream, peer_addr) = self.listener.accept().await?;
        debug!("Received connection attempt from {}", peer_addr);

        if let Ok(tls_stream) = self.acceptor.accept(tcp_stream).await {
            debug!("Completed TLS handshake with {}", peer_addr);
            Ok(Some(Connection::from(TlsConnectionMetadata::Server {
                local_addr: self.local_addrs.clone(),
                peer_addr,
                stream: tls_stream,
            })))
        } else {
            warn!("Could not encrypt connection with TLS from {}", peer_addr);
            Ok(None)
        }
    }
}
