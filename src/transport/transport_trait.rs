use std::{io, net::SocketAddr};

pub trait Transport {
    fn send(&mut self, addr: SocketAddr, payload: Box<[u8]>) -> io::Result<usize>;
    fn recv(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;
}
