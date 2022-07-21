//! TODO: Docs
#![deny(missing_docs)]
#![feature(const_slice_index)]
#![feature(seek_stream_len)]
#[cfg(feature = "aamp")]
pub mod aamp;
#[cfg(feature = "byml")]
pub mod byml;
#[cfg(feature = "sarc")]
pub mod sarc;
pub mod types;
mod util;
mod yaml;
#[cfg(feature = "yaz0")]
pub mod yaz0;

/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Incorrect magic: found `{0}`, expected `{1}`.")]
    InvalidMagic(String, &'static str),
    #[error("Data too short: found {0:#x} bytes, expected >= {1:#x}.")]
    InsufficientData(usize, usize),
    #[cfg(feature = "byml")]
    #[error(transparent)]
    BymlError(#[from] byml::BymlError),
    #[cfg(feature = "aamp")]
    #[error(transparent)]
    AampError(#[from] aamp::AampError),
    #[cfg(feature = "yaz0")]
    #[error(transparent)]
    Yaz0Error(#[from] yaz0::Yaz0Error),
    #[cfg(feature = "sarc")]
    #[error(transparent)]
    SarcError(#[from] sarc::SarcError),
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
        match self {
            #[cfg(feature = "yaz0")]
            Error::Yaz0Error(e) => Error::Any(join_str::jstr!("Yaz0Error: {&e.to_string()}")),
            other => other.clone(),
        }
    }
}
