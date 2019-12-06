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
    ReplySlotEmpty,
    ReplySlotWaiting,
    ReplySlotComplete,
    ReplySlotReceiving,
    ReplySlotOptBufMissing,
    ReplyBodyTooLong,
    ReplyOptBufTooLong,
    ReplyOptBufUnexpected,
}

pub type Result<T> = core::result::Result<T, Error>;

impl convert::From<postcard::Error> for Error {
    fn from(error: postcard::Error) -> Self {
        Self::SerializeDeserialize(error)
    }
}

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
    pub fn take_reply(&mut self, rpc_client: &mut RpcClient) -> Option<Result<(R::P, Vec<u8>)>> {
        match rpc_client.take_reply(self.chan_id) {
            None => None,
            Some((rep_header, rep_body_buf, opt_buf)) => Some(
                postcard::from_bytes(&rep_body_buf)
                    .map(|r| (r, opt_buf.unwrap()))
                    .map_err(|e| e.into()),
            ),
        }
    }
}

impl<R: Request, QB: OptBuf> RequestType<R, QB, OptBufNo> {
    pub fn take_reply(&mut self, rpc_client: &mut RpcClient) -> Option<Result<R::P>> {
        match rpc_client.take_reply(self.chan_id) {
            None => None,
            Some((rep_header, rep_body_buf, _opt_buf)) => {
                Some(postcard::from_bytes(&rep_body_buf).map_err(|e| e.into()))
            }
        }
    }
}

enum State {
    WaitHeader,
    WaitBody(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
    RecvdBody(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
    WaitBuf(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
    Reply(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
}

enum ReplyState {
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
    reply_slots: Vec<ReplyState>,
}

impl RpcClient {
    pub fn new() -> Self {
        let reply_slots = (0..256).map(|_| ReplyState::Empty).collect();
        RpcClient {
            chan_id: 0,
            state: State::WaitHeader,
            reply_slots,
        }
    }

    // Serialize a request packet built from (header, body, req_body_buf) into buf.  Prepare a
    // reply slot with (rep_body_buf, rep_opt_buf).
    pub fn req<S: Serialize>(
        &mut self,
        header: &mut RequestHeader,
        body: &S,
        req_body_buf: Option<&[u8]>,
        rep_body_buf: Vec<u8>,
        rep_opt_buf: Option<Vec<u8>>,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let body_buf = postcard::to_slice(&body, &mut buf[REQ_HEADER_LEN..])?;
        header.body_len = body_buf.len() as u16;
        header.chan_id = self.chan_id;
        // Make sure that the reply slot for this channel is not busy.
        match self.reply_slots[header.chan_id as usize] {
            ReplyState::Empty => (),
            ReplyState::Receiving => return Err(Error::ReplySlotReceiving),
            ReplyState::Waiting { .. } => return Err(Error::ReplySlotWaiting),
            ReplyState::Complete { .. } => return Err(Error::ReplySlotComplete),
        }
        // Set the reply slot for this channel as waiting with the buffers to store the reply.
        self.reply_slots[header.chan_id as usize] = ReplyState::Waiting {
            rep_body_buf,
            opt_buf: rep_opt_buf,
        };
        // TODO: Use channels id's wisely
        self.chan_id += 1;
        // Serialize the request (with the optional buffer)
        if let Some(req_body_buf) = req_body_buf {
            header.buf_len = req_body_buf.len() as u16;
            buf[REQ_HEADER_LEN + header.body_len as usize
                ..REQ_HEADER_LEN + header.body_len as usize + req_body_buf.len()]
                .copy_from_slice(&req_body_buf);
        }
        postcard::to_slice(&header, &mut buf)?;
        Ok(REQ_HEADER_LEN + header.body_len as usize + header.buf_len as usize)
    }

    // Parse an received buffer in order to advance the deserialization of a reply.  Returns the
    // number of bytes needed to keep advancing, and optionally the channel number of the completed
    // deserialized reply.
    pub fn parse(&mut self, rcv_buf: &[u8]) -> Result<(usize, Option<u8>)> {
        loop {
            let mut state = State::WaitHeader;
            swap(&mut state, &mut self.state);
            match state {
                // Initial state: waiting for the header bytes
                State::WaitHeader => {
                    let rep_header = rep_header_from_bytes(&rcv_buf)?;
                    match self.reply_slots[rep_header.chan_id as usize].take_waiting() {
                        // Check that there's a valid reply slot for this reply.
                        None => match self.reply_slots[rep_header.chan_id as usize] {
                            ReplyState::Empty => return Err(Error::ReplySlotEmpty),
                            ReplyState::Complete { .. } => return Err(Error::ReplySlotComplete),
                            ReplyState::Receiving => return Err(Error::ReplySlotReceiving),
                            ReplyState::Waiting { .. } => unreachable!(),
                        },
                        Some((rep_body_buf, opt_buf)) => {
                            // Check that the body buffer will fit in the reply slot.
                            if rep_header.body_len as usize > rep_body_buf.len() {
                                return Err(Error::ReplyBodyTooLong);
                            }
                            // Check that the optional buffer length in the reply header is
                            // compatible with the reply slot that the requester stored.
                            match &opt_buf {
                                None => {
                                    if rep_header.buf_len > 0 {
                                        return Err(Error::ReplyOptBufUnexpected);
                                    }
                                }
                                Some(b) => {
                                    if rep_header.buf_len as usize > b.len() {
                                        return Err(Error::ReplyOptBufTooLong);
                                    }
                                }
                            }
                            let n = rep_header.body_len as usize;
                            if n == 0 {
                                self.state = State::RecvdBody(rep_header, rep_body_buf, opt_buf);
                            } else {
                                self.state = State::WaitBody(rep_header, rep_body_buf, opt_buf);
                                return Ok((n, None));
                            }
                        }
                    }
                }
                // Received body bytes
                State::WaitBody(rep_header, mut rep_body_buf, opt_buf) => {
                    let rep_header_buf_len = rep_header.buf_len;
                    rep_body_buf[..rcv_buf.len()].copy_from_slice(rcv_buf);
                    self.state = State::RecvdBody(rep_header, rep_body_buf, opt_buf);
                }
                State::RecvdBody(rep_header, mut rep_body_buf, opt_buf) => {
                    if rep_header.buf_len == 0 {
                        self.state = State::Reply(rep_header, rep_body_buf, opt_buf);
                    } else {
                        let n = rep_header.buf_len as usize;
                        self.state = State::WaitBuf(rep_header, rep_body_buf, opt_buf);
                        return Ok((n, None));
                    }
                }
                // Received optional buffer bytes
                State::WaitBuf(rep_header, rep_body_buf, mut opt_buf) => {
                    match &mut opt_buf {
                        Some(buf) => {
                            buf[..rcv_buf.len()].copy_from_slice(rcv_buf);
                        }
                        None => return Err(Error::ReplySlotOptBufMissing),
                    }
                    self.state = State::Reply(rep_header, rep_body_buf, opt_buf);
                }
                // Received all the bytes necessary to build the complete reply
                State::Reply(rep_header, rep_body_buf, opt_buf) => {
                    let chan_id = rep_header.chan_id;
                    self.reply_slots[chan_id as usize] = ReplyState::Complete {
                        rep_header,
                        rep_body_buf,
                        opt_buf,
                    };
                    self.state = State::WaitHeader;
                    return Ok((REP_HEADER_LEN, Some(chan_id)));
                }
            }
        }
    }

    // Take the reply of the slot in a channel id if it's complete.
    pub fn take_reply(&mut self, chan_id: u8) -> Option<(ReplyHeader, Vec<u8>, Option<Vec<u8>>)> {
        self.reply_slots[chan_id as usize].take_complete()
    }
}
