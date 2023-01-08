pub mod connection_manager;
pub mod packet;
pub mod socket;
pub mod transport;

fn find_first_14_consecutive(s: &[u8]) -> usize {
    s.windows(14)
        .position(|window| {
            let mut prev_lookup: u32 = 0;
            let mut lookup: u32 = 0;

            for val in window.iter() {
                lookup |= 1 << (val - 97);
                if lookup == prev_lookup {
                    return false;
                }
                prev_lookup = lookup;
            }
            return true;
        })
        .map(|pos| pos + 14)
        .unwrap()
}

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
