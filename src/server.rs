use super::consts::*;
use super::*;

use core::marker::PhantomData;
use core::mem::swap;

use postcard;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct RequestType<Q: DeserializeOwned, P: Serialize> {
    chan_id: u8,
    pub body: Q,
    phantom: PhantomData<P>,
}

impl<Q: DeserializeOwned, P: Serialize> RequestType<Q, P> {
    pub fn from_bytes(header: RequestHeader, buf: &[u8]) -> Result<Self> {
        Ok(Self {
            chan_id: header.chan_id,
            body: postcard::from_bytes(buf)?,
            phantom: PhantomData::<P>,
        })
    }
    pub fn reply(self, payload: P, mut reply_buf: &mut [u8]) -> Result<()> {
        let body_buf = postcard::to_slice(&payload, &mut reply_buf[REP_HEADER_LEN..])?;
        let header = ReplyHeader {
            chan_id: self.chan_id,
            opts: 0,
            body_len: body_buf.len() as u16,
            buf_len: 0,
        };
        postcard::to_slice(&header, &mut reply_buf)?;
        Ok(())
    }
    pub fn reply_err(self, err: u8) -> () {
        unimplemented!();
    }
}

enum State<R> {
    RecvHeader,
    RecvBody(RequestHeader),
    RecvBuf(Result<R>),
    Request(Result<R>),
}

pub enum ParseResult<'a, R> {
    NeedBytes(usize),
    Request(Result<R>, Option<&'a [u8]>),
}

pub trait Request {
    type R;

    fn from_bytes(header: RequestHeader, buf: &[u8]) -> Result<Self::R>;
}

pub struct RpcServer<R: Request> {
    state: State<R::R>,
}

impl<R: Request> RpcServer<R> {
    pub fn new() -> Self {
        Self {
            state: State::RecvHeader,
        }
    }

    pub fn parse<'a>(&mut self, rcv_buf: &'a [u8]) -> ParseResult<'a, R::R> {
        let mut opt_buf: Option<&'a [u8]> = None;
        loop {
            let mut state = State::RecvHeader;
            swap(&mut state, &mut self.state);
            match state {
                State::RecvHeader => {
                    let req_header = req_header_from_bytes(&rcv_buf).unwrap();
                    let ret = ParseResult::NeedBytes(req_header.body_len as usize);
                    self.state = State::RecvBody(req_header);
                    return ret;
                }
                State::RecvBody(req_header) => {
                    let req_header_buf_len = req_header.buf_len;
                    let req = R::from_bytes(req_header, &rcv_buf[..]);
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
