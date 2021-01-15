pub mod schema;
pub mod tcp;
// @todo pub mod tls;
mod reader;
mod writer;

pub use crate::reader::StitchConnectionReader;
use crate::schema::StitchMessage;
pub use crate::writer::StitchConnectionWriter;
use async_channel::RecvError;
use async_std::net::SocketAddr;
use async_std::pin::Pin;
use bytes::{Buf, BytesMut};
use futures::task::{Context, Poll};
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, Sink, Stream};
use log::*;
use protobuf::Message;
use std::convert::TryInto;

pub use futures::SinkExt;
pub use futures::StreamExt;
use protobuf::well_known_types::Any;

pub struct StitchConnection {
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    reader: StitchConnectionReader,
    writer: StitchConnectionWriter,
}

#[allow(dead_code)]
impl StitchConnection {
    pub(crate) fn new(
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        read_stream: Box<dyn AsyncRead + Send + Sync + Unpin>,
        write_stream: Box<dyn AsyncWrite + Send + Sync + Unpin>,
    ) -> Self {
        Self {
            local_addr,
            peer_addr,
            reader: StitchConnectionReader::new(local_addr, peer_addr, read_stream),
            writer: StitchConnectionWriter::new(local_addr, peer_addr, write_stream),
        }
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr.clone()
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr.clone()
    }

    pub fn split(self) -> (StitchConnectionReader, StitchConnectionWriter) {
        (self.reader, self.writer)
    }

    pub fn join(reader: StitchConnectionReader, writer: StitchConnectionWriter) -> Self {
        Self {
            local_addr: reader.local_addr(),
            peer_addr: reader.peer_addr(),
            reader,
            writer,
        }
    }

    pub fn reader(&mut self) -> &mut StitchConnectionReader {
        &mut self.reader
    }

    pub fn writer(&mut self) -> &mut StitchConnectionWriter {
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
