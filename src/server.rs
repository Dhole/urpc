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
        if header.buf_len() > 0 {
            return Err(postcard::Error::WontImplement);
        }
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
        let buf_start = header.body_len();
        Ok((
            Self {
                chan_id: header.chan_id,
                body: postcard::from_bytes(buf)?,
                phantom: PhantomData::<(OptBufYes, P, PB)>,
            },
            &buf[buf_start..buf_start + header.buf_len()],
        ))
    }
}

impl<Q: DeserializeOwned, QB: OptBuf, P: Serialize> RequestType<Q, QB, P, OptBufNo> {
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
        Ok(REP_HEADER_LEN + header.body_len() + header.buf_len())
    }
}

impl<Q: DeserializeOwned, QB: OptBuf, P: Serialize> RequestType<Q, QB, P, OptBufYes> {
    pub fn get_opt_buf<'a>(&self, reply_buf: &'a mut [u8]) -> &'a mut [u8] {
        &mut reply_buf[REP_HEADER_LEN..]
    }

    /// Serialize a reply packet build from a payload.  Returns the number of bytes written to
    /// `reply_buf`.
    pub fn reply(self, payload: P, opt_buf_len: u16, mut reply_buf: &mut [u8]) -> Result<usize> {
        let body_buf = postcard::to_slice(
            &payload,
            &mut reply_buf[REP_HEADER_LEN + opt_buf_len as usize..],
        )?;
        let header = ReplyHeader {
            chan_id: self.chan_id,
            opts: 0,
            body_len: body_buf.len() as u16,
            buf_len: opt_buf_len,
        };
        postcard::to_slice(&header, &mut reply_buf)?;
        Ok(REP_HEADER_LEN + header.body_len() + header.buf_len())
    }
}

impl<Q: DeserializeOwned, QB: OptBuf, P: Serialize, PB: OptBuf> RequestType<Q, QB, P, PB> {
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
pub enum ParseResult<T> {
    NeedBytes(usize),
    Request(T),
}

/// RPC Call request.
pub trait Request<'a>
where
    Self: Sized + 'a,
{
    // where Self: std::marker::Sized
    // type R;

    fn from_bytes(header: RequestHeader, buf: &'a [u8]) -> Result<Self>;

    fn from_rpc(rpc_server: &mut RpcServer, rcv_buf: &'a [u8]) -> Result<ParseResult<Self>> {
        match rpc_server.parse(&rcv_buf)? {
            ParseResult::NeedBytes(n) => Ok(ParseResult::NeedBytes(n)),
            ParseResult::Request((header, body_buf)) => {
                Ok(ParseResult::Request(Self::from_bytes(header, body_buf)?))
            }
        }
    }
}

/// Main component of the RPC Server.  The server keeps the state of the parsed bytes and outputs
/// the requests once they are received.
pub struct RpcServer {
    max_buf_len: u16,
    state: State,
}

impl RpcServer {
    pub fn new(max_buf_len: u16) -> Self {
        Self {
            max_buf_len,
            state: State::WaitHeader,
        }
    }

    /// Parse incoming bytes and return wether a request has been received, or more bytes are
    /// needed to build a complete request.
    pub fn parse<'a>(
        &mut self,
        rcv_buf: &'a [u8],
    ) -> Result<ParseResult<(RequestHeader, &'a [u8])>> {
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
                        // let req = R::from_bytes(req_header, &[]);
                        self.state = State::WaitHeader;
                        return Ok(ParseResult::Request((req_header, &[])));
                    } else {
                        let ret =
                            ParseResult::NeedBytes(req_header.body_len() + req_header.buf_len());
                        self.state = State::WaitBody(req_header);
                        return Ok(ret);
                    }
                }
                State::WaitBody(req_header) => {
                    // let req = R::from_bytes(req_header, &rcv_buf[..]);
                    self.state = State::WaitHeader;
                    return Ok(ParseResult::Request((req_header, &rcv_buf[..])));
                }
            }
        }
    }
}
