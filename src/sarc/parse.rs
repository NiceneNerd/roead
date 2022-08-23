use super::*;
use crate::{Error, Result};
use binrw::{BinRead, BinReaderExt};
use core::mem::size_of;
use join_str::jstr;
use num_integer::Integer;
use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
    io::Cursor,
};

fn find_null(data: &[u8]) -> Result<usize> {
    data.iter()
        .position(|b| b == &0u8)
        .ok_or(Error::InvalidData(
            "SARC filename contains unterminated string",
        ))
}

#[inline(always)]
fn read<T: BinRead>(endian: Endian, reader: &mut Cursor<&[u8]>) -> Result<T>
where
    <T as binrw::BinRead>::Args: std::default::Default,
{
    Ok(match endian {
        Endian::Big => reader.read_be()?,
        Endian::Little => reader.read_le()?,
    })
}

/// Iterator over [`File`] entries in a [`Sarc`].
#[derive(Debug)]
pub struct FileIterator<'a> {
    sarc: &'a Sarc<'a>,
    index: usize,
    entry_offset: usize,
    entry: ResFatEntry,
}

impl<'a> Iterator for FileIterator<'a> {
    type Item = File<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.sarc.num_files as usize {
            None
        } else {
            self.entry_offset =
                self.sarc.entries_offset as usize + size_of::<ResFatEntry>() * self.index;
            self.entry = read(
                self.sarc.endian,
                &mut Cursor::new(&self.sarc.data[self.entry_offset..]),
            )
            .ok()?;
            self.index += 1;
            Some(File {
                name: if self.entry.rel_name_opt_offset != 0 {
                    let name_offset = self.sarc.names_offset as usize
                        + (self.entry.rel_name_opt_offset & 0xFFFFFF) as usize * 4;
                    let term_pos = find_null(&self.sarc.data[name_offset..]).ok()?;
                    Some(
                        std::str::from_utf8(&self.sarc.data[name_offset..name_offset + term_pos])
                            .ok()?,
                    )
                } else {
                    None
                },
                data: &self.sarc.data[(self.sarc.data_offset + self.entry.data_begin) as usize
                    ..(self.sarc.data_offset + self.entry.data_end) as usize],
                index: self.index,
                sarc: self.sarc,
            })
        }
    }
}

#[derive(Clone)]
/// A simple SARC archive reader
pub struct Sarc<'a> {
    num_files: u16,
    entries_offset: u16,
    hash_multiplier: u32,
    data_offset: u32,
    names_offset: u32,
    endian: Endian,
    data: Cow<'a, [u8]>,
}

impl std::fmt::Debug for Sarc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sarc")
            .field("num_files", &self.num_files)
            .field("entries_offset", &self.entries_offset)
            .field("hash_multiplier", &self.hash_multiplier)
            .field("data_offset", &self.data_offset)
            .field("names_offset", &self.names_offset)
            .field("endian", &self.endian)
            .finish()
    }
}

impl PartialEq for Sarc<'_> {
    /// Returns true if and only if the raw archive data is identical
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Sarc<'_> {}

impl Hash for Sarc<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

impl<'a, S: std::borrow::Borrow<str>> std::ops::Index<S> for Sarc<'a> {
    type Output = [u8];

    fn index(&self, index: S) -> &Self::Output {
        self.get_data(index.borrow()).unwrap().unwrap()
    }
}

impl<'a> Sarc<'_> {
    /// Parses a SARC archive from binary data.
    ///
    /// **Note**: If and only if the `yaz0` feature is enabled, this function
    /// automatically decompresses the SARC when necessary.
    pub fn new<T: Into<Cow<'a, [u8]>>>(data: T) -> crate::Result<Sarc<'a>> {
        let mut data = data.into();

        #[cfg(feature = "yaz0")]
        {
            if data.starts_with(b"Yaz0") {
                data = crate::yaz0::decompress(&data)?.into();
            }
        }

        let mut reader = Cursor::new(data.as_ref());
        reader.set_position(6);
        let endian: Endian = Endian::read(&mut reader).map_err(Error::from)?;
        reader.set_position(0);

        let header: ResHeader = read(endian, &mut reader)?;
        if header.version != 0x0100 {
            return Err(Error::InvalidData("Invalid SARC version (expected 0x100)"));
        }
        if header.header_size as usize != 0x14 {
            return Err(Error::InvalidData("SARC header wrong size (expected 0x14)"));
        }

        let fat_header: ResFatHeader = read(endian, &mut reader)?;
        if fat_header.header_size as usize != 0x0C {
            return Err(Error::InvalidData("SFAT header wrong size (expected 0x0C)"));
        }
        if (fat_header.num_files >> 0xE) != 0 {
            return Err(Error::InvalidDataD(jstr!(
                "Too many files in SARC ({&fat_header.num_files.to_string()})"
            )));
        }

        let num_files = fat_header.num_files;
        let entries_offset = reader.position() as u16;
        let hash_multiplier = fat_header.hash_multiplier;
        let data_offset = header.data_offset;

        let fnt_header_offset = entries_offset as usize + 0x10 * num_files as usize;
        reader.set_position(fnt_header_offset as u64);
        let fnt_header: ResFntHeader = read(endian, &mut reader)?;
        if fnt_header.header_size as usize != 0x08 {
            return Err(Error::InvalidData("SFNT header wrong size (expected 0x8)"));
        }

        let names_offset = reader.position() as u32;
        if data_offset < names_offset {
            return Err(Error::InvalidData("Invalid name table offset in SARC"));
        }
        Ok(Sarc {
            data,
            data_offset,
            endian,
            entries_offset,
            num_files,
            hash_multiplier,
            names_offset,
        })
    }

    /// Get the number of files that are stored in the archive
    pub fn len(&self) -> usize {
        self.num_files as usize
    }

    /// Check if the SARC contains no files.
    pub fn is_empty(&self) -> bool {
        self.num_files == 0
    }

    /// Get the offset to the beginning of file data
    pub fn data_offset(&self) -> usize {
        self.data_offset as usize
    }

    /// Get the archive endianness
    pub fn endian(&self) -> Endian {
        self.endian
    }

    #[inline(always)]
    fn find_file(&self, file: &str) -> Result<Option<usize>> {
        if self.num_files == 0 {
            return Ok(None);
        }
        let needle_hash = hash_name(self.hash_multiplier, file);
        let mut a: u32 = 0;
        let mut b: u32 = self.num_files as u32 - 1;
        let mut reader = Cursor::new(self.data.as_ref());
        while a <= b {
            let m: u32 = (a + b) as u32 / 2;
            reader.set_position(self.entries_offset as u64 + 0x10 * m as u64);
            let hash: u32 = read(self.endian, &mut reader)?;
            match needle_hash.cmp(&hash) {
                std::cmp::Ordering::Less => b = m - 1,
                std::cmp::Ordering::Greater => a = m + 1,
                std::cmp::Ordering::Equal => return Ok(Some(m as usize)),
            }
        }
        Ok(None)
    }

    /// Get a file by name
    pub fn get_file(&self, file: &str) -> Result<Option<File>> {
        let file_index = self.find_file(file)?;
        file_index.map(|i| self.file_at(i)).transpose()
    }

    /// Get file data by name.
    pub fn get_data(&self, file: &str) -> Result<Option<&[u8]>> {
        let file_index = self.find_file(file)?;
        file_index
            .map(|i| -> Result<&[u8]> {
                let entry_offset = self.entries_offset as usize + size_of::<ResFatEntry>() * i;
                let entry: ResFatEntry =
                    read(self.endian, &mut Cursor::new(&self.data[entry_offset..]))?;
                Ok(&self.data[(self.data_offset + entry.data_begin) as usize
                    ..(self.data_offset + entry.data_end) as usize])
            })
            .transpose()
    }

    /// Get a file by index. Returns error if index > file count.
    pub fn file_at(&self, index: usize) -> Result<File> {
        if index >= self.num_files as usize {
            return Err(Error::InvalidDataD(jstr!(
                "No file in SARC at index {&index.to_string()}"
            )));
        }

        let entry_offset = self.entries_offset as usize + size_of::<ResFatEntry>() * index;
        let entry: ResFatEntry = read(self.endian, &mut Cursor::new(&self.data[entry_offset..]))?;

        Ok(File {
            name: if entry.rel_name_opt_offset != 0 {
                let name_offset = self.names_offset as usize
                    + (entry.rel_name_opt_offset & 0xFFFFFF) as usize * 4;
                let term_pos = find_null(&self.data[name_offset..])?;
                Some(std::str::from_utf8(
                    &self.data[name_offset..name_offset + term_pos],
                )?)
            } else {
                None
            },
            data: &self.data[(self.data_offset + entry.data_begin) as usize
                ..(self.data_offset + entry.data_end) as usize],
            index,
            sarc: self,
        })
    }

    /// Returns an iterator over the contained files
    pub fn files(&self) -> FileIterator<'_> {
        FileIterator {
            entry: ResFatEntry {
                name_hash: 0,
                rel_name_opt_offset: 0,
                data_begin: 0,
                data_end: 0,
            },
            index: 0,
            entry_offset: self.entries_offset as usize,
            sarc: self,
        }
    }

    /// Guess the minimum data alignment for files that are stored in the
    /// archive
    pub fn guess_min_alignment(&self) -> usize {
        const MIN_ALIGNMENT: u32 = 4;
        let mut gcd = MIN_ALIGNMENT;
        let mut reader = Cursor::new(&self.data[self.entries_offset as usize..]);
        for _ in 0..self.num_files {
            let entry: ResFatEntry = read(self.endian, &mut reader).unwrap();
            gcd = gcd.gcd(&(self.data_offset + entry.data_begin));
        }

        if !is_valid_alignment(gcd as usize) {
            return MIN_ALIGNMENT as usize;
        }
        gcd as usize
    }

    /// Returns true is each archive contains the same files
    pub fn are_files_equal(sarc1: &Sarc, sarc2: &Sarc) -> bool {
        if sarc1.len() != sarc2.len() {
            return false;
        }

        for (file1, file2) in sarc1.files().zip(sarc2.files()) {
            if file1 != file2 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read;
    #[test]
    fn parse_sarc() {
        let data = read("test/sarc/Dungeon119.pack").unwrap();
        let sarc = Sarc::new(&data).unwrap();
        assert_eq!(sarc.endian(), Endian::Big);
        assert_eq!(sarc.len(), 10);
        assert_eq!(sarc.guess_min_alignment(), 4);
        for file in &[
            "NavMesh/CDungeon/Dungeon119/Dungeon119.shknm2",
            "Map/CDungeon/Dungeon119/Dungeon119_Static.smubin",
            "Map/CDungeon/Dungeon119/Dungeon119_Dynamic.smubin",
            "Actor/Pack/DgnMrgPrt_Dungeon119.sbactorpack",
            "Physics/StaticCompound/CDungeon/Dungeon119.shksc",
            "Map/CDungeon/Dungeon119/Dungeon119_TeraTree.sblwp",
            "Map/CDungeon/Dungeon119/Dungeon119_Clustering.sblwp",
            "Map/DungeonData/CDungeon/Dungeon119.bdgnenv",
            "Model/DgnMrgPrt_Dungeon119.sbfres",
            "Model/DgnMrgPrt_Dungeon119.Tex2.sbfres",
        ] {
            sarc.get_file(file)
                .unwrap()
                .unwrap_or_else(|| panic!("Could not find file {}", file));
        }
    }
}
