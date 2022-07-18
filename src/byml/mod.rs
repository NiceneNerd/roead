mod writer;
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
    version >= 2 && version <= 4
}

#[derive(Debug, thiserror::Error)]
pub enum BymlError {
    #[error("Incorrect BYML node type: found `{0}`, expected `{1}`.")]
    TypeError(std::string::String, std::string::String),
    #[error(transparent)]
    BinaryRwError(#[from] binrw::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Error parsing BYML data: {0}")]
    ParseError(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Byml {
    String(String),
    Binary(Vec<u8>),
    Array(Vec<Byml>),
    Hash(im::OrdMap<String, Byml>),
    Bool(bool),
    Int(i32),
    Float(f32),
    UInt(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
    Null,
}

impl Byml {
    fn get_node_type(&self) -> NodeType {
        match self {
            Byml::String(_) => NodeType::String,
            Byml::Binary(_) => NodeType::Binary,
            Byml::Array(_) => NodeType::Array,
            Byml::Hash(_) => NodeType::Hash,
            Byml::Bool(_) => NodeType::Bool,
            Byml::Int(_) => NodeType::Int,
            Byml::Float(_) => NodeType::Float,
            Byml::UInt(_) => NodeType::UInt,
            Byml::Int64(_) => NodeType::Int64,
            Byml::UInt64(_) => NodeType::UInt64,
            Byml::Double(_) => NodeType::Double,
            Byml::Null => NodeType::Null,
        }
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
