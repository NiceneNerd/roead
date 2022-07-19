//! Provides for reading and writing SARC archives.
//!
//! Unlike the other modules in this crate, this does not provide bindings to
//! `oead` but is a port of its SARC implementation. Why? Because SARC is a
//! simple format, so much so that it is easier to reimplement than to do FFI.
//!
//! Sample usage, just reading a SARC:
//! ```
//! # use roead::sarc::*;
//! # fn do_stuff_with_data(data: &[u8]) -> () {}
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = std::fs::read("test/sarc/Dungeon119.pack")?;
//! let sarc: Sarc = Sarc::new(&data)?; // In this case we borrow data, but we can also own
//! assert_eq!(sarc.len(), 10); // Get the number of files
//! assert_eq!(sarc.guess_min_alignment(), 4);
//! for file in sarc.files() {
//!     println!("File name: {}", file.name().unwrap());
//!     do_stuff_with_data(file.data());
//! }
//! # Ok(())
//! # }
//! ```
//! And writing a SARC:
//! ```
//! # use roead::sarc::*;
//! # use roead::Endian;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut sarc_writer = SarcWriter::new(Endian::Big); // Create an empty SARC
//! sarc_writer.set_min_alignment(4); // Set the alignment, if needed
//! sarc_writer.files.insert("A/Dummy/File.txt".into(), b"This is a test".to_vec()); // Add a couple files
//! sarc_writer.files.insert("A/Dummy/File2.txt".into(), b"This is another test".to_vec());
//! sarc_writer.files.remove("A/Dummy/File.txt"); // Never mind!
//! let data = sarc_writer.to_binary(); // Write to an in-memory buffer
//! // We can also take construct a SARC writer from an existing SARC
//! let sarc = Sarc::new(data.as_slice())?;
//! let another_sarc_writer = SarcWriter::from_sarc(&sarc);
//! # Ok(())
//! # }
//! ```
mod parse;
mod write;
use crate::Endian;
use binrw::{BinRead, BinWrite};
pub use parse::Sarc;
use thiserror::Error;
pub use write::SarcWriter;

#[derive(Error, Debug)]
/// An enum for all SARC-related errors
pub enum SarcError {
    #[error("File index {0} out of range")]
    OutOfRange(usize),
    #[error("Invalid {0} value: \"{1}\"")]
    InvalidData(String, String),
    #[error("A string in the name table was not terminated")]
    UnterminatedStringError,
    #[error("Invalid UTF file name")]
    InvalidFileName(#[from] std::str::Utf8Error),
    #[error(transparent)]
    BinaryRWError(#[from] binrw::Error),
    #[error("{0} is not a valid alignment")]
    InvalidAlignmentError(usize),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

/// Provides readonly access to a file that is stored in a SARC archive.
#[derive(Debug, PartialEq, Eq)]
pub struct File<'a> {
    name: Option<&'a str>,
    data: &'a [u8],
    index: usize,
}

impl File<'_> {
    /// File name. May be empty for file entries that do not use the file name
    /// table.
    #[inline(always)]
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    /// File name. May be empty for file entries that do not use the file name
    /// table. Panics if the file has no name.
    #[inline(always)]
    pub fn unwrap_name(&self) -> &str {
        self.name.unwrap()
    }

    /// File name. May be empty for file entries that do not use the file name
    /// table.
    ///
    /// # Safety
    /// Calling this function on a file without a name is undefined behavior.
    #[inline(always)]
    pub unsafe fn unwrap_name_unchecked(&self) -> &str {
        self.name.unwrap_unchecked()
    }

    /// File data (as a slice).
    #[inline(always)]
    pub fn data(&self) -> &[u8] {
        self.data
    }

    /// File index in the SARC archive.
    #[inline(always)]
    pub fn index(&self) -> usize {
        self.index
    }

    #[cfg(feature = "yaz0")]
    /// Returns a decompressed copy of the file data.
    #[inline(always)]
    pub fn decompressed_data(&self) -> crate::Result<Vec<u8>> {
        crate::yaz0::decompress(self.data)
    }

    /// Check if the file is a SARC.
    #[inline(always)]
    pub fn is_sarc(&self) -> bool {
        &self.data[0..4] == b"SARC" || &self.data[0x11..0x15] == b"SARC"
    }

    /// Attempt to parse file as SARC.
    pub fn parse_sarc(&self) -> crate::Result<Sarc> {
        Sarc::new(self.data)
    }

    /// Check if the file is yaz0 compressed.
    #[inline(always)]
    pub fn is_compressed(&self) -> bool {
        &self.data[0..4] == b"Yaz0"
    }

    /// Check if the file is an AAMP.
    #[inline(always)]
    pub fn is_aamp(&self) -> bool {
        &self.data[0..4] == b"AAMP"
    }

    /// Check if the file is a BYML document.
    #[inline(always)]
    pub fn is_byml(&self) -> bool {
        &self.data[0..2] == b"BY"
            || &self.data[0..2] == b"YB"
            || &self.data[0x11..0x13] == b"BY"
            || &self.data[0x11..0x13] == b"YB"
    }
}

const SARC_MAGIC: [u8; 4] = *b"SARC";
const SFAT_MAGIC: [u8; 4] = *b"SFAT";
const SFNT_MAGIC: [u8; 4] = *b"SFNT";

#[inline]
const fn hash_name(multiplier: u32, name: &str) -> u32 {
    let mut hash = 0u32;
    let bytes = name.as_bytes();
    let mut i = 0;
    while i < name.len() {
        hash = hash
            .wrapping_mul(multiplier)
            // This is sound because obviously the index is within the string
            // length.
            .wrapping_add(unsafe { *bytes.get_unchecked(i) as u32 });
        i += 1;
    }
    hash
}

/// Size = 0x14
#[derive(Debug, Eq, PartialEq, Copy, Clone, BinRead, BinWrite)]
struct ResHeader {
    magic: [u8; 4],
    header_size: u16,
    bom: Endian,
    file_size: u32,
    data_offset: u32,
    version: u16,
    reserved: u16,
}

/// Size = 0x0C
#[derive(Debug, Copy, Clone, Eq, PartialEq, BinRead, BinWrite)]
struct ResFatHeader {
    magic: [u8; 4],
    header_size: u16,
    num_files: u16,
    hash_multiplier: u32,
}

/// Size = 0x10
#[derive(Debug, PartialEq, Eq, Copy, Clone, BinRead, BinWrite)]
struct ResFatEntry {
    name_hash: u32,
    rel_name_opt_offset: u32,
    data_begin: u32,
    data_end: u32,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, BinRead, BinWrite)]
struct ResFntHeader {
    magic: [u8; 4],
    header_size: u16,
    reserved: u16,
}

#[inline(always)]
const fn is_valid_alignment(alignment: usize) -> bool {
    alignment != 0 && (alignment & (alignment - 1)) == 0
}
