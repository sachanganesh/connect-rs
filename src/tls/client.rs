use async_std::net::{TcpStream, ToSocketAddrs};
use async_tls::client;
use async_tls::TlsConnector;
use futures::AsyncReadExt;
use log::*;

use crate::tls::TlsConnectionMetadata;
use crate::Connection;

impl Connection {
    /// Creates a [`Connection`] that uses a TLS transport
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// let mut conn = Connection::tls_client("127.0.0.1:3456", "localhost", client_config.into()).await?;
    /// ```
    ///
    /// Please see the [tls-client](https://github.com/sachanganesh/connect-rs/blob/main/examples/tls-client/src/main.rs)
    /// example program for a more thorough showcase.
    pub async fn tls_client<A: ToSocketAddrs + std::fmt::Display>(
        ip_addrs: A,
        domain: &str,
        connector: TlsConnector,
    ) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(&ip_addrs).await?;
        info!("Established client TCP connection to {}", ip_addrs);
        stream.set_nodelay(true)?;

        let local_addr = stream.peer_addr()?;
        let peer_addr = stream.peer_addr()?;

        let encrypted_stream: client::TlsStream<TcpStream> =
            connector.connect(domain, stream).await?;
        info!("Completed TLS handshake with {}", peer_addr);

        Ok(Self::from(TlsConnectionMetadata::Client {
            local_addr,
            peer_addr,
            stream: encrypted_stream,
        }))
    }
}

impl From<TlsConnectionMetadata> for Connection {
    /// Creates a [`Connection`] using a TLS transport from [`TlsConnectionMetadata`].
    fn from(metadata: TlsConnectionMetadata) -> Self {
        match metadata {
            TlsConnectionMetadata::Client {
                local_addr,
                peer_addr,
                stream,
            } => {
                let (read_stream, write_stream) = stream.split();

                Self::new(
                    local_addr,
                    peer_addr,
                    Box::pin(read_stream),
                    Box::pin(write_stream),
                )
            }

            TlsConnectionMetadata::Listener {
                local_addr,
                peer_addr,
                stream,
            } => {
                let (read_stream, write_stream) = stream.split();

                Self::new(
                    local_addr,
                    peer_addr,
                    Box::pin(read_stream),
                    Box::pin(write_stream),
                )
            }
        }
    }
}
