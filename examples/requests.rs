use std::io;
use urpc;

use postcard::from_bytes;
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
    Ping([u8; 4]),
    SendBytes(&'a [u8]),
}

// Auto
enum ReplyBody {
    Ping([u8; 4]),
    SendBytes(()),
}

// Auto
enum Reply {
    Ack,
    Error(Error),
    Body(ReplyBody),
}

fn req_body_from_bytes<'a>(header: &urpc::RequestHeader, buf: &'a [u8]) -> Request<'a> {
    match header.method_idx {
        0 => Request::Ping(from_bytes::<[u8; 4]>(buf).unwrap()),
        1 => Request::SendBytes(from_bytes::<&'a [u8]>(buf).unwrap()),
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
        Ok([0; 4]) // Example
    }

    fn send_bytes(&self, bytes: &[u8]) -> Result<(), io::Error> {
        Ok(())
    }
}

fn main() -> () {
    let req_header_buf = [0; 5];
    let req_header = urpc::req_header_from_bytes(&req_header_buf);

    let req_body_buf = vec![0; req_header.length as usize];
    let req_body = req_body_from_bytes(&req_header, &req_body_buf[..]);
    let reply = match req_body {
        Request::SendBytes(_body) => Reply::Ack,
        Request::Ping(body) => Reply::Body(ReplyBody::Ping(body)),
    };
}
