//! This crate provides a simple brokerless message-queue abstraction over asynchronous network
//! streams.
//!
//! # Examples
//! Please use the [example programs](https://github.com/sachanganesh/connect-rs/tree/main/examples)
//! provided to help understand crate usage.
//!
//! # Why?
//! When building networked applications, developers shouldn't have to focus on repeatedly solving
//! the problem of reliable, fault-tolerant message delivery over byte-streams. By using a message
//! queue abstraction, crate users can focus on core application logic and leave the low-level
//! networking and message-queue guarantees to the abstraction.
//!

#![feature(doc_cfg)]

mod protocol;
mod reader;
pub mod tcp;

#[cfg(feature = "tls")]
#[doc(cfg(feature = "tls"))]
pub mod tls;

mod writer;

pub use crate::protocol::{ConnectDatagram, DatagramEmptyError};
pub use crate::reader::ConnectionReader;
pub use crate::writer::{ConnectionWriter, ConnectionWriteError};
use async_std::{net::SocketAddr, pin::Pin};
use futures::{AsyncRead, AsyncWrite};
pub use futures::{SinkExt, StreamExt};

/// Wrapper around a [`ConnectionReader`] and [`ConnectionWriter`] to read and write on a network
/// connection.
pub struct Connection {
    reader: ConnectionReader,
    writer: ConnectionWriter,
}

#[allow(dead_code)]
impl Connection {
    pub(crate) fn new(
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        read_stream: Pin<Box<dyn AsyncRead + Send + Sync>>,
        write_stream: Pin<Box<dyn AsyncWrite + Send + Sync>>,
    ) -> Self {
        Self {
            reader: ConnectionReader::new(local_addr, peer_addr, read_stream),
            writer: ConnectionWriter::new(local_addr, peer_addr, write_stream),
        }
    }

    /// Get the local IP address and port.
    pub fn local_addr(&self) -> SocketAddr {
        self.reader.local_addr()
    }

    /// Get the peer IP address and port.
    pub fn peer_addr(&self) -> SocketAddr {
        self.reader.peer_addr()
    }

    /// Consume the [`Connection`] to split into separate [`ConnectionReader`] and
    /// [`ConnectionWriter`] halves.
    ///
    /// [`Connection`]s are split when reading and writing must be concurrent operations.
    pub fn split(self) -> (ConnectionReader, ConnectionWriter) {
        (self.reader, self.writer)
    }

    /// Re-wrap the [`ConnectionReader`] and [`ConnectionWriter`] halves into a [`Connection`].
    pub fn join(reader: ConnectionReader, writer: ConnectionWriter) -> Self {
        Self { reader, writer }
    }

    /// Get mutable access to the underlying [`ConnectionReader`].
    pub fn reader(&mut self) -> &mut ConnectionReader {
        &mut self.reader
    }

    /// Get mutable access to the underlying [`ConnectionWriter`].
    pub fn writer(&mut self) -> &mut ConnectionWriter {
        &mut self.writer
    }

    /// Close the connection by closing both the reading and writing halves.
    pub async fn close(self) -> SocketAddr {
        let peer_addr = self.peer_addr();
        let (reader, writer) = self.split();

        drop(reader);

        // writer.close().await;
        drop(writer);

        return peer_addr;
    }
}
