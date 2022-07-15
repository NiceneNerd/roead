//! Bindings for the oead::yaz0 module, which supports Yaz0 decompression and
//! fast compression (using syaz0).
use std::borrow::Cow;

use crate::{Error, Result};
pub use ffi::Header;

/// Error type for Yaz0 handling.
#[derive(Debug, thiserror::Error)]
pub enum Yaz0Error {
    #[error("Buffer too small to decompress: only {0} bytes, need {1}.")]
    InsufficientBuffer(usize, usize),
    #[error(transparent)]
    CxxError(#[from] cxx::Exception),
}

/// Get the header of Yaz0 compressed data, if it exists.
pub fn get_header(data: impl AsRef<[u8]>) -> Option<Header> {
    let data = data.as_ref();
    ffi::GetHeader(data).ok()
}

/// Decompress Yaz0 data to vector.
pub fn decompress(data: impl AsRef<[u8]>) -> Result<Vec<u8>> {
    let data = data.as_ref();
    if data.len() < 0x16 {
        return Err(Error::InsufficientData(data.len(), 0x16));
    }
    let header: Header = ffi::GetHeader(data).map_err(Yaz0Error::from)?;
    if &header.magic != b"Yaz0" {
        return Err(Error::InvalidMagic(
            String::from_utf8_lossy(header.magic.as_slice()).to_string(),
            "Yaz0",
        ));
    }
    let mut out = vec![0; header.uncompressed_size as usize];
    ffi::DecompressIntoBuffer(data, &mut out).map_err(Yaz0Error::from)?;
    Ok(out)
}

/// Decompress Yaz0 data into an existing buffer, returning the number of
/// bytes written.
pub fn decompress_into(data: impl AsRef<[u8]>, mut buffer: impl AsMut<[u8]>) -> Result<usize> {
    let data = data.as_ref();
    if data.len() < 0x16 {
        return Err(Error::InsufficientData(data.len(), 0x16));
    }
    let header: Header = ffi::GetHeader(data).map_err(Yaz0Error::from)?;
    if &header.magic != b"Yaz0" {
        return Err(Error::InvalidMagic(
            String::from_utf8_lossy(header.magic.as_slice()).to_string(),
            "Yaz0",
        ));
    }
    let buffer = buffer.as_mut();
    if buffer.len() < header.uncompressed_size as usize {
        return Err(Error::Yaz0Error(Yaz0Error::InsufficientBuffer(
            buffer.len(),
            header.uncompressed_size as usize,
        )));
    }
    ffi::DecompressIntoBuffer(data, buffer).map_err(Yaz0Error::from)?;
    Ok(header.uncompressed_size as usize)
}

/// Decompress Yaz0 data into an existing buffer, returning the number of
/// bytes written.
///
/// # Safety
/// This function is extremely unsafe. The caller must needs be quite
/// certain that the provided slice is fully valid and complete Yaz0 data.
/// The destination buffer must also be large enough to hold the decompressed
/// data. **Do not use this function on untrusted data.**
pub unsafe fn decompress_unchecked(data: impl AsRef<[u8]>, mut buffer: impl AsMut<[u8]>) -> usize {
    let data = data.as_ref();
    ffi::DecompressUnsafe(data, buffer.as_mut())
        .map_err(Yaz0Error::from)
        .unwrap_unchecked();
    u32::from_be_bytes(data.get_unchecked(0x4..0x8).try_into().unwrap_unchecked()) as usize
}

/// Conditionally decompress Yaz0 data to a vector. Returns a [`Cow`] which
/// contains the original data if the data is not Yaz0 compressed or
/// decompression fails, or containing the decompressed data otherwise.
#[inline]
pub fn decompress_if(data: &[u8]) -> Cow<'_, [u8]> {
    if data.len() < 0x16 {
        return Cow::Borrowed(data);
    }
    if let Ok(header) = ffi::GetHeader(data).map_err(Yaz0Error::from) {
        if &header.magic != b"Yaz0" {
            return Cow::Borrowed(data);
        }
        let mut out = vec![0; header.uncompressed_size as usize];
        if ffi::DecompressIntoBuffer(data, &mut out).is_ok() {
            Cow::Owned(out)
        } else {
            Cow::Borrowed(data)
        }
    } else {
        Cow::Borrowed(data)
    }
}

/// Compress data with default compression settings (no alignment, compression
/// level 7).
pub fn compress(data: impl AsRef<[u8]>) -> Vec<u8> {
    let data = data.as_ref();
    ffi::Compress(data, 0, 7)
}

/// Yaz0 compression options.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CompressOptions {
    /// Buffer alignment hint for decompression
    pub alignment: u8,
    /// Compression level (6 to 9; 6 is fastest and 9 is slowest)
    pub compression_level: u8,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            alignment: 0,
            compression_level: 7,
        }
    }
}

/// Compress data with custom compression settings.
///
/// Automatically clamps the compression level to 6 to 9.
pub fn compress_with_options(data: impl AsRef<[u8]>, options: CompressOptions) -> Vec<u8> {
    let data = data.as_ref();
    ffi::Compress(
        data,
        options.alignment as u32,
        options.compression_level as i32,
    )
}

/// Compress data conditionally, if an associated path has a Yaz0-associated
/// file extension (starts with `s`, but does not equal `sarc`). Returns a
/// [`Cow`] which contains the original data if the data does not need to be
/// compressed, or containing the compressed data otherwise.
#[inline]
pub fn compress_if(data: &[u8], path: impl AsRef<std::path::Path>) -> Cow<'_, [u8]> {
    if path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.starts_with('s') && e != "sarc")
        .unwrap_or(false)
    {
        Cow::Owned(compress(data))
    } else {
        Cow::Borrowed(data)
    }
}

#[cxx::bridge(namespace = "oead::yaz0")]
mod ffi {
    /// The header of Yaz0 compressed data.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    struct Header {
        /// Should be "Yaz0".
        pub magic: [u8; 4],
        /// The size of the uncompressed data.
        pub uncompressed_size: u32,
        /// [Newer files only] Required buffer alignment
        pub data_alignment: u32,
        #[doc(hidden)]
        reserved: [u8; 4],
    }

    unsafe extern "C++" {
        include!("roead/src/include/oead/yaz0.h");
        fn GetHeader(data: &[u8]) -> Result<Header>;
        #[rust_name = "DecompressIntoBuffer"]
        fn Decompress(data: &[u8], dest: &mut [u8]) -> Result<()>;
        unsafe fn DecompressUnsafe(data: &[u8], dest: &mut [u8]) -> Result<()>;
        fn Compress(data: &[u8], data_alignment: u32, level: i32) -> Vec<u8>;
    }
}
