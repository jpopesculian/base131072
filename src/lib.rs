//! I originally made this crate in order to pack some data into tweets. However halfway through
//! making the crate, I discovered [with the help of a very helpful
//! table](https://github.com/qntm/base2048) that Twitter weights its characters, and that
//! Base131072 is not actually the most efficient way to encode information on Twitter, but rather
//! Base2048. [Another very good crate](https://docs.rs/base2048/2.0.2/base2048) implements
//! Base2048.
//!
//! However, this crate should still work, should you want to encode something Base131072 for some
//! reason!

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

mod lookup_table;

use core::cmp::Ordering;
use core::fmt;
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
        if self.bit_offset > 9 {
            let mut next = (self.data[self.index].0 << (self.bit_offset - 9)) as u8;
            self.index += 1;
            if self.index >= self.data.len() {
                if self.bit_offset == 17 {
                    return None;
                }
                return Some(next);
            }
            self.bit_offset -= 9;
            next |= (self.data[self.index].0 >> (17 - self.bit_offset)) as u8;
            Some(next)
        } else {
            let next = (self.data[self.index].0 >> (9 - self.bit_offset)) as u8;
            self.bit_offset += 8;
            Some(next)
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Padding {
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

/// Encode some bytes to a base131072 encoded string
pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    let input = input.as_ref();
    let mut out = String::with_capacity(input.len() * 8 / 17);
    for b17 in B8ToB17Iter::new(input) {
        out.push(b17.encode());
    }
    if let Some(padding) = calc_padding(input.len()) {
        match padding {
            Padding::Pad1 => out.push(unsafe { char::from_u32_unchecked(PAD1) }),
            Padding::Pad2 => out.push(unsafe { char::from_u32_unchecked(PAD2) }),
        }
    }
    out
}

/// The error encountered when decoding an invalid Base2048 string
#[derive(Debug, Clone, Copy)]
pub struct InvalidChar(pub usize, pub char);

impl fmt::Display for InvalidChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "invalid char '{}' encountered at character number {}",
            self.1, self.0
        ))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidChar {}

/// Decode a base131072 encoded string
pub fn decode<T: AsRef<str>>(input: T) -> Result<Vec<u8>, InvalidChar> {
    let mut string = input.as_ref();
    if string.is_empty() {}
    let padding = if let Some(ch) = string.chars().last() {
        match ch as u32 {
            PAD1 => 1,
            PAD2 => 2,
            _ => 0,
        }
    } else {
        return Ok(Vec::new());
    };
    if padding > 0 {
        let last_char_index = string.char_indices().last().unwrap().0;
        string = &string[..last_char_index];
    }
    let mut b17s = Vec::with_capacity(string.len());
    for (idx, ch) in string.chars().enumerate() {
        if let Some(b17) = B17::decode(ch) {
            b17s.push(b17);
        } else {
            return Err(InvalidChar(idx, ch));
        }
    }
    let mut bytes = B17ToB8Iter::new(&b17s).collect::<Vec<_>>();
    bytes.truncate(bytes.len() - padding);
    Ok(bytes)
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

    #[test]
    fn encoding() {
        assert_eq!(decode(encode(&[])).unwrap(), &[]);
        assert_eq!(decode(encode(&[1])).unwrap(), &[1]);
        assert_eq!(decode(encode(&[1, 2])).unwrap(), &[1, 2]);
        assert_eq!(decode(encode(&[1, 2, 3])).unwrap(), &[1, 2, 3]);
        assert_eq!(
            decode(encode((0..17).collect::<Vec<_>>())).unwrap(),
            (0..17).collect::<Vec<_>>()
        );
        assert_eq!(
            decode(encode((0..255).collect::<Vec<_>>())).unwrap(),
            (0..255).collect::<Vec<_>>()
        );
        assert_eq!(decode(encode(vec![100; 1024])).unwrap(), vec![100; 1024]);
    }
}
