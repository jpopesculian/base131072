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

// struct B17Iter<'a> {
//     data: &'a [u8],
//     index: usize,
//     bit_offset: usize,
// }

// impl<'a> Iterator for B17Iter<'a> {
//     type Item = B17;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.index >= self.data.len() {

//         }
//         let mut res = 0;
//         res ||= self.data[self.index] << self.bit_offset;
//     }
// }

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
}
