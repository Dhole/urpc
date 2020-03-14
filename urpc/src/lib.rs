#![cfg_attr(not(feature = "std"), no_std)]

//! uRPC (pronounced micro RPC) is a simple and lightweight RPC framework designed with embedded
//! systems in mind.  The server side works in a heapless environment with `no_std` and is
//! supposed to use very low resources.  The current client side implementation requires `std`.
//!
//! # Features
//!
//! - ✓ Support for 256 different methods.
//! - ✓ Low level primitives for the server side regarding request parsing and
//!   reply serializing.
//! - ✓ Single argument of any `sedre::Serialize + serde::DeserializeOwned` type.
//! - ✓ Optional byte buffer for the argument that doesn't involve any buffer copy.
//!     - This feature is designed to optimize the transfer of bytes between client
//!       and server minimizing the amount of used memory in the server.
//! - ✓ Single return value of any `sedre::Serialize + serde::DeserializeOwned` type.
//! - ✓ Optional byte buffer for the reply ~~that doesn't involve any buffer copy~~.
//!     - This feature is designed to optimize the transfer of bytes between client
//!       and server ~~minimizing the amount of used memory in the client~~.
//! - ✗ Methods can return custom errors.
//! - ✗ Asyncrhonous methods.
//!     - ✗ Support for holding 255 async uncompleted requests.
//! - ✗ Stream methods.
//!
//! # Packet format
//!
//! - The request packet consists of a 7 byte header, an optional body and an
//!   optional byte buffer.
//! - The reply packet consists of a 6 byte header, an optional body and an
//!   optional byte buffer.
//!
//! # Header Format
//!
//! ## Request
//!
//! length | desc
//! -------|-----
//! 8b | method index
//! 8b | channel id
//! 8b | options
//! 16b | body length (little endian)
//! 16b | optional buffer length (little endian)
//!
//! ## Reply
//!
//! length | desc
//! -------|-----
//! 8b | channel id
//! 8b | options
//! 16b | body length (little endian)
//! 16b | optional buffer length (little endian)
//!
//! # Usage
//!
//! The best way to use this library is by using the macros `server_requets` and `client_requests`.
//! You can see complete examples in the documentation: [server_requests](macro.server_requests.html),
//! [client_requests](macro.client_requests.html).

#[macro_use]
mod macros;

#[cfg(feature = "std")]
/// Client side implementation
pub mod client;

/// Constant parameters
pub mod consts;

/// Server side implementation
pub mod server;

use postcard::from_bytes;
use serde::{Deserialize, Serialize};

// Auto
// enum Error {
//     InvalidMethod,
//     InvalidBody,
//     Busy,
// }

// pub type Result<T> = postcard::Result<T>;
// pub type Error = postcard::Error;

/// Options of a request/reply packet
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Opts {
    pub sync: bool,
    pub once: bool,
    pub cancel: bool,
}

/// Header of a request packet
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestHeader {
    pub method_idx: u8,
    pub chan_id: u8,
    pub opts: u8,
    pub body_len: u16,
    pub buf_len: u16,
}

/// Header of a reply packet
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReplyHeader {
    pub chan_id: u8,
    pub opts: u8,
    pub body_len: u16,
    pub buf_len: u16,
}

// enum ReplyType {
//     Ack,
//     Error,
//     Data,
// }

fn req_header_from_bytes(buf: &[u8]) -> Result<RequestHeader, postcard::Error> {
    from_bytes(buf)
}

fn rep_header_from_bytes(buf: &[u8]) -> Result<ReplyHeader, postcard::Error> {
    from_bytes(buf)
}

/// Trait used to allow building RPC calls with optional buffer.
pub trait OptBuf {}

/// Indicate that the RPC Call contains an optional buffer.
#[derive(Debug)]
pub struct OptBufYes {}
impl OptBuf for OptBufYes {}

/// Indicate that the RPC Call doesn't contain an optional buffer.
#[derive(Debug)]
pub struct OptBufNo {}
impl OptBuf for OptBufNo {}
