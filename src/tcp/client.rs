use async_std::task;
use log::*;

use crate::StitchConnection;
use async_std::net::{TcpStream, ToSocketAddrs};

impl StitchConnection {
    pub fn tcp_client<A: ToSocketAddrs + std::fmt::Display>(
        ip_addrs: A,
    ) -> anyhow::Result<StitchConnection> {
        let read_stream = task::block_on(TcpStream::connect(&ip_addrs))?;
        info!("Established client TCP connection to {}", ip_addrs);

        Ok(Self::from(read_stream))
    }
}

impl From<TcpStream> for StitchConnection {
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
            Box::new(stream),
            Box::new(write_stream),
        )
    }
}
