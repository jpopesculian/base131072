use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process::Command;

const LOOKUP_TABLE_SIZE: usize = (1 << 17) + 2;

const OUT_FILE: &str = "src/lookup_table.rs";

const UNICODE_DATA_PATH: &str = "ucd/UnicodeData.txt";
const HANGUL_DATA_PATH: &str = "ucd/HangulSyllableType.txt";
const UNIHAN_DATA_PATHS: &[&str] = &[
    "ucd/Unihan_IRGSources.txt",
    "ucd/Unihan_NumericValues.txt",
    "ucd/Unihan_OtherMappings.txt",
    "ucd/Unihan_RadicalStrokeCounts.txt",
    "ucd/Unihan_Readings.txt",
    "ucd/Unihan_Variants.txt",
];

fn parse_code_point(code_point: &str) -> io::Result<u32> {
    u32::from_str_radix(code_point, 16).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("bad unicode code point: {err}"),
        )
    })
}

fn read_all_codepoints() -> io::Result<Vec<u32>> {
    let mut code_points = BTreeSet::new();

    for line_res in BufReader::new(File::open(UNICODE_DATA_PATH)?).lines() {
        let line = line_res?;
        let mut attrs = line.split(';');
        let code_point = attrs.next().unwrap();
        let descriptor = attrs.next().unwrap();
        if descriptor.starts_with('<') {
            continue;
        }
        code_points.insert(parse_code_point(code_point)?);
    }

    for line_res in BufReader::new(File::open(HANGUL_DATA_PATH)?).lines() {
        let line = line_res?;
        if line.is_empty() {
            continue;
        }
        let mut attrs = line.split(' ');
        let code_point = attrs.next().unwrap();
        if code_point.starts_with('#') {
            continue;
        }
        let mut code_point_range = code_point.split("..");
        let code_point_start = parse_code_point(code_point_range.next().unwrap())?;
        if let Some(code_point_end) = code_point_range.next() {
            for code_point in code_point_start..parse_code_point(code_point_end)? {
                code_points.insert(code_point);
            }
        } else {
            code_points.insert(code_point_start);
        }
    }

    for path in UNIHAN_DATA_PATHS {
        for line_res in BufReader::new(File::open(path)?).lines() {
            let line = line_res?;
            let mut attrs = line.split('\t');
            let code_point = attrs.next().unwrap();
            if !code_point.starts_with("U+") {
                continue;
            }
            code_points.insert(parse_code_point(&code_point[2..])?);
        }
    }

    Ok(code_points.into_iter().take(LOOKUP_TABLE_SIZE).collect())
}

pub struct Ranges<'a> {
    values: &'a [u32],
    index: usize,
}

impl<'a> Ranges<'a> {
    pub fn new(values: &'a [u32]) -> Self {
        Self { values, index: 0 }
    }
}

impl<'a> Iterator for Ranges<'a> {
    type Item = (u32, u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.values.len() {
            return None;
        }
        let index_start = self.index;
        let mut range = (
            index_start as u32,
            self.values[self.index],
            self.values[self.index],
        );
        loop {
            self.index += 1;
            if self.index >= self.values.len() {
                return Some(range);
            }
            let new_end = self.values[self.index];
            if new_end == range.2 + 1 {
                range.2 = new_end;
            } else {
                return Some(range);
            }
        }
    }
}

fn main() -> io::Result<()> {
    if Path::new(OUT_FILE).exists() {
        return Ok(());
    }
    let mut code_points = read_all_codepoints()?;
    let pad1 = code_points.pop().unwrap();
    let pad2 = code_points.pop().unwrap();
    let code_point_ranges = Ranges::new(&code_points).collect::<Vec<_>>();
    let code = format!(
        r#"
//! AUTOMATICALLY GENERATED! DO NOT EDIT
//!
//! If you wish to update this file, do the following:
//! 1. Download a the UCD with the `download-ucd.sh` script
//! 2. Delete the `src/lookup_table.rs` file
//! 3. Run `cargo build`

/// The extra symbol if the encoding was padded by 1 byte
pub const PAD1: u32 = {pad1};
/// The extra symbol if the encoding was padded by 2 bytes
pub const PAD2: u32 = {pad2};
/// A Lookup Table with `(table_offset, range_start, range_end)` of valid unicode code points
pub const LOOKUP_TABLE: &[(u32, u32, u32)] = &{code_point_ranges:?};
    "#
    );
    fs::write(OUT_FILE, code)?;
    Command::new("rustfmt")
        .args([OUT_FILE])
        .output()
        .expect("failed to execute rustfmt");
    Ok(())
}
