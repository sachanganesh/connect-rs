use crate::Connection;
use async_std::net::{SocketAddr, TcpListener as AsyncListener, ToSocketAddrs};
use async_std::pin::Pin;
use async_std::task::{Context, Poll};
use futures::{Stream, StreamExt};
use log::*;

/// Listens on a bound socket for incoming TCP connections to be handled as independent
/// [`Connection`]s.
///
/// Implements the [`Stream`] trait to asynchronously accept incoming TCP connections.
///
/// # Example
///
/// Basic usage:
///
/// ```ignore
/// let mut server = TcpListener::bind(ip_address).await?;
///
/// // wait for a connection to come in and be accepted
/// while let Some(mut conn) = server.next().await {
///     // do something with connection
/// }
/// ```
#[allow(dead_code)]
pub struct TcpListener {
    local_addrs: SocketAddr,
    listener: AsyncListener,
}

impl TcpListener {
    /// Creates a [`TcpListener`] by binding to an IP address and port and listens for incoming TCP
    /// connections.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// let mut server = TcpListener::bind("127.0.0.1:3456").await?;
    /// ```
    pub async fn bind<A: ToSocketAddrs + std::fmt::Display>(ip_addrs: A) -> anyhow::Result<Self> {
        let listener = AsyncListener::bind(&ip_addrs).await?;
        info!("Started TCP server at {}", &ip_addrs);

        Ok(Self {
            local_addrs: listener.local_addr()?,
            listener,
        })
    }

    /// Creates a [`Connection`] for the next `accept`ed TCP connection at the bound socket.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// let mut server = TcpListener::bind("127.0.0.1:3456").await?;
    /// while let Ok(mut conn) = server.accept().await? {
    ///     // handle the connection
    /// }
    /// ```
    pub async fn accept(&self) -> anyhow::Result<Connection> {
        let (stream, ip_addr) = self.listener.accept().await?;
        debug!("Received connection attempt from {}", ip_addr);

        Ok(Connection::from(stream))
    }
}

impl Stream for TcpListener {
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
