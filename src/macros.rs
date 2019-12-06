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
