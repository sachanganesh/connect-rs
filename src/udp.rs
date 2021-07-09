use crate::reader::BUFFER_SIZE;
use crate::{ConnectDatagram, Connection, ConnectionWriteError};
use async_std::net::UdpSocket;
use async_stream::stream;
use futures::sink::unfold;
use log::*;
use std::convert::TryFrom;
use std::sync::Arc;

impl TryFrom<UdpSocket> for Connection {
    type Error = anyhow::Error;

    fn try_from(socket: UdpSocket) -> Result<Self, Self::Error> {
        let local_addr = socket.local_addr()?;
        let peer_addr = socket.peer_addr()?;

        let socket = Arc::new(socket);

        let read_socket = socket.clone();
        let reader = Box::pin(stream! {
            let mut buffer = vec![0; BUFFER_SIZE];

            while let Ok(bytes_read) = read_socket.recv(&mut buffer).await {
                if let Ok(datagram) = ConnectDatagram::from_bytes(&buffer[..bytes_read]) {
                    yield datagram
                } else {
                    warn!("Could not deserialize message from UDP message");
                }
            }
        });

        let write_socket = socket;
        let writer = Box::pin(unfold(0, move |_, datagram: ConnectDatagram| {
            let socket = write_socket.clone();
            async move {
                match socket.send(datagram.into_bytes().as_slice()).await {
                    Ok(bytes_written) => Ok(bytes_written),

                    Err(io_err) => Err(ConnectionWriteError::IoError(io_err)),
                }
            }
        }));

        Ok(Self::join(local_addr, peer_addr, reader, writer))
    }
}
