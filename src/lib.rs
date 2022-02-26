mod lookup_table;

use core::cmp::Ordering;
use lookup_table::{LOOKUP_TABLE, PAD1, PAD2};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct B17(u32);

impl B17 {
    fn encode(self) -> char {
        match LOOKUP_TABLE.binary_search_by_key(&self.0, |&(idx, _, _)| idx) {
            Ok(lookup_idx) => unsafe { char::from_u32_unchecked(LOOKUP_TABLE[lookup_idx].1) },
            Err(lookup_idx) => {
                let (idx, start, _) = LOOKUP_TABLE[lookup_idx - 1];
                unsafe { char::from_u32_unchecked(self.0 - idx + start) }
            }
        }
    }

    fn decode(ch: char) -> Option<Self> {
        let code_point = ch as u32;
        let lookup_idx = LOOKUP_TABLE
            .binary_search_by(|&(_, start, stop)| {
                if start > code_point {
                    Ordering::Greater
                } else if code_point > stop {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .ok()?;
        let (idx, start, _) = LOOKUP_TABLE[lookup_idx];
        Some(Self(code_point - start + idx))
    }
}

struct B8ToB17Iter<'a> {
    data: &'a [u8],
    index: usize,
    bit_offset: usize,
}

impl<'a> B8ToB17Iter<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            index: 0,
            bit_offset: 0,
        }
    }
}

impl<'a> Iterator for B8ToB17Iter<'a> {
    type Item = B17;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.len() {
            return None;
        }
        let mut res: u32 = 0;
        res |= ((self.data[self.index] << self.bit_offset) as u32) << 9;

        self.index += 1;
        if self.index >= self.data.len() {
            return Some(B17(res));
        }
        res |= (self.data[self.index] as u32) << (1 + self.bit_offset);

        self.index += 1;
        if self.index >= self.data.len() {
            return Some(B17(res));
        }
        res |= (self.data[self.index] >> (7 - self.bit_offset)) as u32;

        self.bit_offset += 1;
        if self.bit_offset > 7 {
            self.index += 1;
            self.bit_offset = 0;
        }
        Some(B17(res))
    }
}

struct B17ToB8Iter<'a> {
    data: &'a [B17],
    index: usize,
    bit_offset: usize,
}

impl<'a> B17ToB8Iter<'a> {
    fn new(data: &'a [B17]) -> Self {
        Self {
            data,
            index: 0,
            bit_offset: 0,
        }
    }
}

impl<'a> Iterator for B17ToB8Iter<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.len() {
            return None;
        }
        println!("this {:#019b}", self.data[self.index].0);
        println!("{}", self.bit_offset);
        if self.bit_offset > 9 {
            let mut next = (self.data[self.index].0 << self.bit_offset >> 9) as u8;
            println!("res* {:#010b} {}", next, next);
            self.index += 1;
            if self.index >= self.data.len() {
                if self.bit_offset == 17 {
                    return None;
                }
                return Some(next);
            }
            self.bit_offset -= 9;
            println!("next {:#019b}", self.data[self.index].0);
            next |= (self.data[self.index].0 >> (17 - self.bit_offset)) as u8;
            println!("res! {:#010b} {}", next, next);
            Some(next)
        } else {
            let next = (self.data[self.index].0 >> (9 - self.bit_offset)) as u8;
            self.bit_offset += 8;
            println!("res= {:#010b} {}", next, next);
            Some(next)
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Padding {
    Pad1,
    Pad2,
}

fn calc_padding(byte_size: usize) -> Option<Padding> {
    let bits = byte_size * 8;
    if bits % 17 == 0 {
        None
    } else {
        Some(match (16 - (bits % 17)) / 8 {
            0 => Padding::Pad1,
            1 => Padding::Pad2,
            _ => unreachable!(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b17_encoding() {
        const B17_TEST_CASES: &[u32] = &[
            0,
            1,
            2,
            (1 << 17) - 1,
            (1 << 17) - 2,
            (1 << 17) / 2,
            (1 << 17) / 3,
            (1 << 17) / 4,
            (1 << 17) / 5,
            (1 << 17) / 6,
            (1 << 17) / 7,
            (1 << 17) / 8,
            (1 << 17) / 9,
        ];
        for &test_case in B17_TEST_CASES {
            assert_eq!(
                B17::decode(B17(test_case).encode()).unwrap(),
                B17(test_case)
            )
        }
    }

    #[test]
    fn b8_to_b17_iter() {
        assert_eq!(B8ToB17Iter::new(&[]).collect::<Vec<_>>(), vec![]);
        assert_eq!(
            B8ToB17Iter::new(&[1]).collect::<Vec<_>>(),
            vec![
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0000_0)
            ]
        );
        assert_eq!(
            B8ToB17Iter::new(&[1, 2]).collect::<Vec<_>>(),
            vec![
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0010_0)
            ]
        );
        assert_eq!(
            B8ToB17Iter::new(&[1, 2, 3]).collect::<Vec<_>>(),
            vec![
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0010_0),
                B17(0b000_0011_0000_0000_00)
            ]
        );
        assert_eq!(
            B8ToB17Iter::new(&[
                0b0001_0001,
                0b0010_0010,
                0b0011_0011,
                0b0100_0100,
                0b0101_0101,
                0b0110_0110,
                0b0111_0111,
                0b1000_1000,
            ])
            .collect::<Vec<_>>(),
            vec![
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0001_0001_0010_0010_0),
                B17(0b011_0011_0100_0100_01),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b01_0101_0110_0110_011),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b1_0111_1000_1000_0000)
            ]
        );
        assert_eq!(
            B8ToB17Iter::new(&[
                0b0000_0001,
                0b0000_0010,
                0b0000_0011,
                0b0000_0100,
                0b0000_0101,
                0b0000_0110,
                0b0000_0111,
                0b0000_1000,
                0b0000_1001,
                0b0000_1010,
                0b0000_1011,
                0b0000_1100,
                0b0000_1101,
                0b0000_1110,
                0b0000_1111,
                0b0001_0000,
                0b0001_0001,
            ])
            .collect::<Vec<_>>(),
            vec![
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0010_0),
                B17(0b000_0011_0000_0100_00),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b00_0101_0000_0110_000),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0_0111_0000_1000_0000),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b1001_0000_1010_0000_1),
                B17(0b011_0000_1100_0000_11),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b01_0000_1110_0000_111),
                B17(0b1_0001_0000_0001_0001),
            ]
        );
        assert_eq!(
            B8ToB17Iter::new(&[
                0b0000_0001,
                0b0000_0010,
                0b0000_0011,
                0b0000_0100,
                0b0000_0101,
                0b0000_0110,
                0b0000_0111,
                0b0000_1000,
                0b0000_1001,
                0b0000_1010,
                0b0000_1011,
                0b0000_1100,
                0b0000_1101,
                0b0000_1110,
                0b0000_1111,
                0b0001_0000,
                0b0001_0001,
                0b0001_0010,
                0b0001_0011,
                0b0001_0100,
            ])
            .collect::<Vec<_>>(),
            vec![
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0010_0),
                B17(0b000_0011_0000_0100_00),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b00_0101_0000_0110_000),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0_0111_0000_1000_0000),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b1001_0000_1010_0000_1),
                B17(0b011_0000_1100_0000_11),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b01_0000_1110_0000_111),
                B17(0b1_0001_0000_0001_0001),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0001_0010_0001_0011_0),
                B17(0b001_0100_0000_0000_00),
            ]
        );
    }

    #[test]
    fn b17_to_b8_iter() {
        assert_eq!(B17ToB8Iter::new(&[]).collect::<Vec<_>>(), vec![]);
        assert_eq!(
            B17ToB8Iter::new(&[
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0000_0)
            ])
            .collect::<Vec<_>>(),
            vec![1, 0, 0]
        );
        assert_eq!(
            B17ToB8Iter::new(&[
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0010_0)
            ])
            .collect::<Vec<_>>(),
            vec![1, 2, 0],
        );
        println!();
        assert_eq!(
            B17ToB8Iter::new(&[
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0010_0),
                B17(0b000_0011_0000_0000_00)
            ])
            .collect::<Vec<_>>(),
            vec![1, 2, 3, 0, 0],
        );
        assert_eq!(
            B17ToB8Iter::new(&[
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0001_0001_0010_0010_0),
                B17(0b011_0011_0100_0100_01),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b01_0101_0110_0110_011),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b1_0111_1000_1000_0000)
            ])
            .collect::<Vec<_>>(),
            vec![
                0b0001_0001,
                0b0010_0010,
                0b0011_0011,
                0b0100_0100,
                0b0101_0101,
                0b0110_0110,
                0b0111_0111,
                0b1000_1000,
                0
            ]
        );
        assert_eq!(
            B17ToB8Iter::new(&[
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0010_0),
                B17(0b000_0011_0000_0100_00),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b00_0101_0000_0110_000),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0_0111_0000_1000_0000),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b1001_0000_1010_0000_1),
                B17(0b011_0000_1100_0000_11),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b01_0000_1110_0000_111),
                B17(0b1_0001_0000_0001_0001),
            ])
            .collect::<Vec<_>>(),
            vec![
                0b0000_0001,
                0b0000_0010,
                0b0000_0011,
                0b0000_0100,
                0b0000_0101,
                0b0000_0110,
                0b0000_0111,
                0b0000_1000,
                0b0000_1001,
                0b0000_1010,
                0b0000_1011,
                0b0000_1100,
                0b0000_1101,
                0b0000_1110,
                0b0000_1111,
                0b0001_0000,
                0b0001_0001,
            ]
        );
        assert_eq!(
            B17ToB8Iter::new(&[
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0000_0001_0000_0010_0),
                B17(0b000_0011_0000_0100_00),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b00_0101_0000_0110_000),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0_0111_0000_1000_0000),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b1001_0000_1010_0000_1),
                B17(0b011_0000_1100_0000_11),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b01_0000_1110_0000_111),
                B17(0b1_0001_0000_0001_0001),
                #[allow(clippy::unusual_byte_groupings)]
                B17(0b0001_0010_0001_0011_0),
                B17(0b001_0100_0000_0000_00),
            ])
            .collect::<Vec<_>>(),
            vec![
                0b0000_0001,
                0b0000_0010,
                0b0000_0011,
                0b0000_0100,
                0b0000_0101,
                0b0000_0110,
                0b0000_0111,
                0b0000_1000,
                0b0000_1001,
                0b0000_1010,
                0b0000_1011,
                0b0000_1100,
                0b0000_1101,
                0b0000_1110,
                0b0000_1111,
                0b0001_0000,
                0b0001_0001,
                0b0001_0010,
                0b0001_0011,
                0b0001_0100,
                0,
                0
            ]
        );
    }

    #[test]
    fn test_padding() {
        assert_eq!(calc_padding(0), None);
        assert_eq!(calc_padding(1), Some(Padding::Pad2));
        assert_eq!(calc_padding(2), Some(Padding::Pad1));
        assert_eq!(calc_padding(3), Some(Padding::Pad2));
        assert_eq!(calc_padding(8), Some(Padding::Pad1));
        assert_eq!(calc_padding(17), None);
        assert_eq!(calc_padding(20), Some(Padding::Pad2));
    }
}
