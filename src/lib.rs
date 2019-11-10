#![no_std]

use postcard::from_bytes;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Opts {
    pub sync: bool,
    pub once: bool,
    pub cancel: bool,
}

#[derive(Deserialize)]
pub struct RequestHeader {
    pub method_idx: u8,
    pub chan_id: u8,
    pub opts: Opts,
    pub length: u16,
}

// struct ReplyHeader {
//     chan_id: u8,
//     reply_type: ReplyType,
//     length: u16,
// }

// enum ReplyType {
//     Ack,
//     Error,
//     Data,
// }

#[macro_export]
macro_rules! setup {
    ( methods: [
      $( {
          name: $method_name:ident,
          request_type: $req_body_type:ty
      } ),* ],
      errors: [ $( $err_name:ident ),* ]) => {
        enum RequestBody {
            $(
                $method_name($req_body_type),
            )*
        }
        enum Error {
            $(
                $err_name,
            )*
        }
    };
}

pub fn req_header_from_bytes(buf: &[u8]) -> RequestHeader {
    from_bytes(buf).unwrap()
}

// enum Reply {
//     Ack,
//     Error(Error),
//     Data(ReplyData),
// }
//
// enum Request {
//     Body(RequestBody),
// }

// struct Method {
//     idx: u8,
//     body:
// }

// fn req_header_from_bytes(&[u8]) -> RequestHeader {
//     unimplemented!();
// }

// fn req_body_from_bytes(&[u8]) -> ReplyBody {
//
// }
//
// // Auto generated
//
// enum RequestBody {
//     SendBytes(BodySendBytes),
//     RecvBytes(BodyRecvBytes),
//     Reset(BodyReset),
//     Ping(BodyPing),
// }
//
// enum Error {
//     Example
// }
//
// enum ReplyBody {
//     None,
//
// }
//
// // User
//
// enum Method {
//     SendBytes,
//     RecvBytes,
//     Reset,
//     Ping,
//     // EndHandshake,
//     // Handshake,
//     // Status,
// }
//
// register_methods![
//     (SendBytes, [u8]),
//     (RecvBytes, ()),
//     (Reset, ()),
//     (Ping, [4; u8]),
// ]
//
// register_errors![
//     Busy,
//     InvalidBody,
// ]
//
// enum ReplyBody {
//
// }
//
// {
//     let req_header_buf = [0; 5];
//     let req_header = req_header_from_bytes(&req_header_buf);
//
//     let req_body_buf = [0; req_header.len()]:
//     let req_body = req_body_from_bytes(&req_header, req_header_body_buf);
//     match req_body {
//         RequestBody::SendBytes(body) => {},
//         RequestBody::RecvBytes(body) => {},
//         RequestBody::Reset(body) => {},
//         RequestBody::Ping(body) => {},
//     }
// }
