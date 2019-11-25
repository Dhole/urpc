use core::mem::swap;
use hex;
use std::io;
use urpc;

use serde::de::DeserializeOwned;
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
// #[derive(Debug)]
// enum RequestBody {
//     Ping([u8; 4]),
//     SendBytes(()),
// }

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
// enum ReplyBody {
//     Ping([u8; 4]),
//     SendBytes(()),
// }

// Auto
// enum Reply<T> {
//     Ack,
//     Error(Error),
//     Body(T),
// }

// TODO: Macro this
// fn req_to_bytes(
//     chan_id: u8,
//     req: &mut ClientRequest,
//     buf: &mut [u8],
// ) -> postcard::Result<urpc::RequestHeader> {
//     let (method_idx, body_buf) = match req {
//         ClientRequest::Ping(req) => {
//             req.chan_id = chan_id;
//             (
//                 0,
//                 postcard::to_slice(&req.body, &mut buf[REQ_HEADER_LEN..])?,
//             )
//         }
//         ClientRequest::SendBytes(req) => {
//             req.chan_id = chan_id;
//             (
//                 1,
//                 postcard::to_slice(&req.body, &mut buf[REQ_HEADER_LEN..])?,
//             )
//         }
//     };
//     Ok(urpc::RequestHeader {
//         method_idx,
//         chan_id,
//         opts: 0,
//         body_len: body_buf.len() as u16,
//         buf_len: 0,
//     })
// }

mod consts {
    pub const REQ_HEADER_LEN: usize = 7;
    pub const REP_HEADER_LEN: usize = 6;
}

mod client {
    use super::consts::*;

    use core::mem::swap;

    use postcard;
    use serde::{de::DeserializeOwned, Serialize};

    pub trait Request {
        type Q: Serialize;
        type P: DeserializeOwned;

        fn method_idx() -> u8;
    }

    #[derive(Debug)]
    pub struct RequestType<R: Request> {
        chan_id: u8,
        body: R::Q,
    }

    impl<R: Request> RequestType<R> {
        pub fn new(req: R::Q) -> Self {
            Self {
                chan_id: 0,
                body: req,
            }
        }

        pub fn reply(
            &mut self,
            rpc_client: &mut RpcClient,
        ) -> Option<postcard::Result<(R::P, Option<Vec<u8>>)>> {
            match rpc_client.replies[self.chan_id as usize].take() {
                None => None,
                Some((rep_header, rep_body_buf, opt_buf)) => {
                    match postcard::from_bytes(&rep_body_buf) {
                        Ok(r) => Some(Ok((r, opt_buf))),
                        Err(e) => Some(Err(e)),
                    }
                }
            }
        }

        pub fn request(
            &mut self,
            req_buf: Option<&[u8]>,
            rpc_client: &mut RpcClient,
            mut buf: &mut [u8],
        ) -> postcard::Result<()> {
            let mut header = urpc::RequestHeader {
                method_idx: R::method_idx(),
                chan_id: 0,
                opts: 0,
                body_len: 0,
                buf_len: 0,
            };
            rpc_client.req(&mut header, &self.body, req_buf, &mut buf)?;
            self.chan_id = header.chan_id;
            Ok(())
        }
    }

    enum State {
        RecvHeader,
        RecvBody(urpc::ReplyHeader),
        RecvBuf(urpc::ReplyHeader, Vec<u8>),
        Reply(urpc::ReplyHeader, Vec<u8>, Option<Vec<u8>>),
    }

    pub struct RpcClient {
        chan_id: u8,
        state: State,
        pub replies: Vec<Option<(urpc::ReplyHeader, Vec<u8>, Option<Vec<u8>>)>>,
        // req_to_bytes: fn(chan_id: u8, req: R, buf: &mut [u8]) -> postcard::Result<urpc::RequestHeader>,
    }

    impl RpcClient {
        pub fn new(// req_to_bytes: fn(
        //    chan_id: u8,
        //    req: R,
        //     buf: &mut [u8],
        //) -> postcard::Result<urpc::RequestHeader>,
        ) -> Self {
            RpcClient {
                chan_id: 0,
                state: State::RecvHeader,
                replies: vec![None; 256],
                // req_to_bytes,
            }
        }

        pub fn req<S: Serialize>(
            &mut self,
            header: &mut urpc::RequestHeader,
            body: &S,
            req_buf: Option<&[u8]>,
            mut buf: &mut [u8],
        ) -> postcard::Result<()> {
            // let mut header = (self.req_to_bytes)(self.chan_id, req, &mut buf[REQ_HEADER_LEN..])?;
            let body_buf = postcard::to_slice(&body, &mut buf[REQ_HEADER_LEN..])?;
            header.body_len = body_buf.len() as u16;
            header.chan_id = self.chan_id;
            self.chan_id += 1;
            //let body_buf_len = body_buf.len();
            if let Some(req_buf) = req_buf {
                header.buf_len = req_buf.len() as u16;
                buf[REQ_HEADER_LEN + header.body_len as usize
                    ..REQ_HEADER_LEN + header.body_len as usize + req_buf.len()]
                    .copy_from_slice(&req_buf);
            }
            println!("client header: {:?}", header);
            postcard::to_slice(&header, &mut buf)?;
            Ok(())
        }

        pub fn parse(&mut self, rcv_buf: &[u8]) -> postcard::Result<usize> {
            let mut opt_buf: Option<Vec<u8>> = None;
            loop {
                let mut state = State::RecvHeader;
                swap(&mut state, &mut self.state);
                match state {
                    State::RecvHeader => {
                        let rep_header = urpc::rep_header_from_bytes(&rcv_buf).unwrap();
                        let n = rep_header.body_len as usize;
                        self.state = State::RecvBody(rep_header);
                        return Ok(n);
                    }
                    State::RecvBody(rep_header) => {
                        let rep_header_buf_len = rep_header.buf_len;
                        let rep_buf = Vec::from(rcv_buf);
                        if rep_header_buf_len == 0 {
                            self.state = State::Reply(rep_header, rep_buf, None);
                        } else {
                            let n = rep_header_buf_len as usize;
                            self.state = State::RecvBuf(rep_header, rep_buf);
                            return Ok(n);
                        }
                    }
                    State::RecvBuf(rep_header, rep_buf) => {
                        opt_buf = Some(Vec::from(rcv_buf));
                        self.state = State::Reply(rep_header, rep_buf, opt_buf);
                    }
                    State::Reply(rep_header, rep_buf, opt_buf) => {
                        let chan_id = rep_header.chan_id;
                        self.replies[chan_id as usize] = Some((rep_header, rep_buf, opt_buf));
                        self.state = State::RecvHeader;
                        return Ok(REP_HEADER_LEN);
                    }
                }
            }
        }
    }
}

//
// Client
//

// struct RpcClient;

// impl RpcClient {
//     fn ping(&self, data: &[u8; 4]) -> Result<[u8; 4], io::Error> {
//         let mut echo = [0; 4];
//         echo.copy_from_slice(data);
//         Ok(echo)
//     }
//
//     fn send_bytes(&self, bytes: &[u8]) -> Result<(), io::Error> {
//         Ok(())
//     }
// }

// trait Rpc {
//     type Request;
//     type Reply;
//
//     fn
// }

mod server {
    use super::consts::*;

    use core::marker::PhantomData;
    use core::mem::swap;

    use postcard;
    use serde::{de::DeserializeOwned, Serialize};

    #[derive(Debug)]
    pub struct RequestType<Q: DeserializeOwned, P: Serialize> {
        // header: urpc::RequestHeader,
        chan_id: u8,
        pub body: Q,
        phantom: PhantomData<P>,
    }

    impl<Q: DeserializeOwned, P: Serialize> RequestType<Q, P> {
        pub fn from_bytes(header: urpc::RequestHeader, buf: &[u8]) -> postcard::Result<Self> {
            Ok(Self {
                chan_id: header.chan_id,
                body: postcard::from_bytes(buf)?,
                phantom: PhantomData::<P>,
            })
        }
        pub fn reply(self, payload: P, mut reply_buf: &mut [u8]) -> postcard::Result<()> {
            let body_buf = postcard::to_slice(&payload, &mut reply_buf[REP_HEADER_LEN..])?;
            let header = urpc::ReplyHeader {
                chan_id: self.chan_id,
                opts: 0,
                body_len: body_buf.len() as u16,
                buf_len: 0,
            };
            postcard::to_slice(&header, &mut reply_buf)?;
            Ok(())
        }
        pub fn reply_err(self, err: u8) -> () {}
    }

    enum State<R> {
        RecvHeader,
        RecvBody(urpc::RequestHeader),
        RecvBuf(R),
        Request(R),
    }

    pub enum ParseResult<'a, R> {
        NeedBytes(usize),
        Request(R, Option<&'a [u8]>),
    }

    pub struct RpcServer<R> {
        state: State<R>,
        req_from_bytes: fn(header: urpc::RequestHeader, buf: &[u8]) -> R,
    }

    impl<R> RpcServer<R> {
        pub fn new(req_from_bytes: fn(header: urpc::RequestHeader, buf: &[u8]) -> R) -> Self {
            Self {
                state: State::RecvHeader,
                req_from_bytes,
            }
        }

        pub fn parse<'a>(&mut self, rcv_buf: &'a [u8]) -> ParseResult<'a, R> {
            let mut opt_buf: Option<&'a [u8]> = None;
            loop {
                let mut state = State::RecvHeader;
                swap(&mut state, &mut self.state);
                match state {
                    State::RecvHeader => {
                        let req_header = urpc::req_header_from_bytes(&rcv_buf).unwrap();
                        let ret = ParseResult::NeedBytes(req_header.body_len as usize);
                        self.state = State::RecvBody(req_header);
                        return ret;
                    }
                    State::RecvBody(req_header) => {
                        let req_header_buf_len = req_header.buf_len;
                        let req = (self.req_from_bytes)(req_header, &rcv_buf[..]);
                        if req_header_buf_len == 0 {
                            self.state = State::Request(req);
                        } else {
                            let ret = ParseResult::NeedBytes(req_header_buf_len as usize);
                            self.state = State::RecvBuf(req);
                            return ret;
                        }
                    }
                    State::RecvBuf(req) => {
                        opt_buf = Some(rcv_buf);
                        self.state = State::Request(req);
                    }
                    State::Request(req) => {
                        self.state = State::RecvHeader;
                        return ParseResult::Request(req, opt_buf);
                    }
                }
            }
        }
    }
}

// pub trait ServerRequest1 {
//     type Q: DeserializeOwned;
//     type P: Serialize;
//
//     fn method_idx() -> u8;
// }

//
// Client
//

struct ClientRequestPing;
impl client::Request for ClientRequestPing {
    type Q = [u8; 4];
    type P = [u8; 4];

    fn method_idx() -> u8 {
        0
    }
}

//
// Server
//

// TODO: Macro this
#[derive(Debug)]
enum ServerRequest {
    Ping(server::RequestType<[u8; 4], [u8; 4]>),
    SendBytes(server::RequestType<(), ()>),
}

// TODO: Macro this
fn req_from_bytes(header: urpc::RequestHeader, buf: &[u8]) -> postcard::Result<ServerRequest> {
    Ok(match header.method_idx {
        0 => ServerRequest::Ping(server::RequestType::from_bytes(header, buf)?),
        1 => ServerRequest::SendBytes(server::RequestType::from_bytes(header, buf)?),
        _ => {
            return Err(postcard::Error::WontImplement);
        }
    })
}

fn main() -> () {
    let mut read_buf = vec![0; 32];
    let mut write_buf = vec![0; 32];

    let mut rpc_client = client::RpcClient::new();
    let req_buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut req = client::RequestType::<ClientRequestPing>::new([0, 1, 2, 3]);
    req.request(Some(&req_buf), &mut rpc_client, &mut read_buf);
    // rpc_client
    //     .req(
    //         &mut ClientRequest::SendBytes(req),
    //         Some(&req_buf),
    //         &mut read_buf,
    //     )
    //     .unwrap();
    println!("{}, {}", read_buf.len(), hex::encode(&read_buf));

    let mut rpc_server = server::RpcServer::new(req_from_bytes);
    let mut pos = 0;
    let mut read_len = consts::REQ_HEADER_LEN;
    loop {
        let buf = &read_buf[pos..pos + read_len];
        println!("pos: {}, buf: {}", pos, hex::encode(buf));
        pos += read_len;
        match rpc_server.parse(&buf) {
            server::ParseResult::NeedBytes(n) => {
                read_len = n;
            }
            server::ParseResult::Request(req, opt_buf) => {
                read_len = consts::REQ_HEADER_LEN;
                println!("request: {:?}, {:?}", req, opt_buf);
                match req.unwrap() {
                    ServerRequest::Ping(ping) => {
                        let ping_body = ping.body;
                        ping.reply(ping_body, &mut write_buf).unwrap();
                    }
                    ServerRequest::SendBytes(send_bytes) => {
                        println!("send_bytes: {}", hex::encode(opt_buf.unwrap()));
                        send_bytes.reply((), &mut write_buf).unwrap();
                    }
                }
                break;
            }
        }
    }
    println!("{}, {}", write_buf.len(), hex::encode(&write_buf));

    let mut pos = 0;
    let mut read_len = consts::REP_HEADER_LEN;
    loop {
        let buf = &write_buf[pos..pos + read_len];
        println!("pos: {}, buf: {}", pos, hex::encode(buf));
        pos += read_len;
        read_len = rpc_client.parse(&buf).unwrap();
        match req.reply(&mut rpc_client) {
            Some(r) => {
                println!("reply: {:?}", r.unwrap());
                break;
            }
            None => {}
        }
    }
}
