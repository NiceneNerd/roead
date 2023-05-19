use std::{
    collections::BTreeMap,
    io::{Cursor, Seek, SeekFrom, Write},
    rc::Rc,
};

use binrw::prelude::*;
use rustc_hash::FxHashMap;

use super::*;
use crate::{
    util::{align, u24},
    Endian, Error, Result,
};

impl Byml {
    /// Serialize the document to binary into the given writer. This can only
    /// be done for Null, Array, or Hash nodes.
    pub fn write<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        version: u16,
    ) -> Result<()> {
        if !is_valid_version(version) {
            return Err(Error::InvalidData("Unsupported BYML version (2-4 only)"));
        }

        if !matches!(
            self,
            Byml::Map(_) | Byml::HashMap(_) | Byml::ValueHashMap(_) | Byml::Array(_) | Byml::Null
        ) {
            return Err(Error::TypeError(
                format!("{:?}", self).into(),
                "Map, HashMap, ValueHashMap, Array, or Null",
            ));
        }

        let mut ctx = WriteContext::new(self, writer, endian);
        ctx.write(match endian {
            Endian::Little => b"YB",
            Endian::Big => b"BY",
        })?;
        ctx.write(version)?;
        ctx.write(0u32)?; // Hash key table offset
        ctx.write(0u32)?; // String table offset
        ctx.write(0u32)?; // Root node offset

        if let &Byml::Null = self {
            Ok(())
        } else {
            if !ctx.hash_key_table.is_empty() {
                let pos = ctx.writer.stream_position()? as u32;
                ctx.write_at(pos, 0x4)?;
                ctx.write_string_table(ctx.hash_key_table.clone())?;
            }

            if !ctx.string_table.is_empty() {
                let pos = ctx.writer.stream_position()? as u32;
                ctx.write_at(pos, 0x8)?;
                ctx.write_string_table(ctx.string_table.clone())?;
            }

            let pos = ctx.writer.stream_position()? as u32;
            ctx.write_at(pos, 0xC)?;
            ctx.align()?;
            ctx.write_container_node(self)?;
            ctx.align()?;
            ctx.writer.flush()?;
            Ok(())
        }
    }

    /// Serialize the document to bytes with the specified endianness and
    /// default version (2). This can only be done for Null, Array, or Hash
    /// nodes.
    pub fn to_binary(&self, endian: Endian) -> Vec<u8> {
        let mut buf = Vec::new();
        self.write(&mut Cursor::new(&mut buf), endian, 2)
            .expect("BYML should serialize to binary without error");
        buf
    }

    /// Serialize the document to BYML with the specified endianness and
    /// version number. This can only be done for Null, Array, or Hash nodes.
    pub fn to_binary_with_version(&self, endian: Endian, version: u16) -> Vec<u8> {
        let mut buf = Vec::new();
        self.write(&mut Cursor::new(&mut buf), endian, version)
            .expect("BYML should serialize to binary without error");
        buf
    }
}

struct NonInlineNode<'a> {
    data:   &'a Byml,
    offset: u32,
}

#[derive(Debug, Default)]
struct StringTable<'a> {
    table: FxHashMap<&'a String, u32>,
    sorted_strings: Vec<&'a String>,
}

impl<'a> StringTable<'a> {
    #[inline]
    fn add<'b>(&'b mut self, s: &'a String) {
        self.table.insert(s, 0);
    }

    #[inline]
    fn get_index(&self, s: &String) -> u32 {
        unsafe { self.table.get(s).copied().unwrap_unchecked() }
    }

    fn build(&mut self) {
        self.sorted_strings = self.table.keys().copied().collect();
        self.sorted_strings.sort();
        self.table = self
            .sorted_strings
            .iter()
            .enumerate()
            .map(|(i, s)| (*s, i as u32))
            .collect();
    }

    #[inline]
    fn len(&self) -> usize {
        self.table.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

struct WriteContext<'a, W: Write + Seek> {
    writer: W,
    endian: binrw::Endian,
    hash_key_table: Rc<StringTable<'a>>,
    string_table: Rc<StringTable<'a>>,
    non_inline_node_data: FxHashMap<&'a Byml, u32>,
}

impl<'a, W: Write + Seek> WriteContext<'a, W> {
    fn new(byml: &'a Byml, writer: W, endian: Endian) -> Self {
        let mut non_inline_node_count = 0;
        let mut string_table = StringTable::default();
        let mut hash_key_table = StringTable::default();
        fn traverse<'a>(
            byml: &'a Byml,
            count: &mut usize,
            string_table: &mut StringTable<'a>,
            hash_key_table: &mut StringTable<'a>,
        ) {
            match byml {
                Byml::String(s) => {
                    string_table.add(s);
                }
                Byml::Array(arr) => {
                    for node in arr.iter() {
                        traverse(node, count, string_table, hash_key_table);
                    }
                }
                Byml::Map(hash) => {
                    for (key, node) in hash.iter() {
                        hash_key_table.add(key);
                        traverse(node, count, string_table, hash_key_table);
                    }
                }
                Byml::HashMap(hash) => {
                    for node in hash.values() {
                        traverse(node, count, string_table, hash_key_table);
                    }
                }
                Byml::ValueHashMap(hash) => {
                    for (node, _) in hash.values() {
                        traverse(node, count, string_table, hash_key_table);
                    }
                }
                Byml::BinaryData(_)
                | Byml::FileData(_)
                | Byml::I64(_)
                | Byml::U64(_)
                | Byml::Double(_) => {}
                _ => return,
            }
            *count += 1;
        }
        traverse(
            byml,
            &mut non_inline_node_count,
            &mut string_table,
            &mut hash_key_table,
        );
        string_table.build();
        hash_key_table.build();
        WriteContext {
            writer,
            endian: match endian {
                Endian::Little => binrw::Endian::Little,
                Endian::Big => binrw::Endian::Big,
            },
            hash_key_table: Rc::new(hash_key_table),
            string_table: Rc::new(string_table),
            non_inline_node_data: FxHashMap::with_capacity_and_hasher(
                non_inline_node_count,
                Default::default(),
            ),
        }
    }

    #[inline(always)]
    fn write<'b, T: BinWrite<Args<'b> = ()>>(&mut self, val: T) -> binrw::BinResult<()> {
        val.write_options(&mut self.writer, self.endian, ())
    }

    #[inline(always)]
    fn align(&mut self) -> binrw::BinResult<()> {
        let pos = self.writer.stream_position()? as u32;
        let aligned = align(pos, 4);
        self.writer.seek(SeekFrom::Start(aligned as u64))?;
        Ok(())
    }

    #[inline(always)]
    fn write_at<'b, T: BinWrite<Args<'b> = ()>>(
        &mut self,
        val: T,
        offset: u32,
    ) -> binrw::BinResult<()> {
        let old_pos = self.writer.stream_position()?;
        self.writer.seek(SeekFrom::Start(offset as u64))?;
        self.write(val)?;
        self.writer.seek(SeekFrom::Start(old_pos))?;
        Ok(())
    }

    fn write_value_node(&mut self, node: &Byml) -> binrw::BinResult<()> {
        match node {
            Byml::Null => self.write(0u32),
            Byml::String(s) => self.write(self.string_table.get_index(s)),
            Byml::BinaryData(data) => {
                self.write(data.len() as u32)?;
                self.write(data)
            }
            Byml::FileData(data) => {
                self.write(data.len() as u32)?;
                self.write(0x1000u32)?; // unknown
                self.write(data)
            }
            Byml::Bool(b) => self.write(*b as u32),
            Byml::I32(i) => self.write(*i),
            Byml::U32(u) => self.write(*u),
            Byml::Float(f) => self.write(f.to_bits()),
            Byml::I64(i) => self.write(*i),
            Byml::U64(u) => self.write(*u),
            Byml::Double(d) => self.write(d.to_bits()),
            _ => {
                BinResult::Err(binrw::Error::Custom {
                    pos: self.writer.stream_position()?,
                    err: Box::new("Invalid value node type"),
                })
            }
        }
    }

    fn write_container_node<'b>(&'b mut self, node: &'a Byml) -> binrw::BinResult<()> {
        let mut non_inline_nodes = Vec::new();

        #[inline]
        fn write_container_item<'parent, W: Write + Seek>(
            ctx: &mut WriteContext<'parent, W>,
            item: &'parent Byml,
            non_inline_nodes: &mut Vec<NonInlineNode<'parent>>,
        ) -> binrw::BinResult<()> {
            if item.is_non_inline_type() {
                non_inline_nodes.push(NonInlineNode {
                    data:   item,
                    offset: ctx.writer.stream_position()? as u32,
                });
                ctx.write(0u32)?;
            } else {
                ctx.write_value_node(item)?;
            }
            Ok(())
        }

        match node {
            Byml::Array(arr) => {
                non_inline_nodes.reserve(arr.len());
                self.write(NodeType::Array)?;
                self.write(u24(arr.len() as u32))?;
                let types_pos = self.writer.stream_position()? as u32;
                self.writer.seek(SeekFrom::Current(arr.len() as i64))?;
                self.align()?;
                for (i, item) in arr.iter().enumerate() {
                    self.write_at(item.get_node_type(), types_pos + i as u32)?;
                    write_container_item(self, item, &mut non_inline_nodes)?;
                }
            }
            Byml::Map(map) => {
                non_inline_nodes.reserve(map.len());
                self.write(NodeType::Map)?;
                self.write(u24(map.len() as u32))?;
                let sorted = map.iter().collect::<BTreeMap<_, _>>();
                for (key, item) in sorted {
                    self.write(u24(self.hash_key_table.get_index(key)))?;
                    self.write(item.get_node_type())?;
                    write_container_item(self, item, &mut non_inline_nodes)?;
                }
            }
            Byml::HashMap(hash) => {
                non_inline_nodes.reserve(hash.len());
                self.write(NodeType::HashMap)?;
                self.write(u24(hash.len() as u32))?;
                let sorted = hash.iter().collect::<BTreeMap<_, _>>();
                for (hash, item) in &sorted {
                    self.write(*hash)?;
                    write_container_item(self, item, &mut non_inline_nodes)?;
                }
                for item in sorted.values() {
                    self.write(item.get_node_type())?;
                }
                self.align()?;
            }
            Byml::ValueHashMap(hash) => {
                non_inline_nodes.reserve(hash.len());
                self.write(NodeType::ValueHashMap)?;
                self.write(u24(hash.len() as u32))?;
                let sorted = hash.iter().collect::<BTreeMap<_, _>>();
                for (hash, (item, unknown)) in &sorted {
                    write_container_item(self, item, &mut non_inline_nodes)?;
                    self.write(hash)?;
                    self.write(unknown)?;
                }
                for (item, _) in sorted.values() {
                    self.write(item.get_node_type())?;
                }
                self.align()?;
            }
            _ => {
                return BinResult::Err(binrw::Error::Custom {
                    pos: self.writer.stream_position()?,
                    err: Box::new("Invalid container node type"),
                });
            }
        }

        for node in non_inline_nodes {
            self.align()?;
            if let Some(pos) = self.non_inline_node_data.get(&node.data).copied() {
                self.write_at(pos, node.offset)?;
            } else {
                let offset = self.writer.stream_position()? as u32;
                self.write_at(offset, node.offset)?;
                self.non_inline_node_data.insert(node.data, offset);
                match node.data {
                    Byml::Array(_) | Byml::Map(_) | Byml::HashMap(_) | Byml::ValueHashMap(_) => {
                        self.write_container_node(node.data)?
                    }
                    _ => self.write_value_node(node.data)?,
                }
            }
        }

        Ok(())
    }

    fn write_string_table(&mut self, table: Rc<StringTable<'_>>) -> binrw::BinResult<()> {
        let start = self.writer.stream_position()? as u32;
        self.write(NodeType::StringTable)?;
        self.write(u24(table.len() as u32))?;

        let offset_table_offset = self.writer.stream_position()? as u32;
        self.writer.seek(SeekFrom::Start(
            (offset_table_offset as usize + 0x4 * (table.len() + 1)) as u64,
        ))?;

        let mut pos;
        for (i, string_) in table.sorted_strings.iter().enumerate() {
            pos = self.writer.stream_position()? as u32;
            self.write_at(pos - start, (offset_table_offset as usize + 0x4 * i) as u32)?;
            self.write(string_.as_bytes())?;
            self.write(0u8)?;
        }

        let end = self.writer.stream_position()? as u32;
        self.write_at(
            end - start,
            (offset_table_offset as usize + 0x4 * table.len()) as u32,
        )?;
        self.align()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn binary_roundtrip() {
        println!("{}", std::mem::size_of::<Map>());
        for file in FILES {
            println!("{}", file);
            let bytes =
                std::fs::read(std::path::Path::new("test/byml").join([file, ".byml"].join("")))
                    .unwrap();
            let byml = Byml::from_binary(bytes).unwrap();
            let new_le_bytes = byml.to_binary(Endian::Little);
            let mut new_byml = Byml::from_binary(new_le_bytes).unwrap();
            assert_eq!(byml, new_byml);
            let new_be_bytes = byml.to_binary(Endian::Big);
            new_byml = Byml::from_binary(new_be_bytes).unwrap();
            assert_eq!(byml, new_byml);
        }
    }
}
