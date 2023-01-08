pub struct Packet {
    pub ordering_type: OrderingType,
    // pub delivery_guarantee: DeliveryGuarantee,
    pub payload: Box<u8>,
}

pub enum OrderingType {
    OrderedReliable,
    SequencedUnReliable,
    UnorderedReliable,
    UnorderedUnreliable,
}

// Reliable header
// Sequence nr
// Ack index
// Ack bitfield

// Arranging header
// Arranging id,
// Stream id,

// Fragment header
// :wq

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
