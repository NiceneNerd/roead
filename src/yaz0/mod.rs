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
use std::{borrow::Cow, path::Path};

use crate::ffi;
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

pub type Result<T> = std::result::Result<T, Yaz0Error>;

/// Decompress yaz0 compressed data.
pub fn decompress<B: AsRef<[u8]>>(data: B) -> Result<Vec<u8>> {
    let data = data.as_ref();
    if data.len() < 16 {
        return Err(Yaz0Error::InsufficientDataError(data.len()));
    }
    if &data[0..4] != b"Yaz0" {
        return Err(Yaz0Error::MagicError(
            String::from_utf8_lossy(&data[0..4]).to_string(),
        ));
    }
    Ok(ffi::decompress(data)?)
}

/// Check if data is yaz0 compressed and decompress if needed.
#[inline]
pub fn decompress_if<'a, B: Into<Cow<'a, [u8]>>>(data: B) -> Result<Cow<'a, [u8]>> {
    let data = data.into();
    if data.len() < 16 {
        return Err(Yaz0Error::InsufficientDataError(data.len()));
    }
    if &data[0..4] == b"Yaz0" {
        decompress(data).map(|d| d.into())
    } else {
        Ok(data)
    }
}

/// Compress data with default compression level (7).
pub fn compress<B: AsRef<[u8]>>(data: B) -> Vec<u8> {
    ffi::compress(data.as_ref(), 7)
}

/// Compress data with specified compression level. Available levels are 6-9, from
/// fastest (and generally largest) to slowest (and generally smallest).
pub fn compress_with_level<B: AsRef<[u8]>>(data: B, level: u8) -> Result<Vec<u8>> {
    if !(6..=9).contains(&level) {
        return Err(Yaz0Error::InvalidLevelError(level));
    }
    Ok(ffi::compress(data.as_ref(), level))
}

/// Compress data conditionally, if an associated path has a yaz0-associated
/// file extension (starts with 's', but does not equal 'sarc').
#[inline]
pub fn compress_if<'a, B: Into<Cow<'a, [u8]>>, P: AsRef<Path>>(data: B, path: P) -> Cow<'a, [u8]> {
    if let Some(ext) = path.as_ref().extension() {
        if let Some(ext) = ext.to_str() {
            if ext.starts_with('s') && UniCase::new("sarc") != UniCase::new(ext) {
                return compress(data.into()).into();
            }
        }
    }
    data.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decompress_test() {
        let data = std::fs::read("test/Cargo.stoml").unwrap();
        let contents = String::from_utf8(decompress(&data).unwrap()).unwrap();
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
        assert_eq!(compress_if(&data[..], "Test/File.ssarc"), &compressed[..]);
        assert_eq!(compress_if(&data[..], "Test/File.sarc"), &same_data[..]);
    }
}
