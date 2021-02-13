use crate::Connection;
use async_std::net::{SocketAddr, TcpListener as AsyncListener, TcpStream, ToSocketAddrs};
use async_std::pin::Pin;
use async_std::task::{Context, Poll};
use async_stream::stream;
use futures::Stream;
use futures_lite::StreamExt;
use log::*;

/// Listens on a bound socket for incoming TCP connections to be handled as independent
/// [`Connection`]s.
///
/// Implements the [`Stream`] trait to asynchronously accept incoming TCP connections.
///
/// # Example
///
/// Please see the [tcp-echo-server](https://github.com/sachanganesh/connect-rs/blob/main/examples/tcp-echo-server/src/main.rs)
/// example program for a more thorough showcase.
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
    // listener: AsyncListener,
    conn_stream:
        Pin<Box<dyn Stream<Item = Option<Result<TcpStream, std::io::Error>>> + Send + Sync>>,
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

        let local_addrs = listener.local_addr()?;

        let stream = Box::pin(stream! {
            loop {
                yield listener.incoming().next().await;
            }
        });

        Ok(Self {
            local_addrs,
            // listener,
            conn_stream: stream,
        })
    }

    // /// Creates a [`Connection`] for the next `accept`ed TCP connection at the bound socket.
    // ///
    // /// # Example
    // ///
    // /// Basic usage:
    // ///
    // /// ```ignore
    // /// let mut server = TcpListener::bind("127.0.0.1:3456").await?;
    // /// while let Ok(mut conn) = server.accept().await? {
    // ///     // handle the connection
    // /// }
    // /// ```
    // pub async fn accept(&self) -> anyhow::Result<Connection> {
    //     let (stream, ip_addr) = self.listener.accept().await?;
    //     debug!("Received connection attempt from {}", ip_addr);
    //
    //     Ok(Connection::from(stream))
    // }
}

impl Stream for TcpListener {
    type Item = Connection;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.conn_stream.poll_next(cx) {
            Poll::Ready(Some(Some(Ok(tcp_stream)))) => {
                let peer_addr = tcp_stream
                    .peer_addr()
                    .expect("Could not retrieve peer IP address");
                debug!("Received connection attempt from {}", peer_addr);

                Poll::Ready(Some(Connection::from(tcp_stream)))
            }

            Poll::Ready(Some(Some(Err(err)))) => {
                error!(
                    "Encountered error when trying to accept new connection {}",
                    err
                );
                Poll::Pending
            }

            Poll::Ready(Some(None)) => Poll::Ready(None),

            Poll::Ready(None) => Poll::Ready(None),

            Poll::Pending => Poll::Pending,
        }
    }
}
