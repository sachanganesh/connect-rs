//! TLS transport client and listener implementations.
//!
//! <br/>
//!
//! This module primarily exposes the TLS client implementation over a [`Connection`] type and the
//! TLS listener implementation as [`TlsListener`].
//!

#[allow(unused_imports)]
pub(crate) use crate::Connection;

pub(crate) mod client;
pub(crate) mod listener;

use async_std::net::TcpStream;
use async_tls::server;
use std::net::SocketAddr;

pub use client::*;
pub use listener::*;

#[cfg(feature = "tls")]
// #[doc(cfg(feature = "tls"))]
pub use async_tls;

#[cfg(feature = "tls")]
// #[doc(cfg(feature = "tls"))]
pub use rustls;

/// Used to differentiate between an outgoing connection ([Client](`TlsConnectionMetadata::Client`))
/// or incoming connection listener ([Listener](`TlsConnectionMetadata::Listener`)).
///
/// The async TLS library used by this crate has two differing stream types based on whether the
/// connection being established is either a client or server. This is to aid with handling that
/// distinction during connection instantiation.
pub enum TlsConnectionMetadata {
    Client {
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        stream: async_tls::client::TlsStream<TcpStream>,
    },
    Listener {
        local_addr: SocketAddr,
        peer_addr: SocketAddr,
        stream: server::TlsStream<TcpStream>,
    },
}
