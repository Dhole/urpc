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
    ) -> Result<usize> {
        let mut header = RequestHeader {
            method_idx: R::method_idx(),
            chan_id: 0,
            opts: 0,
            body_len: 0,
            buf_len: 0,
        };
        let n = rpc_client.req(&mut header, &self.body, req_buf, &mut buf)?;
        self.chan_id = header.chan_id;
        Ok(n)
    }
}

enum State {
    RecvHeader,
    RecvBody(ReplyHeader),
    RecvBuf(ReplyHeader, Vec<u8>),
    Reply(ReplyHeader, Vec<u8>, Option<Vec<u8>>),
}

pub struct RpcClient {
    chan_id: u8,
    state: State,
    pub replies: Vec<Option<(ReplyHeader, Vec<u8>, Option<Vec<u8>>)>>,
}

impl RpcClient {
    pub fn new() -> Self {
        RpcClient {
            chan_id: 0,
            state: State::RecvHeader,
            replies: vec![None; 256],
        }
    }

    pub fn req<S: Serialize>(
        &mut self,
        header: &mut RequestHeader,
        body: &S,
        req_buf: Option<&[u8]>,
        mut buf: &mut [u8],
    ) -> Result<usize> {
        let body_buf = postcard::to_slice(&body, &mut buf[REQ_HEADER_LEN..])?;
        header.body_len = body_buf.len() as u16;
        header.chan_id = self.chan_id;
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
        let mut opt_buf: Option<Vec<u8>> = None;
        loop {
            let mut state = State::RecvHeader;
            swap(&mut state, &mut self.state);
            match state {
                State::RecvHeader => {
                    let rep_header = rep_header_from_bytes(&rcv_buf).unwrap();
                    let n = rep_header.body_len as usize;
                    self.state = State::RecvBody(rep_header);
                    return Ok((n, None));
                }
                State::RecvBody(rep_header) => {
                    let rep_header_buf_len = rep_header.buf_len;
                    let rep_buf = Vec::from(rcv_buf);
                    if rep_header_buf_len == 0 {
                        self.state = State::Reply(rep_header, rep_buf, None);
                    } else {
                        let n = rep_header_buf_len as usize;
                        self.state = State::RecvBuf(rep_header, rep_buf);
                        return Ok((n, None));
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
                    return Ok((REP_HEADER_LEN, Some(chan_id)));
                }
            }
        }
    }
}
