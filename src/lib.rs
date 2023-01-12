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
//! * [AAMP](https://zeldamods.org/wiki/AAMP) (binary parameter archive): Only
//!   version 2 is supported.
//! * [BYML](https://zeldamods.org/wiki/BYML) (binary YAML): Versions 2, 3, and
//!   4 are supported.
//! * [SARC](https://zeldamods.org/wiki/SARC) (archive)
//! * [Yaz0](https://zeldamods.org/wiki/Yaz0) (compression algorithm)
//!
//! The roead project brings oead's core functionality, by directly porting or
//! (for the yaz0 module) providing safe and idiomatic bindings to oead's
//! features. (The Grezzo datasheets are not supported.) For more info on oead
//! itself, visit [its GitHub repo](https://github.com/zeldamods/oead/).
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
//! Most of roead is pure Rust and can compiled with any relatively recent
//! *nightly* release. However, the yaz0 module provides FFI bindings to oead
//! code, so to use it the following additional requirements are necessary:
//!
//! - CMake 3.12+
//! - A compiler that supports C++17
//! - Everything necessary to build zlib
//!
//! First, clone the repository, then enter the roead directory and run
//! `git submodule update --init --recursive`.
//!
//! ## Contributing
//!
//! Issue tracker: <https://github.com/NiceneNerd/roead/issues>  
//! Source code: <https://github.com/NiceneNerd/roead>
//!
//! This project is licensed under the GPLv3+ license. oead is licensed under
//! the GPLv2+ license.
#![feature(const_slice_index, doc_auto_cfg, let_chains, seek_stream_len)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
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
    TypeError(smartstring::alias::String, &'static str),
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
    #[cfg(feature = "yaml")]
    #[error("Parsing YAML binary data failed: {0}")]
    InvalidYamlBinary(#[from] base64::DecodeError),
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

pub type Result<T> = std::result::Result<T, Error>;

impl Clone for Error {
    fn clone(&self) -> Self {
        todo!()
    }
}
