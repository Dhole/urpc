#[macro_export(local_inner_macros)]
macro_rules! client_request {
    ($id: expr, $method:ident ( $req_type:ty, $rep_type:ty) ) => {
        struct $method;
        impl $crate::client::Request for $method {
            type Q = $req_type;
            type P = $rep_type;
            const METHOD_ID: u8 = $id;
        }
    };
}

#[macro_export(local_inner_macros)]
macro_rules! server_requests {
    ($request_enum:ident; $( ($id: expr, $method:ident ($req_type:ty, $rep_type:ty)) ),*) => {
        #[derive(Debug)]
        enum $request_enum {
            $(
                $method($crate::server::RequestType<$req_type, $rep_type>),
            )*
        }

        impl server::Request for $request_enum {
            type R = Self;

            fn from_bytes(header: $crate::RequestHeader, buf: &[u8]) -> $crate::Result<Self> {
                Ok(match header.method_idx {
                    $(
                        $id => $request_enum::$method($crate::server::RequestType::from_bytes(header, buf)?),
                    )*
                    _ => {
                        return Err($crate::Error::WontImplement);
                    }
                })
            }
        }
    }
}
