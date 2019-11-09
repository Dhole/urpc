use urpc;

urpc::setup!(
    methods: [
        {name: SendBytes, request_type: [u8]},
        {name: RecvBytes, request_type: ()},
        {name: Reset, request_type: ()},
        {name: Ping, request_type: [u8; 4]}
    ],
    errors: [
        InvalidMethod,
        InvalidBody,
        Busy
    ]
);

fn main() -> () {}
