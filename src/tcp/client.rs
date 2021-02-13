use log::*;

use crate::Connection;
use async_std::net::{TcpStream, ToSocketAddrs};

impl Connection {
    /// Creates a [`Connection`] that uses a TCP transport.
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// let mut conn = Connection::tcp_client("127.0.0.1:3456").await?;
    /// ```
    pub async fn tcp_client<A: ToSocketAddrs + std::fmt::Display>(
        ip_addrs: A,
    ) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(&ip_addrs).await?;
        info!("Established client TCP connection to {}", ip_addrs);

        stream.set_nodelay(true)?;
        Ok(Self::from(stream))
    }
}

impl From<TcpStream> for Connection {
    /// Creates a [`Connection`] using a TCP transport from an async [`TcpStream`].
    fn from(stream: TcpStream) -> Self {
        let write_stream = stream.clone();

        let local_addr = stream
            .local_addr()
            .expect("Local address could not be retrieved");

        let peer_addr = stream
            .peer_addr()
            .expect("Peer address could not be retrieved");

        Self::new(
            local_addr,
            peer_addr,
            Box::pin(stream),
            Box::pin(write_stream),
        )
    }
}
