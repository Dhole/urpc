use urpc::{
    self,
    server::{self, RequestType},
};
use urpc_derive::urpc_request;

#[urpc_request]
enum ServerRequests {
    Ping(RequestType<[u8; 4], [u8; 4]>),
    SendBytes(RequestType<(), ()>),
    RecvBytes(RequestType<(), ()>),
}

impl server::Request for ServerRequests {
    type R = Self;

    fn from_bytes(header: urpc::RequestHeader, buf: &[u8]) -> server::Result<Self> {
        Ok(match header.method_idx {
            0 => ServerRequests::Ping(RequestType::from_bytes(header, buf)?),
            1 => ServerRequests::SendBytes(RequestType::from_bytes(header, buf)?),
            2 => ServerRequests::RecvBytes(RequestType::from_bytes(header, buf)?),
            _ => {
                return Err(server::Error::WontImplement);
            }
        })
    }
}

fn main() {}
