#[macro_use]
extern crate urpc;

use urpc::{client, client::OptBufNo, client::OptBufYes, consts, server};

use hex;

client_requests! {
    client_requests;
    (0, ClientRequestPing([u8; 4], OptBufNo, [u8; 4], OptBufNo)),
    (1, ClientRequestSendBytes((), OptBufYes, (), OptBufNo)),
    (2, ClientRequestRecvBytes((), OptBufNo, (), OptBufYes))
}

server_requests! {
    ServerRequest;
    (0, Ping([u8; 4], OptBufNo, [u8; 4], OptBufNo)),
    (1, SendBytes((), OptBufYes, (), OptBufNo)),
    (2, RecvBytes((), OptBufNo, (), OptBufYes))
}

fn main() -> () {
    const buf_len: usize = 4096;
    let mut read_buf = vec![0; buf_len];
    let mut write_buf = vec![0; buf_len];

    let mut rpc_client = client::RpcClient::new();

    let mut req0 = None;
    let mut req1 = None;
    let mut req2 = None;
    for i in 0..3 {
        read_buf.iter_mut().for_each(|x| *x = 0);
        write_buf.iter_mut().for_each(|x| *x = 0);
        let mut read_buf_len = 0;
        match i {
            0 => {
                println!("--- Ping ---");
                let mut req = ClientRequestPing::new([0, 1, 2, 3]);
                read_buf_len = req
                    .request(&mut rpc_client, vec![0; buf_len], &mut read_buf)
                    .unwrap();
                req0 = Some(req);
            }
            1 => {
                println!("--- SendBytes ---");
                let req_buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
                let mut req = ClientRequestSendBytes::new(());
                read_buf_len = req
                    .request(&req_buf, &mut rpc_client, vec![0; buf_len], &mut read_buf)
                    .unwrap();
                req1 = Some(req);
            }
            2 => {
                println!("--- RecvBytes ---");
                let mut req = ClientRequestRecvBytes::new(());
                read_buf_len = req
                    .request(
                        &mut rpc_client,
                        vec![0; buf_len],
                        vec![0; buf_len],
                        &mut read_buf,
                    )
                    .unwrap();
                req2 = Some(req);
            }
            _ => {}
        }

        println!(
            "read_buf: {}, {}",
            read_buf_len,
            hex::encode(&read_buf[..read_buf_len])
        );

        let mut rpc_server = server::RpcServer::<ServerRequest>::new(buf_len as u16);
        let mut pos = 0;
        let mut read_len = consts::REQ_HEADER_LEN;
        let mut write_buf_len = 0;
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
                            write_buf_len = ping.reply(ping_body, &mut write_buf).unwrap();
                        }
                        ServerRequest::SendBytes(send_bytes) => {
                            println!("send_bytes: {}", hex::encode(opt_buf.unwrap()));
                            write_buf_len = send_bytes.reply((), &mut write_buf).unwrap();
                        }
                        ServerRequest::RecvBytes(recv_bytes) => {
                            write_buf_len = recv_bytes.reply((), &mut write_buf).unwrap();
                        }
                    }
                    break;
                }
            }
        }
        println!(
            "write_buf: {}, {}",
            write_buf_len,
            hex::encode(&write_buf[..write_buf_len])
        );

        match i {
            0 => {
                let mut req = req0.unwrap();
                let mut pos = 0;
                let mut read_len = consts::REP_HEADER_LEN;
                loop {
                    let buf = &write_buf[pos..pos + read_len];
                    println!("pos: {}, buf: {}", pos, hex::encode(buf));
                    pos += read_len;
                    read_len = rpc_client.parse(&buf).unwrap().0;
                    match req.take_reply(&mut rpc_client) {
                        Some(r) => {
                            println!("reply: {:?}", r.unwrap());
                            break;
                        }
                        None => {}
                    }
                }
                req0 = Some(req);
            }
            1 => {
                let mut req = req1.unwrap();
                let mut pos = 0;
                let mut read_len = consts::REP_HEADER_LEN;
                loop {
                    let buf = &write_buf[pos..pos + read_len];
                    println!("pos: {}, buf: {}", pos, hex::encode(buf));
                    pos += read_len;
                    read_len = rpc_client.parse(&buf).unwrap().0;
                    match req.take_reply(&mut rpc_client) {
                        Some(r) => {
                            println!("reply: {:?}", r.unwrap());
                            break;
                        }
                        None => {}
                    }
                }
                req1 = Some(req);
            }
            2 => {
                let mut req = req2.unwrap();
                let mut pos = 0;
                let mut read_len = consts::REP_HEADER_LEN;
                loop {
                    let buf = &write_buf[pos..pos + read_len];
                    println!("pos: {}, buf: {}", pos, hex::encode(buf));
                    pos += read_len;
                    read_len = rpc_client.parse(&buf).unwrap().0;
                    match req.take_reply(&mut rpc_client) {
                        Some(r) => {
                            println!("reply: {:?}", r.unwrap());
                            break;
                        }
                        None => {}
                    }
                }
                req2 = Some(req);
            }
            _ => {}
        }
    }
}
