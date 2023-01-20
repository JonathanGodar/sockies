pub mod connection_manager;
pub mod packet;
pub mod socket;
pub mod transport;

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::UdpSocket;

    #[test]
    fn sending_on_closed_socket() {
        let socket = UdpSocket::bind("127.0.0.1:3000").unwrap();
    }
}

// Crates that can be used: Fxhash, bincode ( or use crc 32 from wikipedia)
