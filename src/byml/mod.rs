//! Port of the `oead::byml` module.
//!
//! A `Byml` type will usually be constructed from binary data or a YAML string,
//! e.g.
//! ```
//! # use roead::byml::Byml;
//! # use std::{fs::read, error::Error};
//! # fn docttest() -> Result<(), Box<dyn Error>> {
//! let buf: Vec<u8> = std::fs::read("test/byml/A-1_Dynamic.byml")?;
//! let map_unit = Byml::from_binary(&buf)?;
//! let text: String = std::fs::read_to_string("test/byml/A-1_Dynamic.yml")?;
//! //let map_unit2 = Byml::from_text(&text)?;
//! //assert_eq!(map_unit, map_unit2);
//! # Ok(())
//! # }
//! ```
//! You can also easily serialize to binary or a YAML string.
//! ```no_run
//! # use roead::{byml::Byml, Endian};
//! # fn docttest() -> Result<(), Box<dyn std::error::Error>> {
//! let buf: Vec<u8> = std::fs::read("test/aamp/A-1_Dynamic.byml")?;
//! let map_unit = Byml::from_binary(&buf)?;
//! //std::fs::write("A-1_Static.yml", &map_unit.to_text())?;
//! std::fs::write("test/aamp/A-1_Dynamic.byml", &map_unit.to_binary(Endian::Big))?;
//! # Ok(())
//! # }
//! ```
//!
//! A number of convenience getters are available which return a result for a variant value:
//! ```
//! # use roead::byml::Byml;
//! # fn docttest() -> Result<(), Box<dyn std::error::Error>> {
//! # let some_data = b"BYML";
//! let doc = Byml::from_binary(some_data)?;
//! let hash = doc.as_hash().unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! Most of the node types are fairly self-explanatory. Arrays are implemented as `Vec<Byml>`, and
//! hash nodes as `FxHashMap<String, Byml>`.
//!
//! For convenience, a `Byml` *known* to be an array or hash node can be indexed. **Panics if the
//! node has the wrong type, the index has the wrong type, or the index is not found**.
//! ```
//! # use roead::byml::Byml;
//! # fn docttest() -> Result<(), Box<dyn std::error::Error>> {
//! let buf: Vec<u8> = std::fs::read("test/byml/ActorInfo.product.byml")?;
//! let actor_info = Byml::from_binary(&buf)?;
//! assert_eq!(actor_info["Actors"].as_array().unwrap().len(), 7934);
//! assert_eq!(*actor_info["Hashes"][0].as_i32().unwrap(), 31119);
//! # Ok(())
//! # }
//! ```
mod writer;
#[cfg(feature = "yaml")]
mod yaml;
use enum_as_inner::EnumAsInner;
use smartstring::alias::String;
mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[binrw::binrw]
#[brw(repr = u8)]
#[repr(u8)]
enum NodeType {
    String = 0xa0,
    Binary = 0xa1,
    Array = 0xc0,
    Hash = 0xc1,
    StringTable = 0xc2,
    Bool = 0xd0,
    I32 = 0xd1,
    Float = 0xd2,
    U32 = 0xd3,
    I64 = 0xd4,
    U64 = 0xd5,
    Double = 0xd6,
    Null = 0xff,
}

#[inline(always)]
const fn is_container_type(node_type: NodeType) -> bool {
    matches!(node_type, NodeType::Array | NodeType::Hash)
}

#[inline(always)]
const fn is_valid_version(version: u16) -> bool {
    version >= 2 && version <= 4
}

#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum BymlError {
    #[error("Invalid version: {0}")]
    InvalidVersion(u16),
    #[error("Incorrect BYML node type: found `{0}`, expected `{1}`.")]
    TypeError(std::string::String, std::string::String),
    #[error(transparent)]
    BinaryRwError(#[from] binrw::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Error parsing BYML data: {0}")]
    ParseError(&'static str),
}

/// A BYML hash node.
#[cfg(feature = "im-rc")]
pub type Hash = im_rc::HashMap<String, Byml>;
/// A BYML hash node.
#[cfg(not(feature = "im-rc"))]
pub type Hash = rustc_hash::FxHashMap<String, Byml>;

/// Convenience type used for indexing into `Byml`s
pub enum BymlIndex<'a> {
    /// Index into a hash node. The key is a string.
    HashIdx(&'a str),
    /// Index into an array node. The index is an integer.
    ArrayIdx(usize),
}

impl<'a> From<&'a str> for BymlIndex<'a> {
    fn from(s: &'a str) -> Self {
        Self::HashIdx(s)
    }
}

impl<'a> From<usize> for BymlIndex<'a> {
    fn from(idx: usize) -> Self {
        Self::ArrayIdx(idx)
    }
}

/// Represents a Nintendo binary YAML (BYML) document or node.
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, EnumAsInner)]
pub enum Byml {
    /// String value.
    String(String),
    /// Binary data (not used in BOTW).
    BinaryData(Vec<u8>),
    /// Array of BYML nodes.
    Array(Vec<Byml>),
    /// Hash map of BYML nodes.
    Hash(Hash),
    /// Boolean value.
    Bool(bool),
    /// 32-bit signed integer.
    I32(i32),
    /// 32-bit float.
    Float(f32),
    /// 32-bit unsigned integer.
    U32(u32),
    /// 64-bit signed integer.
    I64(i64),
    /// 64-bit unsigned integer.
    U64(u64),
    /// 64-bit float.
    Double(f64),
    /// Null value.
    Null,
}

impl PartialEq for Byml {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Byml::String(s1), Byml::String(s2)) => s1 == s2,
            (Byml::BinaryData(d1), Byml::BinaryData(d2)) => d1 == d2,
            (Byml::Array(a1), Byml::Array(a2)) => a1 == a2,
            (Byml::Hash(h1), Byml::Hash(h2)) => h1 == h2,
            (Byml::Bool(b1), Byml::Bool(b2)) => b1 == b2,
            (Byml::I32(i1), Byml::I32(i2)) => i1 == i2,
            (Byml::Float(f1), Byml::Float(f2)) => almost::equal(*f1, *f2),
            (Byml::U32(u1), Byml::U32(u2)) => u1 == u2,
            (Byml::I64(i1), Byml::I64(i2)) => i1 == i2,
            (Byml::U64(u1), Byml::U64(u2)) => u1 == u2,
            (Byml::Double(d1), Byml::Double(d2)) => almost::equal(*d1, *d2),
            (Byml::Null, Byml::Null) => true,
            _ => false,
        }
    }
}

impl Eq for &Byml {}

impl std::hash::Hash for Byml {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Byml::String(s) => s.hash(state),
            Byml::BinaryData(b) => b.hash(state),
            Byml::Array(a) => a.hash(state),
            Byml::Hash(h) => {
                for (k, v) in h.iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Byml::Bool(b) => b.hash(state),
            Byml::I32(i) => i.hash(state),
            Byml::Float(f) => {
                b"f".hash(state);
                f.to_bits().hash(state)
            }
            Byml::U32(u) => u.hash(state),
            Byml::I64(i) => i.hash(state),
            Byml::U64(u) => u.hash(state),
            Byml::Double(d) => {
                b"d".hash(state);
                d.to_bits().hash(state)
            }
            Byml::Null => std::hash::Hash::hash(&0, state),
        }
    }
}

impl<'a, I: Into<BymlIndex<'a>>> std::ops::Index<I> for Byml {
    type Output = Byml;

    fn index(&self, index: I) -> &Self::Output {
        match (self, index.into()) {
            (Byml::Array(a), BymlIndex::ArrayIdx(i)) => &a[i],
            (Byml::Hash(h), BymlIndex::HashIdx(k)) => h.get(k).unwrap(),
            _ => panic!("Wrong index type or node type."),
        }
    }
}

impl<'a, I: Into<BymlIndex<'a>>> std::ops::IndexMut<I> for Byml {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        match (self, index.into()) {
            (Byml::Array(a), BymlIndex::ArrayIdx(i)) => &mut a[i],
            (Byml::Hash(h), BymlIndex::HashIdx(k)) => h.get_mut(k).unwrap(),
            _ => panic!("Wrong index type or node type."),
        }
    }
}

impl Byml {
    #[inline]
    fn get_node_type(&self) -> NodeType {
        match self {
            Byml::String(_) => NodeType::String,
            Byml::BinaryData(_) => NodeType::Binary,
            Byml::Array(_) => NodeType::Array,
            Byml::Hash(_) => NodeType::Hash,
            Byml::Bool(_) => NodeType::Bool,
            Byml::I32(_) => NodeType::I32,
            Byml::Float(_) => NodeType::Float,
            Byml::U32(_) => NodeType::U32,
            Byml::I64(_) => NodeType::I64,
            Byml::U64(_) => NodeType::U64,
            Byml::Double(_) => NodeType::Double,
            Byml::Null => NodeType::Null,
        }
    }

    #[inline(always)]
    fn is_non_inline_type(&self) -> bool {
        matches!(
            self,
            Byml::Array(_)
                | Byml::Hash(_)
                | Byml::BinaryData(_)
                | Byml::I64(_)
                | Byml::U64(_)
                | Byml::Double(_)
        )
    }
}

#[cfg(test)]
pub(self) static FILES: &[&str] = &[
    "A-1_Dynamic",
    "ActorInfo.product",
    "EventInfo.product",
    "GameROMPlayer",
    "LevelSensor",
    "MainFieldLocation",
    "MainFieldStatic",
    "Preset0_Field",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors() {
        let mut actorinfo =
            Byml::from_binary(std::fs::read("test/byml/ActorInfo.product.byml").unwrap()).unwrap();
        let actorinfo_hash = actorinfo.as_hash_mut().unwrap();
        for obj in actorinfo_hash
            .get_mut("Actors")
            .unwrap()
            .as_array_mut()
            .unwrap()
        {
            let hash = obj.as_hash_mut().unwrap();
            *hash.get_mut("name").unwrap().as_string_mut().unwrap() = "test".into();
            assert_eq!(hash["name"].as_string().unwrap(), "test");
        }
    }
}
