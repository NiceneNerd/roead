//! # roead
//!
//! [![crates.io](https://img.shields.io/crates/v/roead)](https://crates.io/crates/roead)
//! [![api](https://img.shields.io/badge/api-rustdoc-558b2f)](https://nicenenerd.github.io/roead/roead/)
//! [![license](https://img.shields.io/badge/license-GPL-blue)](https://spdx.org/licenses/GPL-3.0-or-later.html)
//! [![build](https://img.shields.io/github/actions/workflow/status/NiceneNerd/roead/test.yml)](https://github.com/NiceneNerd/roead/actions/workflows/test.yml)
//!
//! ## A Rusty child of the oead C++ library
//! **oead** is a C++ library for common file formats that are used in modern
//! first-party Nintendo EAD (now EPD) titles.
//!
//! Currently, oead only handles very common formats that are extensively used
//! in recent games such as *Breath of the Wild* and *Super Mario Odyssey*.
//!
//! * [AAMP](https://zeldamods.org/wiki/AAMP) (binary parameter archive): Only version 2 is
//!   supported.
//! * [BYML](https://zeldamods.org/wiki/BYML) (binary YAML): Versions 2, 3, and 4 are supported.
//! * [SARC](https://zeldamods.org/wiki/SARC) (archive)
//! * [Yaz0](https://zeldamods.org/wiki/Yaz0) (compression algorithm)
//!
//! The roead project brings oead's core functionality by directly porting it.
//! (The Grezzo datasheets are not supported.) For more info on oead itself,
//! visit [its GitHub repo](https://github.com/zeldamods/oead/).
//!
//! Each of roead's major modules is configurable as a feature. The default
//! feature set includes `byml`, `aamp`, `sarc,` and `yaz0`. For compatibility
//! with many existing tools for these formats, there is also a `yaml` feature
//! which enables serializing/deserializing AAMP and BYML files as YAML
//! documents. Finally, serde support is available using the `with-serde`
//! feature.
//!
//! For API documentation, see the docs for each module.
//!
//! ## Building from Source
//!
//! roead is pure Rust and builds with any relatively recent stable release
//! (MSRV 1.100), with no C++ toolchain, CMake, or submodules. The optional
//! `yaml` feature is the exception, as it depends on the `ryml` crate.
//!
//! ## Feature model
//!
//! roead is `no_std` by default: `--no-default-features` builds a pure
//! `core` library. The `alloc` feature adds `alloc`-only allocations, and the
//! `std` feature (implying `alloc`) enables the standard library. All modules
//! and APIs are gated behind Cargo features; see `Cargo.toml` for the matrix.
//!
//! ## Contributing
//!
//! Issue tracker: <https://github.com/NiceneNerd/roead/issues>  
//! Source code: <https://github.com/NiceneNerd/roead>
//!
//! This project is licensed under the GPLv3+ license. oead is licensed under
//! the GPLv2+ license.
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
// In test builds the harness requires std, so no_std is not applied; the
// cfg-gated code under test is still selected by features alone.
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "alloc")]
pub mod aamp; // TODO(step 4): arena readers for pure no_std
#[cfg(feature = "alloc")]
pub mod byml; // TODO(step 4): arena readers for pure no_std
// TODO(step 5): split reader (alloc-free) from writer (std)
#[cfg(all(feature = "sarc", feature = "std"))]
pub mod sarc;
pub mod types;
mod util;
#[cfg(feature = "yaml")]
mod yaml;
#[cfg(feature = "alloc")]
pub mod yaz0; // TODO(step 5): `decompress_into` becomes core-only, module gating narrows

/// Error type for this crate.
///
/// The core of `Error` is alloc-free; richer variants require `alloc` or
/// `std`. `Error` is intentionally not `Clone`: wrapping errors are not
/// generally cloneable (this removes the panicking `Clone` stub from v1).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bad magic value: found `{0}`, expected `{1}`.")]
    BadMagic(TinyString, &'static str),
    #[error("Data too short: found {0:#x} bytes, expected >= {1:#x}.")]
    InsufficientData(usize, usize),
    #[error("{0}")]
    InvalidData(&'static str),
    #[error("Found {0}, expected {1}")]
    TypeError(&'static str, &'static str),
    #[error(transparent)]
    InvalidUtf8(#[from] core::str::Utf8Error),
    #[error("Index {index} out of range (len {len})")]
    IndexOutOfRange { index: usize, len: usize },
    #[error("Too many files in SARC ({0})")]
    TooManyFiles(u32),
    #[error("Document does not fit in the fixed-capacity arena (capacity {0})")]
    CapacityExceeded(usize),
    #[cfg(feature = "alloc")]
    #[error("{0}")]
    InvalidDataD(String),
    #[cfg(feature = "alloc")]
    #[error("{0}")]
    Any(String),
    #[cfg(feature = "alloc")]
    #[error("Invalid BYML value node type: {0}")]
    InvalidNodeType(&'static str),
    #[cfg(feature = "std")]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(feature = "binrw")]
    #[cfg_attr(feature = "std", error(transparent))]
    #[cfg_attr(not(feature = "std"), error("binary format error: {0}"))]
    BinarySerde(#[cfg_attr(feature = "std", from)] binrw::Error),
    #[cfg(feature = "yaml")]
    #[error(transparent)]
    InvalidNumber(#[from] lexical::Error),
    #[cfg(feature = "yaml")]
    #[error("Parsing YAML failed: {0}")]
    InvalidYaml(#[from] ryml::Error),
    #[cfg(feature = "yaml")]
    #[error("Parsing YAML binary data failed: {0}")]
    InvalidYamlBinary(#[from] base64::DecodeError),
}

#[cfg(all(feature = "binrw", not(feature = "std")))]
impl From<binrw::Error> for Error {
    fn from(err: binrw::Error) -> Self {
        Error::BinarySerde(err)
    }
}

#[cfg(all(feature = "binrw", not(feature = "std")))]
impl From<binrw::io::Error> for Error {
    fn from(err: binrw::io::Error) -> Self {
        Error::BinarySerde(err.into())
    }
}

/// A copyable, fixed-capacity inline string holding at most 16 bytes.
///
/// Used for error messages in alloc-free contexts. The contents are always
/// valid UTF-8: [`From<&str>`] truncates input longer than 16 bytes (in
/// release) and panics on overflow in debug builds via `debug_assert!`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TinyString {
    data: [u8; 16],
    len: u8,
}

impl TinyString {
    /// Extracts a string slice from the tiny string.
    pub fn as_str(&self) -> &str {
        // SAFETY: the contents are validated to be UTF-8 when constructed.
        unsafe { core::str::from_utf8_unchecked(&self.data[..usize::from(self.len)]) }
    }

    /// Creates a `TinyString` by lossily converting `bytes`, replacing
    /// invalid UTF-8 with the replacement character.
    #[cfg(feature = "alloc")]
    pub fn from_lossy(bytes: &[u8]) -> Self {
        let s: alloc::string::String = alloc::string::String::from_utf8_lossy(bytes).into_owned();
        Self::from(s.as_str())
    }
}

impl From<&str> for TinyString {
    fn from(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len().min(16);
        debug_assert!(
            len == bytes.len(),
            "TinyString can hold at most 16 bytes; input was truncated"
        );
        let mut data = [0; 16];
        data[..len].copy_from_slice(&bytes[..len]);
        Self {
            data,
            len: len as u8,
        }
    }
}

impl core::fmt::Display for TinyString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Hash map type used by the owned BYML/AAMP models.
///
/// With `std` this is `rustc_hash::FxHashMap`; in `alloc`-only builds
/// (where `rustc_hash::FxHashMap` is unavailable) it is a
/// `hashbrown::HashMap` with `rustc_hash::FxBuildHasher`, so the public
/// `Map`/`HashMap`/`ValueHashMap` aliases behave identically in both modes.
#[cfg(feature = "std")]
pub(crate) type FxHashMap<K, V> = rustc_hash::FxHashMap<K, V>;
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub(crate) type FxHashMap<K, V> = hashbrown::HashMap<K, V, rustc_hash::FxBuildHasher>;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub(crate) use hashbrown::hash_map::Entry;
#[cfg(feature = "std")]
pub(crate) use std::collections::hash_map::Entry;

#[cfg_attr(feature = "binrw", binrw::binread, brw(repr = u16))]
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

pub type Result<T> = core::result::Result<T, Error>;
