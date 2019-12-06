use super::consts::*;
use super::*;

use core::marker::PhantomData;
use core::mem::swap;

use postcard;
use serde::{de::DeserializeOwned, Serialize};

pub trait Request {
    type Q: Serialize;
    type P: DeserializeOwned;
    const METHOD_ID: u8;
}

// enum RequestState {}

#[derive(Debug)]
pub struct RequestType<R: Request, QB: OptBuf, PB: OptBuf> {
    chan_id: u8,
    body: R::Q,
    phantom: PhantomData<(QB, PB)>,
    // state: RequestState,
}

pub trait OptBuf {}

pub struct OptBufYes {}
impl OptBuf for OptBufYes {}
pub struct OptBufNo {}
impl OptBuf for OptBufNo {}

impl<R: Request, QB: OptBuf, PB: OptBuf> RequestType<R, QB, PB> {
    pub fn new(req: R::Q) -> Self {
        Self {
            chan_id: 0,
            body: req,
            phantom: PhantomData::<(QB, PB)>,
        }
    }
}

impl<R: Request> RequestType<R, OptBufNo, OptBufNo> {
    pub fn request(
        &mut self,
        rpc_client: &mut RpcClient,
        rep_body_buf: Vec<u8>,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let mut header = RequestHeader {
            method_idx: R::METHOD_ID,
            chan_id: 0,
            opts: 0,
            body_len: 0,
            buf_len: 0,
        };
        let n = rpc_client.req(&mut header, &self.body, None, rep_body_buf, None, &mut buf)?;
        self.chan_id = header.chan_id;
        Ok(n)
    }
}

impl<R: Request> RequestType<R, OptBufNo, OptBufYes> {
    pub fn request(
        &mut self,
        rpc_client: &mut RpcClient,
        rep_body_buf: Vec<u8>,
        rep_opt_buf: Vec<u8>,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let mut header = RequestHeader {
            method_idx: R::METHOD_ID,
            chan_id: 0,
            opts: 0,
            body_len: 0,
            buf_len: 0,
        };
        let n = rpc_client.req(
            &mut header,
            &self.body,
            None,
            rep_body_buf,
            Some(rep_opt_buf),
            &mut buf,
        )?;
        self.chan_id = header.chan_id;
        Ok(n)
    }
}

impl<R: Request> RequestType<R, OptBufYes, OptBufNo> {
    pub fn request(
        &mut self,
        req_body_buf: &[u8],
        rpc_client: &mut RpcClient,
        rep_body_buf: Vec<u8>,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let mut header = RequestHeader {
            method_idx: R::METHOD_ID,
            chan_id: 0,
            opts: 0,
            body_len: 0,
            buf_len: 0,
        };
        let n = rpc_client.req(
            &mut header,
            &self.body,
            Some(req_body_buf),
            rep_body_buf,
            None,
            &mut buf,
        )?;
        self.chan_id = header.chan_id;
        Ok(n)
    }
}

impl<R: Request> RequestType<R, OptBufYes, OptBufYes> {
    pub fn request(
        &mut self,
        req_body_buf: &[u8],
        rpc_client: &mut RpcClient,
        rep_body_buf: Vec<u8>,
        rep_opt_buf: Vec<u8>,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let mut header = RequestHeader {
            method_idx: R::METHOD_ID,
            chan_id: 0,
            opts: 0,
            body_len: 0,
            buf_len: 0,
        };
        let n = rpc_client.req(
            &mut header,
            &self.body,
            Some(req_body_buf),
            rep_body_buf,
            Some(rep_opt_buf),
            &mut buf,
        )?;
        self.chan_id = header.chan_id;
        Ok(n)
    }
}

impl<R: Request, QB: OptBuf> RequestType<R, QB, OptBufYes> {
    pub fn reply(&mut self, rpc_client: &mut RpcClient) -> Option<Result<(R::P, Vec<u8>)>> {
        match rpc_client.take_reply(self.chan_id) {
            None => None,
            Some((rep_header, rep_body_buf, opt_buf)) => {
                match postcard::from_bytes(&rep_body_buf) {
                    Ok(r) => Some(Ok((r, opt_buf.unwrap()))),
                    Err(e) => Some(Err(e)),
                }
            }
        }
    }
}

impl<R: Request, QB: OptBuf> RequestType<R, QB, OptBufNo> {
    pub fn reply(&mut self, rpc_client: &mut RpcClient) -> Option<Result<R::P>> {
        match rpc_client.take_reply(self.chan_id) {
            None => None,
            Some((rep_header, rep_body_buf, _opt_buf)) => {
                match postcard::from_bytes(&rep_body_buf) {
                    Ok(r) => Some(Ok(r)),
                    Err(e) => Some(Err(e)),
                }
            }
        }
    }
}

enum State {
    RecvHeader,
    RecvBody(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
    RecvBuf(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
    Reply(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
}

enum ReplyState {
    // TODO: Use field name in enum like { rep_body_buf: Vec<u8>, ... }
    Empty,
    Waiting {
        rep_body_buf: Vec<u8>,
        opt_buf: Option<Vec<u8>>,
    },
    Receiving,
    Complete {
        rep_header: ReplyHeader,
        rep_body_buf: Vec<u8>,
        opt_buf: Option<Vec<u8>>,
    },
}

impl ReplyState {
    fn take_waiting(&mut self) -> Option<(Vec<u8>, Option<Vec<u8>>)> {
        match self {
            ReplyState::Waiting { .. } => (),
            _ => return None,
        }
        let mut reply = ReplyState::Receiving;
        swap(&mut reply, self);
        if let ReplyState::Waiting {
            rep_body_buf,
            opt_buf,
        } = reply
        {
            return Some((rep_body_buf, opt_buf));
        }
        unreachable!();
    }
    fn take_complete(&mut self) -> Option<(ReplyHeader, Vec<u8>, Option<Vec<u8>>)> {
        match self {
            ReplyState::Complete { .. } => (),
            _ => return None,
        }
        let mut reply = ReplyState::Empty;
        swap(&mut reply, self);
        if let ReplyState::Complete {
            rep_header,
            rep_body_buf,
            opt_buf,
        } = reply
        {
            return Some((rep_header, rep_body_buf, opt_buf));
        }
        unreachable!();
    }
}

pub struct RpcClient {
    chan_id: u8,
    state: State,
    replies: Vec<ReplyState>,
}

impl RpcClient {
    pub fn new() -> Self {
        let replies = (0..256).map(|_| ReplyState::Empty).collect();
        RpcClient {
            chan_id: 0,
            state: State::RecvHeader,
            replies,
        }
    }

    pub fn req<S: Serialize>(
        &mut self,
        header: &mut RequestHeader,
        body: &S,
        req_body_buf: Option<&[u8]>,
        rep_body_buf: Vec<u8>,
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
            ReplyState::Waiting { .. } => panic!("Reply at chan_id is at waiting state"),
            // TODO: Handle this error properly
            ReplyState::Complete { .. } => panic!("Reply at chan_id is at complete state"),
        }
        self.replies[header.chan_id as usize] = ReplyState::Waiting {
            rep_body_buf,
            opt_buf,
        };
        self.chan_id += 1;
        if let Some(req_body_buf) = req_body_buf {
            header.buf_len = req_body_buf.len() as u16;
            buf[REQ_HEADER_LEN + header.body_len as usize
                ..REQ_HEADER_LEN + header.body_len as usize + req_body_buf.len()]
                .copy_from_slice(&req_body_buf);
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
                                ReplyState::Complete { .. } => {
                                    panic!("Reply at chan_id is at complete state")
                                }
                                // TODO: Handle this error properly
                                ReplyState::Receiving => {
                                    panic!("Reply at chan_id is at receiving state")
                                }
                                ReplyState::Waiting { .. } => unreachable!(),
                            }
                        }
                        Some((rep_body_buf, opt_buf)) => {
                            let n = rep_header.body_len as usize;
                            self.state = State::RecvBody(rep_header, rep_body_buf, opt_buf);
                            return Ok((n, None));
                        }
                    }
                }
                State::RecvBody(rep_header, mut rep_body_buf, opt_buf) => {
                    let rep_header_buf_len = rep_header.buf_len;
                    rep_body_buf[..rcv_buf.len()].copy_from_slice(rcv_buf);
                    if rep_header_buf_len == 0 {
                        self.state = State::Reply(rep_header, rep_body_buf, opt_buf);
                    } else {
                        let n = rep_header_buf_len as usize;
                        self.state = State::RecvBuf(rep_header, rep_body_buf, opt_buf);
                        return Ok((n, None));
                    }
                }
                State::RecvBuf(rep_header, rep_body_buf, mut opt_buf) => {
                    match &mut opt_buf {
                        Some(buf) => {
                            buf[..rcv_buf.len()].copy_from_slice(rcv_buf);
                        }
                        None => {
                            // TODO: Handle this error properly
                            panic!("Optional buffer expected but none provided");
                        }
                    }
                    self.state = State::Reply(rep_header, rep_body_buf, opt_buf);
                }
                State::Reply(rep_header, rep_body_buf, opt_buf) => {
                    let chan_id = rep_header.chan_id;
                    self.replies[chan_id as usize] = ReplyState::Complete {
                        rep_header,
                        rep_body_buf,
                        opt_buf,
                    };
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
