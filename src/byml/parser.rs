use super::*;
use crate::Endian;
use binrw::{binrw, BinRead};
use static_assertions::const_assert;
use std::io::{Read, Seek, SeekFrom};

struct BinReader<R: Read + Seek> {
    reader: R,
    opts: binrw::ReadOptions,
}

impl<R: Read + Seek> BinReader<R> {
    fn new(reader: R, endian: Endian) -> Self {
        Self {
            reader,
            opts: binrw::ReadOptions::default().with_endian(match endian {
                Endian::Little => binrw::Endian::Little,
                Endian::Big => binrw::Endian::Big,
            }),
        }
    }

    fn read<T: BinRead>(&mut self) -> binrw::BinResult<T>
    where
        T::Args: Default,
    {
        T::read_options(&mut self.reader, &self.opts, T::Args::default())
    }

    fn seek(&mut self, pos: u64) -> std::io::Result<()> {
        self.reader.seek(SeekFrom::Start(pos))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[binrw]
struct ResHeaderInner {
    /// Format version (2 or 3).
    version: u16,
    /// Offset to the hash key table, relative to start (usually 0x010)
    /// May be 0 if no hash nodes are used. Must be a string table node (0xc2).
    hash_key_table_offset: u32,
    /// Offset to the string table, relative to start. May be 0 if no strings are used.
    /// Must be a string table node (0xc2).
    string_table_offset: u32,
    /// Offset to the root node, relative to start. May be 0 if the document is totally empty.
    /// Must be either an array node (0xc0) or a hash node (0xc1).
    root_node_offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[binrw]
struct ResHeader {
    /// “BY” (big endian) or “YB” (little endian).
    magic: [u8; 2],
    #[br(is_little = &magic == b"YB")]
    inner: ResHeaderInner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
enum NodeType {
    String = 0xa0,
    Binary = 0xa1,
    Array = 0xc0,
    Hash = 0xc1,
    StringTable = 0xc2,
    Bool = 0xd0,
    Int = 0xd1,
    Float = 0xd2,
    UInt = 0xd3,
    Int64 = 0xd4,
    UInt64 = 0xd5,
    Double = 0xd6,
    Null = 0xff,
}

#[inline(always)]
const fn is_container_type(node_type: NodeType) -> bool {
    matches!(node_type, NodeType::Array | NodeType::Hash)
}

#[inline(always)]
const fn is_long_type(node_type: NodeType) -> bool {
    matches!(
        node_type,
        NodeType::Int64 | NodeType::UInt64 | NodeType::Double
    )
}

#[inline(always)]
const fn is_non_inline_type(node_type: NodeType) -> bool {
    is_container_type(node_type) || is_long_type(node_type) || matches!(node_type, NodeType::Binary)
}

#[inline(always)]
const fn is_valid_version(version: u16) -> bool {
    version == 2 || version == 3
}

#[derive(Debug, Default)]
struct StringTableParser {
    offset: u32,
    size: u32,
}

impl StringTableParser {
    fn new<R: Read + Seek>(offset: u32, reader: &mut BinReader<R>) -> Result<Self, BymlError> {
        if offset == 0 {
            Ok(Self::default())
        } else {
            reader.seek(offset as u64)?;
            let type_: NodeType = reader.read()?;
            let num_entries: crate::util::u24 = reader.read()?;
            if type_ != NodeType::StringTable {
                return Err(BymlError::TypeError(
                    format!("{:?}", type_),
                    format!("{:?}", NodeType::StringTable),
                ));
            }
            Ok(Self {
                offset,
                size: num_entries.as_u32(),
            })
        }
    }

    fn get_string<R: Read + Seek>(
        &self,
        index: u32,
        reader: &mut BinReader<R>,
    ) -> Result<String, BymlError> {
        if index >= self.size {
            return Err(BymlError::ParseError("Invalid string table entry index"));
        }
        reader.seek((self.offset + 4 + 4 + index) as u64)?;
        let offset: u32 = reader.read()?;
        let next_offset: u32 = reader.read()?;
        let max_len = next_offset - offset;
        reader.seek((self.offset + offset) as u64)?;
        let string_: binrw::NullString = reader.read()?;
        if string_.len() > max_len as usize {
            return Err(BymlError::ParseError("String table entry too long"));
        }
        Ok(string_.into_string().into())
    }
}

struct Parser<R: Read + Seek> {
    reader: BinReader<R>,
    string_table: StringTableParser,
    hash_key_table: StringTableParser,
    root_node_offset: u32,
}

impl<R: Read + Seek> Parser<R> {
    fn new(mut reader: R) -> Result<Self, BymlError> {
        if reader.stream_len()? < 0x10 {
            return Err(BymlError::ParseError("Insufficient data for header"));
        }
        let header = ResHeader::read(&mut reader)?;
        let endian = if &header.magic == b"BY" {
            Endian::Big
        } else {
            Endian::Little
        };
        if !is_valid_version(header.inner.version) {
            return Err(BymlError::ParseError(
                "Unsupported BYML version (2 or 3 only)",
            ));
        }
        let mut reader = BinReader::new(reader, endian);
        Ok(Self {
            string_table: StringTableParser::new(header.inner.string_table_offset, &mut reader)?,
            hash_key_table: StringTableParser::new(
                header.inner.hash_key_table_offset,
                &mut reader,
            )?,
            root_node_offset: header.inner.root_node_offset,
            reader,
        })
    }

    fn parse(&mut self) -> Result<Byml, BymlError> {
        if self.root_node_offset == 0 {
            Ok(Byml::Null)
        } else {
            self.parse_container_node(self.root_node_offset)
        }
    }

    fn parse_container_node(&mut self, offset: u32) -> Result<Byml, BymlError> {
        todo!()
    }
}
