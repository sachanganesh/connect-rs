//! TCP transport client and listener implementations.
//!
//! <br/>
//!
//! This module primarily exposes the TCP client implementation over a [`Connection`] type and the
//! TCP listener implementation as [`TcpListener`].

#[allow(unused_imports)]
pub(crate) use crate::Connection;

pub(crate) mod client;
pub(crate) mod listener;

pub use client::*;
pub use listener::*;
