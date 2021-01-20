use crate::tls::TlsConnectionMetadata;
use crate::Connection;
use async_std::net::*;
use async_std::pin::Pin;
use async_tls::TlsAcceptor;
use futures::Stream;
use futures::task::{Context, Poll};
use futures_lite::StreamExt;
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
}

impl Stream for TlsServer {
    type Item = Connection;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.listener.incoming().poll_next(cx) {
            Poll::Pending => Poll::Pending,

            Poll::Ready(Some(Ok(tcp_stream))) => {
                let local_addr = tcp_stream
                    .local_addr()
                    .expect("Local address could not be retrieved");

                let peer_addr = tcp_stream
                    .peer_addr()
                    .expect("Peer address could not be retrieved");

                debug!(
                    "Received connection attempt from {}", peer_addr
                );

                if let Ok(tls_stream) = futures::executor::block_on(self.acceptor.accept(tcp_stream)) {
                    debug!("Completed TLS handshake with {}", peer_addr);
                    Poll::Ready(Some(Connection::from(TlsConnectionMetadata::Server {
                        local_addr,
                        peer_addr,
                        stream: tls_stream,
                    })))
                } else {
                    warn!("Could not encrypt connection with TLS from {}", peer_addr);
                    Poll::Pending
                }
            },

            Poll::Ready(Some(Err(e))) => {
                error!(
                    "Encountered error when accepting connection attempt: {}", e
                );

                Poll::Pending
            }

            Poll::Ready(None) => {
                info!("Shutting TLS server down at {}", self.local_addrs);
                Poll::Ready(None)
            },
        }
    }
}
