mod writer;
use decorum::{R32, R64};
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
pub type Hash = rustc_hash::FxHashMap<String, Byml>;

/// Convenience type used for indexing into `Byml`s
pub enum BymlIndex<'a> {
    HashIdx(&'a str),
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
#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Byml {
    String(String),
    BinaryData(Vec<u8>),
    Array(Vec<Byml>),
    Hash(Hash),
    Bool(bool),
    I32(i32),
    Float(R32),
    U32(u32),
    I64(i64),
    U64(u64),
    Double(R64),
    Null,
}

#[allow(clippy::derive_hash_xor_eq)]
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
            Byml::Float(f) => f.hash(state),
            Byml::U32(u) => u.hash(state),
            Byml::I64(i) => i.hash(state),
            Byml::U64(u) => u.hash(state),
            Byml::Double(d) => d.hash(state),
            Byml::Null => std::hash::Hash::hash(&0, state),
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
