//! Port of the `oead::sarc` module.
//!
//! Unlike the other modules in this crate, this does not provide bindings to
//! `oead` but is a port of its SARC implementation. Why? Because SARC is a
//! simple format, so much so that it is easier to reimplement than to do FFI.
//!
//! Sample usage, just reading a SARC:
//! ```
//! # #[cfg(feature = "std")]
//! # fn read_sarc() -> Result<(), Box<dyn std::error::Error>> {
//! # use roead::sarc::*;
//! # let data = std::fs::read("test/sarc/Dungeon119.pack")?;
//! # let sarc: Sarc = Sarc::new(&data)?; // In this case we borrow data, but we can also own
//! # assert_eq!(sarc.len(), 10); // Get the number of files
//! # assert_eq!(sarc.guess_min_alignment(), 4);
//! # for file in sarc.files() {
//! #     println!("File name: {}", file.name().unwrap());
//! # }
//! # Ok(())
//! # }
//! ```
//! And writing a SARC:
//! ```
//! # #[cfg(feature = "std")]
//! # fn write_sarc() -> Result<(), Box<dyn std::error::Error>> {
//! # use roead::sarc::*;
//! # use roead::Endian;
//! # let mut sarc_writer = SarcWriter::new(Endian::Big); // Create an empty SARC
//! # sarc_writer.set_min_alignment(4); // Set the alignment, if needed
//! # sarc_writer
//! #     .files
//! #     .insert("A/Dummy/File.txt".into(), b"This is a test".to_vec()); // Add a couple files
//! # sarc_writer
//! #     .files
//! #     .insert("A/Dummy/File2.txt".into(), b"This is another test".to_vec());
//! # sarc_writer.remove_file("A/Dummy/File.txt"); // Never mind!
//! # let data = sarc_writer.to_binary(); // Write to an in-memory buffer
//! # // We can also take construct a SARC writer from an existing SARC
//! # let sarc = Sarc::new(data.as_slice())?;
//! # let another_sarc_writer = SarcWriter::from_sarc(&sarc);
//! # Ok(())
//! # }
//! ```
mod parse;
#[cfg(feature = "std")]
mod write;
pub use parse::Sarc;
#[cfg(feature = "std")]
pub use write::SarcWriter;

/// Provides readonly access to a file that is stored in a SARC archive.
#[derive(Debug, Eq)]
pub struct File<'a> {
    /// File name. May be empty for file entries that do not use the file name
    /// table.
    pub name: Option<&'a str>,
    /// File data (as a slice).
    pub data: &'a [u8],
    index: usize,
}

impl<'a> PartialEq for File<'a> {
    /// Compares `name`, `data`, and `index` only — deliberately not the
    /// backing `Sarc`, so equality is O(file) rather than O(archive size).
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.data == other.data && self.index == other.index
    }
}

impl<'a> File<'a> {
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
        self.name.expect("File should have name")
    }

    /// File name. May be empty for file entries that do not use the file name
    /// table.
    ///
    /// # Safety
    /// Calling this function on a file without a name is undefined behavior.
    #[inline(always)]
    pub unsafe fn unwrap_name_unchecked(&self) -> &str {
        unsafe { self.name.unwrap_unchecked() }
    }

    /// File data (as a slice).
    #[inline(always)]
    pub fn data(&self) -> &'a [u8] {
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
        (self.data.len() > 4 && &self.data[0..4] == b"SARC")
            || (self.data.len() > 0x15 && &self.data[0x11..0x15] == b"SARC")
    }

    /// Attempt to parse file as SARC.
    pub fn parse_sarc(&self) -> crate::Result<Sarc<'_>> {
        Sarc::new(self.data)
    }

    /// Check if the file is yaz0 compressed.
    #[inline(always)]
    pub fn is_compressed(&self) -> bool {
        self.data.len() > 4 && &self.data[0..4] == b"Yaz0"
    }

    /// Check if the file is an AAMP.
    #[inline(always)]
    pub fn is_aamp(&self) -> bool {
        self.data.len() > 4 && &self.data[0..4] == b"AAMP"
    }

    /// Check if the file is a BYML document.
    #[inline(always)]
    pub fn is_byml(&self) -> bool {
        self.data.len() > 0x13
            && (&self.data[0..2] == b"BY"
                || &self.data[0..2] == b"YB"
                || &self.data[0x11..0x13] == b"BY"
                || &self.data[0x11..0x13] == b"YB")
    }
}

#[inline]
const fn hash_name(multiplier: u32, name: &str) -> u32 {
    let mut hash = 0u32;
    let bytes = name.as_bytes();
    let mut i = 0;
    while i < name.len() {
        hash = hash.wrapping_mul(multiplier).wrapping_add(bytes[i] as u32);
        i += 1;
    }
    hash
}

/// Check if a potential alignment is valid for building a SARC
#[inline(always)]
pub const fn is_valid_alignment(alignment: usize) -> bool {
    alignment != 0 && (alignment & (alignment - 1)) == 0
}
