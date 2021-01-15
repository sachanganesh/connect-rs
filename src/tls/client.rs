use async_std::task;
use async_tls::TlsConnector;
use log::*;

use crate::Connection;
use async_std::net::{TcpStream, SocketAddr, ToSocketAddrs};
use async_tls::client;
use async_tls::server;
use futures::AsyncReadExt;

pub enum TlsConnectionMetadata {
    Client { local_addr: SocketAddr, peer_addr: SocketAddr, stream: client::TlsStream<TcpStream> },
    Server { local_addr: SocketAddr, peer_addr: SocketAddr, stream: server::TlsStream<TcpStream> },
}

impl Connection {
    pub fn tls_client<A: ToSocketAddrs + std::fmt::Display>(
        ip_addrs: A,
        domain: &str,
        connector: TlsConnector,
    ) -> anyhow::Result<Self> {
        let stream = task::block_on(TcpStream::connect(&ip_addrs))?;
        info!("Established client TCP connection to {}", ip_addrs);
        stream.set_nodelay(true)?;

        let local_addr = stream.peer_addr()?;
        let peer_addr = stream.peer_addr()?;

        let encrypted_stream: client::TlsStream<TcpStream> =
            task::block_on(connector.connect(domain, stream))?;
        info!("Completed TLS handshake with {}", peer_addr);

        Ok(Self::from(TlsConnectionMetadata::Client { local_addr, peer_addr, stream: encrypted_stream }))
    }
}

impl From<TlsConnectionMetadata> for Connection {
    fn from(metadata: TlsConnectionMetadata) -> Self {
        match metadata {
            TlsConnectionMetadata::Client { local_addr, peer_addr, stream } => {
                let (read_stream, write_stream) = stream.split();

                Self::new(
                    local_addr,
                    peer_addr,
                    Box::new(read_stream),
                    Box::new(write_stream),
                )
            },

            TlsConnectionMetadata::Server { local_addr, peer_addr, stream } => {
                let (read_stream, write_stream) = stream.split();

                Self::new(
                    local_addr,
                    peer_addr,
                    Box::new(read_stream),
                    Box::new(write_stream),
                )
            }
        }


    }
}
