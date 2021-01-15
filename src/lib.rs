mod reader;
pub mod schema;
pub mod tcp;
pub mod tls;
mod writer;

pub use crate::reader::ConnectionReader;
pub use crate::writer::ConnectionWriter;
use async_std::net::SocketAddr;
use futures::{AsyncRead, AsyncWrite};
pub use futures::{SinkExt, StreamExt};

pub struct Connection {
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    reader: ConnectionReader,
    writer: ConnectionWriter,
}

#[allow(dead_code)]
impl Connection {
    pub(crate) fn new(
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        read_stream: Box<dyn AsyncRead + Send + Sync + Unpin>,
        write_stream: Box<dyn AsyncWrite + Send + Sync + Unpin>,
    ) -> Self {
        Self {
            local_addr,
            peer_addr,
            reader: ConnectionReader::new(local_addr, peer_addr, read_stream),
            writer: ConnectionWriter::new(local_addr, peer_addr, write_stream),
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr.clone()
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr.clone()
    }

    pub fn split(self) -> (ConnectionReader, ConnectionWriter) {
        (self.reader, self.writer)
    }

    pub fn join(reader: ConnectionReader, writer: ConnectionWriter) -> Self {
        Self {
            local_addr: reader.local_addr(),
            peer_addr: reader.peer_addr(),
            reader,
            writer,
        }
    }

    pub fn reader(&mut self) -> &mut ConnectionReader {
        &mut self.reader
    }

    pub fn writer(&mut self) -> &mut ConnectionWriter {
        &mut self.writer
    }

    pub async fn close(self) -> SocketAddr {
        let peer_addr = self.peer_addr();

        drop(self.reader);
        // self.writer.close().await;
        drop(self.writer);

        return peer_addr;
    }
}
