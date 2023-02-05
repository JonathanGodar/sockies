use crate::{
    headers::ReliableHeader,
    // headers::reliable_header::ReliableHeader,
    packet::{self, OrderingHeader, Packet},
    transport::transport_trait::Transport,
};
use crc::{Crc, CRC_32_CKSUM};
use std::{
    collections::HashMap,
    io::{self, Cursor, Error, ErrorKind},
    net::SocketAddr,
};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct ConnectionManager<T: Transport> {
    open_connections: HashMap<SocketAddr, Connection>,
    transport: T,
    proto_version: u64,
}

pub struct Connection {
    addr: SocketAddr,
    id_counter: u16,

    unreliable_seq: u16,
    unreliable_ack: u16,

    seq: u16,
    ack: u16,
    ack_bits: u32,


    seq_buffer: [],
    // Packets that the other peer has not acked
    unacked_packets: HashMap<u16, Packet>,
}

st:

impl Connection {
    fn new(sock_addr: SocketAddr) -> Self {
        Self {
            addr: sock_addr,
            id_counter: 0,

            unreliable_seq: 0,
            unreliable_ack: 0,

            seq: 0,
            ack: 0,
            ack_bits: 0,
            unacked_packets: HashMap::new(),
        }
    }

    fn handle_incomming(&mut self, rx: &[u8], protocol_version: u64) {
        let packet = Packet::deserialize(rx, protocol_version);
        if packet.is_err() {
            return;
        }
        let packet = packet.unwrap();

        match packet.delivery_guarantee {
            packet::DeliveryGuarantee::Reliable(header) => {
                self.handle_reliable(packet.packet_id, header);
            }
            packet::DeliveryGuarantee::Unreliable => {}
        }

    }

    fn handle_reliable(&mut self, packet_id: u16, reliable_header: ReliableHeader) {
        for acked in reliable_header.get_acked_packages() {
            self.unacked_packets.remove(&acked);
        }

        let shift = packet_id as i32 - self.ack as i32;

        if shift < -32 {
            todo!();
        } else if shift < 0 {
            self.ack_bits |= 1 << (-shift - 1);
        } else if shift > 0 {
            self.ack += shift as u16;
            self.ack_bits <<= shift;

            let prev_ack_num_in_ack_bits = 1 << (shift - 1);
            self.ack_bits |= prev_ack_num_in_ack_bits;
        }
    }

    fn create_reliable(&mut self, payload: Vec<u8>) -> Packet {
        Packet {
            payload,
            packet_id: {
                self.seq += 1;
                self.seq - 1
            },
            fragment_header: None,
            ordering_guarantee: packet::OrderingGuarantee::Unordered,
            delivery_guarantee: packet::DeliveryGuarantee::Reliable(ReliableHeader {
                ack_num: self.ack,
                ack_bits: self.ack_bits,
            }),
        }
    }
}

// Standard header
// CRC32

const CRC_32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);
impl<T: Transport> ConnectionManager<T> {
    // fn calculate_hash(&self, recieved: &[u8]) -> u32 {
    //     let mut digest = CRC_32.digest();
    //     digest.update(&self.proto_version.to_le_bytes());
    //     digest.update(recieved);
    //     digest.finalize()
    // }

    pub fn start_polling(&mut self) {
        const MAX_BUFFER_SIZE_BYTES: usize = 2000;
        let mut rx_buff: Vec<u8> = vec![0; MAX_BUFFER_SIZE_BYTES];

        let mut crc32 = Crc::<u32>::new(&CRC_32_CKSUM);

        loop {
            let result = self.transport.recv(&mut rx_buff);
            // Todo handle errors better

            let (n_bytes, from) = result.expect("Could not recieve read bytes.");
            let rx_buff = &rx_buff[..n_bytes];

            let connection = self
                .open_connections
                .entry(from.clone())
                .or_insert_with(|| Connection::new(from.clone()));

            connection.handle_incomming(rx_buff, self.proto_version);
        }
    }
}

// struct FragmentHeader {
//     total_fragments: u8,
//     fragment_id: u8,
// }

// struct StandardHeader {
//     packet_type: PacketTypes,
//     is_fragment: bool,
// }

// impl StandardHeader {
//     fn with_checksum(mut cursor: Cursor<&[u8]>, real_checksum: u32) -> io::Result<Self> {
//         let checksum = cursor.read_u32::<LittleEndian>()?;

//         if real_checksum != checksum {
//             Err(Error::new(ErrorKind::Other, "Bad checksum"))?;
//         }

//         // Todo remove unwrap
//         // let packet_type = PacketTypes::try_from(cursor.read_u8()?).unwrap();
//         // let is_fragment= cursor.read_u8()? as bool;
//         let is_fragment = cursor.read_u8()? != 0;

//         let packet_type = PacketTypes::try_from(cursor.read_u8()?)
//             .map_err(|e| Error::new(ErrorKind::Other, "Bad packettype"))?;

//         // if let Err(e) = packet_type {}
//         // .map_err(|e| Err(Error::new(ErrorKind::Other, "Bad packet type")))?;

//         Ok(Self {
//             packet_type: PacketTypes::Ordered,
//             is_fragment,
//         })
//     }
// }

// struct ReliableOrderedHeader {}
