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
/// use urpc::{client_requests, client, consts, OptBufNo, OptBufYes};
///
/// mod cli {
///     use urpc::client_requests;
///
///     client_requests! {
///         client_requests;
///         (0, ping, Ping([u8; 4], OptBufNo, [u8; 4], OptBufNo)),
///         (1, send_bytes, SendBytes((), OptBufYes, (), OptBufNo))
///     }
/// }
///
/// let mut rpc_client = client::RpcClient::new(32);
/// let mut send_buf = vec![0; 32];
/// let mut recv_buf = vec![0; 32];
///
/// let mut req1 = cli::Ping::new([0, 1, 2, 3]);
/// let send_buf_bytes = req1.request(&mut rpc_client, &mut send_buf).unwrap();
/// println!("request bytes: {:02x?}", &send_buf[..send_buf_bytes]);
///
/// // Send send_buf over the network
/// // [...]
///
/// // Read from the network into recv_buf
/// // [...]
/// // We fill recv_buf with some precalculated replies to simulate a server reply
/// recv_buf[..10].copy_from_slice(&[0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03]);
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
/// let mut req2 = cli::SendBytes::new(());
/// let send_buf_bytes = req2.request(&req_buf, &mut rpc_client, &mut send_buf).unwrap();
/// println!("request bytes: {:02x?}", &send_buf[..send_buf_bytes]);
///
/// // Send send_buf over the network
/// // [...]
///
/// // Read from the network into recv_buf
/// // [...]
/// // We fill recv_buf with some precalculated replies to simulate a server reply
/// recv_buf[..10].copy_from_slice(&[0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03]);
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
        $( ($id:expr, $_fn:expr, $method:ident ( $req_type:ty, $req_opt_buf:ident, $rep_type:ty, $rep_opt_buf:ident)) ),*) => {
            use urpc::{OptBufNo, OptBufYes};

            mod methodid {
                $(
                        pub struct $method;
                        impl $crate::client::MethodId for $method {
                            const METHOD_ID: u8 = $id;
                        }
                )*
            }
            $(
                    // client_request_export!($request_mod, $method, $req_opt_buf, $rep_opt_buf);
                    pub type $method = $crate::client::RequestType<
                    methodid::$method,
                    $req_type,
                    $req_opt_buf,
                    $rep_type,
                    $rep_opt_buf,
                    >;
            )*
    };
}

#[macro_export(local_inner_macros)]
macro_rules! rpc_client_io_fn {
    ($fn:ident, $method:ident ( $req_type:ty, OptBufNo, $rep_type:ty, OptBufNo)) => {
        pub fn $fn(
            &mut self,
            arg: $req_type,
        ) -> Result<$rep_type, $crate::client::RpcClientIOError> {
            let mut req = $method::new(arg);
            let write_len = req.request(
                &mut self.rpc.client,
                std::vec![0; self.rpc.buf_len],
                &mut self.rpc.stream_buf,
            )?;
            self.rpc.request(req.chan_id(), write_len)?;
            let (r, _) = req.take_reply(&mut self.rpc.client).unwrap()?;
            Ok(r)
        }
    };
    ($fn:ident, $method:ident ( $req_type:ty, OptBufYes, $rep_type:ty, OptBufNo)) => {
        pub fn $fn(
            &mut self,
            arg: $req_type,
            req_buf: &[u8],
        ) -> Result<$rep_type, $crate::client::RpcClientIOError> {
            let mut req = $method::new(arg);
            let write_len = req.request(
                req_buf,
                &mut self.rpc.client,
                std::vec![0; self.rpc.buf_len],
                &mut self.rpc.stream_buf,
            )?;
            self.rpc.request(req.chan_id(), write_len)?;
            let (r, _) = req.take_reply(&mut self.rpc.client).unwrap()?;
            Ok(r)
        }
    };
    ($fn:ident, $method:ident ( $req_type:ty, OptBufNo, $rep_type:ty, OptBufYes)) => {
        pub fn $fn(
            &mut self,
            arg: $req_type,
        ) -> Result<($rep_type, Vec<u8>), $crate::client::RpcClientIOError> {
            let mut req = $method::new(arg);
            let write_len = req.request(
                &mut self.rpc.client,
                std::vec![0; self.rpc.buf_len],
                std::vec![0; self.rpc.buf_len],
                &mut self.rpc.stream_buf,
            )?;
            self.rpc.request(req.chan_id(), write_len)?;
            let (r, buf, _) = req.take_reply(&mut self.rpc.client).unwrap()?;
            Ok((r, buf))
        }
    };
    ($fn:ident, $method:ident ( $req_type:ty, OptBufYes, $rep_type:ty, OptBufYes)) => {
        pub fn $fn(
            &mut self,
            arg: $req_type,
            req_buf: &[u8],
        ) -> Result<($rep_type, Vec<u8>), $crate::client::RpcClientIOError> {
            let mut req = $method::new(arg);
            let write_len = req.request(
                req_buf,
                &mut self.rpc.client,
                std::vec![0; self.rpc.buf_len],
                std::vec![0; self.rpc.buf_len],
                &mut self.rpc.stream_buf,
            )?;
            self.rpc.request(req.chan_id(), write_len)?;
            let (r, buf, _) = req.take_reply(&mut self.rpc.client).unwrap()?;
            Ok((r, buf))
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! rpc_client_io {
    ($client:ident;
     $request_mod:ident;
        $( ($id:expr, $fn:ident, $method:ident ( $req_type:ty, $req_opt_buf:ident, $rep_type:ty, $rep_opt_buf:ident)) ),*) => {
        use std::io::{self, Read, Write};

        client_requests! {
            $request_mod;
            $(
                ($id, $fn, $method ( $req_type, $req_opt_buf, $rep_type, $rep_opt_buf))
            ),*
        }

        pub struct $client<S: io::Read + io::Write> {
            rpc: $crate::client::RpcClientIO<S>,
        }

        impl<S: Read + Write> $client<S> {
            pub fn new(stream: S, buf_len: usize) -> Self {
                Self {
                    rpc: $crate::client::RpcClientIO::new(stream, buf_len),
                }
            }
            $(
                rpc_client_io_fn!($fn, $method ( $req_type, $req_opt_buf, $rep_type, $rep_opt_buf));
                // pub fn $fn(&mut self, arg: $req_type) -> Result<$rep_type, $crate::client::RpcClientIOError> {
                //     let mut req = $method::new(arg);
                //     let write_len = req.request(
                //         &mut self.rpc.client,
                //         self.rpc.body_buf.take().unwrap(),
                //         &mut self.rpc.stream_buf,
                //     )?;
                //     self.rpc.request(req.chan_id(), write_len)?;
                //     let (r, body_buf) = req.take_reply(&mut self.rpc.client).unwrap()?;
                //     self.rpc.body_buf = Some(body_buf);
                //     Ok(r)
                // }
            )*
        }
    };
}

/// Macro that builds the required types to handle calls via RPC from the server.
///
/// Examples
///
/// ```
/// use urpc::{server_requests, server::{self, Request}, consts, OptBufNo, OptBufYes};
///
/// server_requests! {
///     ServerRequest;
///     (0, ping, Ping([u8; 4], OptBufNo, [u8; 4], OptBufNo)),
///     (1, send_bytes, SendBytes((), OptBufYes, (), OptBufNo))
/// }
///
/// let mut rpc_server = server::RpcServer::new(32);
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
///     match ServerRequest::from_rpc(&mut rpc_server, &buf).unwrap() {
///         server::ParseResult::NeedBytes(n) => {
///             read_len = n;
///         }
///         server::ParseResult::Request(req) => {
///             read_len = consts::REQ_HEADER_LEN;
///             match req {
///                 ServerRequest::Ping(ping) => {
///                     println!("request ping: {:?}", ping.body);
///                     let ping_body = ping.body;
///                     send_buf_bytes = ping.reply(ping_body, &mut send_buf).unwrap();
///                     println!("reply bytes: {:02x?}", &send_buf[..send_buf_bytes]);
///                 }
///                 ServerRequest::SendBytes((send_bytes, buf)) => {
///                     println!("request send_bytes: {:?}, {:?}", send_bytes.body, buf);
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
macro_rules! server_requests_variant {
    ($req_type:ty, OptBufNo, $rep_type:ty, $rep_opt_buf:ident) => {
        $crate::server::RequestType<$req_type, OptBufNo, $rep_type, $rep_opt_buf>
    };
    ($req_type:ty, OptBufYes, $rep_type:ty, $rep_opt_buf:ident) => {
        ($crate::server::RequestType<$req_type, OptBufYes, $rep_type, $rep_opt_buf>, &'a [u8])
    };
}

#[macro_export(local_inner_macros)]
macro_rules! server_requests {
    ($request_enum:ident;
     $( ($id: expr, $_fn:ident, $method:ident ($req_type:ty, $req_opt_buf:ident, $rep_type:ty, $rep_opt_buf:ident)) ),*) => {
        #[derive(Debug)]
        enum $request_enum<'a> {
            $(
                $method(server_requests_variant!($req_type, $req_opt_buf, $rep_type, $rep_opt_buf)),
            )*
        }

        impl<'a> server::Request<'a> for $request_enum<'a> {
            fn from_bytes(header: $crate::RequestHeader, buf: &'a [u8]) -> $crate::server::Result<Self> {
                Ok(match header.method_idx {
                    $(
                        $id => $request_enum::$method(
                            $crate::server::RequestType::<_, $req_opt_buf, _, _>::from_bytes(header, buf)?),
                    )*
                    _ => {
                        return Err($crate::server::Error::WontImplement);
                    }
                })
            }
        }
    }
}
