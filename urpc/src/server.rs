use super::consts::*;
use super::*;

use core::marker::PhantomData;
use core::mem::swap;

use postcard;
use serde::{de::DeserializeOwned, Serialize};

// TODO Use custom Error
pub type Result<T> = postcard::Result<T>;
pub type Error = postcard::Error;

/// Type used to handle a Request for a particular RPC Call.
#[derive(Debug)]
pub struct RequestType<Q: DeserializeOwned, QB: OptBuf, P: Serialize, PB: OptBuf> {
    chan_id: u8,
    pub body: Q,
    phantom: PhantomData<(QB, P, PB)>,
}

impl<Q: DeserializeOwned, P: Serialize, PB: OptBuf> RequestType<Q, OptBufNo, P, PB> {
    /// Deserialize the body of a Request.
    pub fn from_bytes(header: RequestHeader, buf: &[u8]) -> Result<Self> {
        Ok(Self {
            chan_id: header.chan_id,
            body: postcard::from_bytes(buf)?,
            phantom: PhantomData::<(OptBufNo, P, PB)>,
        })
    }
}

impl<Q: DeserializeOwned, P: Serialize, PB: OptBuf> RequestType<Q, OptBufYes, P, PB> {
    /// Deserialize the body of a Request.
    pub fn from_bytes<'a>(header: RequestHeader, buf: &'a [u8]) -> Result<(Self, &'a [u8])> {
        let buf_start = header.body_len as usize;
        Ok((
            Self {
                chan_id: header.chan_id,
                body: postcard::from_bytes(buf)?,
                phantom: PhantomData::<(OptBufYes, P, PB)>,
            },
            &buf[buf_start..buf_start + header.buf_len as usize],
        ))
    }
}

impl<Q: DeserializeOwned, QB: OptBuf, P: Serialize, PB: OptBuf> RequestType<Q, QB, P, PB> {
    /// Serialize a reply packet build from a payload.  Returns the number of bytes written to
    /// `reply_buf`.
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
    /// Serialize an error reply packet.  Returns the number of bytes written to `reply_buf`.
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

enum State {
    WaitHeader,
    WaitBody(RequestHeader),
    // RecvdBody(Result<R>, u16),
    // WaitBuf(Result<R>),
    // Request(Result<R>),
}

/// Result of parsing some bytes by the RPC Server.
pub enum ParseResult<R> {
    NeedBytes(usize),
    Request(Result<R>),
}

/// RPC Call request.
pub trait Request<'a>
where
    Self: Sized,
{
    // where Self: std::marker::Sized
    // type R;

    fn from_bytes(header: RequestHeader, buf: &'a [u8]) -> Result<Self>;
}

/// Main component of the RPC Server.  The server keeps the state of the parsed bytes and outputs
/// the requests once they are received.
pub struct RpcServer<'a, R: Request<'a>> {
    max_buf_len: u16,
    state: State,
    phantom: PhantomData<&'a R>,
}

impl<'a, R: Request<'a>> RpcServer<'a, R> {
    pub fn new(max_buf_len: u16) -> Self {
        Self {
            max_buf_len,
            state: State::WaitHeader,
            phantom: PhantomData::<&'a R>,
        }
    }

    /// Parse incoming bytes and return wether a request has been received, or more bytes are
    /// needed to build a complete request.
    pub fn parse(&mut self, rcv_buf: &'a [u8]) -> Result<ParseResult<R>> {
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
                    if req_header_body_len + req_header_buf_len == 0 {
                        let req = R::from_bytes(req_header, &[]);
                        self.state = State::WaitHeader;
                        return Ok(ParseResult::Request(req));
                    } else {
                        let ret = ParseResult::NeedBytes(
                            (req_header.body_len + req_header.buf_len) as usize,
                        );
                        self.state = State::WaitBody(req_header);
                        return Ok(ret);
                    }
                }
                State::WaitBody(req_header) => {
                    // let req_header_buf_len = req_header.buf_len;
                    let req = R::from_bytes(req_header, &rcv_buf[..]);
                    self.state = State::WaitHeader;
                    return Ok(ParseResult::Request(req));
                    // self.state = State::RecvdBody(req, req_header_buf_len)
                } // State::RecvdBody(req, req_header_buf_len) => {
                  //     if req_header_buf_len == 0 {
                  //         self.state = State::Request(req);
                  //     } else {
                  //         let ret = ParseResult::NeedBytes(req_header_buf_len as usize);
                  //         self.state = State::WaitBuf(req);
                  //         return Ok(ret);
                  //     }
                  // }
                  // State::WaitBuf(req) => {
                  //     opt_buf = Some(rcv_buf);
                  //     self.state = State::Request(req);
                  // }
                  // State::Request(req) => {
                  //     self.state = State::WaitHeader;
                  //     return Ok(ParseResult::Request(req, opt_buf));
                  // }
            }
        }
    }
}
