use super::consts::*;
use super::*;

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

    pub fn reply(&mut self, rpc_client: &mut RpcClient) -> Option<Result<(R::P, Option<Vec<u8>>)>> {
        match rpc_client.take_reply(self.chan_id) {
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
        rep_buf: Vec<u8>,
        opt_buf: Option<Vec<u8>>,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let mut header = RequestHeader {
            method_idx: R::method_idx(),
            chan_id: 0,
            opts: 0,
            body_len: 0,
            buf_len: 0,
        };
        let n = rpc_client.req(&mut header, &self.body, req_buf, rep_buf, opt_buf, &mut buf)?;
        self.chan_id = header.chan_id;
        Ok(n)
    }
}

enum State {
    RecvHeader,
    RecvBody(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
    RecvBuf(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
    Reply(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
}

enum ReplyState {
    Empty,
    Waiting(Vec<u8>, Option<Vec<u8>>),
    Receiving,
    Complete(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
}

impl ReplyState {
    fn take_waiting(&mut self) -> Option<(Vec<u8>, Option<Vec<u8>>)> {
        match self {
            ReplyState::Waiting(_, _) => (),
            _ => return None,
        }
        let mut reply = ReplyState::Receiving;
        swap(&mut reply, self);
        if let ReplyState::Waiting(rep_buf, opt_buf) = reply {
            return Some((rep_buf, opt_buf));
        }
        unreachable!();
    }
    fn take_complete(&mut self) -> Option<(ReplyHeader, Vec<u8>, Option<Vec<u8>>)> {
        match self {
            ReplyState::Complete(_, _, _) => (),
            _ => return None,
        }
        let mut reply = ReplyState::Empty;
        swap(&mut reply, self);
        if let ReplyState::Complete(header, rep_buf, opt_buf) = reply {
            return Some((header, rep_buf, opt_buf));
        }
        unreachable!();
    }
}

pub struct RpcClient {
    chan_id: u8,
    state: State,
    // pub replies: Vec<Option<(ReplyHeader, Vec<u8>, Option<Vec<u8>>)>>,
    replies: [ReplyState; 256],
}

// https://stackoverflow.com/a/36259524
macro_rules! array {
    (@accum (0, $($_es:expr),*) -> ($($body:tt)*))
        => {array!(@as_expr [$($body)*])};
    (@accum (1, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (0, $($es),*) -> ($($body)* $($es,)*))};
    (@accum (2, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (0, $($es),*) -> ($($body)* $($es,)* $($es,)*))};
    (@accum (3, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (2, $($es),*) -> ($($body)* $($es,)*))};
    (@accum (4, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (2, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (5, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es),*) -> ($($body)* $($es,)*))};
    (@accum (6, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es),*) -> ($($body)* $($es,)* $($es,)*))};
    (@accum (7, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es),*) -> ($($body)* $($es,)* $($es,)* $($es,)*))};
    (@accum (8, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (4, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (16, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (8, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (32, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (16, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (64, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (32, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (128, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (64, $($es,)* $($es),*) -> ($($body)*))};
    (@accum (256, $($es:expr),*) -> ($($body:tt)*))
        => {array!(@accum (128, $($es,)* $($es),*) -> ($($body)*))};

    (@as_expr $e:expr) => {$e};

    [$e:expr; $n:tt] => { array!(@accum ($n, $e) -> ()) };
}

impl RpcClient {
    pub fn new() -> Self {
        RpcClient {
            chan_id: 0,
            state: State::RecvHeader,
            replies: array![ReplyState::Empty; 256],
        }
    }

    pub fn req<S: Serialize>(
        &mut self,
        header: &mut RequestHeader,
        body: &S,
        req_buf: Option<&[u8]>,
        rep_buf: Vec<u8>,
        opt_buf: Option<Vec<u8>>,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let body_buf = postcard::to_slice(&body, &mut buf[REQ_HEADER_LEN..])?;
        header.body_len = body_buf.len() as u16;
        header.chan_id = self.chan_id;
        match self.replies[header.chan_id as usize] {
            ReplyState::Empty => (),
            // TODO: Handle this error properly
            ReplyState::Receiving => panic!("Reply at chan_id is at receiving state"),
            // TODO: Handle this error properly
            ReplyState::Waiting(_, _) => panic!("Reply at chan_id is at waiting state"),
            // TODO: Handle this error properly
            ReplyState::Complete(_, _, _) => panic!("Reply at chan_id is at complete state"),
        }
        self.replies[header.chan_id as usize] = ReplyState::Waiting(rep_buf, opt_buf);
        self.chan_id += 1;
        if let Some(req_buf) = req_buf {
            header.buf_len = req_buf.len() as u16;
            buf[REQ_HEADER_LEN + header.body_len as usize
                ..REQ_HEADER_LEN + header.body_len as usize + req_buf.len()]
                .copy_from_slice(&req_buf);
        }
        postcard::to_slice(&header, &mut buf)?;
        Ok(REQ_HEADER_LEN + header.body_len as usize + header.buf_len as usize)
    }

    pub fn parse(&mut self, rcv_buf: &[u8]) -> Result<(usize, Option<u8>)> {
        loop {
            let mut state = State::RecvHeader;
            swap(&mut state, &mut self.state);
            match state {
                State::RecvHeader => {
                    let rep_header = rep_header_from_bytes(&rcv_buf).unwrap();
                    match self.replies[rep_header.chan_id as usize].take_waiting() {
                        None => {
                            match self.replies[rep_header.chan_id as usize] {
                                // TODO: Handle this error properly
                                ReplyState::Empty => panic!("Unexpected reply at chan_id"),
                                // TODO: Handle this error properly
                                ReplyState::Complete(_, _, _) => {
                                    panic!("Reply at chan_id is at complete state")
                                }
                                // TODO: Handle this error properly
                                ReplyState::Receiving => {
                                    panic!("Reply at chan_id is at receiving state")
                                }
                                ReplyState::Waiting(_, _) => unreachable!(),
                            }
                        }
                        Some((rep_buf, opt_buf)) => {
                            let n = rep_header.body_len as usize;
                            self.state = State::RecvBody(rep_header, rep_buf, opt_buf);
                            return Ok((n, None));
                        }
                    }
                }
                State::RecvBody(rep_header, mut rep_buf, opt_buf) => {
                    let rep_header_buf_len = rep_header.buf_len;
                    rep_buf[..rcv_buf.len()].copy_from_slice(rcv_buf);
                    if rep_header_buf_len == 0 {
                        self.state = State::Reply(rep_header, rep_buf, opt_buf);
                    } else {
                        let n = rep_header_buf_len as usize;
                        self.state = State::RecvBuf(rep_header, rep_buf, opt_buf);
                        return Ok((n, None));
                    }
                }
                State::RecvBuf(rep_header, rep_buf, mut opt_buf) => {
                    match &mut opt_buf {
                        Some(buf) => {
                            buf[..rcv_buf.len()].copy_from_slice(rcv_buf);
                        }
                        None => {
                            // TODO: Handle this error properly
                            panic!("Optional buffer expected but none provided");
                        }
                    }
                    self.state = State::Reply(rep_header, rep_buf, opt_buf);
                }
                State::Reply(rep_header, rep_buf, opt_buf) => {
                    let chan_id = rep_header.chan_id;
                    self.replies[chan_id as usize] =
                        ReplyState::Complete(rep_header, rep_buf, opt_buf);
                    self.state = State::RecvHeader;
                    return Ok((REP_HEADER_LEN, Some(chan_id)));
                }
            }
        }
    }

    pub fn take_reply(&mut self, chan_id: u8) -> Option<(ReplyHeader, Vec<u8>, Option<Vec<u8>>)> {
        self.replies[chan_id as usize].take_complete()
    }
}
