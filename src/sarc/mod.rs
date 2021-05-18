//! Bindings for the oead SARC types.
//! 
//! Sample usage, just reading a SARC:
//! ```
//! # use roead::sarc::*;
//! # fn do_stuff_with_data(data: &[u8]) -> () {}
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = std::fs::read("test/Enemy_Lynel_Dark.sbactorpack")?;
//! let sarc: Sarc = Sarc::read(&data)?; // In this case we borrow data, but we can also own
//! assert_eq!(sarc.len(), 125); // Get the number of files
//! assert_eq!(sarc.guess_min_alignment(), 4);
//! for (name, data) in sarc.files() {
//!     println!("File name: {}", name);
//!     do_stuff_with_data(data);
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
//! sarc_writer.set_alignment(4); // Set the alignment, if needed
//! sarc_writer.add_file("A/Dummy/File.txt", b"This is a test".to_vec()); // Add a couple files
//! sarc_writer.add_file("A/Dummy/File2.txt", b"This is another test".to_vec());
//! sarc_writer.delete_file("A/Dummy/File.txt"); // Never mind!
//! let data: Vec<u8> = sarc_writer.to_binary(); // Write to an in-memory buffer
//! // We can also take construct a SARC writer from an existing SARC
//! let sarc = Sarc::read(&data)?;
//! let another_sarc_writer: SarcWriter = sarc.into();
//! # Ok(())
//! # }
//! ```
use crate::{ffi, Endian};
use std::{borrow::Cow, hash::Hash, io};
use thiserror::Error;

/// Error type for SARC parsing and writing.
#[derive(Error, Debug)]
pub enum SarcError {
    #[error("Invalid SARC magic")]
    MagicError,
    #[error("Not enough data for valid SARC, expected >40 bytes, found {0}")]
    InsufficientDataError(usize),
    #[error("Compressed SARC could not be decompressed: {0}")]
    Yaz0Error(#[from] crate::yaz0::Yaz0Error),
    #[error("Failed to parse SARC: {0}")]
    OeadError(#[from] cxx::Exception),
}

type Result<T> = std::result::Result<T, SarcError>;

/// A simple SARC archive reader.
pub struct Sarc<'a> {
    inner: cxx::UniquePtr<ffi::Sarc>,
    _data: Cow<'a, [u8]>
}

impl std::fmt::Debug for Sarc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sarc")
            .field("data_offset", &self.data_offset())
            .field("alignment", &self.guess_min_alignment())
            .field("endian", &self.endian())
            .field("files", &self.list_filenames())
            .finish()
    }
}

impl Hash for Sarc<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self._data.hash(state)
    }
}

impl Clone for Sarc<'_> {
    fn clone(&self) -> Self {
        match &self._data {
            Cow::Borrowed(data) => Self::read(*data).unwrap(),
            Cow::Owned(data) => Self::read(data.clone()).unwrap()
        }
    }
}

impl PartialEq for Sarc<'_> {
    fn eq(&self, other: &Sarc) -> bool {
        self._data == other._data
    }
}

impl Sarc<'_> {
    /// Get the number of files that are stored in the archive.
    pub fn len(&self) -> usize {
        self.inner.num_files() as usize
    }

    /// Check if the SARC contains no files.
    pub fn is_empty(&self) -> bool {
        self.inner.num_files() == 0
    }

    /// Get an iterator over the contained files.
    pub fn files(&self) -> impl Iterator<Item = (&str, &[u8])> {
        (0..self.len())
            .into_iter()
            .filter_map(move |i| self.get_file_by_index(i))
    }

    /// Get a vector of file names.
    pub fn list_filenames(&self) -> Vec<&str> {
        (0..self.len())
            .into_iter()
            .filter_map(|i| self.inner.idx_file_name(i as u16).ok())
            .collect()
    }

    /// Get an option containing the data belonging to a file 
    /// if it exists in the SARC, otherwise None.
    pub fn get_file_data(&self, name: &str) -> Option<&[u8]> {
        self.inner.get_file_data(name).ok()
    }

    /// Get a file name and data by its index.
    pub fn get_file_by_index(&self, idx: usize) -> Option<(&str, &[u8])> {
        if idx >= self.len() {
            return None;
        }
        let name = self.inner.idx_file_name(idx as u16);
        let data = self.inner.idx_file_data(idx as u16);
        if let Ok(name) = name {
            Some((name, data.unwrap()))
        } else {
            None
        }
    }

    /// Get the endianness of the SARC.
    pub fn endian(&self) -> Endian {
        if self.inner.big_endian() {
            Endian::Big
        } else {
            Endian::Little
        }
    }

    /// Get the offset to the beginning of file data.
    pub fn data_offset(&self) -> usize {
        self.inner.get_offset() as usize
    }

    /// Guess the minimum data alignment for files that are stored in the archive.
    pub fn guess_min_alignment(&self) -> usize {
        self.inner.guess_align()
    }

    /// Compare the contents of two SARCs.
    pub fn files_are_equal(&self, other: &Sarc) -> bool {
        self.inner.files_eq(&other.inner)
    }

    /// Read a SARC from binary data. The data can be owned (so the SARC
    /// can be freely moved) or passed as a reference.
    pub fn read<'a, D: Into<Cow<'a, [u8]>>>(data: D) -> Result<Sarc<'a>> {
        let data = data.into();
        if &data[0..4] == b"Yaz0" {
            let data = crate::yaz0::decompress(data)?;
            Ok(Sarc {
                inner: ffi::sarc_from_binary(&data)?,
                _data: Cow::Owned(data)
            })
        } else if data.len() < 40 {
            Err(SarcError::InsufficientDataError(data.len()))
        } else if &data[0..4] != b"SARC" {
            Err(SarcError::MagicError)
        } else {
            Ok(Sarc {
                inner: ffi::sarc_from_binary(data.as_ref())?,
                _data: data
            })
        }
    }
}

/// A simple SARC archive reader.
///
/// *Note about the two modes:*
/// Legacy mode is used for games with an old-style resource system that requires
/// aligning nested SARCs and manual alignment of file data in archives.
/// Legacy mode is not needed for games with a new-style resource system that
/// automatically takes care of data alignment and does not require manual
/// alignment nor nested SARC alignment.
pub struct SarcWriter(cxx::UniquePtr<ffi::SarcWriter>);

impl From<&Sarc<'_>> for SarcWriter {
    fn from(sarc: &Sarc) -> Self {
        SarcWriter(ffi::WriterFromSarc(&sarc.inner))
    }
}

impl From<Sarc<'_>> for SarcWriter {
    fn from(sarc: Sarc) -> Self {
        SarcWriter(ffi::WriterFromSarc(&sarc.inner))
    }
}

impl SarcWriter {
    /// Construct a new SARC with the specified endianness.
    pub fn new(endian: Endian) -> SarcWriter {
        SarcWriter(ffi::NewSarcWriter(endian == Endian::Big, false))
    }

    /// Construct a new SARC with the specified endianness in legacy mode
    /// (for manual alignment).
    pub fn new_legacy_mode(endian: Endian) -> SarcWriter {
        SarcWriter(ffi::NewSarcWriter(endian == Endian::Big, true))
    }

    /// Get the number of files that are stored in the archive.
    pub fn len(&self) -> usize {
        self.0.NumFiles()
    }

    /// Checks if the SARC contains no files.
    pub fn is_empty(&self) -> bool {
        self.0.NumFiles() == 0
    }

    /// Add a file to the SARC.
    pub fn add_file<B: Into<Vec<u8>>>(&mut self, name: &str, data: B) {
        self.0.pin_mut().SetFile(name, data.into());
    }

    /// Delete a file from the SARC.
    pub fn delete_file(&mut self, name: &str) -> bool {
        self.0.pin_mut().DelFile(name)
    }

    /// Set the minimum data alignment for files that are stored in the archive.
    pub fn set_alignment(&mut self, alignment: u8) {
        self.0.pin_mut().SetMinAlignment(alignment as usize)
    }

    /// Set the endianness of the SARC.
    pub fn set_endian(&mut self, endian: Endian) {
        self.0.pin_mut().SetEndianness(endian == Endian::Big)
    }

    /// Set whether the SARC uses legacy alignment.
    pub fn set_legacy_mode(&mut self, legacy: bool) {
        self.0.pin_mut().SetMode(legacy)
    }

    /// Write a SARC archive to an in-memory buffer.
    #[allow(clippy::clippy::wrong_self_convention)]
    pub fn to_binary(&mut self) -> Vec<u8> {
        self.0.pin_mut().Write().data
    }

    /// Write a SARC archive to an in-memory buffer, returning a tuple containing
    /// both the file data and the final alignment.
    #[allow(clippy::clippy::wrong_self_convention)]
    pub fn to_binary_and_check_alignment(&mut self) -> (Vec<u8>, usize) {
        let result = self.0.pin_mut().Write();
        (result.data, result.alignment)
    }

    /// Write a SARC archive to any writer.
    pub fn write<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_binary())
    }

    /// Write a SARC archive with yaz0 compression to any writer.
    pub fn write_compressed<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let bytes = crate::yaz0::compress(self.to_binary());
        writer.write_all(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{sarc, Endian};

    #[test]
    fn read_sarc() {
        let data = std::fs::read("test/Enemy_Lynel_Dark.sbactorpack").unwrap();
        let sarc = sarc::Sarc::read(&data).unwrap();
        dbg!(&sarc);
        assert_eq!(sarc.data_offset(), 6492);
        assert_eq!(sarc.guess_min_alignment(), 4);
        assert_eq!(sarc.endian(), Endian::Big);
        assert_eq!(sarc.files().count(), 125);
        assert_eq!(
            sarc.get_file_data("Actor/AS/Lynel_StunEnd.bas")
                .expect("Could not find file data")
                .len(),
            132
        );
    }

    #[test]
    fn read_sarc_with_owned_data() {
        let sarc = {
            let data = std::fs::read("test/Enemy_Lynel_Dark.sbactorpack").unwrap();
            sarc::Sarc::read(data).unwrap()
        };
        dbg!(&sarc);
        assert_eq!(sarc.data_offset(), 6492);
        assert_eq!(sarc.guess_min_alignment(), 4);
        assert_eq!(sarc.endian(), Endian::Big);
        assert_eq!(sarc.files().count(), 125);
        assert_eq!(
            sarc.get_file_data("Actor/AS/Lynel_StunEnd.bas")
                .expect("Could not find file data")
                .len(),
            132
        );
    }

    #[test]
    fn sarc_eq() {
        let data = std::fs::read("test/Enemy_Lynel_Dark.sbactorpack").unwrap();
        let data2 = std::fs::read("test/Enemy_Lynel_Dark.sbactorpack").unwrap();
        let sarc = sarc::Sarc::read(data.as_slice()).unwrap();
        let sarc2 = sarc::Sarc::read(&data2).unwrap();
        assert_eq!(sarc, sarc2)
    }

    #[test]
    fn build_a_sarc() {
        let mut writer = sarc::SarcWriter::new(Endian::Big);
        writer.add_file("Test/Test.txt", b"This is some test data".to_vec());
        writer.add_file("Test/Test2.txt", b"This is some more test data".to_vec());
        assert_eq!(writer.len(), 2);
        assert!(writer.delete_file("Test/Test2.txt"));
        assert_eq!(writer.len(), 1);
        let bytes = writer.to_binary();
        let sarc = sarc::Sarc::read(&bytes).unwrap();
        assert_eq!(
            sarc.get_file_data("Test/Test.txt").unwrap(),
            b"This is some test data"
        );
        let mut bytes2: Vec<u8> = vec![];
        writer.write_compressed(&mut bytes2).unwrap();
        assert_eq!(bytes, crate::yaz0::decompress(bytes2).unwrap());
    }

    #[test]
    fn sarc_to_writer() {
        let bytes = std::fs::read("test/Enemy_Lynel_Dark.sbactorpack").unwrap();
        let sarc = sarc::Sarc::read(&bytes).unwrap();
        let mut writer = sarc::SarcWriter::from(&sarc);
        assert_eq!(writer.len(), sarc.len());
        assert_eq!(writer.to_binary(), sarc._data.as_ref());
    }
}
