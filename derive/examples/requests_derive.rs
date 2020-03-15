use urpc::{
    self,
    server::{Request, RequestType},
    OptBufNo, OptBufYes,
};
// use urpc_derive::Request;

#[derive(Debug, Request)]
enum ServerRequests<'a> {
    Ping(RequestType<[u8; 4], OptBufNo, [u8; 4], OptBufNo>),
    SendBytes(RequestType<u32, OptBufYes, u32, OptBufNo>, &'a [u8]),
    RecvBytes(RequestType<u32, OptBufNo, u32, OptBufYes>),
}

// {
//     0: Ping ([u8;4]) -> ([u8; 4]),
//     1: SendBytes (u32, &'a [u8]) -> (u32),
//     2: RecvBytes (u32) -> (u32, &'a [u8]),
// }

// impl<'a> server::Request<'a> for ServerRequests<'a> {
//     // type R = Self;
//
//     fn from_bytes(header: urpc::RequestHeader, buf: &'a [u8]) -> server::Result<Self> {
//         Ok(match header.method_idx {
//             0 => ServerRequests::Ping(RequestType::<_, OptBufNo, _, _>::from_bytes(header, buf)?),
//             1 => ServerRequests::SendBytes(RequestType::<_, OptBufNo, _, _>::from_bytes(
//                 header, buf,
//             )?),
//             2 => ServerRequests::RecvBytes(RequestType::<_, OptBufYes, _, _>::from_bytes(
//                 header, buf,
//             )?),
//             _ => {
//                 return Err(server::Error::WontImplement);
//             }
//         })
//     }
// }

fn main() {
    // let _x = ServerRequests::Ping(RequestType<[u8; 4], OptBufNo, [u8; 4], OptBufNo>);
}
