//! TODO: Docs
#![deny(missing_docs)]
#![feature(const_slice_index)]
#![feature(seek_stream_len)]
#![feature(let_chains)]
#[cfg(feature = "aamp")]
pub mod aamp;
#[cfg(feature = "byml")]
pub mod byml;
#[cfg(feature = "sarc")]
pub mod sarc;
pub mod types;
mod util;
#[cfg(feature = "yaml")]
mod yaml;
#[cfg(feature = "yaz0")]
pub mod yaz0;

/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Bad magic value: found `{0}`, expected `{1}`.")]
    BadMagic(String, &'static str),
    #[error("Data too short: found {0:#x} bytes, expected >= {1:#x}.")]
    InsufficientData(usize, usize),
    #[error("{0}")]
    InvalidData(&'static str),
    #[error("{0}")]
    InvalidDataD(String),
    #[error("Found {0}, expected {1}")]
    TypeError(String, &'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(feature = "binrw")]
    #[error(transparent)]
    BinarySerde(#[from] binrw::Error),
    #[error(transparent)]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[cfg(feature = "yaml")]
    #[error(transparent)]
    InvalidNumber(#[from] lexical::Error),
    #[cfg(feature = "yaml")]
    #[error("Parsing YAML failed: {0}")]
    InvalidYaml(#[from] ryml::Error),
    #[cfg(feature = "yaz0")]
    #[error(transparent)]
    Yaz0Error(#[from] cxx::Exception),
    #[error("{0}")]
    Any(String),
}

#[cfg_attr(feature = "sarc", binrw::binread, brw(repr = u16))]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(u16)]
/// Represents endianness where applicable.
///
/// Generally in the game ROM, big endian is used for Wii U and little endian
/// is used for Switch.
pub enum Endian {
    /// Big Endian (Wii U)
    Big = 0xFFFE,
    /// Little Endian (Switch)
    Little = 0xFEFF,
}

type Result<T> = std::result::Result<T, Error>;

impl Clone for Error {
    fn clone(&self) -> Self {
        todo!()
    }
}
