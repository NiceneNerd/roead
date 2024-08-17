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
//! std::fs::write(
//!     "test/aamp/A-1_Dynamic.byml",
//!     &map_unit.to_binary(Endian::Big),
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! A number of convenience getters are available which return a result for a
//! variant value:
//! ```
//! # use roead::byml::Byml;
//! # fn docttest() -> Result<(), Box<dyn std::error::Error>> {
//! # let some_data = b"BYML";
//! let doc = Byml::from_binary(some_data)?;
//! let map = doc.as_map().unwrap();
//! # Ok(())
//! # }
//! ```
//!
//! Most of the node types are fairly self-explanatory. Arrays are implemented
//! as `Vec<Byml>`, and maps as `FxHashMap<String, Byml>`. The new v7 hash maps
//! are `FxHashMap<u32, Byml>` and `FxHashMap<u32, (Byml, u32)>`.
//!
//! For convenience, a `Byml` *known* to be an array or map can be
//! indexed. **Panics if the node has the wrong type, the index has the wrong
//! type, or the index is not found**.
//! ```
//! # use roead::byml::Byml;
//! # fn docttest() -> Result<(), Box<dyn std::error::Error>> {
//! let buf: Vec<u8> = std::fs::read("test/byml/ActorInfo.product.byml")?;
//! let actor_info = Byml::from_binary(&buf)?;
//! assert_eq!(actor_info["Actors"].as_array().unwrap().len(), 7934);
//! assert_eq!(actor_info["Hashes"][0].as_i32().unwrap(), 31119);
//! # Ok(())
//! # }
//! ```
#[cfg(feature = "yaml")]
mod text;
mod writer;
use num_traits::AsPrimitive;
use smartstring::alias::String;

use crate::{Error, Result};
mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[binrw::binrw]
#[brw(repr = u8)]
#[repr(u8)]
enum NodeType {
    HashMap = 0x20,
    ValueHashMap = 0x21,
    String = 0xa0,
    Binary = 0xa1,
    File = 0xa2,
    Array = 0xc0,
    Map = 0xc1,
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
    matches!(
        node_type,
        NodeType::Array | NodeType::Map | NodeType::ValueHashMap | NodeType::HashMap
    )
}

#[inline(always)]
const fn is_valid_version(version: u16) -> bool {
    version >= 1 && version < 8
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
pub type Map = rustc_hash::FxHashMap<String, Byml>;
pub type HashMap = rustc_hash::FxHashMap<u32, Byml>;
pub type ValueHashMap = rustc_hash::FxHashMap<u32, (Byml, u32)>;

/// Convenience type used for indexing into `Byml`s
pub enum BymlIndex<'a> {
    /// Index into a hash node. The key is a string.
    StringIdx(&'a str),
    /// Index into a hash node. The key is a u32 hash.
    HashIdx(u32),
    /// Index into an array node. The index is an integer.
    ArrayIdx(usize),
}

impl<'a> From<&'a str> for BymlIndex<'a> {
    fn from(s: &'a str) -> Self {
        Self::StringIdx(s)
    }
}

impl<'a> From<&'a String> for BymlIndex<'a> {
    fn from(s: &'a String) -> Self {
        Self::StringIdx(s)
    }
}

impl<'a> From<usize> for BymlIndex<'a> {
    fn from(idx: usize) -> Self {
        Self::ArrayIdx(idx)
    }
}

impl<'a> From<i32> for BymlIndex<'a> {
    fn from(value: i32) -> Self {
        assert!(!value.is_negative());
        Self::ArrayIdx(value as usize)
    }
}

impl<'a> From<u32> for BymlIndex<'a> {
    fn from(value: u32) -> Self {
        Self::HashIdx(value)
    }
}

/// Represents a Nintendo binary YAML (BYML) document or node.
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum Byml {
    /// String value.
    String(String),
    /// Binary data (not used in BOTW).
    BinaryData(Vec<u8>),
    /// File data
    FileData(Vec<u8>),
    /// Array of BYML nodes.
    Array(Vec<Byml>),
    /// Hash map of BYML nodes with string keys.
    Map(Map),
    /// Hash map of BYML nodes with u32 keys.
    HashMap(HashMap),
    /// Hash map of BYML nodes with u32 keys and additional value.
    ValueHashMap(ValueHashMap),
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

impl Byml {
    fn type_name(&self) -> String {
        match self {
            Byml::String(_) => "String".into(),
            Byml::BinaryData(_) => "Binary".into(),
            Byml::FileData(_) => "File".into(),
            Byml::Array(_) => "Array".into(),
            Byml::Map(_) => "Map".into(),
            Byml::HashMap(_) => "HashMap".into(),
            Byml::ValueHashMap(_) => "ValueHashMap".into(),
            Byml::Bool(_) => "Bool".into(),
            Byml::I32(_) => "I32".into(),
            Byml::Float(_) => "Float".into(),
            Byml::U32(_) => "U32".into(),
            Byml::I64(_) => "I64".into(),
            Byml::U64(_) => "U64".into(),
            Byml::Double(_) => "Double".into(),
            Byml::Null => "Null".into(),
        }
    }

    /// Checks if the BYML node is a null node
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Get a reference to the inner bool value.
    pub fn as_bool(&self) -> Result<bool> {
        if let Self::Bool(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "Bool"))
        }
    }

    /// Get a reference to the inner i32 value.
    pub fn as_i32(&self) -> Result<i32> {
        if let Self::I32(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "I32"))
        }
    }

    /// Get a reference to the inner u32 value.
    pub fn as_u32(&self) -> Result<u32> {
        if let Self::U32(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "U32"))
        }
    }

    /// Get a reference to the inner i64 value.
    pub fn as_i64(&self) -> Result<i64> {
        if let Self::I64(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "I64"))
        }
    }

    /// Get a reference to the inner u64 value.
    pub fn as_u64(&self) -> Result<u64> {
        if let Self::U64(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "U64"))
        }
    }

    /// Get the inner value as an integer of any type. Casts the value using
    /// [`as`](https://doc.rust-lang.org/std/keyword.as.html) where necessary.
    /// Note that this is subject to all the normal risks of casting with `as`.
    pub fn as_int<T>(&self) -> Result<T>
    where
        T: Copy + 'static,
        i32: AsPrimitive<T>,
        u32: AsPrimitive<T>,
        i64: AsPrimitive<T>,
        u64: AsPrimitive<T>,
    {
        match self {
            Byml::I32(i) => Ok(i.as_()),
            Byml::I64(i) => Ok(i.as_()),
            Byml::U32(i) => Ok(i.as_()),
            Byml::U64(i) => Ok(i.as_()),
            _ => Err(Error::TypeError(self.type_name(), "an integer")),
        }
    }

    /// Get the inner value as a number of any type. Casts the value using
    /// [`as`](https://doc.rust-lang.org/std/keyword.as.html) where necessary.
    /// Note that this is subject to all the normal risks of casting with `as`.
    pub fn as_num<T>(&self) -> Result<T>
    where
        T: Copy + 'static,
        i32: AsPrimitive<T>,
        u32: AsPrimitive<T>,
        i64: AsPrimitive<T>,
        u64: AsPrimitive<T>,
        f32: AsPrimitive<T>,
        f64: AsPrimitive<T>,
    {
        match self {
            Byml::I32(i) => Ok(i.as_()),
            Byml::I64(i) => Ok(i.as_()),
            Byml::U32(i) => Ok(i.as_()),
            Byml::U64(i) => Ok(i.as_()),
            Byml::Float(i) => Ok(i.as_()),
            Byml::Double(i) => Ok(i.as_()),
            _ => Err(Error::TypeError(self.type_name(), "a number")),
        }
    }

    /// Get a reference to the inner f32 value.
    pub fn as_float(&self) -> Result<f32> {
        if let Self::Float(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "Float"))
        }
    }

    /// Get a reference to the inner f64 value.
    pub fn as_double(&self) -> Result<f64> {
        if let Self::Double(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "Double"))
        }
    }

    /// Get a reference to the inner string value.
    pub fn as_string(&self) -> Result<&String> {
        if let Self::String(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "String"))
        }
    }

    /// Get a reference to the inner byte slice.
    pub fn as_binary_data(&self) -> Result<&[u8]> {
        if let Self::BinaryData(v) = self {
            Ok(v.as_slice())
        } else {
            Err(Error::TypeError(self.type_name(), "BinaryData"))
        }
    }

    /// Get a reference to the inner array of BYML nodes.
    pub fn as_array(&self) -> Result<&[Byml]> {
        if let Self::Array(v) = self {
            Ok(v.as_slice())
        } else {
            Err(Error::TypeError(self.type_name(), "Array"))
        }
    }

    /// Get a reference to the inner string-keyed hash map of BYML nodes.
    pub fn as_map(&self) -> Result<&Map> {
        if let Self::Map(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Map"))
        }
    }

    /// Get a reference to the inner u32-keyed hash map of BYML nodes.
    pub fn as_hash_map(&self) -> Result<&HashMap> {
        if let Self::HashMap(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "HashMap"))
        }
    }

    /// Get a reference to the inner u32-keyed hash map of BYML nodes.
    pub fn as_value_hash_map(&self) -> Result<&ValueHashMap> {
        if let Self::ValueHashMap(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "ValueHashMap"))
        }
    }

    /// Get a mutable reference to the inner string value.
    pub fn as_mut_string(&mut self) -> Result<&mut String> {
        if let Self::String(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "String"))
        }
    }

    /// Get a mutable reference to the inner bool value.
    pub fn as_mut_bool(&mut self) -> Result<&mut bool> {
        if let Self::Bool(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Bool"))
        }
    }

    /// Get a mutable reference to the inner i32 value.
    pub fn as_mut_i32(&mut self) -> Result<&mut i32> {
        if let Self::I32(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "I32"))
        }
    }

    /// Get a mutable reference to the inner u32 value.
    pub fn as_mut_u32(&mut self) -> Result<&mut u32> {
        if let Self::U32(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "U32"))
        }
    }

    /// Get a mutable reference to the inner i64 value.
    pub fn as_mut_i64(&mut self) -> Result<&mut i64> {
        if let Self::I64(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "I64"))
        }
    }

    /// Get a mutable reference to the inner u64 value.
    pub fn as_mut_u64(&mut self) -> Result<&mut u64> {
        if let Self::U64(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "U64"))
        }
    }

    /// Get a mutable reference to the inner f32 value.
    pub fn as_mut_float(&mut self) -> Result<&mut f32> {
        if let Self::Float(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Float"))
        }
    }

    /// Get a mutable reference to the inner f64 value.
    pub fn as_mut_double(&mut self) -> Result<&mut f64> {
        if let Self::Double(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Double"))
        }
    }

    /// Get a mutable reference to the inner byte slice.
    pub fn as_mut_binary_data(&mut self) -> Result<&mut [u8]> {
        if let Self::BinaryData(v) = self {
            Ok(v.as_mut_slice())
        } else {
            Err(Error::TypeError(self.type_name(), "BinaryData"))
        }
    }

    /// Get a mutable reference to the inner array of BYML nodes.
    pub fn as_mut_array(&mut self) -> Result<&mut Vec<Byml>> {
        if let Self::Array(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Array"))
        }
    }

    /// Get a mutable reference to the inner hash map of BYML nodes.
    pub fn as_mut_map(&mut self) -> Result<&mut Map> {
        if let Self::Map(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Hash"))
        }
    }

    /// Get a reference to the inner u32-keyed hash map of BYML nodes.
    pub fn as_mut_hash_map(&mut self) -> Result<&mut HashMap> {
        if let Self::HashMap(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "HashMap"))
        }
    }

    /// Get a reference to the inner u32-keyed hash map of BYML nodes.
    pub fn as_mut_value_hash_map(&mut self) -> Result<&mut ValueHashMap> {
        if let Self::ValueHashMap(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "ValueHashMap"))
        }
    }

    /// Extract the inner string value.
    pub fn into_string(self) -> Result<String> {
        if let Self::String(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "String"))
        }
    }

    /// Extract the inner bool value.
    pub fn into_bool(self) -> Result<bool> {
        if let Self::Bool(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Bool"))
        }
    }

    /// Extract the inner i32 value.
    pub fn into_i32(self) -> Result<i32> {
        if let Self::I32(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "I32"))
        }
    }

    /// Extract the inner u32 value.
    pub fn into_u32(self) -> Result<u32> {
        if let Self::U32(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "U32"))
        }
    }

    /// Extract the inner i64 value.
    pub fn into_i64(self) -> Result<i64> {
        if let Self::I64(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "I64"))
        }
    }

    /// Extract the inner u64 value.
    pub fn into_u64(self) -> Result<u64> {
        if let Self::U64(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "U64"))
        }
    }

    /// Extract the inner f32 value.
    pub fn into_float(self) -> Result<f32> {
        if let Self::Float(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Float"))
        }
    }

    /// Extract the inner f64 value.
    pub fn into_double(self) -> Result<f64> {
        if let Self::Double(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Double"))
        }
    }

    /// Extract the inner byte slice value.
    pub fn into_binary_data(self) -> Result<Vec<u8>> {
        if let Self::BinaryData(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "BinaryData"))
        }
    }

    /// Extract the inner Byml array value.
    pub fn into_array(self) -> Result<Vec<Byml>> {
        if let Self::Array(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Array"))
        }
    }

    /// Extract the inner map value.
    pub fn into_map(self) -> Result<Map> {
        if let Self::Map(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Map"))
        }
    }

    /// Extract the inner hash map value.
    pub fn into_hash_map(self) -> Result<HashMap> {
        if let Self::HashMap(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "HashMap"))
        }
    }

    /// Extract the inner value hash map value.
    pub fn into_value_hash_map(self) -> Result<ValueHashMap> {
        if let Self::ValueHashMap(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "ValueHashMap"))
        }
    }
}

impl From<bool> for Byml {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl TryFrom<Byml> for bool {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        match value {
            Byml::Bool(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<i32> for Byml {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl TryFrom<Byml> for i32 {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        value.as_int().map_err(|_| value)
    }
}

impl From<u32> for Byml {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl TryFrom<Byml> for u32 {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        value.as_int().map_err(|_| value)
    }
}

impl From<i64> for Byml {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl TryFrom<Byml> for i64 {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        value.as_int().map_err(|_| value)
    }
}

impl From<u64> for Byml {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl TryFrom<Byml> for u64 {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        value.as_int().map_err(|_| value)
    }
}

impl From<f32> for Byml {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl TryFrom<Byml> for f32 {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        value.as_num().map_err(|_| value)
    }
}

impl From<f64> for Byml {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl TryFrom<Byml> for f64 {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        value.as_num().map_err(|_| value)
    }
}

impl TryFrom<Byml> for Vec<u8> {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        match value {
            Byml::BinaryData(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Vec<Byml>> for Byml {
    fn from(value: Vec<Byml>) -> Self {
        Self::Array(value)
    }
}

impl TryFrom<Byml> for Vec<Byml> {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        match value {
            Byml::Array(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Map> for Byml {
    fn from(value: Map) -> Self {
        Self::Map(value)
    }
}

impl TryFrom<Byml> for Map {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        match value {
            Byml::Map(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<HashMap> for Byml {
    fn from(value: HashMap) -> Self {
        Self::HashMap(value)
    }
}

impl TryFrom<Byml> for HashMap {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        match value {
            Byml::HashMap(map) => Ok(map),
            _ => Err(value),
        }
    }
}

impl From<ValueHashMap> for Byml {
    fn from(value: ValueHashMap) -> Self {
        Self::ValueHashMap(value)
    }
}

impl TryFrom<Byml> for ValueHashMap {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        match value {
            Byml::ValueHashMap(map) => Ok(map),
            _ => Err(value),
        }
    }
}

impl From<&str> for Byml {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<String> for Byml {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for Byml {
    fn from(value: &String) -> Self {
        Self::String(value.clone())
    }
}

impl From<std::string::String> for Byml {
    fn from(value: std::string::String) -> Self {
        Self::String(value.into())
    }
}

impl From<&std::string::String> for Byml {
    fn from(value: &std::string::String) -> Self {
        Self::String(value.into())
    }
}

impl TryFrom<Byml> for String {
    type Error = Byml;

    fn try_from(value: Byml) -> std::result::Result<Self, Self::Error> {
        match value {
            Byml::String(v) => Ok(v),
            _ => Err(value),
        }
    }
}

// impl From<&[u8]> for Byml {
//     fn from(value: &[u8]) -> Self {
//         Self::BinaryData(value.to_vec())
//     }
// }

impl From<&[Byml]> for Byml {
    fn from(value: &[Byml]) -> Self {
        Self::Array(value.to_vec())
    }
}

impl<S: Into<String>> FromIterator<(S, Byml)> for Byml {
    fn from_iter<T: IntoIterator<Item = (S, Byml)>>(iter: T) -> Self {
        Self::Map(iter.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

impl FromIterator<Byml> for Byml {
    fn from_iter<T: IntoIterator<Item = Byml>>(iter: T) -> Self {
        Self::Array(iter.into_iter().collect())
    }
}

impl Default for Byml {
    fn default() -> Self {
        Self::Null
    }
}

impl PartialEq for Byml {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Byml::String(s1), Byml::String(s2)) => s1 == s2,
            (Byml::BinaryData(d1), Byml::BinaryData(d2)) => d1 == d2,
            (Byml::FileData(d1), Byml::FileData(d2)) => d1 == d2,
            (Byml::Array(a1), Byml::Array(a2)) => a1 == a2,
            (Byml::Map(h1), Byml::Map(h2)) => h1 == h2,
            (Byml::HashMap(h1), Byml::HashMap(h2)) => h1 == h2,
            (Byml::ValueHashMap(h1), Byml::ValueHashMap(h2)) => h1 == h2,
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
            Byml::FileData(b) => b.hash(state),
            Byml::Array(a) => a.hash(state),
            Byml::Map(h) => {
                for (k, v) in h.iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Byml::HashMap(h) => {
                for (k, v) in h.iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Byml::ValueHashMap(h) => {
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
            (Byml::Map(h), BymlIndex::StringIdx(k)) => &h[k],
            (Byml::HashMap(h), BymlIndex::HashIdx(i)) => &h[&i],
            (Byml::ValueHashMap(h), BymlIndex::HashIdx(i)) => &h[&i].0,
            _ => panic!("Wrong index type or node type."),
        }
    }
}

impl<'a, I: Into<BymlIndex<'a>>> std::ops::IndexMut<I> for Byml {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        match (self, index.into()) {
            (Byml::Array(a), BymlIndex::ArrayIdx(i)) => &mut a[i],
            (Byml::Map(h), BymlIndex::StringIdx(k)) => h.get_mut(k).expect("Key should be in hash"),
            (Byml::HashMap(h), BymlIndex::HashIdx(i)) => {
                h.get_mut(&i).expect("Key should be in hash")
            }
            (Byml::ValueHashMap(h), BymlIndex::HashIdx(i)) => {
                &mut h.get_mut(&i).expect("Key should be in hash").0
            }
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
            Byml::FileData(_) => NodeType::File,
            Byml::Array(_) => NodeType::Array,
            Byml::Map(_) => NodeType::Map,
            Byml::HashMap(_) => NodeType::HashMap,
            Byml::ValueHashMap(_) => NodeType::ValueHashMap,
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
                | Byml::Map(_)
                | Byml::HashMap(_)
                | Byml::ValueHashMap(_)
                | Byml::BinaryData(_)
                | Byml::FileData(_)
                | Byml::I64(_)
                | Byml::U64(_)
                | Byml::Double(_)
        )
    }
}

/// Convenience macro to construct a [`Byml`] map using map literal syntax.
/// Example:
///
/// ```
/// # use roead::byml::*;
/// let bmap = map!(
///     "SomeKey" => Byml::Bool(true),
///     "AnotherKey" => Byml::I32(0)
/// );
/// ```
///
/// Adapted from <https://github.com/bluss/maplit/blob/master/src/lib.rs>
#[macro_export]
macro_rules! map {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(map!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { map!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = map!(@count $($key),*);
            let mut _map = $crate::byml::Map::default();
            _map.reserve(_cap);

            $(
                let _ = _map.insert(::smartstring::alias::String::from($key), $value);
            )*
            $crate::byml::Byml::Map(_map)
        }
    };
}
pub use map;

/// Convenience macro to construct a [`Byml`] array using array literal syntax.
/// Example:
///
/// ```
/// # use roead::byml::*;
/// let arr = array!(Byml::Bool(true), Byml::I32(0), Byml::String("test".into()));
/// ```
#[macro_export]
macro_rules! array {
    () => (
        $crate::byml::Byml::Array(Default::default())
    );
    ($elem:expr; $n:expr) => (
        $crate::byml::Byml::Array(vec![$elem, $n])
    );
    ($($x:expr),+ $(,)?) => (
        $crate::byml::Byml::Array(vec![$($x),+])
    );
}
pub use array;

#[cfg(test)]
static FILES: &[&str] = &[
    "A-1_Dynamic",
    "D-3_Dynamic",
    "EventInfo.product",
    "GameROMPlayer",
    "LevelSensor",
    "MainFieldLocation",
    "MainFieldStatic",
    "Preset0_Field",
    "ActorInfo.product",
    "ElectricGenerator.Nin_NX_NVN.esetb",
    "USen",
    "Mrg_01e57204_MrgD100_B4-B3-B2-1A90E17A.bcett",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors() {
        let mut actorinfo =
            Byml::from_binary(std::fs::read("test/byml/ActorInfo.product.byml").unwrap()).unwrap();
        let actorinfo_hash = actorinfo.as_mut_map().unwrap();
        for obj in actorinfo_hash
            .get_mut("Actors")
            .unwrap()
            .as_mut_array()
            .unwrap()
        {
            let hash = obj.as_mut_map().unwrap();
            *hash.get_mut("name").unwrap().as_mut_string().unwrap() = "test".into();
            assert_eq!(hash["name"].as_string().unwrap(), "test");
        }
    }

    #[test]
    fn macro_test() {
        let map = map!(
            "test" => "bob".into()
        );
        assert_eq!(map["test"], Byml::String("bob".into()));
        let arr = array!(Byml::String("bob".into()), Byml::Bool(true));
        assert_eq!(arr.as_array().unwrap().len(), 2);
    }
}
