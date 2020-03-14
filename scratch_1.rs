pub trait Request {
    type R;
    
    fn from_bytes(id: u8, buf: &[u8]) -> Self::R;
}

pub struct RequestType<Q: Default> {
    pub body: Q,
}

impl<Q: Default> RequestType<Q> {
    pub fn from_bytes(buf: &[u8]) -> Self {
        Self {
            body: Q::default(),
        }
    }
}


enum ServerRequests {
    Ping(RequestType<u32>),
    SendBytes(RequestType<()>),
}

impl Request for ServerRequests {
    type R = Self;
    
    fn from_bytes(id: u8, buf: &[u8]) -> Self {
        match id {
            0 => ServerRequests::Ping(RequestType::from_bytes(buf)),
            1 => ServerRequests::SendBytes(RequestType::from_bytes(buf)),
            _ => unreachable!(),
        }
    }
}


fn main() {
    
}
