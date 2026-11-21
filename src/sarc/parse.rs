use core::hash::{Hash, Hasher};
use core::mem::size_of;

use super::*;
use crate::{Endian, Error, Result, TinyString};

fn find_null(data: &[u8]) -> Result<usize> {
    data.iter()
        .position(|b| b == &0u8)
        .ok_or(Error::InvalidData(
            "SARC filename contains unterminated string",
        ))
}

/// Read a u16 at `offset` in `data` according to `endian`, bounds-checked.
fn read_u16(data: &[u8], offset: usize, endian: Endian) -> Result<u16> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or(Error::InsufficientData(data.len(), offset + 2))?;
    Ok(match endian {
        Endian::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
        Endian::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
    })
}

/// Read a u32 at `offset` in `data` according to `endian`, bounds-checked.
fn read_u32(data: &[u8], offset: usize, endian: Endian) -> Result<u32> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or(Error::InsufficientData(data.len(), offset + 4))?;
    Ok(match endian {
        Endian::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        Endian::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
    })
}

/// Iterator over [`File`] entries in a [`Sarc`].
#[derive(Debug)]
pub struct FileIterator<'a> {
    sarc: &'a Sarc<'a>,
    index: usize,
    entry: ResFatEntry,
}

impl<'a> Iterator for FileIterator<'a> {
    type Item = File<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.sarc.num_files as usize {
            None
        } else {
            let entry_offset =
                self.sarc.entries_offset as usize + size_of::<ResFatEntry>() * self.index;
            self.entry = self.sarc.read_entry(entry_offset).ok()?;
            self.index += 1;
            Some(File {
                name: if self.entry.rel_name_opt_offset != 0 {
                    let name_offset = self.sarc.names_offset as usize
                        + (self.entry.rel_name_opt_offset & 0xFFFFFF) as usize * 4;
                    let term_pos = find_null(&self.sarc.data[name_offset..]).ok()?;
                    Some(
                        core::str::from_utf8(&self.sarc.data[name_offset..name_offset + term_pos])
                            .ok()?,
                    )
                } else {
                    None
                },
                data: self.sarc.data.get(
                    (self.sarc.data_offset + self.entry.data_begin) as usize
                        ..(self.sarc.data_offset + self.entry.data_end) as usize,
                )?,
                index: self.index,
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
    data: &'a [u8],
}

impl core::fmt::Debug for Sarc<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

impl<'a, S: core::borrow::Borrow<str>> core::ops::Index<S> for Sarc<'a> {
    type Output = [u8];

    fn index(&self, index: S) -> &Self::Output {
        self.get_data(index.borrow())
            .expect("File not found in SARC")
    }
}

impl<'a> Sarc<'_> {
    /// Parses a SARC archive from binary data.
    ///
    /// The data is borrowed, not owned: `Sarc` holds a `&'a [u8]` and never
    /// allocates.
    ///
    /// ```rust
    /// let raw: &[u8] = b"SARC\x00\x14\xfe\xff\x00\x00\x00\x28\x00\x00\x00\x28\x01\x00\x00\x00SFAT\x00\x0c\x00\x00\x00\x00\x00\x65SFNT\x00\x08\x00\x00";
    /// let sarc = roead::sarc::Sarc::new(raw).unwrap();
    /// assert_eq!(sarc.len(), 0);
    /// ```
    ///
    /// Yaz0-compressed archives are **not** decompressed here; decompress them
    /// with [`crate::yaz0::decompress_if`] first and pass the resulting slice:
    ///
    /// ```rust
    /// # #[cfg(feature = "yaz0")]
    /// # {
    /// # use roead::yaz0;
    /// # let raw: &[u8] = b"";
    /// let decompressed = yaz0::decompress_if(raw);
    /// assert!(decompressed.is_empty());
    /// # }
    /// ```
    pub fn new(data: &'a [u8]) -> crate::Result<Sarc<'a>> {
        if data.len() < 0x14 {
            return Err(Error::InsufficientData(data.len(), 0x14));
        }
        if &data[0..4] != b"SARC" {
            return Err(Error::BadMagic(TinyString::from_bytes(&data[0..4]), "SARC"));
        }

        // The BOM at offset 6 is self-describing. Read it as a little-endian
        // u16 (matching the original `Endian::read_ne` behavior on little-endian
        // hosts) and map the discriminant via `Endian::from_bom`.
        let bom_bytes = data
            .get(6..8)
            .ok_or(Error::InsufficientData(data.len(), 8))?;
        let endian = Endian::from_bom(u16::from_le_bytes([bom_bytes[0], bom_bytes[1]]))
            .ok_or(Error::InvalidData("Invalid SARC BOM"))?;

        let header = ResHeader {
            header_size: read_u16(data, 4, endian)?,
            bom: endian,
            file_size: read_u32(data, 8, endian)?,
            data_offset: read_u32(data, 12, endian)?,
            version: read_u16(data, 16, endian)?,
            reserved: read_u16(data, 18, endian)?,
        };
        if header.header_size != 0x14 {
            return Err(Error::InvalidData("SARC header wrong size (expected 0x14)"));
        }
        let data_offset = header.data_offset;
        if header.version != 0x0100 {
            return Err(Error::InvalidData("Invalid SARC version (expected 0x100)"));
        }

        let fat_header_offset = header.header_size as usize;
        if &data[fat_header_offset..fat_header_offset + 4] != b"SFAT" {
            return Err(Error::BadMagic(
                TinyString::from_bytes(&data[fat_header_offset..fat_header_offset + 4]),
                "SFAT",
            ));
        }
        let fat_header = ResFatHeader {
            header_size: read_u16(data, fat_header_offset + 4, endian)?,
            num_files: read_u16(data, fat_header_offset + 6, endian)?,
            hash_multiplier: read_u32(data, fat_header_offset + 8, endian)?,
        };
        if fat_header.header_size != 0x0C {
            return Err(Error::InvalidData("SFAT header wrong size (expected 0x0C)"));
        }
        let num_files = fat_header.num_files;
        if (num_files >> 0xE) != 0 {
            return Err(Error::TooManyFiles(num_files));
        }
        let hash_multiplier = fat_header.hash_multiplier;

        let entries_offset = fat_header_offset + 0x0C;
        let fat_end = entries_offset + 0x10 * num_files as usize;
        if data.len() < fat_end {
            return Err(Error::InsufficientData(data.len(), fat_end));
        }

        let fnt_header_offset = fat_end;
        if fnt_header_offset + 4 > data.len() {
            return Err(Error::InsufficientData(data.len(), fnt_header_offset + 4));
        }
        if &data[fnt_header_offset..fnt_header_offset + 4] != b"SFNT" {
            return Err(Error::BadMagic(
                TinyString::from_bytes(&data[fnt_header_offset..fnt_header_offset + 4]),
                "SFNT",
            ));
        }
        let fnt_header = ResFntHeader {
            header_size: read_u16(data, fnt_header_offset + 4, endian)?,
            reserved: read_u16(data, fnt_header_offset + 6, endian)?,
        };
        if fnt_header.header_size != 0x08 {
            return Err(Error::InvalidData("SFNT header wrong size (expected 0x8)"));
        }

        let names_offset = (fnt_header_offset + 0x08) as u32;
        if data_offset < names_offset {
            return Err(Error::InvalidData("Invalid name table offset in SARC"));
        }

        Ok(Sarc {
            data,
            data_offset,
            endian,
            entries_offset: entries_offset as u16,
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
        while a <= b {
            let m: u32 = (a + b) / 2;
            let entry_offset = self.entries_offset as usize + 0x10 * m as usize;
            let hash = read_u32(self.data, entry_offset, self.endian)?;
            match needle_hash.cmp(&hash) {
                core::cmp::Ordering::Less => match m.checked_sub(1) {
                    Some(v) => b = v,
                    None => return Ok(None),
                },
                core::cmp::Ordering::Greater => a = m + 1,
                core::cmp::Ordering::Equal => return Ok(Some(m as usize)),
            }
        }
        Ok(None)
    }

    /// Get a file by name, returning `None` on its absence or any error.
    /// If you need to know the error, use [`Sarc::try_get`].
    pub fn get(&self, file: &str) -> Option<File<'_>> {
        let file_index = self.find_file(file).ok()?;
        file_index.and_then(|i| self.file_at(i).ok())
    }

    /// Get a file by name, returning a [`Result`] of an [`Option`]. This
    /// distinguishes between failed parsing (e.g. due to a corrupted SARC)
    /// and the absence of the file. If you don't care about any potential
    /// errors, just whether you can get the file data, use [`Sarc::get`].
    pub fn try_get(&self, file: &str) -> Result<Option<File<'_>>> {
        let file_index = self.find_file(file)?;
        file_index.map(|i| self.file_at(i)).transpose()
    }

    /// Get file data by name, returning a [`Result`] of an [`Option`]. This
    /// distinguishes between failed parsing (e.g. due to a corrupted SARC)
    /// and the absence of the file. If you don't care about any potential
    /// errors, just whether you can get the file data, use [`Sarc::get_data`].
    pub fn try_get_data(&self, file: &str) -> Result<Option<&[u8]>> {
        let file_index = self.find_file(file)?;
        file_index
            .map(|i| -> Result<&[u8]> {
                let entry_offset = self.entries_offset as usize + size_of::<ResFatEntry>() * i;
                let entry = self.read_entry(entry_offset)?;
                let data_begin = self.data_offset as usize + entry.data_begin as usize;
                let data_end = self.data_offset as usize + entry.data_end as usize;
                self.data
                    .get(data_begin..data_end)
                    .ok_or(Error::InsufficientData(self.data.len(), data_end))
            })
            .transpose()
    }

    /// Get file data by name, returning `None` on its absence or any error.
    /// If you need to know the error, use [`Sarc::try_get_data`].
    pub fn get_data(&self, file: &str) -> Option<&[u8]> {
        self.try_get_data(file).ok().flatten()
    }

    /// Get a file by index. Returns error if index > file count.
    pub fn file_at(&self, index: usize) -> Result<File<'_>> {
        if index >= self.num_files as usize {
            return Err(Error::IndexOutOfRange {
                index,
                len: self.num_files as usize,
            });
        }

        let entry_offset = self.entries_offset as usize + size_of::<ResFatEntry>() * index;
        let entry = self.read_entry(entry_offset)?;

        let data_begin = self.data_offset as usize + entry.data_begin as usize;
        let data_end = self.data_offset as usize + entry.data_end as usize;
        let data = self
            .data
            .get(data_begin..data_end)
            .ok_or(Error::InsufficientData(self.data.len(), data_end))?;

        Ok(File {
            name: if entry.rel_name_opt_offset != 0 {
                let name_offset = self.names_offset as usize
                    + (entry.rel_name_opt_offset & 0xFFFFFF) as usize * 4;
                let term_pos = find_null(&self.data[name_offset..])?;
                Some(core::str::from_utf8(
                    &self.data[name_offset..name_offset + term_pos],
                )?)
            } else {
                None
            },
            data,
            index,
        })
    }

    fn read_entry(&self, offset: usize) -> Result<ResFatEntry> {
        Ok(ResFatEntry {
            name_hash: read_u32(self.data, offset, self.endian)?,
            rel_name_opt_offset: read_u32(self.data, offset + 4, self.endian)?,
            data_begin: read_u32(self.data, offset + 8, self.endian)?,
            data_end: read_u32(self.data, offset + 12, self.endian)?,
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
            sarc: self,
        }
    }

    /// Guess the minimum data alignment for files that are stored in the
    /// archive
    pub fn guess_min_alignment(&self) -> usize {
        const MIN_ALIGNMENT: u32 = 4;
        let mut gcd = MIN_ALIGNMENT;
        let mut offset = self.entries_offset as usize;
        for _ in 0..self.num_files {
            let entry = self
                .read_entry(offset)
                .expect("Data should have valid ResFatEntry");
            offset += size_of::<ResFatEntry>();
            gcd = gcd_usize(gcd as usize, (self.data_offset + entry.data_begin) as usize) as u32;
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

/// Size = 0x14
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) struct ResHeader {
    pub header_size: u16,
    pub bom: Endian,
    pub file_size: u32,
    pub data_offset: u32,
    pub version: u16,
    pub reserved: u16,
}

/// Size = 0x0C
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct ResFatHeader {
    pub header_size: u16,
    pub num_files: u16,
    pub hash_multiplier: u32,
}

/// Size = 0x10
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) struct ResFatEntry {
    pub name_hash: u32,
    pub rel_name_opt_offset: u32,
    pub data_begin: u32,
    pub data_end: u32,
}

/// Size = 0x08
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) struct ResFntHeader {
    pub header_size: u16,
    pub reserved: u16,
}

/// Check if a potential alignment is valid for building a SARC
#[inline(always)]
pub const fn is_valid_alignment(alignment: usize) -> bool {
    alignment != 0 && (alignment & (alignment - 1)) == 0
}

/// Greatest common divisor of two alignments.
fn gcd_usize(a: usize, b: usize) -> usize {
    use num_integer::Integer;
    a.gcd(&b)
}

#[cfg(test)]
mod tests {
    use std::fs::read;

    use super::*;
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
            sarc.get(file)
                .unwrap_or_else(|| panic!("Could not find file {}", file));
        }
    }
}
