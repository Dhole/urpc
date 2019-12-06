use super::consts::*;
use super::*;

use core::marker::PhantomData;
use core::mem::swap;

use postcard;
use serde::{de::DeserializeOwned, Serialize};

// TODO Use custom Error
pub type Result<T> = postcard::Result<T>;
pub type Error = postcard::Error;

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
    pub fn reply(self, payload: P, mut reply_buf: &mut [u8]) -> Result<usize> {
        let body_buf = postcard::to_slice(&payload, &mut reply_buf[REP_HEADER_LEN..])?;
        let header = ReplyHeader {
            chan_id: self.chan_id,
            opts: 0,
            body_len: body_buf.len() as u16,
            buf_len: 0,
        };
        postcard::to_slice(&header, &mut reply_buf)?;
        Ok(REP_HEADER_LEN + header.body_len as usize + header.buf_len as usize)
    }
    pub fn reply_err(self, err: u8, mut reply_buf: &mut [u8]) -> Result<usize> {
        let header = ReplyHeader {
            chan_id: self.chan_id,
            opts: 1,
            body_len: 0,
            buf_len: 0,
        };
        postcard::to_slice(&header, &mut reply_buf)?;
        Ok(REP_HEADER_LEN)
    }
}

enum State<R> {
    WaitHeader,
    WaitBody(RequestHeader),
    RecvdBody(Result<R>, u16),
    WaitBuf(Result<R>),
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
    max_buf_len: u16,
    state: State<R::R>,
}

impl<R: Request> RpcServer<R> {
    pub fn new(max_buf_len: u16) -> Self {
        Self {
            max_buf_len,
            state: State::WaitHeader,
        }
    }

    pub fn parse<'a>(&mut self, rcv_buf: &'a [u8]) -> Result<ParseResult<'a, R::R>> {
        let mut opt_buf: Option<&'a [u8]> = None;
        loop {
            let mut state = State::WaitHeader;
            swap(&mut state, &mut self.state);
            match state {
                State::WaitHeader => {
                    let req_header = req_header_from_bytes(&rcv_buf)?;
                    if req_header.body_len >= self.max_buf_len {
                        // TODO: Make custom error
                        return Err(postcard::Error::WontImplement);
                    }
                    if req_header.buf_len >= self.max_buf_len {
                        // TODO: Make custom error
                        return Err(postcard::Error::WontImplement);
                    }
                    let req_header_body_len = req_header.body_len;
                    let req_header_buf_len = req_header.buf_len;
                    if req_header_body_len == 0 {
                        let req = R::from_bytes(req_header, &[]);
                        self.state = State::RecvdBody(req, req_header_buf_len);
                    } else {
                        let ret = ParseResult::NeedBytes(req_header.body_len as usize);
                        self.state = State::WaitBody(req_header);
                        return Ok(ret);
                    }
                }
                State::WaitBody(req_header) => {
                    let req_header_buf_len = req_header.buf_len;
                    let req = R::from_bytes(req_header, &rcv_buf[..]);
                    self.state = State::RecvdBody(req, req_header_buf_len)
                }
                State::RecvdBody(req, req_header_buf_len) => {
                    if req_header_buf_len == 0 {
                        self.state = State::Request(req);
                    } else {
                        let ret = ParseResult::NeedBytes(req_header_buf_len as usize);
                        self.state = State::WaitBuf(req);
                        return Ok(ret);
                    }
                }
                State::WaitBuf(req) => {
                    opt_buf = Some(rcv_buf);
                    self.state = State::Request(req);
                }
                State::Request(req) => {
                    self.state = State::WaitHeader;
                    return Ok(ParseResult::Request(req, opt_buf));
                }
            }
        }
    }
}
