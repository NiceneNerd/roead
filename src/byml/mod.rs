//! Bindings for the `oead::byml` module.
//!
//! A `Byml` type will usually be constructed from binary data or a YAML string,
//! e.g.
//! ```
//! # use roead::byml::Byml;
//! # use std::{fs::read, error::Error};
//! # fn docttest() -> Result<(), Box<dyn Error>> {
//! let buf: Vec<u8> = std::fs::read("A-1_Static.smubin")?;
//! let map_unit = Byml::from_binary(&buf)?;
//! let text: String = std::fs::read_to_string("A-1_Static.yml")?;
//! let map_unit2 = Byml::from_text(&text)?;
//! assert_eq!(map_unit, map_unit2);
//! # Ok(())
//! # }
//! ```
//! You can also easily serialize to binary or a YAML string.
//! ```
//! # use roead::{byml::Byml, Endian};
//! # fn docttest() -> Result<(), Box<dyn std::error::Error>> {
//! let buf: Vec<u8> = std::fs::read("A-1_Static.smubin")?;
//! let map_unit = Byml::from_binary(&buf)?;
//! std::fs::write("A-1_Static.yml", &map_unit.to_text())?;
//! std::fs::write("A-1_Static.copy.mubin", &map_unit.to_binary(Endian::Big))?;
//! # Ok(())
//! # }
//! ```
//!
//! A number of convenience getters are available which return a result for a variant value:
//! ```
//! # use roead::byml::Byml;
//! # use std::collections::BTreeMap;
//! # fn docttest() -> Result<(), Box<dyn std::error::Error>> {
//! # let some_data = b"BYML";
//! let doc = Byml::from_binary(some_data)?;
//! let hash: &BTreeMap<String, Byml> = doc.as_hash()?;
//! # Ok(())
//! # }
//! ```
//!
//! Most of the node types are fairly self-explanatory. Arrays are implemented as `Vec<Byml>`, and
//! hash nodes as `BTreeMap<String, Byml>`.
//!
//! For convenience, a `Byml` *known* to be an array or hash node can be indexed. **Panics if the
//! node has the wrong type, the index has the wrong type, or the index is not found**.
//! ```
//! # use roead::byml::Byml;
//! # fn docttest() -> Result<(), Box<dyn std::error::Error>> {
//! let buf: Vec<u8> = std::fs::read("ActorInfo.product.sbyml")?;
//! let actor_info = Byml::from_binary(&buf)?;
//! assert_eq!(actor_info["Actors"].as_array()?.len(), 7934);
//! assert_eq!(actor_info["Hashes"][0].as_int()?, 31119);
//! # Ok(())
//! # }
//! ```
use crate::{ffi, Endian};
use std::{
    collections::BTreeMap,
    iter::FromIterator,
    ops::{Index, IndexMut},
};
use thiserror::Error;

/// An error when serializing/deserializing BYML documents
#[derive(Error, Debug)]
pub enum BymlError {
    #[error("Invalid BYML magic, expected \"BY\" or \"YB\", found {0}")]
    MagicError(String),
    #[error("Compressed BYML could not be decompressed: {0}")]
    Yaz0Error(#[from] crate::yaz0::Yaz0Error),
    #[error("BYML value is not of expected type")]
    TypeError,
    /// Wraps any other error returned by `oead` in C++
    #[error("Failed to parse BYML: {0}")]
    OeadError(#[from] cxx::Exception),
}

pub type Result<T> = std::result::Result<T, BymlError>;
pub type Hash = BTreeMap<String, Byml>;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Byml {
    Null,
    String(String),
    Binary(Vec<u8>),
    Array(Vec<Byml>),
    Hash(Hash),
    Bool(bool),
    Int(i32),
    Float(f32),
    UInt(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
}

impl Default for Byml {
    fn default() -> Self {
        Self::Null
    }
}

impl<'a, I: Into<BymlIndex<'a>>> Index<I> for Byml {
    type Output = Byml;
    fn index(&self, index: I) -> &Self::Output {
        let index: BymlIndex = index.into();
        match self {
            Self::Array(a) => {
                if let BymlIndex::ArrayIdx(idx) = index {
                    &a[idx]
                } else {
                    panic!("Wrong index type for Byml::Array")
                }
            }
            Self::Hash(h) => {
                if let BymlIndex::HashIdx(key) = index {
                    &h[key]
                } else {
                    panic!("Wrong index type for Byml::Hash")
                }
            }
            _ => panic!("Cannot index, Byml type is not Hash or Array"),
        }
    }
}

impl<'a, I: Into<BymlIndex<'a>>> IndexMut<I> for Byml {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let index: BymlIndex = index.into();
        match self {
            Self::Array(a) => {
                if let BymlIndex::ArrayIdx(idx) = index {
                    &mut a[idx]
                } else {
                    panic!("Wrong index type for Byml::Array")
                }
            }
            Self::Hash(h) => {
                if let BymlIndex::HashIdx(key) = index {
                    h.get_mut(key).unwrap()
                } else {
                    panic!("Wrong index type for Byml::Hash")
                }
            }
            _ => panic!("Cannot index, Byml type is not Hash or Array"),
        }
    }
}

impl FromIterator<Byml> for Byml {
    fn from_iter<T: IntoIterator<Item = Byml>>(iter: T) -> Self {
        Self::Array(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<(&'a str, Byml)> for Byml {
    fn from_iter<T: IntoIterator<Item = (&'a str, Byml)>>(iter: T) -> Self {
        Self::Hash(iter.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }
}

impl FromIterator<(String, Byml)> for Byml {
    fn from_iter<T: IntoIterator<Item = (String, Byml)>>(iter: T) -> Self {
        Self::Hash(iter.into_iter().collect())
    }
}

impl Byml {
    /// Returns a result with the inner boolean value or a type error
    pub fn as_bool(&self) -> Result<bool> {
        if let Byml::Bool(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner s32 value or a type error
    pub fn as_int(&self) -> Result<i32> {
        if let Byml::Int(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner u32 value or a type error
    pub fn as_uint(&self) -> Result<u32> {
        if let Byml::UInt(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner i64 value or a type error
    pub fn as_int64(&self) -> Result<i64> {
        if let Byml::Int64(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner u64 value or a type error
    pub fn as_uint64(&self) -> Result<u64> {
        if let Byml::UInt64(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner f32 value or a type error
    pub fn as_float(&self) -> Result<f32> {
        if let Byml::Float(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner f64 value or a type error
    pub fn as_double(&self) -> Result<f64> {
        if let Byml::Double(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner string slice or a type error
    pub fn as_string(&self) -> Result<&str> {
        if let Byml::String(v) = self {
            Ok(v.as_str())
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner byte slice or a type error
    pub fn as_binary(&self) -> Result<&[u8]> {
        if let Byml::Binary(v) = self {
            Ok(v.as_slice())
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the inner Byml array slice or a type error
    pub fn as_array(&self) -> Result<&[Byml]> {
        if let Byml::Array(v) = self {
            Ok(v.as_slice())
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with a reference to the inner hash or a type error
    pub fn as_hash(&self) -> Result<&Hash> {
        if let Byml::Hash(v) = self {
            Ok(v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the mutable inner string slice or a type error
    pub fn as_mut_string(&mut self) -> Result<&mut str> {
        if let Byml::String(v) = self {
            Ok(v.as_mut_str())
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the mutable inner byte slice or a type error
    pub fn as_mut_binary(&mut self) -> Result<&mut [u8]> {
        if let Byml::Binary(v) = self {
            Ok(v.as_mut_slice())
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with the mutable inner Byml array slice or a type error
    pub fn as_mut_array(&mut self) -> Result<&mut [Byml]> {
        if let Byml::Array(v) = self {
            Ok(v.as_mut_slice())
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Returns a result with a mutable reference to the inner hash or a type error
    pub fn as_mut_hash(&mut self) -> Result<&mut Hash> {
        if let Byml::Hash(v) = self {
            Ok(v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    /// Load a document from binary data.
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        let byml = ffi::BymlFromBinary(data)?;
        Ok(match byml.GetType() {
            ffi::BymlType::Hash | ffi::BymlType::Array | ffi::BymlType::Null => {
                Self::from_ffi(byml.as_ref().unwrap())
            }
            _ => unreachable!(),
        })
    }

    /// Load a document from YAML text.
    pub fn from_text<S: AsRef<str>>(text: S) -> Result<Self> {
        let byml = ffi::BymlFromText(text.as_ref())?;
        Ok(match byml.GetType() {
            ffi::BymlType::Hash | ffi::BymlType::Array | ffi::BymlType::Null => {
                Self::from_ffi(byml.as_ref().unwrap())
            }
            _ => unreachable!(),
        })
    }

    /// Serialize the document to YAML. This can only be done for Null, Array or Hash nodes.
    pub fn to_text(&self) -> String {
        if matches!(self, Byml::Array(_) | Byml::Hash(_) | Byml::Null) {
            ffi::BymlToText(self)
        } else {
            panic!("Root node must be an array, hash, or null value")
        }
    }

    /// Serialize the document to BYML with the specified endianness and default version (2).
    /// This can only be done for Null, Array or Hash nodes.
    pub fn to_binary(&self, endian: Endian) -> Vec<u8> {
        if matches!(self, Byml::Array(_) | Byml::Hash(_) | Byml::Null) {
            ffi::BymlToBinary(self, matches!(endian, Endian::Big), 2)
        } else {
            panic!("Root node must be an array, hash, or null value")
        }
    }

    /// Serialize the document to BYML with the specified endianness and version number.
    /// This can only be done for Null, Array or Hash nodes.
    pub fn to_binary_with_version(&self, endian: Endian, version: u8) -> Vec<u8> {
        if version > 4 {
            panic!("Version must be <= 4")
        }
        if matches!(self, Byml::Array(_) | Byml::Hash(_) | Byml::Null) {
            ffi::BymlToBinary(self, matches!(endian, Endian::Big), version as usize)
        } else {
            panic!("Root node must be an array, hash, or null value")
        }
    }

    fn from_ffi(byml: &ffi::Byml) -> Self {
        match byml.GetType() {
            ffi::BymlType::Hash => Self::Hash({
                let chash = byml.GetHash();
                let keys = ffi::GetHashKeys(chash);
                keys.iter()
                    .map(|k| (k.to_str().unwrap().to_owned(), Self::from_ffi(chash.at(k))))
                    .collect()
            }),
            ffi::BymlType::Bool => Byml::Bool(byml.GetBool()),
            ffi::BymlType::Binary => Byml::Binary(byml.GetBinary().iter().copied().collect()),
            ffi::BymlType::Int => Byml::Int(byml.GetInt()),
            ffi::BymlType::UInt => Byml::UInt(byml.GetUInt()),
            ffi::BymlType::Int64 => Byml::Int64(byml.GetInt64()),
            ffi::BymlType::UInt64 => Byml::UInt64(byml.GetUInt64()),
            ffi::BymlType::Float => Byml::Float(byml.GetFloat()),
            ffi::BymlType::Double => Byml::Double(byml.GetDouble()),
            ffi::BymlType::String => Byml::String(byml.GetString().to_str().unwrap().to_owned()),
            ffi::BymlType::Array => {
                Self::Array(byml.GetArray().iter().map(Self::from_ffi).collect())
            }
            _ => Self::Null,
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Byml::Array(v) => v.len(),
            Byml::Hash(v) => v.len(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn get_key_by_index(&self, index: usize) -> &String {
        if let Byml::Hash(h) = self {
            h.iter().nth(index).unwrap().0
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get(&self, index: usize) -> &Byml {
        match self {
            Byml::Hash(h) => h.iter().nth(index).unwrap().1,
            Byml::Array(a) => a.get(index).unwrap(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn get_ffi_type(&self) -> ffi::BymlType {
        match self {
            Self::Array(_) => ffi::BymlType::Array,
            Self::Hash(_) => ffi::BymlType::Hash,
            Self::Null => ffi::BymlType::Null,
            Self::Bool(_) => ffi::BymlType::Bool,
            Self::Binary(_) => ffi::BymlType::Binary,
            Self::Int(_) => ffi::BymlType::Int,
            Self::UInt(_) => ffi::BymlType::UInt,
            Self::Int64(_) => ffi::BymlType::Int64,
            Self::UInt64(_) => ffi::BymlType::UInt64,
            Self::Float(_) => ffi::BymlType::Float,
            Self::Double(_) => ffi::BymlType::Double,
            Self::String(_) => ffi::BymlType::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Byml, Endian};

    #[test]
    fn read_byml() {
        for file in [
            "include/oead/test/byml/files/MainFieldLocation.byml",
            "include/oead/test/byml/files/ActorInfo.product.byml",
            "include/oead/test/byml/files/EventInfo.product.byml",
            "include/oead/test/byml/files/A-1_Dynamic.byml",
        ]
        .iter()
        {
            let data = std::fs::read(file).unwrap();
            let byml = Byml::from_binary(&data).unwrap();
            assert!(matches!(byml, Byml::Array(_) | Byml::Hash(_)))
        }
    }

    #[test]
    fn read_actorinfo() {
        let text =
            std::fs::read_to_string("include/oead/test/byml/files/ActorInfo.product.yml").unwrap();
        let byml = Byml::from_text(&text).unwrap();
        assert!(matches!(byml, Byml::Hash(_)));
        assert!(byml["Actors"].as_array().unwrap().len() > 7000);
        byml["Actors"]
            .as_array()
            .unwrap()
            .iter()
            .take(20)
            .for_each(|a| println!("{}", a["name"].as_string().unwrap()))
    }

    #[test]
    fn byml_to_yml() {
        let data = std::fs::read("include/oead/test/byml/files/GameROMPlayer.byml").unwrap();
        let byml = Byml::from_binary(&data).unwrap();
        println!("{}", byml.to_text());
    }

    #[test]
    fn binary_roundtrip() {
        let text =
            std::fs::read_to_string("include/oead/test/byml/files/ActorInfo.product.yml").unwrap();
        let byml = Byml::from_text(&text).unwrap();
        let bytes = byml.to_binary(Endian::Big);
        let byml2 = Byml::from_binary(&bytes).unwrap();
        assert_eq!(byml, byml2);
    }
}
