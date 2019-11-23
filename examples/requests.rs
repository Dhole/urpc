use core::mem::swap;
use hex;
use std::io;
use urpc;

use serde::{Deserialize, Serialize};

// use heapless::{consts::*, Vec};
use postcard;
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
#[derive(Debug)]
enum RequestBody {
    Ping([u8; 4]),
    SendBytes(()),
}

#[derive(Debug)]
enum Request {
    Ping(RequestType<[u8; 4]>),
    SendBytes(RequestType<()>),
}

// #[derive(Debug)]
// struct RequestPing {
//     header: urpc::RequestHeader,
//     body: [u8; 4],
// }
//
// impl RequestPing {
//     pub fn reply(self, payload: [u8; 4], mut reply_buf: &mut [u8]) {
//         let body_buf = postcard::to_slice(&payload, &mut reply_buf[REP_HEADER_LEN..]).unwrap();
//         let header = urpc::ReplyHeader {
//             chan_id: self.header.chan_id,
//             opts: 0,
//             body_len: body_buf.len() as u16,
//             buf_len: 0,
//         };
//         let header_buf = postcard::to_slice(&header, &mut reply_buf).unwrap();
//     }
//     pub fn reply_err(self, err: u8) -> () {}
// }

#[derive(Debug)]
struct RequestType<T: Serialize> {
    header: urpc::RequestHeader,
    body: T,
}

impl<T: Serialize> RequestType<T> {
    pub fn reply(self, payload: T, mut reply_buf: &mut [u8]) -> postcard::Result<()> {
        let body_buf = postcard::to_slice(&payload, &mut reply_buf[REP_HEADER_LEN..])?;
        let header = urpc::ReplyHeader {
            chan_id: self.header.chan_id,
            opts: 0,
            body_len: body_buf.len() as u16,
            buf_len: 0,
        };
        postcard::to_slice(&header, &mut reply_buf)?;
        Ok(())
    }
    pub fn reply_err(self, err: u8) -> () {}
}

// enum RequestWithBufState {
//     ExpectBuf,
//     ReceivedBuf,
// }

// #[derive(Debug)]
// struct RequestSendBytes {
//     header: urpc::RequestHeader,
//     body: (),
// }
//
// impl RequestSendBytes {
//     pub fn reply(self, payload: ()) -> () {
//         ()
//     }
//     pub fn reply_err(self, err: u8) -> () {}
// }

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

fn req_body_from_bytes(header: &urpc::RequestHeader, buf: &[u8]) -> RequestBody {
    match header.method_idx {
        0 => RequestBody::Ping(postcard::from_bytes(buf).unwrap()),
        1 => RequestBody::SendBytes(postcard::from_bytes(buf).unwrap()),
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

enum State {
    RecvHeader,
    RecvBody(urpc::RequestHeader),
    RecvBuf(urpc::RequestHeader, RequestBody),
    Request(urpc::RequestHeader, RequestBody),
}

enum RpcServerParseResult<'a> {
    NeedBytes(usize),
    Request(Request, Option<&'a [u8]>),
}

struct RpcServer {
    state: State,
}

const REQ_HEADER_LEN: usize = 7;
const REP_HEADER_LEN: usize = 6;

impl RpcServer {
    pub const fn header_bytes() -> usize {
        REQ_HEADER_LEN
    }
    pub fn new() -> Self {
        Self {
            state: State::RecvHeader,
        }
    }

    pub fn parse<'a>(&mut self, rcv_buf: &'a [u8]) -> RpcServerParseResult<'a> {
        let mut opt_buf: Option<&'a [u8]> = None;
        loop {
            let mut state = State::RecvHeader;
            swap(&mut state, &mut self.state);
            match state {
                State::RecvHeader => {
                    let req_header = urpc::req_header_from_bytes(&rcv_buf).unwrap();
                    let ret = RpcServerParseResult::NeedBytes(req_header.body_len as usize);
                    self.state = State::RecvBody(req_header);
                    return ret;
                }
                State::RecvBody(req_header) => {
                    let req_body = req_body_from_bytes(&req_header, &rcv_buf[..]);
                    if req_header.buf_len == 0 {
                        self.state = State::Request(req_header, req_body);
                    } else {
                        let ret = RpcServerParseResult::NeedBytes(req_header.buf_len as usize);
                        self.state = State::RecvBuf(req_header, req_body);
                        return ret;
                    }
                }
                State::RecvBuf(req_header, req_body) => {
                    opt_buf = Some(rcv_buf);
                    self.state = State::Request(req_header, req_body);
                }
                State::Request(req_header, req_body) => {
                    let request = match req_body {
                        RequestBody::Ping(body) => Request::Ping(RequestType::<[u8; 4]> {
                            header: req_header,
                            body,
                        }),
                        RequestBody::SendBytes(body) => Request::SendBytes(RequestType::<()> {
                            header: req_header,
                            body,
                        }),
                    };
                    self.state = State::RecvHeader;
                    return RpcServerParseResult::Request(request, opt_buf);
                }
            }
        }
    }
}

fn main() -> () {
    let mut read_buf = vec![0; 32];
    let mut write_buf = vec![0; 32];

    // {
    //     let mut buf = &mut read_buf;
    //     let body: [u8; 4] = [0x41, 0x42, 0x43, 0x44];
    //     // println!("buf: {}", hex::encode(&buf));
    //     let body_buf = postcard::to_slice(&body, &mut buf[RpcServer::header_bytes()..]).unwrap();
    //     let header = urpc::RequestHeader {
    //         method_idx: 0,
    //         chan_id: 5,
    //         opts: 0,
    //         body_len: body_buf.len() as u16,
    //         buf_len: 0,
    //     };
    //     // println!("buf: {}", hex::encode(&buf));
    //     postcard::to_slice(&header, &mut buf).unwrap();
    //     // println!("buf: {}", hex::encode(&buf));
    //     // println!("header len: {}", header_buf.len());
    // }
    {
        let mut buf = &mut read_buf;
        // println!("buf: {}", hex::encode(&buf));
        let body_buf = postcard::to_slice(&(), &mut buf[RpcServer::header_bytes()..]).unwrap();
        let body_buf_len = body_buf.len();
        let req_buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        buf[REQ_HEADER_LEN + body_buf_len..REQ_HEADER_LEN + body_buf_len + req_buf.len()]
            .copy_from_slice(&req_buf);
        let header = urpc::RequestHeader {
            method_idx: 1,
            chan_id: 5,
            opts: 0,
            body_len: body_buf_len as u16,
            buf_len: req_buf.len() as u16,
        };
        println!("header: {:?}", header);
        // println!("buf: {}", hex::encode(&buf));
        postcard::to_slice(&header, &mut buf).unwrap();
        // println!("buf: {}", hex::encode(&buf));
        // println!("header len: {}", header_buf.len());
    }
    println!("{}, {}", read_buf.len(), hex::encode(&read_buf));

    let mut rpc_server = RpcServer::new();
    let mut pos = 0;
    let mut read_len = RpcServer::header_bytes();
    loop {
        let buf = &read_buf[pos..pos + read_len];
        println!("pos: {}, buf: {}", pos, hex::encode(buf));
        pos += read_len;
        match rpc_server.parse(&buf) {
            RpcServerParseResult::NeedBytes(n) => {
                read_len = n;
            }
            RpcServerParseResult::Request(req, opt_buf) => {
                read_len = RpcServer::header_bytes();
                println!("request: {:?}, {:?}", req, opt_buf);
                match req {
                    Request::Ping(ping) => {
                        let ping_body = ping.body;
                        ping.reply(ping_body, &mut write_buf).unwrap();
                    }
                    Request::SendBytes(send_bytes) => {
                        println!("send_bytes: {}", hex::encode(opt_buf.unwrap()));
                        send_bytes.reply((), &mut write_buf).unwrap();
                    }
                }
                break;
            }
        }
    }
    println!("{}, {}", write_buf.len(), hex::encode(&write_buf));
}
