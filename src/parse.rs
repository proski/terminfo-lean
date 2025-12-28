// Copyright 2025 Pavel Roskin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Parsing terminfo database files

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read, Seek, SeekFrom},
    mem,
};

const ABSENT_ENTRY: i32 = -1;
const CANCELED_ENTRY: i32 = -2;

const BOOL_NAMES: [&str; 44] = [
    "bw", "am", "xsb", "xhp", "xenl", "eo", "gn", "hc", "km", "hs", "in", "db", "da", "mir",
    "msgr", "os", "eslok", "xt", "hz", "ul", "xon", "nxon", "mc5i", "chts", "nrrmc", "npc",
    "ndscr", "ccc", "bce", "hls", "xhpa", "crxm", "daisy", "xvpa", "sam", "cpix", "lpix", "OTbs",
    "OTns", "OTnc", "OTMT", "OTNL", "OTpt", "OTxr",
];

const NUM_NAMES: [&str; 39] = [
    "cols", "it", "lines", "lm", "xmc", "pb", "vt", "wsl", "nlab", "lh", "lw", "ma", "wnum",
    "colors", "pairs", "ncv", "bufsz", "spinv", "spinh", "maddr", "mjump", "mcs", "mls", "npins",
    "orc", "orl", "orhi", "orvi", "cps", "widcs", "btns", "bitwin", "bitype", "UTug", "OTdC",
    "OTdN", "OTdB", "OTdT", "OTkn",
];

const STR_NAMES: [&str; 414] = [
    "cbt", "bel", "cr", "csr", "tbc", "clear", "el", "ed", "hpa", "cmdch", "cup", "cud1", "home",
    "civis", "cub1", "mrcup", "cnorm", "cuf1", "ll", "cuu1", "cvvis", "dch1", "dl1", "dsl", "hd",
    "smacs", "blink", "bold", "smcup", "smdc", "dim", "smir", "invis", "prot", "rev", "smso",
    "smul", "ech", "rmacs", "sgr0", "rmcup", "rmdc", "rmir", "rmso", "rmul", "flash", "ff", "fsl",
    "is1", "is2", "is3", "if", "ich1", "il1", "ip", "kbs", "ktbc", "kclr", "kctab", "kdch1",
    "kdl1", "kcud1", "krmir", "kel", "ked", "kf0", "kf1", "kf10", "kf2", "kf3", "kf4", "kf5",
    "kf6", "kf7", "kf8", "kf9", "khome", "kich1", "kil1", "kcub1", "kll", "knp", "kpp", "kcuf1",
    "kind", "kri", "khts", "kcuu1", "rmkx", "smkx", "lf0", "lf1", "lf10", "lf2", "lf3", "lf4",
    "lf5", "lf6", "lf7", "lf8", "lf9", "rmm", "smm", "nel", "pad", "dch", "dl", "cud", "ich",
    "indn", "il", "cub", "cuf", "rin", "cuu", "pfkey", "pfloc", "pfx", "mc0", "mc4", "mc5", "rep",
    "rs1", "rs2", "rs3", "rf", "rc", "vpa", "sc", "ind", "ri", "sgr", "hts", "wind", "ht", "tsl",
    "uc", "hu", "iprog", "ka1", "ka3", "kb2", "kc1", "kc3", "mc5p", "rmp", "acsc", "pln", "kcbt",
    "smxon", "rmxon", "smam", "rmam", "xonc", "xoffc", "enacs", "smln", "rmln", "kbeg", "kcan",
    "kclo", "kcmd", "kcpy", "kcrt", "kend", "kent", "kext", "kfnd", "khlp", "kmrk", "kmsg", "kmov",
    "knxt", "kopn", "kopt", "kprv", "kprt", "krdo", "kref", "krfr", "krpl", "krst", "kres", "ksav",
    "kspd", "kund", "kBEG", "kCAN", "kCMD", "kCPY", "kCRT", "kDC", "kDL", "kslt", "kEND", "kEOL",
    "kEXT", "kFND", "kHLP", "kHOM", "kIC", "kLFT", "kMSG", "kMOV", "kNXT", "kOPT", "kPRV", "kPRT",
    "kRDO", "kRPL", "kRIT", "kRES", "kSAV", "kSPD", "kUND", "rfi", "kf11", "kf12", "kf13", "kf14",
    "kf15", "kf16", "kf17", "kf18", "kf19", "kf20", "kf21", "kf22", "kf23", "kf24", "kf25", "kf26",
    "kf27", "kf28", "kf29", "kf30", "kf31", "kf32", "kf33", "kf34", "kf35", "kf36", "kf37", "kf38",
    "kf39", "kf40", "kf41", "kf42", "kf43", "kf44", "kf45", "kf46", "kf47", "kf48", "kf49", "kf50",
    "kf51", "kf52", "kf53", "kf54", "kf55", "kf56", "kf57", "kf58", "kf59", "kf60", "kf61", "kf62",
    "kf63", "el1", "mgc", "smgl", "smgr", "fln", "sclk", "dclk", "rmclk", "cwin", "wingo", "hup",
    "dial", "qdial", "tone", "pulse", "hook", "pause", "wait", "u0", "u1", "u2", "u3", "u4", "u5",
    "u6", "u7", "u8", "u9", "op", "oc", "initc", "initp", "scp", "setf", "setb", "cpi", "lpi",
    "chr", "cvr", "defc", "swidm", "sdrfq", "sitm", "slm", "smicm", "snlq", "snrmq", "sshm",
    "ssubm", "ssupm", "sum", "rwidm", "ritm", "rlm", "rmicm", "rshm", "rsubm", "rsupm", "rum",
    "mhpa", "mcud1", "mcub1", "mcuf1", "mvpa", "mcuu1", "porder", "mcud", "mcub", "mcuf", "mcuu",
    "scs", "smgb", "smgbp", "smglp", "smgrp", "smgt", "smgtp", "sbim", "scsd", "rbim", "rcsd",
    "subcs", "supcs", "docr", "zerom", "csnm", "kmous", "minfo", "reqmp", "getm", "setaf", "setab",
    "pfxl", "devt", "csin", "s0ds", "s1ds", "s2ds", "s3ds", "smglr", "smgtb", "birep", "binel",
    "bicr", "colornm", "defbi", "endbi", "setcolor", "slines", "dispc", "smpch", "rmpch", "smsc",
    "rmsc", "pctrm", "scesc", "scesa", "ehhlm", "elhlm", "elohlm", "erhlm", "ethlm", "evhlm",
    "sgr1", "slength", "OTi2", "OTrs", "OTnl", "OTbs", "OTko", "OTma", "OTG2", "OTG3", "OTG1",
    "OTG4", "OTGR", "OTGL", "OTGU", "OTGD", "OTGH", "OTGV", "OTGC", "meml", "memu", "box1",
];

#[repr(u16)]
enum TerminfoMagic {
    /// Original format, 16-bit numbers
    Magic1 = 0x011a,
    /// 32-bit numbers
    Magic2 = 0x021e,
}

/// Errors reported when parsing a terminfo database
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The magic number is invalid or unsupported
    #[error("Unknown magic number")]
    BadMagic,
    /// A string is not terminated by the NUL byte
    #[error("String without final NUL")]
    UnterminatedString,
    /// Unexpected condition, probably invalid terminfo database
    #[error("Unsupported terminfo format")]
    UnsupportedFormat,
    /// Boolean value not 0 or 1, probably invalid terminfo database
    #[error("Invalid boolean value {0}")]
    InvalidBooleanValue(u8),
    /// Input/output error, probably truncated terminfo database
    #[error("I/O error")]
    IO(#[from] std::io::Error),
    /// A string is not valid UTF-8
    #[error("Invalid UTF-8 string")]
    Utf8(#[from] std::str::Utf8Error),
}

/// Parse terminfo database from the supplied buffer
///
/// Returns `Terminfo` instance with data populated from the buffer.
pub fn parse(buffer: &[u8]) -> Result<Terminfo<'_>, Error> {
    let mut terminfo = Terminfo::new();
    let mut reader = Cursor::new(buffer);
    terminfo.parse_base(&mut reader)?;
    match terminfo.parse_extended(&mut reader) {
        Ok(()) | Err(Error::IO(_)) => {} // missing extended data is OK
        Err(err) => return Err(err),
    }
    Ok(terminfo)
}

fn read_u8(reader: &mut impl Read) -> Result<u8, Error> {
    let mut buffer = [0u8; 1];
    reader.read_exact(&mut buffer)?;
    Ok(buffer[0])
}

fn read_le16(reader: &mut impl Read) -> Result<u16, Error> {
    let mut buffer = [0u8; 2];
    reader.read_exact(&mut buffer)?;
    let value = u16::from_le_bytes(buffer);
    Ok(value)
}

fn read_slice<'a>(reader: &mut Cursor<&'a [u8]>, size: usize) -> Result<&'a [u8], Error> {
    let start = reader.position() as usize;
    let end = reader.seek(SeekFrom::Current(size as i64))? as usize;
    let buffer = &reader.get_ref();
    match buffer.get(start..end) {
        Some(slice) => Ok(slice),
        None => Err(Error::UnsupportedFormat),
    }
}

fn get_string(string_table: &[u8], offset: usize) -> Result<&[u8], Error> {
    let Some(string_slice) = string_table.get(offset..) else {
        return Err(Error::UnsupportedFormat);
    };
    if let Some(string_length) = &string_slice.iter().position(|c| *c == b'\0') {
        Ok(&string_table[offset..offset + string_length])
    } else {
        Err(Error::UnterminatedString)
    }
}

/// Convert ABSENT and CANCELED to None
fn check_offset(size: u16) -> Option<usize> {
    match i32::from(size as i16) {
        ABSENT_ENTRY | CANCELED_ENTRY => None,
        _ => Some(usize::from(size)),
    }
}

/// Skip a byte if needed to ensure 2-byte alignment
fn align_cursor(reader: &mut Cursor<&[u8]>) -> Result<(), Error> {
    let position = reader.position();
    if position & 1 == 1 {
        reader.seek_relative(1)?;
    }
    Ok(())
}

/// Parsed terminfo entry
#[derive(Debug)]
pub struct Terminfo<'a> {
    pub booleans: BTreeSet<&'a str>,
    pub numbers: BTreeMap<&'a str, i32>,
    pub strings: BTreeMap<&'a str, &'a [u8]>,
    number_size: usize,
}

impl<'a> Terminfo<'a> {
    fn new() -> Self {
        Self {
            booleans: BTreeSet::default(),
            numbers: BTreeMap::default(),
            strings: BTreeMap::default(),
            number_size: 0,
        }
    }

    fn read_number(&self, reader: &mut Cursor<&'a [u8]>) -> Result<Option<i32>, Error> {
        let value = if self.number_size == 4 {
            let mut buffer = [0u8; 4];
            reader.read_exact(&mut buffer)?;
            i32::from_le_bytes(buffer)
        } else {
            let mut buffer = [0u8; 2];
            reader.read_exact(&mut buffer)?;
            i32::from(i16::from_le_bytes(buffer))
        };
        if value > 0 { Ok(Some(value)) } else { Ok(None) }
    }

    /// Parse base capabilities
    fn parse_base(&mut self, mut reader: &mut Cursor<&'a [u8]>) -> Result<(), Error> {
        let magic = read_le16(&mut reader)?;
        let name_size = usize::from(read_le16(&mut reader)?);
        let bool_count = usize::from(read_le16(&mut reader)?);
        let num_count = usize::from(read_le16(&mut reader)?);
        let str_count = usize::from(read_le16(&mut reader)?);
        let str_size = usize::from(read_le16(&mut reader)?);

        self.number_size = match magic {
            val if val == TerminfoMagic::Magic1 as u16 => 2,
            val if val == TerminfoMagic::Magic2 as u16 => 4,
            _ => return Err(Error::BadMagic),
        };

        if bool_count > BOOL_NAMES.len()
            || num_count > NUM_NAMES.len()
            || str_count > STR_NAMES.len()
        {
            return Err(Error::UnsupportedFormat);
        }

        // Skip terminal names/aliases, we are not using them
        reader.seek_relative(name_size as i64)?;

        for name in BOOL_NAMES.iter().take(bool_count) {
            let value = read_u8(&mut reader)?;
            match value {
                0 => continue,
                1 => {}
                value => return Err(Error::InvalidBooleanValue(value)),
            }
            self.booleans.insert(*name);
        }

        align_cursor(reader)?;

        for name in NUM_NAMES.iter().take(num_count) {
            if let Some(number) = self.read_number(reader)? {
                self.numbers.insert(*name, number);
            }
        }

        let str_offsets = read_slice(reader, mem::size_of::<u16>() * str_count)?;
        let mut str_offsets_reader = Cursor::new(str_offsets);

        let str_table = read_slice(reader, str_size)?;

        for name in STR_NAMES.iter().take(str_count) {
            let offset = read_le16(&mut str_offsets_reader)?;
            let Some(offset) = check_offset(offset) else {
                continue;
            };
            let value = get_string(str_table, offset)?;
            self.strings.insert(*name, value);
        }

        Ok(())
    }

    /// Parse extended capabilities
    fn parse_extended(&mut self, mut reader: &mut Cursor<&'a [u8]>) -> Result<(), Error> {
        align_cursor(reader)?;

        let bool_count = usize::from(read_le16(&mut reader)?);
        let num_count = usize::from(read_le16(&mut reader)?);
        let str_count = usize::from(read_le16(&mut reader)?);
        let _ext_str_usage = usize::from(read_le16(&mut reader)?);
        let str_limit = usize::from(read_le16(&mut reader)?);

        let bools = read_slice(reader, bool_count)?;
        let mut bools_reader = Cursor::new(bools);
        align_cursor(reader)?;

        let nums = read_slice(reader, self.number_size * num_count)?;
        let mut nums_reader = Cursor::new(nums);

        let strs = read_slice(reader, mem::size_of::<u16>() * str_count)?;
        let mut strs_reader = Cursor::new(strs);

        let name_count = bool_count + num_count + str_count;
        let names = read_slice(reader, mem::size_of::<u16>() * name_count)?;
        let mut names_reader = Cursor::new(names);

        let str_table = read_slice(reader, str_limit)?;

        let mut names_base = 0;
        loop {
            let Ok(offset) = read_le16(&mut strs_reader) else {
                break;
            };
            let Some(offset) = check_offset(offset) else {
                continue;
            };
            names_base += get_string(str_table, offset)?.len() + 1;
        }

        let Some(names_table) = &str_table.get(names_base..) else {
            return Err(Error::UnsupportedFormat);
        };

        loop {
            let Ok(value) = read_u8(&mut bools_reader) else {
                break;
            };
            let Ok(name_offset) = read_le16(&mut names_reader) else {
                return Err(Error::UnsupportedFormat);
            };
            match value {
                0 => continue,
                1 => {}
                value => return Err(Error::InvalidBooleanValue(value)),
            }
            let Some(name_offset) = check_offset(name_offset) else {
                return Err(Error::UnsupportedFormat);
            };
            let name = get_string(names_table, name_offset)?;
            self.booleans.insert(str::from_utf8(name)?);
        }

        loop {
            let Ok(value) = self.read_number(&mut nums_reader) else {
                break;
            };
            let Ok(name_offset) = read_le16(&mut names_reader) else {
                return Err(Error::UnsupportedFormat);
            };
            let Some(value) = value else {
                continue;
            };
            let Some(name_offset) = check_offset(name_offset) else {
                return Err(Error::UnsupportedFormat);
            };
            let name = get_string(names_table, name_offset)?;
            self.numbers.insert(str::from_utf8(name)?, value);
        }

        strs_reader.set_position(0);
        loop {
            let Ok(str_offset) = read_le16(&mut strs_reader) else {
                break;
            };
            let Ok(name_offset) = read_le16(&mut names_reader) else {
                return Err(Error::UnsupportedFormat);
            };
            if let (Some(str_offset), Some(name_offset)) =
                (check_offset(str_offset), check_offset(name_offset))
            {
                let value = get_string(str_table, str_offset)?;
                let name = get_string(names_table, name_offset)?;
                self.strings.insert(str::from_utf8(name)?, value);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use collection_literals::collection;

    use super::*;

    #[derive(Clone, Copy, PartialEq)]
    enum NumberType {
        U16,
        U32,
    }

    #[derive(Clone, PartialEq)]
    enum StringValue {
        Present(Vec<u8>),
        Absent,
        Canceled,
    }

    // Allow conversion from `StringValue` to `Option`
    impl<'a> From<&'a StringValue> for Option<&'a [u8]> {
        fn from(value: &'a StringValue) -> Self {
            match value {
                StringValue::Present(value) => Some(value),
                _ => None,
            }
        }
    }

    // Allow iteration over `StringValue` by converting it to `Option`
    impl<'a> IntoIterator for &'a StringValue {
        type Item = &'a [u8];
        type IntoIter = std::option::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            Option::<&'a [u8]>::from(self).into_iter()
        }
    }

    impl<const N: usize> From<&[u8; N]> for StringValue {
        fn from(value: &[u8; N]) -> Self {
            Self::Present(value.to_vec())
        }
    }

    // Size of byte string in memory with terminating NUL
    fn memlen(byte_string: &[u8]) -> u16 {
        byte_string.len() as u16 + 1
    }

    struct DataSet {
        number_type: NumberType,
        term_name: Vec<u8>,
        base_booleans: Vec<u8>,
        base_numbers: Vec<i32>,
        base_strings: Vec<StringValue>,
        ext_booleans: Vec<(&'static [u8], u8)>,
        ext_numbers: Vec<(&'static [u8], i32)>,
        ext_strings: Vec<(&'static [u8], StringValue)>,
    }

    impl Default for DataSet {
        fn default() -> Self {
            Self {
                number_type: NumberType::U16,
                term_name: b"myterm".to_vec(),
                base_booleans: vec![1, 0, 0, 0, 1],
                base_numbers: vec![80, -2, 25, -1, -10, 0x10005],
                base_strings: vec![
                    StringValue::Absent,
                    StringValue::from(b"Hello"),
                    StringValue::Canceled,
                    StringValue::Absent,
                    StringValue::from(b"World!"),
                ],
                ext_booleans: vec![(b"Curly", 1), (b"Italic", 1), (b"Semi-bold", 1)],
                ext_numbers: vec![(b"Shades", 1100), (b"Variants", 2200)],
                ext_strings: vec![
                    (b"Colors", StringValue::from(b"A lot")),
                    (b"Luminosity", StringValue::from(b"Positive")),
                    (b"Ideas", StringValue::Absent),
                ],
            }
        }
    }

    fn make_buffer(data_set: &DataSet, add_ext: bool) -> Vec<u8> {
        let magic = match data_set.number_type {
            NumberType::U16 => 0x011a,
            NumberType::U32 => 0x021e,
        };
        let str_size = data_set.base_strings.iter().flatten().map(memlen).sum();

        let mut buffer = vec![];
        buffer.extend_from_slice(&u16::to_le_bytes(magic));
        buffer.extend_from_slice(&u16::to_le_bytes(memlen(&data_set.term_name)));
        buffer.extend_from_slice(&u16::to_le_bytes(data_set.base_booleans.len() as u16));
        buffer.extend_from_slice(&u16::to_le_bytes(data_set.base_numbers.len() as u16));
        buffer.extend_from_slice(&u16::to_le_bytes(data_set.base_strings.len() as u16));
        buffer.extend_from_slice(&u16::to_le_bytes(str_size));
        buffer.extend_from_slice(&data_set.term_name);
        buffer.push(0);
        buffer.extend_from_slice(&data_set.base_booleans);
        if !buffer.len().is_multiple_of(2) {
            buffer.push(0);
        }
        for number in &data_set.base_numbers {
            match data_set.number_type {
                NumberType::U16 => buffer.extend_from_slice(&u16::to_le_bytes(*number as u16)),
                NumberType::U32 => buffer.extend_from_slice(&u32::to_le_bytes(*number as u32)),
            }
        }
        let mut offset = 0;
        for string in &data_set.base_strings {
            match string {
                StringValue::Present(string) => {
                    buffer.extend_from_slice(&u16::to_le_bytes(offset));
                    offset += memlen(string);
                }
                StringValue::Absent => buffer.extend_from_slice(&u16::to_le_bytes(0xffff)),
                StringValue::Canceled => buffer.extend_from_slice(&u16::to_le_bytes(0xfffe)),
            }
        }
        for string in data_set.base_strings.iter().flatten() {
            buffer.extend_from_slice(string);
            buffer.push(0);
        }
        if add_ext {
            if !buffer.len().is_multiple_of(2) {
                buffer.push(0);
            }
            buffer.append(&mut make_ext_buffer(data_set));
        }
        buffer
    }

    fn make_ext_buffer(data_set: &DataSet) -> Vec<u8> {
        let booleans = &data_set.ext_booleans;
        let numbers = &data_set.ext_numbers;
        let strings = &data_set.ext_strings;

        let boolean_name_size: u16 = booleans.iter().map(|x| memlen(x.0)).sum();
        let number_name_size: u16 = numbers.iter().map(|x| memlen(x.0)).sum();
        let string_name_size: u16 = strings.iter().map(|x| memlen(x.0)).sum();
        let string_value_size: u16 = strings.iter().flat_map(|x| &x.1).map(memlen).sum();
        let name_size = boolean_name_size + number_name_size + string_name_size;
        let string_size = name_size + string_value_size;

        let mut buffer = vec![];

        // The layout is:
        //
        // extended header, boolean values, align(2), number values, string value offsets,
        // name offsets, string values, boolean names, number names, string names.

        buffer.extend_from_slice(&u16::to_le_bytes(booleans.len() as u16));
        buffer.extend_from_slice(&u16::to_le_bytes(numbers.len() as u16));
        buffer.extend_from_slice(&u16::to_le_bytes(strings.len() as u16));
        buffer.extend_from_slice(&u16::to_le_bytes(0u16)); // unused `ext_str_usage`
        buffer.extend_from_slice(&u16::to_le_bytes(string_size));

        for boolean in booleans {
            buffer.push(boolean.1);
        }
        if !buffer.len().is_multiple_of(2) {
            buffer.push(0);
        }
        for number in numbers {
            match data_set.number_type {
                NumberType::U16 => buffer.extend_from_slice(&u16::to_le_bytes(number.1 as u16)),
                NumberType::U32 => buffer.extend_from_slice(&u32::to_le_bytes(number.1 as u32)),
            }
        }
        let mut offset = 0;
        for string in strings {
            match &string.1 {
                StringValue::Present(string) => {
                    buffer.extend_from_slice(&u16::to_le_bytes(offset));
                    offset += memlen(string);
                }
                StringValue::Absent => buffer.extend_from_slice(&u16::to_le_bytes(0xffff)),
                StringValue::Canceled => buffer.extend_from_slice(&u16::to_le_bytes(0xfffe)),
            }
        }

        offset = 0;
        for boolean in booleans {
            buffer.extend_from_slice(&u16::to_le_bytes(offset));
            offset += memlen(boolean.0);
        }
        for number in numbers {
            buffer.extend_from_slice(&u16::to_le_bytes(offset));
            offset += memlen(number.0);
        }
        for string in strings {
            buffer.extend_from_slice(&u16::to_le_bytes(offset));
            offset += memlen(string.0);
        }

        for string in strings {
            if let StringValue::Present(string) = &string.1 {
                buffer.extend_from_slice(string);
                buffer.push(0);
            }
        }

        for boolean in booleans {
            buffer.extend_from_slice(boolean.0);
            buffer.push(0);
        }
        for number in numbers {
            buffer.extend_from_slice(number.0);
            buffer.push(0);
        }
        for string in strings {
            buffer.extend_from_slice(string.0);
            buffer.push(0);
        }

        buffer
    }

    #[test]
    fn empty_buffer() {
        let terminfo = parse(b"");
        assert!(matches!(terminfo.unwrap_err(), Error::IO(_)));
    }

    #[test]
    fn base_16_bit() {
        let data_set = DataSet::default();
        let buffer = make_buffer(&data_set, false);
        let terminfo = parse(buffer.as_slice()).unwrap();
        assert_eq!(terminfo.booleans, collection!("bw", "xenl"));
        assert_eq!(
            terminfo.numbers,
            collection!(
                "cols" => 80,
                "lines" => 25,
                "pb" => 5,
            )
        );
        assert_eq!(
            terminfo.strings,
            collection!(
                "bel" => b"Hello".as_slice(),
                "tbc" => b"World!",
            )
        );
    }

    #[test]
    fn base_32_bit() {
        let mut data_set = DataSet {
            number_type: NumberType::U32,
            ..Default::default()
        };
        data_set.base_numbers[5] = 0x7fff_ffff;

        let buffer = make_buffer(&data_set, false);
        let terminfo = parse(buffer.as_slice()).unwrap();
        assert_eq!(terminfo.booleans, collection!("bw", "xenl"));
        assert_eq!(
            terminfo.numbers,
            collection!(
                "cols" => 80,
                "lines" => 25,
                "pb" => 0x7fff_ffff,
            )
        );
        assert_eq!(
            terminfo.strings,
            collection!(
                "bel" => b"Hello".as_slice(),
                "tbc" => b"World!",
            )
        );
    }

    #[test]
    fn bad_magic() {
        let data_set = DataSet::default();
        let mut buffer = make_buffer(&data_set, false);
        buffer[1] = 3;
        let terminfo = parse(buffer.as_slice());
        assert!(matches!(terminfo.unwrap_err(), Error::BadMagic));
    }

    #[test]
    fn base_truncated() {
        let data_set = DataSet::default();
        let mut buffer = make_buffer(&data_set, false);
        buffer.pop();
        let terminfo = parse(buffer.as_slice());
        assert!(matches!(terminfo.unwrap_err(), Error::UnsupportedFormat));
    }

    #[test]
    fn base_unterminated_string() {
        let data_set = DataSet::default();
        let mut buffer = make_buffer(&data_set, false);
        let buffer_size = buffer.len();
        buffer[buffer_size - 1] = b'!';
        let terminfo = parse(buffer.as_slice());
        assert!(matches!(terminfo.unwrap_err(), Error::UnterminatedString));
    }

    #[test]
    fn extended_16_bit() {
        let data_set = DataSet::default();
        let buffer = make_buffer(&data_set, true);
        let terminfo = parse(buffer.as_slice()).unwrap();
        assert_eq!(
            terminfo.booleans,
            collection!("Curly", "Italic", "Semi-bold", "bw", "xenl")
        );
        assert_eq!(
            terminfo.numbers,
            collection!(
                "Shades" => 1100,
                "Variants" => 2200,
                "cols" => 80,
                "lines" => 25,
                "pb" => 5,
            )
        );
        assert_eq!(
            terminfo.strings,
            collection!(
                "Colors" => b"A lot".as_slice(),
                "Luminosity" => b"Positive",
                "bel" => b"Hello",
                "tbc" => b"World!",
            )
        );
    }

    #[test]
    fn extended_32_bit() {
        let mut data_set = DataSet {
            number_type: NumberType::U32,
            ..Default::default()
        };
        data_set.base_numbers[5] = 0x7fff_ffff;

        let buffer = make_buffer(&data_set, true);
        let terminfo = parse(buffer.as_slice()).unwrap();
        assert_eq!(
            terminfo.booleans,
            collection!("Curly", "Italic", "Semi-bold", "bw", "xenl")
        );
        assert_eq!(
            terminfo.numbers,
            collection!(
                "Shades" => 1100,
                "Variants" => 2200,
                "cols" => 80,
                "lines" => 25,
                "pb" => 0x7fff_ffff,
            )
        );
        assert_eq!(
            terminfo.strings,
            collection!(
                "Colors" => b"A lot".as_slice(),
                "Luminosity" => b"Positive",
                "bel" => b"Hello",
                "tbc" => b"World!",
            )
        );
    }
}
