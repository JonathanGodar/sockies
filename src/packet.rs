use std::{
    io::{Cursor, Error, ErrorKind},
    mem,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::{Crc, CRC_32_CKSUM};

#[derive(Debug, PartialEq, Eq)]
pub struct Packet {
    pub payload: Box<[u8]>,
    pub fragment_header: Option<FragmentHeader>,

    pub packet_id: u16,

    pub ordering_guarantee: OrderingGuarantee,
    pub delivery_guarantee: DeliveryGuarantee,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OrderingGuarantee {
    Ordered(OrderingHeader),
    Unordered,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeliveryGuarantee {
    Reliable(ReliableHeader),
    Unreliable,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FragmentHeader {
    frag_count: u8,
    frag_id: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OrderingHeader {
    channel: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReliableHeader {
    // seq_num: u16,
    ack_num: u16,
    ack_bits: u16,
}

// pub enum OrderingType {
// OrderedReliable,
// SequencedUnReliable,
// UnorderedReliable,
// UnorderedUnreliable,
// }

// enum PacketTypes {
//     Unreliable = 0b01,
//     Reliable = 0b10,
//     Ordered = 0b11,
// }

// impl TryFrom<u8> for PacketTypes {
//     type Error = &'static str;

//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         match value {
//             // 0 => Ok(Self::Fragment),
//             1 => Ok(Self::Unreliable),
//             2 => Ok(Self::Reliable),
//             3 => Ok(Self::Ordered),
//             _ => Err("Could not convert u8 to PacketType"),
//         }
//     }
// }

bitflags::bitflags! {
    struct PacketFlags: u8 {
        const IS_FRAGMENT = (1 << 0);
        const IS_ORDERED = (1 << 1);
        const IS_RELIABLE = (1 << 2);
        // const IS_HEARBET = (1 << 3);
    }
}

// #[derive(Error)]
// enum Se

const CRC_32: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);
impl Packet {
    fn caluclate_checksum(packet_minus_chksum: &[u8], proto_version: u64) -> u32 {
        let mut digest = CRC_32.digest();
        digest.update(&proto_version.to_le_bytes());
        digest.update(packet_minus_chksum);
        digest.finalize()
    }

    pub fn deserialize(bytes: &[u8], proto_version: u64) -> std::io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let checksum = cursor.read_u32::<LittleEndian>()?;

        if Packet::caluclate_checksum(
            &cursor.get_ref()[cursor.position() as usize..],
            proto_version,
        ) != checksum
        {
            Err(Error::new(ErrorKind::Other, "Bad checksum"))?;
        }

        let mut packet = Packet {
            payload: *Box::default(),
            fragment_header: None,

            packet_id: 0,

            ordering_guarantee: OrderingGuarantee::Unordered,
            delivery_guarantee: DeliveryGuarantee::Unreliable,
        };

        packet.packet_id = cursor.read_u16::<LittleEndian>()?;
        let flags = PacketFlags::from_bits_truncate(cursor.read_u8()?);

        if flags.contains(PacketFlags::IS_RELIABLE) {
            let ack_num = cursor.read_u16::<LittleEndian>()?;
            let ack_bits = cursor.read_u16::<LittleEndian>()?;

            packet.delivery_guarantee =
                DeliveryGuarantee::Reliable(ReliableHeader { ack_num, ack_bits });
        }

        if flags.contains(PacketFlags::IS_ORDERED) {
            let channel = cursor.read_u16::<LittleEndian>()?;

            packet.ordering_guarantee = OrderingGuarantee::Ordered(OrderingHeader { channel });
        }

        if flags.contains(PacketFlags::IS_FRAGMENT) {
            let frag_id = cursor.read_u8()?;
            let frag_count = cursor.read_u8()?;

            packet.fragment_header = Some(FragmentHeader {
                frag_count,
                frag_id,
            });
        }

        let pos = cursor.position() as usize;
        let mut payload: Vec<&u8> = bytes.copy_within(src, dest);
        payload.drain(..pos);

        packet.payload = payload.into_boxed_slice();
        Ok(packet)
    }

    fn serialize(&self, proto_version: u64) -> std::io::Result<Box<[u8]>> {
        let mut builder = Vec::new();

        builder.write_u32::<LittleEndian>(0)?;
        builder.write_u16::<LittleEndian>(self.packet_id)?;
        builder.write_u8(self.get_flags().bits)?;

        if let DeliveryGuarantee::Reliable(reliable_header) = &self.delivery_guarantee {
            builder.write_u16::<LittleEndian>(reliable_header.ack_num)?;
            builder.write_u16::<LittleEndian>(reliable_header.ack_bits)?;
        }

        if let OrderingGuarantee::Ordered(ordered_header) = &self.ordering_guarantee {
            builder.write_u16::<LittleEndian>(ordered_header.channel)?;
        }

        if let Some(fragment_header) = &self.fragment_header {
            builder.write_u8(fragment_header.frag_id)?;
            builder.write_u8(fragment_header.frag_count)?;
        }

        builder.extend_from_slice(&self.payload);

        let checksum = Packet::caluclate_checksum(&builder[mem::size_of::<u32>()..], proto_version);
        builder.splice(..mem::size_of::<u32>(), checksum.to_le_bytes());
        Ok(builder.into_boxed_slice())
    }

    fn get_flags(&self) -> PacketFlags {
        PacketFlags::from_bits_truncate(
            PacketFlags::IS_ORDERED.bits()
                * (matches!(self.ordering_guarantee, OrderingGuarantee::Ordered(..))) as u8
                | PacketFlags::IS_RELIABLE.bits()
                    * (matches!(self.delivery_guarantee, DeliveryGuarantee::Reliable(..))) as u8
                | PacketFlags::IS_FRAGMENT.bits() * (self.fragment_header.is_some()) as u8,
        )
    }

    // fn read_channel(mut cursor: Cursor<&[u8]>) -> std::io::Result<u32> {
    //     let mut channel: u32 = 0;
    //     for i in 0..4 {
    //         channel <<= 8;
    //         let read = cursor.read_u8()?;
    //         channel += read as u32;
    //         if read & (1 << 7) == 0 {
    //             break;
    //         }
    //     }

    //     Ok(channel)
    // }

    // fn write_channel()
}
mod test {
    use super::*;
    const PROTO_VERSION: u64 = 1;

    fn create_simple_packet() -> Packet {
        Packet {
            delivery_guarantee: DeliveryGuarantee::Unreliable,
            fragment_header: None,
            ordering_guarantee: OrderingGuarantee::Unordered,
            packet_id: 1,
            payload: vec![23, 24, 32, 12].into_boxed_slice(),
        }
    }

    fn assert_serialized_packet_is_same(packet: Packet, version: u64) {
        assert!(
            matches!(
                Packet::deserialize(packet.serialize(version).unwrap(), version),
                Ok(packet)
            )
        )
    }

    #[test]
    fn can_serialize_and_deserialize_simple_package() {
        let packet = create_simple_packet();
        
        let serialized = packet.serialize(PROTO_VERSION).unwrap();
        let deserialized = Packet::deserialize(serialized, PROTO_VERSION).unwrap();
        assert_eq!(packet, deserialized);
    }

    #[test]
    fn does_not_accept_corrupt_package(){
        let packet = create_simple_packet();
        let mut serialized = packet.serialize(PROTO_VERSION).unwrap();
        
        for replacement_val in 0..(mem::size_of::<u8>() * 8){
            for i in 0..serialized.len() {
                let prev = serialized[i];
                if prev == replacement_val as u8 {
                    continue;
                } 
                serialized[i] = replacement_val as u8;
                assert!(Packet::deserialize(serialized.clone(), PROTO_VERSION).is_err());
                serialized[i] = prev;
            }
        }
    }

    #[test]
    fn does_not_accept_packet_with_wrong_version(){
        let packet = create_simple_packet();
        let serialized = packet.serialize(PROTO_VERSION).unwrap();
        assert!(Packet::deserialize(serialized, PROTO_VERSION + 1).is_err());
    }

    #[test]
    fn can_serialize_and_deserialize_reliable_pack(){
        let mut packet = create_simple_packet();
        packet.delivery_guarantee = DeliveryGuarantee::Reliable(ReliableHeader { ack_num: 12, ack_bits: 0b00101 });

        assert_serialized_packet_is_same(packet, PROTO_VERSION);
    }

    #[test]
    fn can_serialize_and_deserialize_frag_reliable_pack(){
        let mut packet = create_simple_packet();
        packet.delivery_guarantee = DeliveryGuarantee::Reliable(ReliableHeader { ack_num: 12, ack_bits: 0b00101 });
        packet.fragment_header = Some(FragmentHeader { frag_count: 23, frag_id: 3 });

        assert_serialized_packet_is_same(packet, PROTO_VERSION);
    }
}


// mod test {
//     use super::*;

//     #[test]
//     fn can_read_channel_u8_len() {
//         {
//             let dta = vec![0];
//             let result = Packet::read_channel(Cursor::new(dta.as_slice()));

//             assert_eq!(result.unwrap(), 0);
//         }

//         {
//             let dta = vec![2];
//             let result = Packet::read_channel(Cursor::new(dta.as_slice()));

//             assert_eq!(result.unwrap(), 2);
//         }
//     }

//     #[test]
//     fn can_read_channel_u16_len() {
//         {
//             let dta = vec![255, 1];
//             let result = Packet::read_channel(Cursor::new(dta.as_slice()));

//             println!("{:b}", result.as_ref().unwrap());
//             assert_eq!(result.unwrap(), 0b1111111100000001);
//         }

//         {
//             let dta = vec![128, 4];
//             let result = Packet::read_channel(Cursor::new(dta.as_slice()));

//             assert_eq!(result.unwrap(), 0b1000000000000100);
//         }
//     }

//     #[test]
//     fn can_read_channel_u24_len() {
//         {
//             let dta = vec![128, 128, 4];
//             let result = Packet::read_channel(Cursor::new(dta.as_slice()));

//             println!("{:b}", result.as_ref().unwrap());
//             assert_eq!(result.unwrap(), (1 << 23u32) + (1u32 << 15u32) + 4u32);
//         }

//         {
//             let dta = vec![255, 255, 4];
//             let result = Packet::read_channel(Cursor::new(dta.as_slice()));

//             assert_eq!(result.unwrap(), (255u32 << 16u32) + (255u32 << 8u32) + 4);
//         }
//     }
// }

// Reliable header
// Sequence nr
// Ack index
// Ack bitfield

// Arranging header
// Arranging id,
// Stream id,

// Fragment header

// pub enum DeliveryGuarantee {
//     Reliable,
//     Unreliable,
// }

// What we need to know
// If reliable -> unordered, or ordered?
// If unreliable -> Sequenced or not?

// Is it sequenced, ordered, or unordered
// Is it a fragment?

// Packet types:

// Checksum (cfc32). 32 bits
// Packet type 2 bits
// Sequence number 16 bits
// [data]
//

// fragment id 8 bits
// total fragments 8 bits
