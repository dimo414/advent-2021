use std::collections::VecDeque;
use std::ops::Deref;
use std::str::FromStr;
use anyhow::{anyhow, Context, ensure, Error, Result};

fn main() -> Result<()> {
    let mut input: Bitstream = include_str!("input.txt").parse()?;
    let packet = Packet::deserialize(&mut input)?;
    println!("Versions: {:?}", packet.sum_versions());
    println!("Launches: {:?}", packet.evaluate().expect("Could not evaluate"));

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
enum Type {
    Literal(u64),
    Sum(Vec<Packet>),
    Product(Vec<Packet>),
    Min(Vec<Packet>),
    Max(Vec<Packet>),
    Gt(Box<[Packet; 2]>),
    Lt(Box<[Packet; 2]>),
    Eq(Box<[Packet; 2]>),
}

impl Type {
    fn subpackets(&self) -> &[Packet] {
        match self {
            Type::Literal(_) => &[],
            Type::Sum(v)|Type::Product(v)|Type::Min(v)|Type::Max(v) => v,
            Type::Gt(a)|Type::Lt(a)|Type::Eq(a) => a.deref(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Packet {
    version: u8,
    body: Type,
}

impl Packet {
    // These factory functions are only used in the tests
    #[allow(dead_code)] fn literal(version: u8, value: u64) -> Self { Packet{ version, body: Type::Literal(value), } }
    #[allow(dead_code)] fn sum(version: u8, packets: Vec<Packet>) -> Self { Packet{ version, body: Type::Sum(packets), } }
    #[allow(dead_code)] fn product(version: u8, packets: Vec<Packet>) -> Self { Packet{ version, body: Type::Product(packets), } }
    #[allow(dead_code)] fn max(version: u8, packets: Vec<Packet>) -> Self { Packet{ version, body: Type::Max(packets), } }
    #[allow(dead_code)] fn min(version: u8, packets: Vec<Packet>) -> Self { Packet{ version, body: Type::Min(packets), } }
    #[allow(dead_code)] fn gt(version: u8, left: Packet, right: Packet) -> Self {
        Packet{ version, body: Type::Gt(Box::new([left, right])), }
    }
    #[allow(dead_code)] fn lt(version: u8, left: Packet, right: Packet) -> Self {
        Packet{ version, body: Type::Lt(Box::new([left, right])), }
    }
    #[allow(dead_code)] fn eq(version: u8, left: Packet, right: Packet) -> Self {
        Packet{ version, body: Type::Eq(Box::new([left, right])), }
    }

    fn deserialize(bits: &mut Bitstream) -> Result<Packet> {
        fn deserialize_subpackages(bits: &mut Bitstream) -> Result<Vec<Packet>> {
            let mut packets = Vec::new();
            let sub_packet_type = bits.pop_bits(1)? == 1;
            if sub_packet_type {
                let num_packets = bits.pop_bits(11)?;
                for _ in 0..num_packets {
                    packets.push(Packet::deserialize(bits)?);
                }
            } else {
                let num_bits = bits.pop_bits(15)?;
                let bits_leftover = bits.len() - num_bits as usize;
                while bits.len() > bits_leftover {
                    packets.push(Packet::deserialize(bits)?);
                }
            }
            Ok(packets)
        }

        let version = bits.pop_bits(3)?.try_into()?;
        let packet_type = bits.pop_bits(3)?;
        let packet = match packet_type {
            4 => { // Literal
                let mut value = 0;
                loop {
                    let more = bits.pop_bits(1)? == 1;
                    value = (value << 4) | bits.pop_bits(4)?;
                    if !more { break; }
                }
                Packet { version, body: Type::Literal(value), }
            },
            t => {
                let packets = deserialize_subpackages(bits)?;
                let body = match t {
                    0 => Type::Sum(packets),
                    1 => Type::Product(packets),
                    2 => Type::Min(packets),
                    3 => Type::Max(packets),
                    5 => Type::Gt(Box::new(packets.try_into().map_err(|v| anyhow!("Invalid number of subpackets: {:?}", v))?)),
                    6 => Type::Lt(Box::new(packets.try_into().map_err(|v| anyhow!("Invalid number of subpackets: {:?}", v))?)),
                    7 => Type::Eq(Box::new(packets.try_into().map_err(|v| anyhow!("Invalid number of subpackets: {:?}", v))?)),
                    _ => panic!("Unknown operator {}", t),
                };
                Packet{ version, body }
            },
        };
        Ok(packet)
    }

    fn sum_versions(&self) -> u64 {
        self.version as u64 + self.body.subpackets().iter().map(Packet::sum_versions).sum::<u64>()
    }

    fn evaluate(&self) -> Result<u64> {
        // It doesn't seem possible to directly min()/max() an Iter<Result<u64>>
        fn min(packets: &[Packet]) -> Result<u64> {
            let evaled = packets.iter().map(Packet::evaluate).collect::<Result<Vec<_>>>()?;
            Ok(*evaled.iter().min().ok_or_else(||anyhow!("Invalid sub-pattern"))?)
        }
        fn max(packets: &[Packet]) -> Result<u64> {
            let evaled = packets.iter().map(Packet::evaluate).collect::<Result<Vec<_>>>()?;
            Ok(*evaled.iter().max().ok_or_else(||anyhow!("Invalid sub-pattern"))?)
        }

        Ok(match &self.body {
            Type::Literal(value) => *value,
            Type::Sum(parts) => parts.iter().map(Packet::evaluate).sum::<Result<u64>>()?,
            Type::Product(parts) => parts.iter().map(Packet::evaluate).product::<Result<u64>>()?,
            Type::Min(parts) => min(parts)?,
            Type::Max(parts) => max(parts)?,
            Type::Gt(parts) => if parts[0].evaluate()? > parts[1].evaluate()? { 1 } else { 0 },
            Type::Lt(parts) => if parts[0].evaluate()? < parts[1].evaluate()? { 1 } else { 0 },
            Type::Eq(parts) => if parts[0].evaluate()? == parts[1].evaluate()? { 1 } else { 0 },
        })
    }
}

struct Bitstream {
    bytes: VecDeque<u8>,
    offset: usize,
}

impl Bitstream {
    fn create(bytes: &[u8]) -> Bitstream {
        Bitstream{ bytes: bytes.iter().cloned().collect(), offset: 0, }
    }

    pub fn len(&self) -> usize {
        let byte_bits = self.bytes.len() * 8;
        if self.offset == 0 {
            return byte_bits;
        }
        byte_bits - (8 - self.offset)
    }

    fn pop_from_offset(&mut self, mut bits: usize) -> (usize, u64) {
        if bits > self.offset { bits = self.offset; } // may be zero
        let mut ret = 0;
        for _ in 0..bits {
            let front = self.bytes.front_mut().expect("Unexpected offset");
            ret = (ret << 1) | ((*front as u64 & 128) >> 7);
            *front <<= 1;
        }
        if self.offset > 0 && self.offset == bits { self.bytes.pop_front().expect("Can't be absent"); }
        self.offset -= bits;
        (bits, ret)
    }

    pub fn pop_bits(&mut self, mut bits: usize) -> Result<u64> {
        assert!(bits <= 64);
        ensure!(bits <= self.len(), "Insufficient bits remaining");
        let (offset_bits, mut ret) = self.pop_from_offset(bits);
        bits -= offset_bits;
        while bits >= 8 {
            let popped = self.bytes.pop_front().expect("Already checked len");
            ret = (ret << 8) | popped as u64;
            bits -= 8;
        }
        if bits > 0 {
            self.offset = 8;
            let (offset_bits, offset_value) = self.pop_from_offset(bits);
            assert_eq!(offset_bits, bits);
            ret = (ret << bits) | offset_value;
        }
        Ok(ret)
    }
}

impl FromStr for Bitstream {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let digits: Vec<u8> = s.chars()
            .map(|c| c.to_digit(16)
                .ok_or_else(|| anyhow!("Invalid digit: {}", c))
                .and_then(|d| d.try_into().context("Cannot cast to u8")))
            .collect::<Result<_>>()?;
        let digits = digits.chunks(2)
            .map(|s| match s {
                &[c1, c2] => c1 << 4 | c2,
                &[c1] => c1 << 4,
                _ => panic!("impossible"),
            })
            .collect::<Vec<_>>();
        Ok(Bitstream::create(&digits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitstream() {
        // binary: 1111...
        let mut stream: Bitstream = "FFFFFFFF".parse().unwrap();
        assert_eq!(stream.len(), 32);
        assert_eq!(stream.pop_bits(3).unwrap(), 0b111);
        assert_eq!(stream.len(), 29);
        assert_eq!(stream.pop_bits(8).unwrap(), u8::MAX as u64);
        assert_eq!(stream.len(), 21);
        assert_eq!(stream.pop_bits(16).unwrap(), u16::MAX as u64);
        assert_eq!(stream.len(), 5);
        assert_eq!(stream.pop_bits(1).unwrap(), 0b1);
        assert_eq!(stream.len(), 4);
        assert!(stream.pop_bits(5).is_err());
        assert!(stream.pop_bits(15).is_err());
        assert_eq!(stream.pop_bits(4).unwrap(), 0b1111);
        assert_eq!(stream.len(), 0);

        // binary: 1010...
        let mut stream: Bitstream = "AAAA".parse().unwrap();
        assert_eq!(stream.len(), 16);
        assert_eq!(stream.pop_bits(3).unwrap(), 0b101);
        assert_eq!(stream.len(), 13);
        assert_eq!(stream.pop_bits(3).unwrap(), 0b010);
        assert_eq!(stream.len(), 10);
        assert_eq!(stream.pop_bits(4).unwrap(), 0b1010);
        assert_eq!(stream.len(), 6);
        assert_eq!(stream.pop_bits(5).unwrap(), 0b10101);
        assert_eq!(stream.len(), 1);
        assert_eq!(stream.pop_bits(1).unwrap(), 0b00);
        assert_eq!(stream.len(), 0);
    }

    static EXAMPLE_1: &str = "D2FE28";
    static EXAMPLE_2: &str = "38006F45291200";
    static EXAMPLE_3: &str = "EE00D40C823060";
    static EXAMPLE_4: &str = "8A004A801A8002F478";
    static EXAMPLE_5: &str = "620080001611562C8802118E34";
    static EXAMPLE_6: &str = "C0015000016115A2E0802F182340";
    static EXAMPLE_7: &str = "A0016C880162017C3686B18A3D4780";
    static EXAMPLE_8: &str = "C200B40A82";
    static EXAMPLE_9: &str = "04005AC33890";
    static EXAMPLE_10: &str = "880086C3E88112";
    static EXAMPLE_11: &str = "CE00C43D881120";
    static EXAMPLE_12: &str = "D8005AC2A8F0";
    static EXAMPLE_13: &str = "F600BC2D8F";
    static EXAMPLE_14: &str = "9C005AC2F8F0";
    static EXAMPLE_15: &str = "9C0141080250320F1802104A08";

    parameterized_test::create!{ versions, (input, versions), {
        let mut input: Bitstream = input.parse().unwrap();
        let packet = Packet::deserialize(&mut input).unwrap();
        assert_eq!(packet.sum_versions(), versions);
    } }
    versions! {
        example1: (EXAMPLE_1, 6),
        example2: (EXAMPLE_2, 1+6+2),
        example3: (EXAMPLE_3, 7+2+4+1),
        example4: (EXAMPLE_4, 16),
        example5: (EXAMPLE_5, 12),
        example6: (EXAMPLE_6, 23),
        example7: (EXAMPLE_7, 31),
    }

    parameterized_test::create!{ evals, (input, structure, evaluated), {
        let mut input: Bitstream = input.parse().unwrap();
        let packet = Packet::deserialize(&mut input).unwrap();
        assert_eq!(packet, structure);
        assert_eq!(packet.evaluate().unwrap(), evaluated);
    } }
    evals! {
        example1: (EXAMPLE_1, Packet::literal(6, 2021), 2021),
        example2: (EXAMPLE_2, Packet::lt(1, Packet::literal(6, 10), Packet::literal(2, 20)), 1),  // Not specified
        example3: (EXAMPLE_3, Packet::max(7, vec![Packet::literal(2, 1), Packet::literal(4, 2), Packet::literal(1, 3)]), 3),  // Not specified
        example4: (EXAMPLE_4, Packet::min(4, vec![Packet::min(1, vec![Packet::min(5, vec![Packet::literal(6, 15)])])]), 15),  // Not specified
        // Not specified
        example5: (EXAMPLE_5, Packet::sum(3, vec![
            Packet::sum(0, vec![Packet::literal(0, 10), Packet::literal(5, 11)]),
            Packet::sum(1, vec![Packet::literal(0, 12), Packet::literal(3, 13)])
        ]), 46),
        // Not specified
        example6: (EXAMPLE_6, Packet::sum(6, vec![
            Packet::sum(0, vec![Packet::literal(0, 10), Packet::literal(6, 11)]),
            Packet::sum(4, vec![Packet::literal(7, 12), Packet::literal(0, 13)]),
        ]), 46),
        // Not specified
        example7: (EXAMPLE_7, Packet::sum(5, vec![Packet::sum(1, vec![Packet::sum(3, vec![
            Packet::literal(7, 6),
            Packet::literal(6, 6),
            Packet::literal(5, 12),
            Packet::literal(2, 15),
            Packet::literal(2, 15),
        ])])]), 54),
        // finds the sum of 1 and 2
        example8: (EXAMPLE_8, Packet::sum(6, vec![Packet::literal(6, 1), Packet::literal(2, 2)]), 3),
        // finds the product of 6 and 9
        example9: (EXAMPLE_9, Packet::product(0, vec![Packet::literal(5, 6), Packet::literal(3, 9)]), 54),
        // finds the minimum of 7, 8, and 9
        example10: (EXAMPLE_10, Packet::min(4,
            vec![Packet::literal(5, 7), Packet::literal(6, 8), Packet::literal(0, 9)]), 7),
        // finds the maximum of 7, 8, and 9
        example11: (EXAMPLE_11, Packet::max(6,
            vec![Packet::literal(0, 7), Packet::literal(5, 8), Packet::literal(0, 9)]), 9),
        // produces 1, because 5 is less than 15
        example12: (EXAMPLE_12, Packet::lt(6, Packet::literal(5, 5), Packet::literal(2, 15)), 1),
        // produces 0, because 5 is not greater than 15
        example13: (EXAMPLE_13, Packet::gt(7, Packet::literal(7, 5), Packet::literal(5, 15)), 0),
        // produces 0, because 5 is not equal to 15
        example14: (EXAMPLE_14, Packet::eq(4, Packet::literal(5, 5), Packet::literal(7, 15)), 0),
        // produces 1, because 1 + 3 = 2 * 2
        example15: (EXAMPLE_15, Packet::eq(4,
            Packet::sum(2, vec![Packet::literal(2, 1), Packet::literal(4, 3)]),
            Packet::product(6, vec![Packet::literal(0, 2), Packet::literal(2, 2)]),
        ), 1),
    }
}
