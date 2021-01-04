use crate::{ffi, Endian};
use std::{
    collections::BTreeMap,
    ops::{Deref, Index, IndexMut},
    pin::Pin,
    usize,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BymlError {
    #[error("Invalid BYML magic, expected \"BY\" or \"YB\", found {0}")]
    MagicError(String),
    #[error("Compressed BYML could not be decompressed: {0}")]
    Yaz0Error(#[from] crate::yaz0::Yaz0Error),
    #[error("BYML value is not of expected type")]
    TypeError,
    #[error("Failed to parse BYML: {0}")]
    OeadError(#[from] cxx::Exception),
}

type Result<T> = std::result::Result<T, BymlError>;
pub type Hash = BTreeMap<String, Byml>;

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

impl Byml {
    pub fn as_bool(&self) -> Result<bool> {
        if let Byml::Bool(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_int(&self) -> Result<i32> {
        if let Byml::Int(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_uint(&self) -> Result<u32> {
        if let Byml::UInt(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_int64(&self) -> Result<i64> {
        if let Byml::Int64(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_uint64(&self) -> Result<u64> {
        if let Byml::UInt64(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_float(&self) -> Result<f32> {
        if let Byml::Float(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_double(&self) -> Result<f64> {
        if let Byml::Double(v) = self {
            Ok(*v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_string(&self) -> Result<&str> {
        if let Byml::String(v) = self {
            Ok(v.as_str())
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_binary(&self) -> Result<&[u8]> {
        if let Byml::Binary(v) = self {
            Ok(v.as_slice())
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_array(&self) -> Result<&[Byml]> {
        if let Byml::Array(v) = self {
            Ok(v.as_slice())
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_hash(&self) -> Result<&Hash> {
        if let Byml::Hash(v) = self {
            Ok(v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_mut_string(&mut self) -> Result<&mut str> {
        if let Byml::String(v) = self {
            Ok(v.as_mut_str())
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_mut_binary(&mut self) -> Result<&mut [u8]> {
        if let Byml::Binary(v) = self {
            Ok(v.as_mut_slice())
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_mut_array(&mut self) -> Result<&mut [Byml]> {
        if let Byml::Array(v) = self {
            Ok(v.as_mut_slice())
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn as_mut_hash(&self) -> Result<&Hash> {
        if let Byml::Hash(v) = self {
            Ok(v)
        } else {
            Err(BymlError::TypeError)
        }
    }

    pub fn from_binary(data: &[u8]) -> Result<Self> {
        let byml = ffi::BymlFromBinary(data)?;
        Ok(match byml.GetType() {
            ffi::BymlType::Hash | ffi::BymlType::Array | ffi::BymlType::Null => {
                Self::from_ffi(byml.as_ref().unwrap())
            }
            _ => unreachable!(),
        })
    }

    pub fn from_text(text: &str) -> Result<Self> {
        let byml = ffi::BymlFromText(text)?;
        Ok(match byml.GetType() {
            ffi::BymlType::Hash | ffi::BymlType::Array | ffi::BymlType::Null => {
                Self::from_ffi(byml.as_ref().unwrap())
            }
            _ => unreachable!(),
        })
    }

    pub fn to_text(&self) -> String {
        ffi::BymlToText(self)
    }

    pub fn to_binary(&self, endian: Endian) -> Vec<u8> {
        ffi::BymlToBinary(self, matches!(endian, Endian::Big), 2)
    }

    pub fn to_binary_with_version(&self, endian: Endian, version: u8) -> Vec<u8> {
        if version > 4 {
            panic!("Version must be <= 4")
        }
        ffi::BymlToBinary(self, matches!(endian, Endian::Big), version as usize)
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
    fn dump_yml() {
        let data = std::fs::read("include/oead/test/byml/files/GameROMPlayer.byml").unwrap();
        let byml = Byml::from_binary(&data).unwrap();
        println!("{}", byml.to_text());
    }

    #[test]
    fn binary_roundtrip() {
        let text =
            std::fs::read_to_string("include/oead/test/byml/files/ActorInfo.product.yml").unwrap();
        let byml = Byml::from_text(&text4555555555555555555tntf ff    cvvfndxfxv).unwrap();
        let bytes = byml.to_binary(Endian::Big);
        let byml2 = Byml::from_binary(&bytes).unwrap();
        assert_eq!(byml, byml2);
    }
}
