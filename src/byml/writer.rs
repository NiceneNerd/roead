use super::*;
use crate::Endian;
use binrw::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    io::{Cursor, Seek, Write},
    rc::Rc,
};

struct WriteContext<'a, W: Write + Seek> {
    writer: W,
    options: binrw::WriteOptions,
    hash_key_table: BTreeMap<&'a String, u32>,
    string_table: BTreeMap<&'a String, u32>,
    non_inline_node_data: HashMap<Rc<Byml>, u32>,
}

impl<'a, W: Write + Seek> WriteContext<'a, W> {
    fn new(byml: &'a Byml, writer: W, endian: Endian) -> Self {
        let mut non_inline_node_count = 0;
        let mut string_table = BTreeMap::new();
        let mut hash_key_table = BTreeMap::new();
        fn traverse<'a>(
            byml: &'a Byml,
            count: &mut usize,
            string_table: &mut BTreeMap<&'a String, u32>,
            hash_key_table: &mut BTreeMap<&'a String, u32>,
        ) {
            match byml {
                Byml::String(s) => {
                    let index = string_table.len();
                    string_table.insert(s, index as u32);
                }
                Byml::Array(arr) => {
                    for node in arr.iter() {
                        traverse(node, count, string_table, hash_key_table);
                    }
                }
                Byml::Hash(hash) => {
                    for (key, node) in hash.iter() {
                        let index = hash_key_table.len();
                        hash_key_table.insert(key, index as u32);
                        traverse(node, count, string_table, hash_key_table);
                    }
                }
                Byml::Binary(_) | Byml::Int64(_) | Byml::UInt64(_) | Byml::Double(_) => {}
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
        WriteContext {
            writer,
            options: binrw::WriteOptions::default().with_endian(match endian {
                Endian::Little => binrw::Endian::Little,
                Endian::Big => binrw::Endian::Big,
            }),
            hash_key_table,
            string_table,
            non_inline_node_data: HashMap::with_capacity(non_inline_node_count),
        }
    }

    #[inline(always)]
    fn write<T: BinWrite<Args = ()>>(&mut self, val: T) -> binrw::BinResult<()> {
        val.write_options(&mut self.writer, &self.options, ())
    }

    fn write_value_node(&mut self, node: &Byml) -> binrw::BinResult<()> {
        match node {
            Byml::Null => self.write(0u32),
            Byml::String(s) => self.write(self.string_table[s]),
            Byml::Binary(data) => {
                self.write(data.len() as u32)?;
                self.write(data)
            }
            Byml::Bool(b) => self.write(*b as u32),
            Byml::Int(i) => self.write(*i as i32),
            Byml::UInt(u) => self.write(*u as u32),
            Byml::Float(f) => self.write(*f as f32),
            Byml::Int64(i) => self.write(*i),
            Byml::UInt64(u) => self.write(*u),
            Byml::Double(d) => self.write(*d),
            _ => unreachable!("Invalid value node type"),
        }
    }
}
