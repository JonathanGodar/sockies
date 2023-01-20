use crate::{transport::transport_trait::Transport, packet::{Packet, self}};
use crc::{Crc, CRC_32_CKSUM};
use thiserror::Error;
use std::{
    collections::HashMap,
    io::{self, Cursor, Error, ErrorKind},
    net::SocketAddr,
};

use byteorder::{LittleEndian, ReadBytesExt};

// #[derive(Error)]
// struct NetworkPacketErr {

// }

enum PacketTypes {
    // Fragment = 0b00,
    Unreliable = 0b01,
    Reliable = 0b10,
    Ordered = 0b11,
}

impl TryFrom<u8> for PacketTypes {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            // 0 => Ok(Self::Fragment),
            1 => Ok(Self::Unreliable),
            2 => Ok(Self::Reliable),
            3 => Ok(Self::Ordered),
            _ => Err("Could not convert u8 to PacketType"),
        }
    }
}

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
    ack_bits: u16,
}

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
        }
    }

    fn handle_incomming(&mut self, rx: &[u8], packet_hash: u32) {
        let mut cursor = Cursor::new(rx);
        let standard_header = StandardHeader::with_checksum(cursor, packet_hash);
        if standard_header.is_err() {
            return;
        }

        let standard_header = standard_header.unwrap();
        let packet_type = standard_header.packet_type;
    
        // match packet_type {
        //     // PacketTypes::Orderedj //     // PacketTypes::Fragment {
        //     // }
        // }
    

    }
}

// Standard header
// CRC32

const CRC_32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);
impl<T: Transport> ConnectionManager<T> {
    fn calculate_hash(&self, recieved: &[u8]) -> u32 {
        let mut digest = CRC_32.digest();
        digest.update(&self.proto_version.to_le_bytes());
        digest.update(recieved);
        digest.finalize()
    }

    pub fn start_polling(&mut self) {
        const MAX_BUFFER_SIZE_BYTES: usize = 2000;
        let mut rx_buff: Vec<u8> = vec![0; MAX_BUFFER_SIZE_BYTES];

        let mut crc32 = Crc::<u32>::new(&CRC_32_CKSUM);

        loop {
            let result = self.transport.recv(&mut rx_buff);
            // Todo handle errors better

            let (n_bytes, from) = result.expect("Could not recieve read bytes.");
            let rx_buff = &rx_buff[..n_bytes];
            let acctual_hash = self.calculate_hash(rx_buff);

            let connection = self
                .open_connections
                .entry(from.clone())
                .or_insert_with(|| Connection::new(from.clone()));

            connection.handle_incomming(rx_buff, acctual_hash);
        }
    }
}

struct FragmentHeader {
    total_fragments: u8,
    fragment_id: u8,
}

struct StandardHeader {
    packet_type: PacketTypes,
    is_fragment: bool,
}


impl StandardHeader {
    fn with_checksum(mut cursor: Cursor<&[u8]>, real_checksum: u32) -> io::Result<Self> {
        let checksum = cursor.read_u32::<LittleEndian>()?;

        if real_checksum != checksum {
            Err(Error::new(ErrorKind::Other, "Bad checksum"))?;
        }

        // Todo remove unwrap
        // let packet_type = PacketTypes::try_from(cursor.read_u8()?).unwrap();
        // let is_fragment= cursor.read_u8()? as bool;
        let is_fragment = cursor.read_u8()? != 0;

        let packet_type = PacketTypes::try_from(cursor.read_u8()?)
            .map_err(|e| Error::new(ErrorKind::Other, "Bad packettype"))?;

        // if let Err(e) = packet_type {}
        // .map_err(|e| Err(Error::new(ErrorKind::Other, "Bad packet type")))?;

        Ok(Self {
            packet_type: PacketTypes::Ordered,
            is_fragment,
        })
    }
}

struct ReliableOrderedHeader {}
