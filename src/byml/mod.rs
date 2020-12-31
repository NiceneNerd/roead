use crate::{ffi, Endian};
use std::{collections::BTreeMap, pin::Pin};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BymlError {
    #[error("Invalid BYML magic, expected \"BY\" or \"YB\", found {0}")]
    MagicError(String),
    #[error("Compressed BYML could not be decompressed: {0}")]
    Yaz0Error(#[from] crate::yaz0::Yaz0Error),
    #[error("Failed to parse BYML: {0}")]
    OeadError(#[from] cxx::Exception),
}

type Result<T> = std::result::Result<T, BymlError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Byml {
    Null,
    String(String),
    Binary(Vec<u8>),
    Array(Vec<Byml>),
    Hash(BTreeMap<String, Byml>),
    Bool(bool),
    Int(i32),
    Float(f32),
    UInt(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
}

impl Byml {
    pub fn from_binary(data: &[u8]) -> Result<Self> {
        let byml = ffi::BymlFromBinary(data)?;
        Ok(match byml.GetType() {
            ffi::BymlType::Hash | ffi::BymlType::Array | ffi::BymlType::Null => {
                Self::from_ffi(byml.as_ref().unwrap())
            }
            _ => unreachable!(),
        })
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
            ffi::BymlType::Binary => Byml::Binary(byml.GetBinary().iter().map(|u| *u).collect()),
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
}

#[cfg(test)]
mod tests {
    use super::Byml;

    #[test]
    fn read_byml() {
        for file in ["include/oead/test/byml/files/MainFieldLocation.byml", "include/oead/test/byml/files/ActorInfo.product.byml", "include/oead/test/byml/files/EventInfo.product.byml", "include/oead/test/byml/files/A-1_Dynamic.byml"].iter() {
            let data = std::fs::read(file).unwrap();
            let byml = Byml::from_binary(&data).unwrap();
            dbg!(byml);
        }
    }
}
