//! Bindings for the `oead::yaz0` module, which supports Yaz0 decompression and
//! fast compression (using syaz0).
use std::borrow::Cow;

use binrw::binrw;

use crate::{Error, Result};

/// The header of Yaz0 compressed data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[binrw]
#[brw(big)]
pub struct Header {
    /// Should be "Yaz0".
    pub magic: [u8; 4],
    /// The size of the uncompressed data.
    pub uncompressed_size: u32,
    /// [Newer files only] Required buffer alignment
    pub data_alignment: u32,
    #[doc(hidden)]
    reserved: [u8; 4],
}

/// Get the header of Yaz0 compressed data, if it exists.
pub fn get_header(data: impl AsRef<[u8]>) -> Option<Header> {
    binrw::BinRead::read(&mut std::io::Cursor::new(data.as_ref())).ok()
}

/// Decompress Yaz0 data to vector.
pub fn decompress(data: impl AsRef<[u8]>) -> Result<Vec<u8>> {
    let data = data.as_ref();
    if data.len() < 0x16 {
        return Err(Error::InsufficientData(data.len(), 0x16));
    }
    let header = get_header(data).ok_or(Error::InvalidData("Missing or corrupt Yaz0 header"))?;
    if &header.magic != b"Yaz0" {
        return Err(Error::BadMagic(
            String::from_utf8_lossy(header.magic.as_slice()).to_string(),
            "Yaz0",
        ));
    }
    let mut out = vec![0; header.uncompressed_size as usize];
    ffi::DecompressIntoBuffer(data, &mut out)?;
    Ok(out)
}

/// Decompress Yaz0 data into an existing buffer, returning the number of
/// bytes written.
pub fn decompress_into(data: impl AsRef<[u8]>, mut buffer: impl AsMut<[u8]>) -> Result<usize> {
    let data = data.as_ref();
    if data.len() < 0x16 {
        return Err(Error::InsufficientData(data.len(), 0x16));
    }
    let header = get_header(data).ok_or(Error::InvalidData("Missing or corrupt Yaz0 header"))?;
    if &header.magic != b"Yaz0" {
        return Err(Error::BadMagic(
            String::from_utf8_lossy(header.magic.as_slice()).to_string(),
            "Yaz0",
        ));
    }
    let buffer = buffer.as_mut();
    if buffer.len() < header.uncompressed_size as usize {
        return Err(Error::InsufficientData(
            buffer.len(),
            header.uncompressed_size as usize,
        ));
    }
    ffi::DecompressIntoBuffer(data, buffer)?;
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
    ffi::DecompressUnsafe(data, buffer.as_mut()).unwrap_unchecked();
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
    if let Some(header) = get_header(data) {
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
    unsafe extern "C++" {
        include!("roead/src/include/oead/yaz0.h");
        #[rust_name = "DecompressIntoBuffer"]
        fn Decompress(data: &[u8], dest: &mut [u8]) -> Result<()>;
        unsafe fn DecompressUnsafe(data: &[u8], dest: &mut [u8]) -> Result<()>;
        fn Compress(data: &[u8], data_alignment: u32, level: i32) -> Vec<u8>;
    }
}

#[cfg(test)]
mod tests {
    static FILES: &[(&str, [u8; 4], usize)] = &[
        ("ActorInfo.product.sbyml", [b'Y', b'B', 0x02, 0x0], 1963604),
        ("Demo344_1.sbeventpack", [b'S', b'A', b'R', b'C'], 2847908),
        (
            "ResourceSizeTable.product.srsizetable",
            [b'R', b'S', b'T', b'B'],
            526804,
        ),
        ("0-0.shknm2", [0x57, 0xE0, 0xE0, 0x57], 17584),
    ];

    #[test]
    fn test_header() {
        for (file, _, len) in FILES {
            let path = std::path::Path::new("test/yaz0").join(file);
            let data = std::fs::read(path).unwrap();
            let header = super::get_header(data.as_slice()).unwrap();
            assert_eq!(header.uncompressed_size, *len as u32);
        }
    }

    #[test]
    fn test_decompress() {
        for (file, magic, len) in FILES {
            let path = std::path::Path::new("test/yaz0").join(file);
            let data = std::fs::read(path).unwrap();
            let decompressed = super::decompress(data).unwrap();
            assert_eq!(&decompressed[..4], magic.as_slice());
            assert_eq!(decompressed.len(), *len);
            println!("{} is good", file);
        }
    }

    #[test]
    fn test_roundtrip() {
        for (file, ..) in FILES {
            let path = std::path::Path::new("test/yaz0").join(file);
            let data = std::fs::read(path).unwrap();
            let decompressed = super::decompress(data).unwrap();
            let compressed = super::compress(decompressed.as_slice());
            let decompressed2 = super::decompress(compressed).unwrap();
            assert_eq!(decompressed, decompressed2);
        }
    }

    #[test]
    fn test_unchecked() {
        let data = b"Nothing you have not given away will ever really be yours.";
        let compressed = super::compress(data);
        let mut buffer = vec![0; data.len()];
        let size = unsafe { super::decompress_unchecked(compressed, &mut buffer) };
        assert_eq!(data.as_slice(), &buffer[..size]);
    }
}
