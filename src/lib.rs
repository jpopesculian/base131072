mod lookup_table;

use core::cmp::Ordering;
use lookup_table::LOOKUP_TABLE;

#[derive(Debug, Clone, Eq, PartialEq)]
enum Error {
    InvalidCodePoint(u32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct B17(u32);

impl B17 {
    fn encode(self) -> u32 {
        match LOOKUP_TABLE.binary_search_by_key(&self.0, |&(idx, _, _)| idx) {
            Ok(lookup_idx) => LOOKUP_TABLE[lookup_idx].1,
            Err(lookup_idx) => {
                let (idx, start, _) = LOOKUP_TABLE[lookup_idx - 1];
                println!("{} {} {}", self.0, idx, start);
                self.0 - idx + start
            }
        }
    }

    fn decode(code_point: u32) -> Result<Self, Error> {
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
            .map_err(|_| Error::InvalidCodePoint(code_point))?;
        let (idx, start, _) = LOOKUP_TABLE[lookup_idx];
        Ok(Self(code_point - start + idx))
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
}
