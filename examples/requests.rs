#[macro_use]
extern crate urpc;

use urpc::{client, client::OptBufNo, client::OptBufYes, consts, server};

use hex;

client_request! {
    client_requests;
    (0, ClientRequestPing([u8; 4], OptBufNo, [u8; 4], OptBufNo)),
    (1, ClientRequestSendBytes((), OptBufYes, (), OptBufNo))
}

server_requests! {
    ServerRequest;
    (0, Ping([u8; 4], [u8; 4])),
    (1, SendBytes((), ()))
}

fn main() -> () {
    const buf_len: usize = 32;
    let mut read_buf = vec![0; buf_len];
    let mut write_buf = vec![0; buf_len];

    let mut rpc_client = client::RpcClient::new();

    //let req_buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    //let mut req = client::RequestType::<ClientRequestSendBytes>::new(());
    //req.request(Some(&req_buf), &mut rpc_client, &mut read_buf);

    let mut req = ClientRequestPing::new([0, 1, 2, 3]);
    req.request(&mut rpc_client, vec![0; buf_len], &mut read_buf)
        .unwrap();

    println!("{}, {}", read_buf.len(), hex::encode(&read_buf));

    let mut rpc_server = server::RpcServer::<ServerRequest>::new(buf_len as u16);
    let mut pos = 0;
    let mut read_len = consts::REQ_HEADER_LEN;
    loop {
        let buf = &read_buf[pos..pos + read_len];
        println!("pos: {}, buf: {}", pos, hex::encode(buf));
        pos += read_len;
        match rpc_server.parse(&buf).unwrap() {
            server::ParseResult::NeedBytes(n) => {
                read_len = n;
            }
            server::ParseResult::Request(req, opt_buf) => {
                read_len = consts::REQ_HEADER_LEN;
                println!("request: {:?}, {:?}", req, opt_buf);
                match req.unwrap() {
                    ServerRequest::Ping(ping) => {
                        let ping_body = ping.body;
                        ping.reply(ping_body, &mut write_buf).unwrap();
                    }
                    ServerRequest::SendBytes(send_bytes) => {
                        println!("send_bytes: {}", hex::encode(opt_buf.unwrap()));
                        send_bytes.reply((), &mut write_buf).unwrap();
                    }
                }
                break;
            }
        }
    }
    println!("{}, {}", write_buf.len(), hex::encode(&write_buf));

    let mut pos = 0;
    let mut read_len = consts::REP_HEADER_LEN;
    loop {
        let buf = &write_buf[pos..pos + read_len];
        println!("pos: {}, buf: {}", pos, hex::encode(buf));
        pos += read_len;
        read_len = rpc_client.parse(&buf).unwrap().0;
        match req.reply(&mut rpc_client) {
            Some(r) => {
                println!("reply: {:?}", r.unwrap());
                break;
            }
            None => {}
        }
    }
}
