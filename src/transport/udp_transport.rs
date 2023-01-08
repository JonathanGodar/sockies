use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

use super::transport_trait::Transport;

struct UdpTransport {
    udp_socket: UdpSocket,
}
impl Transport for UdpTransport {
    fn send(&mut self, addr: SocketAddr, payload: Box<[u8]>) -> io::Result<usize> {
        self.udp_socket.send_to(&payload, addr)
    }

    fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, std::net::SocketAddr), std::io::Error> {
        self.udp_socket.recv_from(buf)
    }
}
