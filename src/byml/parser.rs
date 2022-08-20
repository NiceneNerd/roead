use super::*;
use crate::{
    util::{align, u24},
    Endian, Error, Result,
};
use binrw::{binrw, BinRead, VecArgs};
use std::io::{Read, Seek, SeekFrom};

impl Byml {
    /// Read a document from a binary reader.
    pub fn read<R: Read + Seek>(reader: R) -> Result<Byml> {
        Parser::new(reader)?.parse()
    }

    /// Load a document from binary data.
    ///
    /// **Note**: If and only if the `yaz0` feature is enabled, this function
    /// automatically decompresses the SARC when necessary.
    pub fn from_binary(data: impl AsRef<[u8]>) -> Result<Byml> {
        #[cfg(feature = "yaz0")]
        {
            if data.as_ref().starts_with(b"Yaz0") {
                return Parser::new(std::io::Cursor::new(crate::yaz0::decompress(
                    data.as_ref(),
                )?))?
                .parse();
            }
        }
        Parser::new(std::io::Cursor::new(data.as_ref()))?.parse()
    }
}

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

    fn read_at<T: BinRead>(&mut self, offset: u64) -> binrw::BinResult<T>
    where
        T::Args: Default,
    {
        self.reader.seek(SeekFrom::Start(offset))?;
        self.read()
    }

    fn seek(&mut self, pos: u64) -> std::io::Result<()> {
        self.reader.seek(SeekFrom::Start(pos))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[binrw]
struct ResHeaderInner {
    /// Format version (2-4).
    version: u16,
    /// Offset to the hash key table, relative to start (usually 0x010)
    /// May be 0 if no hash nodes are used. Must be a string table node (0xc2).
    hash_key_table_offset: u32,
    /// Offset to the string table, relative to start. May be 0 if no strings
    /// are used. Must be a string table node (0xc2).
    string_table_offset: u32,
    /// Offset to the root node, relative to start. May be 0 if the document is
    /// totally empty. Must be either an array node (0xc0) or a hash node
    /// (0xc1).
    root_node_offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[binrw]
struct ResHeader {
    /// “BY” (big endian) or “YB” (Verslittle endian).
    magic: [u8; 2],
    #[br(is_little = &magic == b"YB")]
    inner: ResHeaderInner,
}

#[derive(Debug, Default)]
struct StringTableParser {
    offset: u32,
    size: u32,
}

impl StringTableParser {
    fn new<R: Read + Seek>(offset: u32, reader: &mut BinReader<R>) -> Result<Self> {
        if offset == 0 {
            Ok(Self::default())
        } else {
            let type_: NodeType = reader.read_at(offset as u64)?;
            let num_entries: crate::util::u24 = reader.read()?;
            if type_ != NodeType::StringTable {
                return Err(Error::TypeError(
                    format!("{:?}", type_).into(),
                    "string table",
                ));
            }
            Ok(Self {
                offset,
                size: num_entries.as_u32(),
            })
        }
    }

    fn get_string<R: Read + Seek>(&self, index: u32, reader: &mut BinReader<R>) -> Result<String> {
        if index >= self.size {
            return Err(Error::InvalidData("Invalid string table entry index"));
        }
        let offset: u32 = reader.read_at((self.offset + 4 + 4 * index) as u64)?;
        let next_offset: u32 = reader.read()?;
        let max_len = next_offset - offset;
        reader.seek((self.offset + offset) as u64)?;
        let mut string_ = String::new_const();
        let mut c: u8 = reader.read()?;
        while c != 0 && string_.len() < max_len as usize {
            string_.push(c as char);
            c = reader.read()?;
        }
        Ok(string_)
    }
}

struct Parser<R: Read + Seek> {
    reader: BinReader<R>,
    string_table: StringTableParser,
    hash_key_table: StringTableParser,
    root_node_offset: u32,
}

impl<R: Read + Seek> Parser<R> {
    fn new(mut reader: R) -> Result<Self> {
        if reader.stream_len()? < 0x10 {
            return Err(Error::InvalidData("Insufficient data for header"));
        }
        let header = ResHeader::read(&mut reader)?;
        let endian = if &header.magic == b"BY" {
            Endian::Big
        } else {
            Endian::Little
        };
        if !is_valid_version(header.inner.version) {
            return Err(Error::InvalidData("Unsupported BYML version (2 or 3 only)"));
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

    fn parse(&mut self) -> Result<Byml> {
        if self.root_node_offset == 0 {
            Ok(Byml::Null)
        } else {
            self.parse_container_node(self.root_node_offset)
        }
    }

    fn parse_value_node(&mut self, offset: u32, node_type: NodeType) -> Result<Byml> {
        let raw: u32 = self.reader.read_at(offset as u64)?;

        let mut read_long = || -> Result<u64> { Ok(self.reader.read_at(offset as u64)?) };

        let value = match node_type {
            NodeType::String => Byml::String(self.string_table.get_string(raw, &mut self.reader)?),
            NodeType::Binary => {
                let size: u32 = self.reader.read_at(raw as u64)?;
                let buf = Vec::read_options(
                    &mut self.reader.reader,
                    &self.reader.opts,
                    VecArgs {
                        count: size as usize,
                        inner: (),
                    },
                )?;
                Byml::BinaryData(buf)
            }
            NodeType::Bool => Byml::Bool(raw != 0),
            NodeType::I32 => Byml::I32(raw as i32),
            NodeType::U32 => Byml::U32(raw),
            NodeType::Float => Byml::Float(f32::from_bits(raw)),
            NodeType::I64 => Byml::I64(read_long()? as i64),
            NodeType::U64 => Byml::U64(read_long()?),
            NodeType::Double => Byml::Double(f64::from_bits(read_long()?)),
            NodeType::Null => Byml::Null,
            _ => unreachable!("Invalid value node type"),
        };
        Ok(value)
    }

    fn parse_container_child_node(&mut self, offset: u32, node_type: NodeType) -> Result<Byml> {
        if is_container_type(node_type) {
            let container_offset = self.reader.read_at(offset as u64)?;
            self.parse_container_node(container_offset)
        } else {
            self.parse_value_node(offset, node_type)
        }
    }

    fn parse_array_node(&mut self, offset: u32, size: u32) -> Result<Byml> {
        let mut array = Vec::with_capacity(size as usize);
        let values_offset = offset + 4 + align(size, 4);
        for i in 0..size {
            let child_offset = offset + 4 + i;
            let child_type: NodeType = self.reader.read_at(child_offset as u64)?;
            array.push(self.parse_container_child_node(values_offset + 4 * i, child_type)?);
        }
        Ok(Byml::Array(array))
    }

    fn parse_hash_node(&mut self, offset: u32, size: u32) -> Result<Byml> {
        let mut hash = Hash::with_capacity_and_hasher(size as usize, Default::default());
        for i in 0..size {
            let entry_offset = offset + 4 + 8 * i;
            let name_idx: u24 = self.reader.read_at(entry_offset as u64)?;
            let node_type: NodeType = self.reader.read_at(entry_offset as u64 + 3)?;
            let key = self
                .hash_key_table
                .get_string(name_idx.as_u32(), &mut self.reader)?;
            hash.insert(
                key,
                self.parse_container_child_node(entry_offset + 4, node_type)?,
            );
        }
        Ok(Byml::Hash(hash))
    }

    fn parse_container_node(&mut self, offset: u32) -> Result<Byml> {
        let node_type: NodeType = self.reader.read_at(offset as u64)?;
        let size: u24 = self.reader.read()?;
        match node_type {
            NodeType::Array => self.parse_array_node(offset, size.as_u32()),
            NodeType::Hash => self.parse_hash_node(offset, size.as_u32()),
            _ => unreachable!("Invalid container node type"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes() {
        for file in FILES {
            println!("{}", file);
            let bytes =
                std::fs::read(std::path::Path::new("test/byml").join([file, ".byml"].join("")))
                    .unwrap();
            let byml = Byml::from_binary(bytes).unwrap();
            match byml {
                Byml::Array(arr) => println!("  Array with {} elements", arr.len()),
                Byml::Hash(hash) => println!("  Hash with {} entries", hash.len()),
                _ => println!("{:?}", byml),
            }
        }
    }
}
