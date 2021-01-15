use crate::Connection;
use crate::tls::TlsConnectionMetadata;
use async_std::net::*;
use async_std::pin::Pin;
use async_std::prelude::*;
use async_std::task;
use async_tls::TlsAcceptor;
use futures::task::{Context, Poll};
use log::*;

#[allow(dead_code)]
pub struct TlsServer {
    local_addrs: SocketAddr,
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsServer {
    pub fn new<A: ToSocketAddrs + std::fmt::Display>(ip_addrs: A, acceptor: TlsAcceptor) -> anyhow::Result<Self> {
        let listener = task::block_on(TcpListener::bind(ip_addrs))?;
        info!("Started TLS server at {}", listener.local_addr()?);

        Ok(Self {
            local_addrs: listener.local_addr()?,
            listener,
            acceptor,
        })
    }
}

impl Stream for TlsServer {
    type Item = Connection;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(Ok(tcp_stream)) = futures::executor::block_on(self.listener.incoming().next()) {
            let local_addr = tcp_stream.local_addr().expect(
                "Local address could not be retrieved",
            );

            let peer_addr = tcp_stream.peer_addr().expect(
                "Peer address could not be retrieved",
            );
            debug!("Received connection attempt from {}", peer_addr);

            if let Ok(tls_stream) = futures::executor::block_on(self.acceptor.accept(tcp_stream)) {
                debug!("Established TLS connection from {}", peer_addr);
                Poll::Ready(Some(Connection::from(TlsConnectionMetadata::Server{ local_addr, peer_addr, stream: tls_stream })))
            } else {
                debug!("Could not encrypt connection with TLS from {}", peer_addr);
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
    }
}
