use urpc::{
    self,
    server::{self, RequestType},
    OptBufNo, OptBufYes,
};
use urpc_derive::urpc_request;

#[urpc_request]
enum ServerRequests<'a> {
    Ping(RequestType<[u8; 4], OptBufNo, [u8; 4], OptBufNo>),
    SendBytes(RequestType<(), OptBufNo, (), OptBufNo>),
    RecvBytes((RequestType<(), OptBufYes, (), OptBufNo>, &'a [u8])),
}

impl<'a> server::Request<'a> for ServerRequests<'a> {
    // type R = Self;

    fn from_bytes(header: urpc::RequestHeader, buf: &'a [u8]) -> server::Result<Self> {
        Ok(match header.method_idx {
            0 => ServerRequests::Ping(RequestType::<_, OptBufNo, _, _>::from_bytes(header, buf)?),
            1 => ServerRequests::SendBytes(RequestType::<_, OptBufNo, _, _>::from_bytes(
                header, buf,
            )?),
            2 => ServerRequests::RecvBytes(RequestType::<_, OptBufYes, _, _>::from_bytes(
                header, buf,
            )?),
            _ => {
                return Err(server::Error::WontImplement);
            }
        })
    }
}

fn main() {}
