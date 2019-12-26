/*
#[macro_export(local_inner_macros)]
macro_rules! client_request_export {
    ($request_mod:ident, $method:ident, None, None) => {
        type $method = $crate::client::RequestType<
            $request_mod::$method,
            $crate::client::OptBufNo,
            $crate::client::OptBufNo,
        >;
    };
    ($request_mod:ident, $method:ident, None, Some<Vec<u8>>) => {
        type $method = $crate::client::RequestType<
            $request_mod::$method,
            $crate::client::OptBufNo,
            $crate::client::OptBufYes,
        >;
    };
    ($request_mod:ident, $method:ident, Some<Vec<u8>>, None) => {
        type $method = $crate::client::RequestType<
            $request_mod::$method,
            $crate::client::OptBufNo,
            $crate::client::OptBufYes,
        >;
    };
}
*/

/// Macro that builds the required types to make calls via RPC from the client.
///
/// # Examples
///
/// ```
/// use urpc::{client_requests, client, consts, client::{OptBufNo, OptBufYes}};
///
/// client_requests! {
///     client_requests;
///     (0, ClientRequestPing([u8; 4], OptBufNo, [u8; 4], OptBufNo)),
///     (1, ClientRequestSendBytes((), OptBufYes, (), OptBufNo))
/// }
///
/// let mut rpc_client = client::RpcClient::new();
/// let mut send_buf = vec![0; 32];
/// let mut recv_buf = vec![0; 32];
///
/// let mut req1 = ClientRequestPing::new([0, 1, 2, 3]);
/// let send_buf_bytes = req1.request(&mut rpc_client, vec![0; 32], &mut send_buf).unwrap();
/// println!("request bytes: {:02x?}", &send_buf[..send_buf_bytes]);
///
/// // Send send_buf over the network
/// // [...]
///
/// // Read from the network into recv_buf
/// // [...]
/// // We fill recv_buf with some precalculated replies to simulate a server reply
/// recv_buf[..6].copy_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
///
/// // Parse read bytes with rpc_client and try to match replies from each request
/// let mut pos = 0;
/// let mut read_len = consts::REP_HEADER_LEN;
/// loop {
///     let buf = &recv_buf[pos..pos + read_len];
///     pos += read_len;
///     read_len = rpc_client.parse(&buf).unwrap().0;
///     if let Some(r) = req1.take_reply(&mut rpc_client) {
///             println!("reply ping: {:?}", r.unwrap());
///             break;
///     }
/// }
///
/// let req_buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
/// let mut req2 = ClientRequestSendBytes::new(());
/// let send_buf_bytes = req2.request(&req_buf, &mut rpc_client, vec![0; 32], &mut send_buf).unwrap();
/// println!("request bytes: {:02x?}", &send_buf[..send_buf_bytes]);
///
/// // Send send_buf over the network
/// // [...]
///
/// // Read from the network into recv_buf
/// // [...]
/// // We fill recv_buf with some precalculated replies to simulate a server reply
/// recv_buf[..10].copy_from_slice(&[0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03]);
///
/// // Parse read bytes with rpc_client and try to match replies from each request
/// let mut pos = 0;
/// let mut read_len = consts::REP_HEADER_LEN;
/// loop {
///     let buf = &recv_buf[pos..pos + read_len];
///     pos += read_len;
///     read_len = rpc_client.parse(&buf).unwrap().0;
///     if let Some(r) = req2.take_reply(&mut rpc_client) {
///             println!("reply send_bytes: {:?}", r.unwrap());
///             break;
///     }
/// }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! client_requests {
    ($request_mod:ident;
        $( ($id:expr, $method:ident ( $req_type:ty, $req_opt_buf:ident, $rep_type:ty, $rep_opt_buf:ident)) ),*) => {
            mod $request_mod {
            $(
                pub struct $method;
                impl $crate::client::Request for $method {
                    type Q = $req_type;
                    type P = $rep_type;
                    const METHOD_ID: u8 = $id;
                }
            )*
            }
            $(
                // client_request_export!($request_mod, $method, $req_opt_buf, $rep_opt_buf);
                type $method = $crate::client::RequestType<
                $request_mod::$method,
                $req_opt_buf,
                $rep_opt_buf,
                >;
            )*
    };
}

/// Macro that builds the required types to handle calls via RPC from the server.
///
/// Examples
///
/// ```
/// use urpc::{server_requests, server, consts};
///
/// server_requests! {
///     ServerRequest;
///     (0, Ping([u8; 4], [u8; 4])),
///     (1, SendBytes((), ()))
/// }
///
/// let mut rpc_server = server::RpcServer::<ServerRequest>::new(32);
/// let mut recv_buf = vec![0; 32];
/// let mut send_buf = vec![0; 32];
///
/// // We fill recv_buf with some precalculated requets to simulate a client request
/// recv_buf[..11].copy_from_slice(&[0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03]);
/// recv_buf[11..11+17].copy_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09]);
///
/// let mut pos = 0;
/// let mut read_len = consts::REQ_HEADER_LEN;
/// let mut send_buf_bytes = 0;
/// loop {
///     let buf = &recv_buf[pos..pos + read_len];
///     match rpc_server.parse(&buf).unwrap() {
///         server::ParseResult::NeedBytes(n) => {
///             read_len = n;
///         }
///         server::ParseResult::Request(req, opt_buf) => {
///             read_len = consts::REQ_HEADER_LEN;
///             match req.unwrap() {
///                 ServerRequest::Ping(ping) => {
///                     println!("request ping: {:?}, {:?}", ping.body, opt_buf);
///                     let ping_body = ping.body;
///                     send_buf_bytes = ping.reply(ping_body, &mut send_buf).unwrap();
///                     println!("reply bytes: {:02x?}", &send_buf[..send_buf_bytes]);
///                 }
///                 ServerRequest::SendBytes(send_bytes) => {
///                     println!("request send_bytes: {:?}, {:?}", send_bytes.body, opt_buf);
///                     send_buf_bytes = send_bytes.reply((), &mut send_buf).unwrap();
///                     println!("reply bytes: {:02x?}", &send_buf[..send_buf_bytes]);
///                 }
///             }
///         }
///     }
///     // Send send_buf over the network
///     // [...]
///
///     pos += read_len;
///     // Break once all the received bytes have been parsed
///     println!("pos: {}, read_len: {}", pos, read_len);
///     if pos == 11 + 17 {
///         break;
///     }
/// }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! server_requests {
    ($request_enum:ident;
     $( ($id: expr, $method:ident ($req_type:ty, $rep_type:ty)) ),*) => {
        #[derive(Debug)]
        enum $request_enum {
            $(
                $method($crate::server::RequestType<$req_type, $rep_type>),
            )*
        }

        impl server::Request for $request_enum {
            type R = Self;

            fn from_bytes(header: $crate::RequestHeader, buf: &[u8]) -> $crate::server::Result<Self> {
                Ok(match header.method_idx {
                    $(
                        $id => $request_enum::$method($crate::server::RequestType::from_bytes(header, buf)?),
                    )*
                    _ => {
                        return Err($crate::server::Error::WontImplement);
                    }
                })
            }
        }
    }
}
