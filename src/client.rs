use super::consts::*;
use super::*;

use core;
use core::convert;
use core::marker::PhantomData;
use core::mem::swap;

use postcard;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub enum Error {
    SerializeDeserialize(postcard::Error),
    ReceivedBufTooShort,
    ReplyBodyTooLong,
    ReplyOptBufTooLong,
    ReplyOptBufUnexpected,
    NotIdle,
    NotExpectingBytes,
    TODO,
}

pub type Result<T> = core::result::Result<T, Error>;

impl convert::From<postcard::Error> for Error {
    fn from(error: postcard::Error) -> Self {
        Self::SerializeDeserialize(error)
    }
}

pub trait MethodId {
    const METHOD_ID: u8;
}

/// Type used to build a Request for a particular RPC Call.
#[derive(Debug)]
pub struct RequestType<M: MethodId, Q: Serialize, QB: OptBuf, P: DeserializeOwned, PB: OptBuf> {
    chan_id: u8,
    body: Q,
    phantom: PhantomData<(M, QB, P, PB)>,
}

impl<M: MethodId, Q: Serialize, QB: OptBuf, P: DeserializeOwned, PB: OptBuf>
    RequestType<M, Q, QB, P, PB>
{
    pub fn new(req: Q) -> Self {
        Self {
            chan_id: 0,
            body: req,
            phantom: PhantomData::<(M, QB, P, PB)>,
        }
    }

    pub fn chan_id(&self) -> u8 {
        self.chan_id
    }
}

impl<M: MethodId, Q: Serialize, P: DeserializeOwned, PB: OptBuf>
    RequestType<M, Q, OptBufNo, P, PB>
{
    /// Build a request and serialize it into buf.
    pub fn request(&mut self, rpc_client: &mut RpcClient, mut buf: &mut [u8]) -> Result<usize> {
        let mut header = RequestHeader {
            method_idx: M::METHOD_ID,
            chan_id: 0,
            opts: 0,
            body_len: 0,
            buf_len: 0,
        };
        let n = rpc_client.req(&mut header, &self.body, None, PB::opt_buf(), &mut buf)?;
        self.chan_id = header.chan_id;
        Ok(n)
    }
}

impl<M: MethodId, Q: Serialize, P: DeserializeOwned, PB: OptBuf>
    RequestType<M, Q, OptBufYes, P, PB>
{
    /// Build a request and serialize it into buf.
    pub fn request(
        &mut self,
        req_body_buf: &[u8],
        rpc_client: &mut RpcClient,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let mut header = RequestHeader {
            method_idx: M::METHOD_ID,
            chan_id: 0,
            opts: 0,
            body_len: 0,
            buf_len: 0,
        };
        let n = rpc_client.req(
            &mut header,
            &self.body,
            Some(req_body_buf),
            PB::opt_buf(),
            &mut buf,
        )?;
        self.chan_id = header.chan_id;
        Ok(n)
    }
}

impl<M: MethodId, Q: Serialize, P: DeserializeOwned, QB: OptBuf>
    RequestType<M, Q, QB, P, OptBufYes>
{
    /// Try to take the reply for this request from the RPC Client.  If no such reply exists,
    /// returns None.
    pub fn take_reply<'a>(
        &mut self,
        rpc_client: &'a mut RpcClient,
    ) -> Option<Result<(P, &'a [u8])>> {
        match rpc_client.take_reply(self.chan_id) {
            None => None,
            Some((_rep_header, rep_body_buf, opt_buf)) => Some(
                postcard::from_bytes(rep_body_buf)
                    .map(|r| (r, opt_buf))
                    .map_err(|e| e.into()),
            ),
        }
    }
}

impl<M: MethodId, Q: Serialize, P: DeserializeOwned, QB: OptBuf>
    RequestType<M, Q, QB, P, OptBufNo>
{
    /// Try to take the reply for this request from the RPC Client.  If no such reply exists,
    /// returns None.
    pub fn take_reply(&mut self, rpc_client: &mut RpcClient) -> Option<Result<P>> {
        match rpc_client.take_reply(self.chan_id) {
            None => None,
            Some((_rep_header, rep_body_buf, _opt_buf)) => {
                Some(postcard::from_bytes(&rep_body_buf).map_err(|e| e.into()))
            }
        }
    }
}

#[derive(Debug)]
enum State {
    Idle,
    WaitHeader { chan_id: u8, opt_buf: bool },
    WaitBody { header: ReplyHeader },
    WaitTakeReply { header: ReplyHeader },
}

/// Main component of the RPC Client.  The client keeps the state of the parsed bytes and stores
/// replies that requests can retreive later.
pub struct RpcClient {
    chan_id: u8,
    state: State,
    buf: Vec<u8>,
}

impl RpcClient {
    /// Create a new RPC Client.
    pub fn new(max_buf_len: u16) -> Self {
        RpcClient {
            chan_id: 1, // Use 1 to avoid a successful parse of a zeroed buffer.
            state: State::Idle,
            buf: vec![0; max_buf_len as usize],
        }
    }

    /// Serialize a request packet built from (`header`, `body`, `req_body_buf`) into `buf`.
    /// Prepare a reply slot with (`rep_body_buf`, `rep_opt_buf`).  Returns the number of bytes
    /// written to `buf`.
    pub fn req<S: Serialize>(
        &mut self,
        header: &mut RequestHeader,
        body: &S,
        req_body_buf: Option<&[u8]>,
        rep_opt_buf: bool,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        match self.state {
            State::Idle => {}
            _ => return Err(Error::NotIdle),
        }
        let body_buf = postcard::to_slice(&body, &mut buf[REQ_HEADER_LEN..])?;
        header.body_len = body_buf.len() as u16;
        header.chan_id = self.chan_id;
        self.state = State::WaitHeader {
            chan_id: header.chan_id,
            opt_buf: rep_opt_buf,
        };
        // Serialize the request (with the optional buffer)
        if let Some(req_body_buf) = req_body_buf {
            header.buf_len = req_body_buf.len() as u16;
            buf[REQ_HEADER_LEN + header.body_len()
                ..REQ_HEADER_LEN + header.body_len() + req_body_buf.len()]
                .copy_from_slice(&req_body_buf);
        }
        postcard::to_slice(&header, &mut buf)?;
        Ok(REQ_HEADER_LEN + header.body_len() + header.buf_len())
    }

    /// Parse an received buffer in order to advance the deserialization of a reply.  Returns the
    /// number of bytes needed to keep advancing, and optionally the channel number of the completed
    /// deserialized reply.
    pub fn parse(&mut self, rcv_buf: &[u8]) -> Result<(usize, Option<u8>)> {
        let rcv_buf = rcv_buf;
        loop {
            let mut state = State::Idle;
            swap(&mut state, &mut self.state);
            match state {
                // Initial state: waiting for the header bytes
                State::WaitHeader { chan_id, opt_buf } => {
                    let rep_header = rep_header_from_bytes(&rcv_buf)?;
                    if rep_header.chan_id != chan_id {
                        return Err(Error::TODO);
                    }
                    // Check that the body buffer will fit in the reply slot.
                    if rep_header.body_len() > self.buf.len() {
                        return Err(Error::ReplyBodyTooLong);
                    }
                    if !opt_buf && rep_header.buf_len != 0 {
                        return Err(Error::ReplyOptBufUnexpected);
                    }
                    let n = rep_header.body_len() + rep_header.buf_len();
                    if opt_buf && n > self.buf.len() {
                        return Err(Error::ReplyOptBufTooLong);
                    }
                    self.state = State::WaitBody { header: rep_header };
                    if n != 0 {
                        return Ok((n, None));
                    }
                }
                // Received body bytes
                State::WaitBody { header: rep_header } => {
                    let n = rep_header.body_len() + rep_header.buf_len();
                    if n > rcv_buf.len() {
                        return Err(Error::ReceivedBufTooShort);
                    }
                    self.buf[..n].copy_from_slice(&rcv_buf[..n]);
                    let chan_id = rep_header.chan_id;
                    self.state = State::WaitTakeReply { header: rep_header };
                    return Ok((REP_HEADER_LEN, Some(chan_id)));
                }
                _ => return Err(Error::NotExpectingBytes),
            }
        }
    }

    /// Take the reply of the slot in a channel id if it's complete.
    pub fn take_reply<'a>(&'a mut self, chan_id: u8) -> Option<(ReplyHeader, &'a [u8], &'a [u8])> {
        let mut state = State::Idle;
        swap(&mut state, &mut self.state);
        match state {
            State::WaitTakeReply { header: rep_header } if rep_header.chan_id == chan_id => {
                let body_len = rep_header.body_len as usize;
                let buf_len = rep_header.buf_len as usize;
                return Some((
                    rep_header,
                    &self.buf[buf_len..buf_len + body_len],
                    &self.buf[..buf_len],
                ));
            }
            _ => {} // TODO: Error
        }
        self.state = state;
        None
    }
}

use std::io;

pub struct RpcClientIO<S: io::Read + io::Write> {
    pub client: RpcClient,
    stream: S,
    pub stream_buf: Vec<u8>,
    pub buf_len: usize,
    // pub body_buf: Option<Vec<u8>>,
    // pub opt_buf: Option<Vec<u8>>,
}

#[derive(Debug)]
pub enum RpcClientIOError {
    Io(io::Error),
    Urpc(Error),
}

impl From<io::Error> for RpcClientIOError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<Error> for RpcClientIOError {
    fn from(err: Error) -> Self {
        Self::Urpc(err)
    }
}

impl<S: io::Read + io::Write> RpcClientIO<S> {
    pub fn new(stream: S, buf_len: usize) -> Self {
        Self {
            client: RpcClient::new(buf_len as u16),
            stream: stream,
            stream_buf: vec![0; buf_len],
            buf_len: buf_len,
            // body_buf: Some(vec![0; buf_len]),
            // opt_buf: Some(vec![0; buf_len]),
        }
    }

    pub fn request(
        &mut self,
        chan_id: u8,
        write_len: usize,
    ) -> core::result::Result<(), RpcClientIOError> {
        self.stream.write_all(&self.stream_buf[..write_len])?;
        self.stream.flush()?;

        let mut read_len = consts::REP_HEADER_LEN;
        loop {
            let mut buf = &mut self.stream_buf[..read_len];
            self.stream.read_exact(&mut buf)?;
            read_len = match self.client.parse(&buf)? {
                (n, None) => n,
                (n, Some(_chan_id)) => {
                    if _chan_id == chan_id {
                        return Ok(());
                    }
                    n
                }
            }
        }
    }
}
