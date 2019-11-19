use std::io;
use urpc;

use postcard::{from_bytes, to_slice};
// urpc::setup!(
//     methods: [
//         {id: 0, name: SendBytes, request: [u8]},
//         {id: 1, name: RecvBytes, request_type: ()},
//         {id: 2, name: Reset, request_type: ()},
//         {id: 3, name: Ping, request_type: [u8; 4]}
//     ],
//     errors: [
//         { InvalidMethod },
//         { InvalidBody },
//         { Busy }
//     ]
// );

//
// Server
//

// Auto
enum Error {
    InvalidMethod,
    InvalidBody,
    Busy,
}

// type PingRequestBody = [u8; 4];
// type SendBytesRequestBody<'a> = &'a [u8];

// Auto
// enum Request<'a> {
//     Ping(PingRequestBody),
//     SendBytes(SendBytesRequestBody<'a>),
// }
enum Request<'a> {
    Ping(RequestPing),
    SendBytes(RequestSendBytes<'a>),
}

struct RequestPing {
    header: urpc::RequestHeader,
    body: [u8; 4],
}

impl RequestPing {
    pub fn reply(self, payload: [u8; 4]) -> () {
        ()
    }
    pub fn reply_err(self, err: u8) -> () {}
}

struct RequestSendBytes<'a> {
    header: urpc::RequestHeader,
    body: &'a [u8],
}

impl<'a> RequestSendBytes<'a> {
    pub fn reply(self, payload: ()) -> () {
        ()
    }
    pub fn reply_err(self, err: u8) -> () {}
}

// Auto
enum ReplyBody {
    Ping([u8; 4]),
    SendBytes(()),
}

// Auto
enum Reply<T> {
    Ack,
    Error(Error),
    Body(T),
}

fn req_body_from_bytes<'a>(header: urpc::RequestHeader, buf: &'a [u8]) -> Request<'a> {
    match header.method_idx {
        0 => Request::Ping(RequestPing {
            header: header,
            body: from_bytes(buf).unwrap(),
        }),
        1 => Request::SendBytes(RequestSendBytes {
            header: header,
            body: from_bytes(buf).unwrap(),
        }),
        _ => {
            unreachable!();
        }
    }
}

//
// Client
//

struct RpcClient;

impl RpcClient {
    fn ping(&self, data: &[u8; 4]) -> Result<[u8; 4], io::Error> {
        let mut echo = [0; 4];
        echo.copy_from_slice(data);
        Ok(echo)
    }

    fn send_bytes(&self, bytes: &[u8]) -> Result<(), io::Error> {
        Ok(())
    }
}

// trait Rpc {
//     type Request;
//     type Reply;
//
//     fn
// }

fn main() -> () {
    let req_header_buf = [0; 5];
    let req_header = urpc::req_header_from_bytes(&req_header_buf);

    let req_body_buf = vec![0; req_header.length as usize];
    let req = req_body_from_bytes(req_header, &req_body_buf[..]);
    let reply = match req {
        Request::SendBytes(send_bytes) => send_bytes.reply(()),
        Request::Ping(ping) => {
            let body = ping.body.clone();
            ping.reply(body)
        }
    };
    let mut reply_buf = [0; 32];
    to_slice(&reply, &mut reply_buf);
}
