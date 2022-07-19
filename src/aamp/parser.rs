use super::*;
use crate::util::u24;
use binrw::{binrw, prelude::*};
use enumflags2::{bitflags, BitFlags};
use std::io::{Read, Seek};

#[binrw]
#[brw(repr(u32), little)]
#[bitflags]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum HeaderFlag {
    LittleEndian = 1 << 0,
    Utf8 = 1 << 1,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResHeader {
    magic: [u8; 4],
    version: u32,
    flags: HeaderFlag,
    file_size: u32,
    pio_version: u32,
    /// Offset to parameter IO (relative to 0x30)
    pio_offset: u32,
    /// Number of lists (including root)
    list_count: u32,
    object_count: u32,
    param_count: u32,
    data_section_size: u32,
    string_section_size: u32,
    unknown_section_size: u32,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameter {
    name: Name,
    data_rel_offset: u24,
    type_: Type,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameterObj {
    name: Name,
    params_rel_offset: u16,
    param_count: u16,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameterList {
    name: Name,
    lists_rel_offset: u16,
    list_count: u16,
    objects_rel_offset: u16,
    object_count: u16,
}

struct Parser<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> Parser<R> {
    fn new(mut reader: R) -> Result<Self, AampError> {
        if reader.stream_len()? < 0x30 {
            return Err(AampError::InvalidData("Incomplete parameter archive"));
        }
        let header = ResHeader::read(&mut reader)?;
        if &header.magic != b"AAMP" {
            return Err(AampError::InvalidData("Invalid magic"));
        }
        if header.version != 2 {
            return Err(AampError::InvalidData(
                "Only version 2 parameter archives are supported",
            ));
        }
        let flags = BitFlags::from_flag(header.flags);
        if !flags.contains(HeaderFlag::LittleEndian) {
            return Err(AampError::InvalidData(
                "Only little endian parameter archives are supported",
            ));
        }
        if !flags.contains(HeaderFlag::Utf8) {
            return Err(AampError::InvalidData(
                "Only UTF-8 parameter archives are supported",
            ));
        }
        Ok(Self { reader })
    }

    #[inline]
    fn seek(&mut self, offset: u32) -> Result<(), AampError> {
        self.reader.seek(std::io::SeekFrom::Start(offset as u64))?;
        Ok(())
    }

    fn parse_parameter(&mut self, offset: u32) -> Result<ResParameter, AampError> {
        self.seek(offset)?;
        let info = ResParameter::read(&mut self.reader)?;
        let crc = info.name.0;
        let data_offset = info.data_rel_offset.as_u32();
        todo!()
    }
}
