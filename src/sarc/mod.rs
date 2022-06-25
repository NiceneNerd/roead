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
use crate::{aamp, byml, cvec_to_vec, ffi, yaz0, Endian};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    hash::Hash,
    io,
};
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

/// Provides readonly access to a file in a SARC
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct File<'a> {
    name: Option<&'a str>,
    data: &'a [u8],
}

impl File<'_> {
    /// The path of the file in the SARC, if present.
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    /// The path of the file in the SARC. Panics if the file has no path.
    pub fn name_unchecked(&self) -> &str {
        self.name.unwrap()
    }

    /// Reference to the file data.
    pub fn data(&self) -> &[u8] {
        self.data
    }

    /// Decompress the file data.
    pub fn decompress_data(&self) -> yaz0::Result<Vec<u8>> {
        yaz0::decompress(self.data)
    }

    /// Check if the file is a SARC.
    #[inline]
    pub fn is_sarc(&self) -> bool {
        &self.data[0..4] == b"SARC" || &self.data[0x11..0x15] == b"SARC"
    }

    /// Attempt to parse file as SARC.
    pub fn parse_as_sarc(&self) -> Result<Sarc> {
        Sarc::read(self.data)
    }

    /// Check if the file is yaz0 compressed.
    #[inline]
    pub fn is_compressed(&self) -> bool {
        &self.data[0..4] == b"Yaz0"
    }

    /// Check if the file is an AAMP.
    #[inline]
    pub fn is_aamp(&self) -> bool {
        &self.data[0..4] == b"AAMP"
    }

    /// Attempt to parse file as AAMP.
    pub fn parse_as_aamp(&self) -> aamp::Result<aamp::ParameterIO> {
        aamp::ParameterIO::from_binary(self.data)
    }

    /// Check if the file is BYML.
    #[inline]
    pub fn is_byml(&self) -> bool {
        &self.data[0..2] == b"BY"
            || &self.data[0..2] == b"YB"
            || &self.data[0x11..0x13] == b"BY"
            || &self.data[0x11..0x13] == b"YB"
    }

    /// Attempt to parse file as BYML.
    pub fn parse_as_byml(&self) -> byml::Result<byml::Byml> {
        byml::Byml::from_binary(self.data)
    }
}

/// A simple SARC archive reader.
pub struct Sarc<'a> {
    inner: cxx::UniquePtr<ffi::Sarc>,
    _data: Cow<'a, [u8]>,
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

unsafe impl Send for Sarc<'_> {}
unsafe impl Sync for Sarc<'_> {}

impl Hash for Sarc<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self._data.hash(state)
    }
}

impl Eq for Sarc<'_> {}

impl Clone for Sarc<'_> {
    fn clone(&self) -> Self {
        match &self._data {
            Cow::Borrowed(data) => Self::read(*data).unwrap(),
            Cow::Owned(data) => Self::read(data.clone()).unwrap(),
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
    pub fn files(&self) -> impl Iterator<Item = File> {
        (0..self.len())
            .into_iter()
            .filter_map(move |i| self.get_file_by_index(i))
    }

    /// Extracts owned filenames and data from the SARC.
    pub fn into_files(self) -> Vec<(Option<String>, Vec<u8>)> {
        self.files()
            .map(|f| (f.name.map(|n| n.to_owned()), f.data.to_vec()))
            .collect()
    }

    /// Create a hash map of files and their data.
    pub fn to_file_map(&self) -> HashMap<Option<String>, Vec<u8>> {
        self.files()
            .map(|f| (f.name.map(|n| n.to_owned()), f.data.to_vec()))
            .collect()
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
    pub fn get_file_by_index(&self, idx: usize) -> Option<File> {
        if idx >= self.len() {
            return None;
        }
        let name = self.inner.idx_file_name(idx as u16);
        let data = self.inner.idx_file_data(idx as u16);
        if let Ok(data) = data {
            Some(File {
                name: name.ok(),
                data,
            })
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
    pub fn has_equal_files(&self, other: &Sarc) -> bool {
        self.inner.files_eq(&other.inner)
    }

    /// Read a SARC from binary data. The data can be owned (so the SARC
    /// can be freely moved) or passed as a reference. Note that if the data
    /// is compressed it will be decompressed first and the resulting Sarc
    /// will own the decompressed data.
    pub fn read<'a, D: Into<Cow<'a, [u8]>>>(data: D) -> Result<Sarc<'a>> {
        let data = data.into();
        if data.len() < 40 {
            Err(SarcError::InsufficientDataError(data.len()))
        } else if &data[0..4] == b"Yaz0" {
            let data = crate::yaz0::decompress(data)?;
            Self::read(data)
        } else if &data[0..4] != b"SARC" {
            Err(SarcError::MagicError)
        } else {
            Ok(Sarc {
                inner: ffi::sarc_from_binary(data.as_ref())?,
                _data: data,
            })
        }
    }
}

pub type FileMap = BTreeMap<String, Vec<u8>>;

/// A simple SARC archive writer.
///
/// *Note about the two modes:*
/// Legacy mode is used for games with an old-style resource system that requires
/// aligning nested SARCs and manual alignment of file data in archives.
/// Legacy mode is not needed for games with a new-style resource system that
/// automatically takes care of data alignment and does not require manual
/// alignment nor nested SARC alignment.
#[derive(Debug, PartialEq, Eq)]
pub struct SarcWriter {
    pub files: FileMap,
    pub endian: Endian,
    pub alignment: u8,
    pub legacy: bool,
}

impl From<&Sarc<'_>> for SarcWriter {
    fn from(sarc: &Sarc) -> Self {
        let alignment = sarc.guess_min_alignment() as u8;
        Self {
            alignment,
            endian: sarc.endian(),
            legacy: alignment != 4,
            files: sarc
                .files()
                .filter_map(|file| file.name.map(|n| (n.to_owned(), file.data.to_vec())))
                .collect(),
        }
    }
}

impl From<Sarc<'_>> for SarcWriter {
    fn from(sarc: Sarc) -> Self {
        let alignment = sarc.guess_min_alignment() as u8;
        Self {
            alignment,
            endian: sarc.endian(),
            legacy: alignment != 4,
            files: sarc
                .into_files()
                .into_iter()
                .filter_map(|(f, d)| f.map(|n| (n, d)))
                .collect(),
        }
    }
}

impl SarcWriter {
    /// Construct a new SARC with the specified endianness.
    pub fn new(endian: Endian) -> Self {
        Self {
            files: FileMap::new(),
            endian,
            alignment: 4,
            legacy: false,
        }
    }

    /// Construct a new SARC with the specified endianness in legacy mode
    /// (for manual alignment).
    pub fn new_legacy_mode(endian: Endian) -> Self {
        Self {
            files: FileMap::new(),
            endian,
            alignment: 4,
            legacy: true,
        }
    }

    /// Builder-style method to set alignment on a new SARC writer.
    #[must_use]
    pub fn with_alignment(mut self, alignment: u8) -> Self {
        self.alignment = alignment;
        self
    }

    /// Builder-style method to add files on a new SARC writer.
    #[must_use]
    pub fn with_files<S: AsRef<str>, B: Into<Vec<u8>>, F: IntoIterator<Item = (S, B)>>(
        mut self,
        files: F,
    ) -> Self {
        self.add_files(files);
        self
    }

    /// Construct a new SARC with the specified endianness, filling it with initial
    /// file data from an iterator.
    pub fn from_files<S: AsRef<str>, B: Into<Vec<u8>>, F: IntoIterator<Item = (S, B)>>(
        endian: Endian,
        files: F,
    ) -> Self {
        Self {
            files: files
                .into_iter()
                .map(|(f, d)| (f.as_ref().to_owned(), d.into()))
                .collect(),
            endian,
            alignment: 4,
            legacy: false,
        }
    }

    /// Shortcut to construct a new SARC from existing data.
    pub fn from_binary<B: AsRef<[u8]>>(data: B) -> Result<Self> {
        Sarc::read(data.as_ref()).map(Self::from)
    }

    /// Get the number of files that are stored in the archive.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns true if the SARC writer contains the specified file name.
    pub fn contains<B: std::borrow::Borrow<str>>(&self, name: B) -> bool {
        self.files.contains_key(name.borrow())
    }

    /// Checks if the SARC contains no files.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Add a file to the SARC.
    pub fn add_file<B: Into<Vec<u8>>>(&mut self, name: &str, data: B) {
        self.files.insert(name.to_owned(), data.into());
    }

    /// Add several files to a SARC from an iterator.
    pub fn add_files<S: AsRef<str>, B: Into<Vec<u8>>, F: IntoIterator<Item = (S, B)>>(
        &mut self,
        files: F,
    ) {
        self.files.extend(
            files
                .into_iter()
                .map(|(f, d)| (f.as_ref().to_owned(), d.into())),
        )
    }

    /// Get the data of a file in the SARC writer.
    pub fn get_file_data(&self, name: &str) -> Option<&[u8]> {
        self.files.get(name).map(|d| d.as_slice())
    }

    /// Delete a file from the SARC.
    pub fn delete_file(&mut self, name: &str) -> bool {
        self.files.remove(name).is_some()
    }

    /// Set the minimum data alignment for files that are stored in the archive.
    pub fn set_alignment(&mut self, alignment: u8) {
        self.alignment = alignment
    }

    /// Set the endianness of the SARC.
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian
    }

    /// Set whether the SARC uses legacy alignment.
    pub fn set_legacy_mode(&mut self, legacy: bool) {
        self.legacy = legacy
    }

    /// Write a SARC archive to an in-memory buffer.
    pub fn to_binary(&self) -> Vec<u8> {
        let ffi::SarcWriteResult { alignment, data } = ffi::WriteSarc(
            self,
            matches!(self.endian, Endian::Big),
            self.legacy,
            self.alignment,
        );
        let data = cvec_to_vec(data);
        data
    }

    /// Write a SARC archive to an in-memory buffer, returning a tuple containing
    /// both the file data and the final alignment.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_binary_and_check_alignment(&mut self) -> (Vec<u8>, usize) {
        let result = ffi::WriteSarc(
            self,
            matches!(self.endian, Endian::Big),
            self.legacy,
            self.alignment,
        );
        (cvec_to_vec(result.data), result.alignment)
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

    pub(crate) fn get_file_by_index(&self, idx: usize) -> &str {
        self.files.keys().nth(idx).unwrap()
    }

    pub(crate) fn get_data_by_index(&self, idx: usize) -> &[u8] {
        self.files.values().nth(idx).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        sarc::{self, SarcWriter},
        Endian,
    };

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
        let writer = sarc::SarcWriter::from(&sarc);
        assert_eq!(writer.len(), sarc.len());
        assert_eq!(writer.to_binary(), sarc._data.as_ref());
    }

    #[test]
    fn destructure() {
        let bytes = std::fs::read("test/Enemy_Lynel_Dark.sbactorpack").unwrap();
        let sarc = sarc::Sarc::read(&bytes).unwrap();
        let files = sarc.into_files();
        for (name, data) in files {
            println!("{:?} is {} bytes long", name, data.len())
        }
    }

    #[test]
    fn multithread_sarcs() {
        use rayon::prelude::*;
        use std::sync::{Arc, Mutex};
        let bytes = std::fs::read("test/Enemy_Lynel_Dark.sbactorpack").unwrap();
        let sarc = Arc::new(sarc::Sarc::read(&bytes).unwrap());
        (0..sarc.len()).into_par_iter().for_each(|i| {
            let file = sarc.get_file_by_index(i).unwrap();
            println!(
                "{} is {} bytes long",
                file.name_unchecked(),
                file.data.len()
            );
        });
        let sarc_writer = Arc::new(Mutex::new(SarcWriter::from(sarc.as_ref())));
        (0..100).into_par_iter().for_each(|i| {
            sarc_writer.lock().unwrap().add_file(
                &format!("File/{}.txt", i),
                format!("Contents for file # {}", i).as_bytes(),
            );
        });
        assert_eq!(
            Arc::try_unwrap(sarc_writer)
                .unwrap()
                .into_inner()
                .unwrap()
                .len(),
            225
        );
    }
}
