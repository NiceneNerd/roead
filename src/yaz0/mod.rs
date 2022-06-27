//! Bindings for the `oead::yaz0` module, which supports Yaz0 decompression and fast compression (using syaz0).
//!
//! ## Performance
//! Decompression performance is on par with existing Yaz0 decoders.
//!
//! As of late December 2019, syaz0 is able to compress files much faster than existing Yaz0 encoders. Files that are representative of Breath of the Wild assets were compressed 20x to 30x faster than with existing public tools for an equivalent or better compression ratio, and 70-80x faster (with a slightly worse ratio) in extreme cases.
//!
//! At the default compression level, file sizes are typically within 1% of Nintendo’s.
//!
//! For detailed benchmarks, see the results files in the [test directory of the syaz0 project](https://github.com/zeldamods/syaz0/tree/master/test).
use crate::{ffi, Bytes};
use std::{ops::Deref, path::Path};
use thiserror::Error;
use unicase::UniCase;

#[derive(Error, Debug)]
pub enum Yaz0Error {
    #[error("Invalid yaz0 magic, expected \"Yaz0\", found {0}")]
    MagicError(String),
    #[error("Invalid compression level, expected 6-9, found {0}")]
    InvalidLevelError(u8),
    #[error("Not enough data to decompress, expected >16 bytes, found {0}")]
    InsufficientDataError(usize),
    #[error("oead could not compress or decompress")]
    OeadError(#[from] cxx::Exception),
}

#[derive(Debug, PartialEq)]
pub enum YazData<'a> {
    Borrowed(&'a [u8]),
    Owned(Bytes),
}

impl<'a> Deref for YazData<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(v) => v.as_slice(),
            Self::Borrowed(v) => *v,
        }
    }
}

impl<'a> From<&'a [u8]> for YazData<'a> {
    fn from(val: &'a [u8]) -> Self {
        Self::Borrowed(val)
    }
}

impl<'a, T: ?Sized + AsRef<[u8]>> PartialEq<T> for YazData<'a> {
    fn eq(&self, other: &T) -> bool {
        (match self {
            Self::Borrowed(v) => *v,
            Self::Owned(v) => v.as_slice(),
        }) == other.as_ref()
    }
}

pub type Result<T> = std::result::Result<T, Yaz0Error>;

/// Decompress yaz0 compressed data.
pub fn decompress<B: AsRef<[u8]>>(data: B) -> Result<Bytes> {
    let data = data.as_ref();
    if data.len() < 16 {
        Err(Yaz0Error::InsufficientDataError(data.len()))
    } else if &data[0..4] != b"Yaz0" {
        Err(Yaz0Error::MagicError(
            String::from_utf8_lossy(&data[0..4]).to_string(),
        ))
    } else {
        Ok(Bytes(ffi::decompress(data)?))
    }
}

/// Check if data is yaz0 compressed and decompress if needed.
#[inline]
pub fn decompress_if(data: &[u8]) -> Result<YazData<'_>> {
    if data.len() < 4 || &data[0..4] != b"Yaz0" {
        Ok(data.into())
    } else {
        decompress(data).map(YazData::Owned)
    }
}

/// Compress data with default compression level (7).
pub fn compress<B: AsRef<[u8]>>(data: B) -> Bytes {
    Bytes(ffi::compress(data.as_ref(), 7))
}

/// Compress data with specified compression level. Available levels are 6-9, from
/// fastest (and generally largest) to slowest (and generally smallest).
/// Panics if called with levels outside 6-9.
pub fn compress_with_level<B: AsRef<[u8]>>(data: B, level: u8) -> Result<Bytes> {
    if !(6..=9).contains(&level) {
        panic!("Invalid yaz0 compression level: {} (expected 6-9)", level);
    }
    Ok(Bytes(ffi::compress(data.as_ref(), level)))
}

/// Compress data conditionally, if an associated path has a yaz0-associated
/// file extension (starts with 's', but does not equal 'sarc').
#[inline]
pub fn compress_if<P: AsRef<Path>>(data: &[u8], path: P) -> YazData<'_> {
    if let Some(ext) = path.as_ref().extension() {
        if let Some(ext) = ext.to_str() {
            if ext.starts_with('s') && UniCase::new("sarc") != UniCase::new(ext) {
                return YazData::Owned(compress(data));
            }
        }
    }
    YazData::Borrowed(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decompress_test() {
        let data = std::fs::read("test/Cargo.stoml").unwrap();
        let decomp = decompress(&data).unwrap();
        let contents = std::str::from_utf8(&decomp).unwrap();
        let decomp = std::fs::read_to_string("test/Cargo.toml").unwrap();
        assert_eq!(&contents[0..9], "[package]");
        assert_eq!(&contents, &decomp);
    }

    #[test]
    fn compress_test() {
        let data = std::fs::read("test/Cargo.toml").unwrap();
        let meta = std::fs::metadata("test/Cargo.stoml").unwrap();
        let comp = compress(&data);
        assert_eq!(comp.len(), meta.len() as usize);
    }

    #[test]
    fn condition_test() {
        let data = b"Some random data";
        let same_data = b"Some random data";
        let compressed =
            b"Yaz0\x00\x00\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\xffSome ran\xffdom data\xff";
        assert_eq!(decompress_if(&compressed[..]).unwrap().as_ref(), &data[..]);
        assert_eq!(decompress_if(&same_data[..]).unwrap().as_ref(), &data[..]);
        assert_eq!(&compress_if(&data[..], "Test/File.ssarc"), &compressed[..]);
        assert_eq!(&compress_if(&data[..], "Test/File.sarc"), &same_data[..]);
    }
}
