use crate::tls::TlsConnectionMetadata;
use crate::Connection;
use async_std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use async_std::pin::Pin;
use async_std::task::{Context, Poll};
use async_stream::stream;
use async_tls::{server::TlsStream, TlsAcceptor};
use futures::Stream;
use futures_lite::StreamExt;
use log::*;

/// Listens on a bound socket for incoming TLS connections to be handled as independent
/// [`Connection`]s.
///
/// Implements the [`Stream`] trait to asynchronously accept incoming TLS connections.
///
/// # Example
///
/// Please see the [tls-echo-server](https://github.com/sachanganesh/connect-rs/blob/main/examples/tls-echo-server/src/main.rs)
/// example program for a more thorough showcase.
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
    conn_stream: Pin<
        Box<
            dyn Stream<
                    Item = Option<
                        Option<(SocketAddr, Result<TlsStream<TcpStream>, std::io::Error>)>,
                    >,
                > + Send
                + Sync,
        >,
    >,
    // listener: TcpListener,
    // acceptor: TlsAcceptor,
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
        let listener = TcpListener::bind(&ip_addrs).await?;
        info!("Started TLS server at {}", &ip_addrs);

        let local_addrs = listener.local_addr()?;

        let stream = Box::pin(stream! {
            loop {
                yield match listener.incoming().next().await {
                    Some(Ok(tcp_stream)) => {
                        let peer_addr = tcp_stream
                            .peer_addr()
                            .expect("Could not retrieve peer IP address");
                        debug!("Received connection attempt from {}", peer_addr);

                        Some(Some((peer_addr, acceptor.accept(tcp_stream).await)))
                    }

                    Some(Err(err)) => {
                        error!(
                            "Encountered error when trying to accept new connection {}",
                            err
                        );
                        Some(None)
                    }

                    None => None,
                }
            }
        });

        Ok(Self {
            local_addrs,
            conn_stream: stream,
            // listener,
            // acceptor,
        })
    }

    // /// Creates a [`Connection`] for the next `accept`ed TCP connection at the bound socket.
    // ///
    // /// # Example
    // ///
    // /// Basic usage:
    // ///
    // /// ```ignore
    // /// let mut server = TlsListener::bind("127.0.0.1:3456", config.into()).await?;
    // /// while let Some(mut conn) = server.next().await {
    // ///     // do something with connection
    // /// }
    // /// ```
    // pub async fn accept(&self) -> anyhow::Result<Connection> {
    //     let (tcp_stream, peer_addr) = self.listener.accept().await?;
    //     debug!("Received connection attempt from {}", peer_addr);
    //
    //     match self.acceptor.accept(tcp_stream).await {
    //         Ok(tls_stream) => {
    //             debug!("Completed TLS handshake with {}", peer_addr);
    //             Ok(Connection::from(TlsConnectionMetadata::Listener {
    //                 local_addr: self.local_addrs.clone(),
    //                 peer_addr,
    //                 stream: tls_stream,
    //             }))
    //         }
    //
    //         Err(e) => {
    //             warn!("Could not encrypt connection with TLS from {}", peer_addr);
    //             Err(anyhow::Error::new(e))
    //         }
    //     }
    // }
}

impl Stream for TlsListener {
    type Item = Connection;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.conn_stream.poll_next(cx) {
            Poll::Ready(Some(Some(Some((peer_addr, Ok(tls_stream)))))) => {
                debug!("Completed TLS handshake with {}", peer_addr);
                Poll::Ready(Some(Connection::from(TlsConnectionMetadata::Listener {
                    local_addr: self.local_addrs.clone(),
                    peer_addr,
                    stream: tls_stream,
                })))
            }

            Poll::Ready(Some(Some(Some((peer_addr, Err(err)))))) => {
                warn!(
                    "Could not encrypt connection with TLS from {}: {}",
                    peer_addr, err
                );
                Poll::Pending
            }

            Poll::Pending => Poll::Pending,

            _ => Poll::Ready(None),
        }
    }
}
