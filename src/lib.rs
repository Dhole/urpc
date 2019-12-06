#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod macros;

#[cfg(feature = "std")]
pub mod client;
pub mod consts;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Opts {
    pub sync: bool,
    pub once: bool,
    pub cancel: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestHeader {
    pub method_idx: u8,
    pub chan_id: u8,
    pub opts: u8,
    pub body_len: u16,
    pub buf_len: u16,
}

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

pub fn req_header_from_bytes(buf: &[u8]) -> Result<RequestHeader, postcard::Error> {
    from_bytes(buf)
}

pub fn rep_header_from_bytes(buf: &[u8]) -> Result<ReplyHeader, postcard::Error> {
    from_bytes(buf)
}
