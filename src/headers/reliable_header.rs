#[derive(Debug, PartialEq, Eq)]
pub struct ReliableHeader {
    pub ack_num: u16,
    pub ack_bits: u32,
}

impl ReliableHeader {
    pub fn get_acked_packages(&self) -> Vec<u16> {
        let mut acked: Vec<u16> = Vec::with_capacity(33);

        acked.push(self.ack_num);
        for bit_num in 0..(std::mem::size_of::<u32>() * 8) {
            let bit_mask = 1 << bit_num;
            if bit_mask & self.ack_bits != 0 {
                acked.push(
                    (std::num::Wrapping(self.ack_num)
                        - std::num::Wrapping(1)
                        - std::num::Wrapping(bit_num as u16))
                    .0,
                );
            }
        }

        acked
    }
}

mod test {
    use super::*;

    #[test]
    fn can_get_acked_packages() {
        let header = ReliableHeader {
            ack_num: 255,
            ack_bits: 0b10000000000000000000000000000001,
        };
        let expected = vec![255, 255 - 1, 255 - 32];

        assert_eq!(header.get_acked_packages(), expected);

        let header = ReliableHeader {
            ack_num: 123,
            ack_bits: 0b11111111111111111111111111111111,
        };

        let expected: Vec<_> = ((123 - 32)..=123).rev().collect();
        assert_eq!(header.get_acked_packages(), expected);
    }

    #[test]
    fn can_handle_underflows() {
        let header = ReliableHeader {
            ack_num: 2,
            ack_bits: 0b100,
        };

        let expected: Vec<_> = vec![2, u16::MAX];
        assert_eq!(header.get_acked_packages(), expected);
    }
}
