//! This Rust crate provides a simple, brokerless message-queue abstraction over asynchronous
//! network streams. It guarantees ordered message delivery and reception, and both TCP and TLS
//! transports are supported.
//!
//! # Examples
//!
//! ````ignore
//! // create a client connection to the server
//! let mut conn = Connection::tcp_client(ip_address).await?;
//!
//! // construct a new message
//! let msg = String::from("Hello world!");
//! let envelope: ConnectDatagram = ConnectDatagram::new(65535, msg.into_bytes())?;
//!
//! // send a message to the server
//! conn.writer().send(envelope).await?;
//!
//! // wait for the echo-server to reply with an echo
//! if let Some(mut envelope) = conn.reader().next().await {
//!     // take the message payload from the envelope
//!     let data: Vec<u8> = envelope.take_data().unwrap();
//!
//!     // reconstruct the original message
//!     let msg = String::from_utf8(data)?;
//!     assert_eq!("Hello world!", msg.as_str());
//! }
//! ````
//!
//! In addition to the [crate documentation](https://docs.rs/connect/latest/connect/), please use
//! the provided [example programs](https://github.com/sachanganesh/connect-rs/tree/main/examples)
//! as a practical reference for crate usage.
//!
//! - TCP
//!     - [TCP Echo Server](https://github.com/sachanganesh/connect-rs/tree/main/examples/tcp-echo-server)
//!     - [TCP Client](https://github.com/sachanganesh/connect-rs/tree/main/examples/tcp-client)
//! - TLS
//!     - [TLS Echo Server](https://github.com/sachanganesh/connect-rs/tree/main/examples/tls-echo-server)
//!     - [TLS Client](https://github.com/sachanganesh/connect-rs/tree/main/examples/tls-client)
//!
//! # Why?
//!
//! When building networked applications, developers shouldn't have to focus on repeatedly solving
//! the problem of reliable, ordered message delivery over byte-streams. By using a message
//! queue abstraction, crate users can focus on core application logic and leave the low-level
//! networking and message-queue guarantees to the abstraction.
//!
//! Connect provides a `ConnectionWriter` and `ConnectionReader` interface to concurrently send
//! and receive messages over a network connection. Each user-provided message is prefixed by 8
//! bytes, containing a size-prefix (4 bytes), version tag (2 bytes), and recipient tag (2 bytes).
//! The size-prefix and version tag are used internally to deserialize messages received from the
//! network connection. The recipient tag is intended for crate users to identify the message
//! recipient, although the library leaves that up to the user's discretion.
//!
//! Library users must serialize their custom messages into bytes (`Vec<u8>`), prior to
//! constructing a `ConnectDatagram`, which can then be passed to a `ConnectionWriter`.
//! Consequently, `ConnectionReader`s will return `ConnectDatagram`s containing the message payload
//! (`Vec<u8>` again) to the user to deserialize.
//!
//! Requiring crate users to serialize data before constructing a datagram may appear redundant, but
//! gives the developer the freedom to use a serialization format of their choosing. This means that
//! library users can do interesting things such as:
//!
//! - Use the recipient tag to signify which serialization format was used for that message
//! - Use the recipient tag to signify the type of message being sent
//!

// #![feature(doc_cfg)]

mod protocol;
mod reader;
pub mod tcp;
mod writer;

#[cfg(feature = "tls")]
#[doc(cfg(feature = "tls"))]
pub mod tls;

use async_std::{net::SocketAddr, pin::Pin};
use futures::{AsyncRead, AsyncWrite};

pub use crate::protocol::{ConnectDatagram, DatagramError};
pub use crate::reader::ConnectionReader;
pub use crate::writer::{ConnectionWriteError, ConnectionWriter};
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

#[cfg(test)]
mod tests {}
