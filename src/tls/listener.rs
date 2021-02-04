use crate::tls::TlsConnectionMetadata;
use crate::Connection;
use async_std::net::{SocketAddr, TcpListener, ToSocketAddrs};
use async_std::pin::Pin;
use async_std::task::{Context, Poll};
use async_tls::TlsAcceptor;
use futures::{Stream, StreamExt};
use log::*;

/// Listens on a bound socket for incoming TLS connections to be handled as independent
/// [`Connection`]s.
///
/// Implements the [`Stream`] trait to asynchronously accept incoming TLS connections.
///
/// # Example
///
/// Basic usage:
///
/// ```ignore
/// let mut server = TlsListener::bind("127.0.0.1:3456", config.into()).await?;
///
/// // wait for a connection to come in and be accepted
/// while let Some(mut conn) = server.next().await {
///     // do something with connection
/// }
/// ```
#[allow(dead_code)]
pub struct TlsListener {
    local_addrs: SocketAddr,
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsListener {
    /// Creates a [`TlsListener`] by binding to an IP address and port and listens for incoming TLS
    /// connections that have successfully been accepted.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// let mut server = TlsListener::bind("127.0.0.1:3456", config.into()).await?;
    /// ```
    pub async fn bind<A: ToSocketAddrs + std::fmt::Display>(
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

    /// Creates a [`Connection`] for the next `accept`ed TCP connection at the bound socket.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// let mut server = TlsListener::bind("127.0.0.1:3456", config.into()).await?;
    /// while let Some(mut conn) = server.next().await {
    ///     // do something with connection
    /// }
    /// ```
    pub async fn accept(&self) -> anyhow::Result<Connection> {
        let (tcp_stream, peer_addr) = self.listener.accept().await?;
        debug!("Received connection attempt from {}", peer_addr);

        match self.acceptor.accept(tcp_stream).await {
            Ok(tls_stream) => {
                debug!("Completed TLS handshake with {}", peer_addr);
                Ok(Connection::from(TlsConnectionMetadata::Listener {
                    local_addr: self.local_addrs.clone(),
                    peer_addr,
                    stream: tls_stream,
                }))
            }

            Err(e) => {
                warn!("Could not encrypt connection with TLS from {}", peer_addr);
                Err(anyhow::Error::new(e))
            }
        }
    }
}

impl Stream for TlsListener {
    type Item = Connection;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match futures::executor::block_on(self.listener.incoming().next()) {
            Some(Ok(tcp_stream)) => {
                let peer_addr = tcp_stream
                    .peer_addr()
                    .expect("Could not retrieve peer IP address");
                debug!("Received connection attempt from {}", peer_addr);

                match futures::executor::block_on(self.acceptor.accept(tcp_stream)) {
                    Ok(tls_stream) => {
                        debug!("Completed TLS handshake with {}", peer_addr);
                        Poll::Ready(Some(Connection::from(TlsConnectionMetadata::Listener {
                            local_addr: self.local_addrs.clone(),
                            peer_addr,
                            stream: tls_stream,
                        })))
                    }

                    Err(_e) => {
                        warn!("Could not encrypt connection with TLS from {}", peer_addr);
                        Poll::Pending
                    }
                }
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
