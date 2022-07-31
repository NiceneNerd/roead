use super::*;
use crate::{Endian, Error, Result};
use binrw::{io::Write, BinReaderExt, BinWrite};
use cached::proc_macro::cached;
use num_integer::Integer;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::{Cursor, Seek, SeekFrom};

const FACTORY_INFO: &str = include_str!("../../data/botw_resource_factory_info.tsv");
const AGLENV_INFO: &str = include_str!("../../data/aglenv_file_info.json");
const HASH_MULTIPLIER: u32 = 0x65;

impl BinWrite for Endian {
    type Args = ();
    #[inline(always)]
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _: &binrw::WriteOptions,
        _: Self::Args,
    ) -> binrw::BinResult<()> {
        match *self {
            Self::Big => [0xFEu8, 0xFFu8].write_to(writer),
            Self::Little => [0xFFu8, 0xFEu8].write_to(writer),
        }
    }
}

#[cached]
fn get_botw_factory_names() -> FxHashSet<&'static str> {
    FACTORY_INFO
        .split('\n')
        .map(|line| line.split('\t').next().unwrap())
        .collect()
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AglEnvInfo {
    id: u16,
    i0: u16,
    ext: String,
    bext: String,
    s: Option<String>,
    align: i32,
    system: String,
    desc: String,
}

#[inline(always)]
fn align(pos: usize, alignment: usize) -> usize {
    ((pos as i64 + alignment as i64 - 1) & (0 - alignment as i64)) as usize
}

#[cached]
fn get_agl_env_alignment_requirements() -> Vec<(String, usize)> {
    serde_json::from_str::<Vec<AglEnvInfo>>(AGLENV_INFO)
        .unwrap()
        .into_iter()
        .filter_map(|e| (e.align >= 0).then_some((e.align as usize, e)))
        .flat_map(|(align, entry)| [(entry.ext, align), (entry.bext, align)].into_iter())
        .collect()
}

/// Newtype wrapper for [`String`] that hashes with the Nintendo algorithm and
/// is ordered by that hash value.
#[derive(Debug, Clone, Eq)]
#[repr(transparent)]
pub struct FileName(String);

impl std::ops::Deref for FileName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for FileName {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl std::hash::Hash for FileName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        hash_name(HASH_MULTIPLIER, &self.0).hash(state)
    }
}

impl PartialOrd for FileName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        hash_name(HASH_MULTIPLIER, &self.0).partial_cmp(&hash_name(HASH_MULTIPLIER, &other.0))
    }
}

impl Ord for FileName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        hash_name(HASH_MULTIPLIER, &self.0).cmp(&hash_name(HASH_MULTIPLIER, &other.0))
    }
}

impl AsRef<str> for FileName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<FileName> for String {
    fn as_ref(&self) -> &FileName {
        // This is sound because [`FileName`] is a newtype around [`String`] with
        // `repr(transparent)`.
        unsafe { &*(self as *const String as *const FileName) }
    }
}

impl From<&str> for FileName {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for FileName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::borrow::Borrow<str> for FileName {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl std::borrow::Borrow<String> for FileName {
    fn borrow(&self) -> &String {
        self.0.borrow()
    }
}

impl std::borrow::Borrow<FileName> for String {
    fn borrow(&self) -> &FileName {
        // This is sound because [`FileName`] is a newtype around [`String`] with
        // `repr(transparent)`.
        unsafe { &*(self as *const String as *const FileName) }
    }
}

impl std::fmt::Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A simple SARC archive writer
#[derive(Clone)]
pub struct SarcWriter {
    endian: Endian,
    legacy: bool,
    hash_multiplier: u32,
    min_alignment: usize,
    alignment_map: FxHashMap<String, usize>,
    options: binrw::WriteOptions,
    /// Files to be written.
    pub files: BTreeMap<FileName, Vec<u8>>,
}

impl std::fmt::Debug for SarcWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SarcWriter")
            .field("endian", &self.endian)
            .field("legacy", &self.legacy)
            .field("hash_multiplier", &self.hash_multiplier)
            .field("min_alignment", &self.min_alignment)
            .field("alignment_map", &self.alignment_map)
            .field("files", &self.files.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl PartialEq for SarcWriter {
    fn eq(&self, other: &Self) -> bool {
        self.endian == other.endian
            && self.legacy == other.legacy
            && self.hash_multiplier == other.hash_multiplier
            && self.min_alignment == other.min_alignment
            && self.alignment_map == other.alignment_map
            && self.files == other.files
    }
}

impl Eq for SarcWriter {}

impl SarcWriter {
    /// A simple SARC archive writer
    pub fn new(endian: Endian) -> SarcWriter {
        SarcWriter {
            endian,
            legacy: false,
            hash_multiplier: HASH_MULTIPLIER,
            alignment_map: FxHashMap::default(),
            files: BTreeMap::new(),
            options: binrw::WriteOptions::default().with_endian(match endian {
                Endian::Big => binrw::Endian::Big,
                Endian::Little => binrw::Endian::Little,
            }),
            min_alignment: 4,
        }
    }

    /// Creates a new SARC writer by taking attributes and files
    /// from an existing SARC reader
    pub fn from_sarc(sarc: &Sarc) -> SarcWriter {
        let endian = sarc.endian();
        SarcWriter {
            endian,
            legacy: false,
            hash_multiplier: HASH_MULTIPLIER,
            alignment_map: FxHashMap::default(),
            files: sarc
                .files()
                .filter_map(|f| f.name.map(|name| (name.into(), f.data.to_vec())))
                .collect(),
            options: binrw::WriteOptions::default().with_endian(match endian {
                Endian::Big => binrw::Endian::Big,
                Endian::Little => binrw::Endian::Little,
            }),
            min_alignment: sarc.guess_min_alignment(),
        }
    }

    /// Write a SARC archive to an in-memory buffer using the specified
    /// endianness. Default alignment requirements may be automatically
    /// added.
    pub fn to_binary(&mut self) -> Vec<u8> {
        let est_size: usize = 0x14
            + 0x0C
            + 0x8
            + self
                .files
                .iter()
                .map(|(n, d)| 0x10 + align(n.len() + 1, 4) + d.len())
                .sum::<usize>();
        let mut buf: Vec<u8> = Vec::with_capacity((est_size as f32 * 1.5) as usize);
        self.write(&mut Cursor::new(&mut buf)).unwrap();
        buf
    }

    /// Write a SARC archive to a Write + Seek writer using the specified
    /// endianness. Default alignment requirements may be automatically
    /// added.
    pub fn write<W: Write + Seek>(&mut self, writer: &mut W) -> Result<()> {
        writer.seek(SeekFrom::Start(0x14))?;
        ResFatHeader {
            header_size: 0x0C,
            num_files: self.files.len() as u16,
            hash_multiplier: self.hash_multiplier,
        }
        .write_options(writer, &self.options, ())?;

        self.add_default_alignments();
        let mut alignments: Vec<usize> = Vec::with_capacity(self.files.len());

        {
            let mut rel_string_offset = 0;
            let mut rel_data_offset = 0;
            for (name, data) in self.files.iter() {
                let alignment = self.get_alignment_for_file(name, data);
                alignments.push(alignment);

                let offset = align(rel_data_offset, alignment);
                ResFatEntry {
                    name_hash: hash_name(self.hash_multiplier, name.as_ref()),
                    rel_name_opt_offset: 1 << 24 | (rel_string_offset / 4),
                    data_begin: offset as u32,
                    data_end: (offset + data.len()) as u32,
                }
                .write_options(writer, &self.options, ())?;

                rel_data_offset = offset + data.len();
                rel_string_offset += align(name.len() + 1, 4) as u32;
            }
        }

        ResFntHeader {
            header_size: 0x8,
            reserved: 0,
        }
        .write_options(writer, &self.options, ())?;
        for (name, _) in self.files.iter() {
            name.0.as_bytes().write_options(writer, &self.options, ())?;
            0u8.write_options(writer, &self.options, ())?;
            let pos = writer.stream_position()? as usize;
            writer.seek(SeekFrom::Start(align(pos, 4) as u64))?;
        }

        let required_alignment = alignments
            .iter()
            .fold(1, |acc: usize, alignment| acc.lcm(alignment));
        let pos = writer.stream_position()? as usize;
        writer.seek(SeekFrom::Start(align(pos, required_alignment) as u64))?;
        let data_offset_begin = writer.stream_position()? as u32;
        for ((_, data), alignment) in self.files.iter().zip(alignments.iter()) {
            let pos = writer.stream_position()? as usize;
            writer.seek(SeekFrom::Start(align(pos, *alignment) as u64))?;
            data.write_to(writer)?;
        }

        let file_size = writer.stream_position()? as u32;
        writer.seek(SeekFrom::Start(0))?;
        ResHeader {
            header_size: 0x14,
            bom: self.endian,
            file_size,
            data_offset: data_offset_begin,
            version: 0x0100,
            reserved: 0,
        }
        .write_options(writer, &self.options, ())?;
        Ok(())
    }

    /// Add or modify a data alignment requirement for a file type. Set the
    /// alignment to 1 to revert.
    ///
    /// # Arguments
    ///
    /// * `ext` - File extension without the dot (e.g. “bgparamlist”)
    /// * `alignment` - Data alignment (must be a power of 2)
    pub fn add_alignment_requirement(&mut self, ext: String, alignment: usize) -> Result<()> {
        if !is_valid_alignment(alignment) {
            return Err(Error::InvalidData("Invalid alignment requirement"));
        }
        self.alignment_map.insert(ext, alignment);
        Ok(())
    }

    /// Builder-style method to add or modify a data alignment requirement for
    /// a file type. Set the alignment to 1 to revert.
    ///
    /// # Arguments
    ///
    /// * `ext` - File extension without the dot (e.g. “bgparamlist”)
    /// * `alignment` - Data alignment (must be a power of 2)
    pub fn with_alignment_requirement(mut self, ext: String, alignment: usize) -> Self {
        self.add_alignment_requirement(ext, alignment).unwrap();
        self
    }

    fn add_default_alignments(&mut self) {
        // This is perfectly sound because all of these alignments are powers
        // of 2 and thus the calls cannot fail.
        unsafe {
            for (ext, alignment) in get_agl_env_alignment_requirements() {
                self.add_alignment_requirement(ext, alignment)
                    .unwrap_unchecked();
            }
            self.add_alignment_requirement("ksky".to_owned(), 8)
                .unwrap_unchecked();
            self.add_alignment_requirement("ksky".to_owned(), 8)
                .unwrap_unchecked();
            self.add_alignment_requirement("bksky".to_owned(), 8)
                .unwrap_unchecked();
            self.add_alignment_requirement("gtx".to_owned(), 0x2000)
                .unwrap_unchecked();
            self.add_alignment_requirement("sharcb".to_owned(), 0x1000)
                .unwrap_unchecked();
            self.add_alignment_requirement("sharc".to_owned(), 0x1000)
                .unwrap_unchecked();
            self.add_alignment_requirement("baglmf".to_owned(), 0x80)
                .unwrap_unchecked();
            self.add_alignment_requirement(
                "bffnt".to_owned(),
                match self.endian {
                    Endian::Big => 0x2000,
                    Endian::Little => 0x1000,
                },
            )
            .unwrap_unchecked();
        }
    }

    /// Set the minimum data alignment
    pub fn set_min_alignment(&mut self, alignment: usize) -> Result<()> {
        if !is_valid_alignment(alignment) {
            return Err(Error::InvalidData("Invalid minimum SARC file alignment"));
        }
        self.min_alignment = alignment;
        Ok(())
    }

    /// Builder-style method to set the minimum data alignment
    pub fn with_min_alignment(mut self, alignment: usize) -> Self {
        self.set_min_alignment(alignment).unwrap();
        self
    }

    /// Set whether to use legacy mode (for games without a BOTW-style
    /// resource system) for addtional alignment restrictions
    pub fn set_legacy_mode(&mut self, value: bool) {
        self.legacy = value
    }

    /// Builder-style method to set whether to use legacy mode (for games
    /// without a BOTW-style resource system) for addtional alignment
    /// restrictions
    pub fn with_legacy_mode(mut self, value: bool) -> Self {
        self.set_legacy_mode(value);
        self
    }

    /// Set the endianness
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian
    }

    /// Builder-style method to set the endianness
    pub fn with_endian(mut self, endian: Endian) -> Self {
        self.set_endian(endian);
        self
    }

    /// Checks if a data slice represents a SARC archive
    pub fn is_file_sarc(data: &[u8]) -> bool {
        data.len() >= 0x20
            && (&data[0..4] == b"SARC" || (&data[0..4] == b"Yaz0" && &data[0x11..0x15] == b"SARC"))
    }

    fn get_alignment_for_new_binary_file(data: &[u8]) -> usize {
        let mut reader = Cursor::new(data);
        if data.len() <= 0x20 {
            return 1;
        }
        reader.set_position(0xC);
        if let Ok(endian) = reader.read_be() {
            reader.set_position(0x1C);
            let file_size: u32 = match endian {
                Endian::Big => reader.read_be().unwrap(),
                Endian::Little => reader.read_le().unwrap(),
            };
            if file_size as usize != data.len() {
                return 1;
            } else {
                return 1 << data[0xE];
            }
        }
        1
    }

    fn get_alignment_for_cafe_bflim(data: &[u8]) -> usize {
        if data.len() <= 0x28 || &data[data.len() - 0x28..data.len() - 0x24] != b"FLIM" {
            1
        } else {
            let mut cur = Cursor::new(&data[data.len() - 0x8..]);
            let alignment: u16 = cur.read_be().unwrap();
            alignment as usize
        }
    }

    fn get_alignment_for_file(&self, name: impl AsRef<str>, data: &[u8]) -> usize {
        let name = name.as_ref();
        let ext = match name.rfind('.') {
            Some(idx) => &name[idx + 1..],
            None => "",
        };
        let mut alignment = self.min_alignment;
        if let Some(requirement) = self.alignment_map.get(ext) {
            alignment = alignment.lcm(requirement);
        }
        if self.legacy && Self::is_file_sarc(data) {
            alignment = alignment.lcm(&0x2000);
        }
        if self.legacy || !get_botw_factory_names().contains(ext) {
            alignment = alignment.lcm(&Self::get_alignment_for_new_binary_file(data));
            if let Endian::Big = self.endian {
                alignment = alignment.lcm(&Self::get_alignment_for_cafe_bflim(data));
            }
        }
        alignment
    }

    /// Add a file to the archive, with greater generic flexibility than using
    /// `insert` on the `files` field.
    pub fn add_file(&mut self, name: impl Into<String>, data: impl Into<Vec<u8>>) {
        self.files.insert(FileName(name.into()), data.into());
    }

    /// Builder-style method to add a file to the archive.
    pub fn with_file(mut self, name: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        self.add_file(name, data);
        self
    }

    /// Add files to the archive from an iterator, with greater generic
    /// flexibility than using `extend` on the `files` field.
    pub fn add_files<N, D>(&mut self, iter: impl IntoIterator<Item = (N, D)>)
    where
        N: Into<String>,
        D: Into<Vec<u8>>,
    {
        self.files.extend(
            iter.into_iter()
                .map(|(name, data)| (FileName(name.into()), data.into())),
        );
    }

    /// Builder-style method to add files to the archive from an iterator.
    pub fn with_files<N, D>(mut self, iter: impl IntoIterator<Item = (N, D)>) -> Self
    where
        N: Into<String>,
        D: Into<Vec<u8>>,
    {
        self.add_files(iter);
        self
    }
}

impl From<&Sarc<'_>> for SarcWriter {
    fn from(sarc: &Sarc) -> Self {
        Self::from_sarc(sarc)
    }
}

#[cfg(test)]
mod tests {
    use crate::sarc::{Sarc, SarcWriter};

    #[test]
    fn make_sarc() {
        for file in [
            "ActorObserverByActorTagTag.sarc",
            "test.sarc",
            "A-1.00.sarc",
        ] {
            let data = std::fs::read(std::path::Path::new("test/sarc").join(file)).unwrap();
            let sarc = Sarc::new(&data).unwrap();
            let mut sarc_writer = SarcWriter::from_sarc(&sarc);
            let new_data = sarc_writer.to_binary();
            let new_sarc = Sarc::new(&new_data).unwrap();
            if !Sarc::are_files_equal(&sarc, &new_sarc) {
                for (f1, f2) in sarc.files().zip(new_sarc.files()) {
                    if f1 != f2 {
                        std::fs::write("test/f1", f1.data).unwrap();
                        std::fs::write("test/f2", f2.data).unwrap();
                        panic!("File {:?} has changed in SARC {:?}", f1.name, file);
                    }
                }
            }
            if data != new_data {
                dbg!(sarc);
                dbg!(new_sarc);
                panic!(
                    "Roundtrip not binary identical, wrong byte at offset {}",
                    data.iter()
                        .zip(new_data.iter())
                        .enumerate()
                        .find(|(_, (b1, b2))| *b1 != *b2)
                        .unwrap()
                        .0
                );
            }
        }
    }
}
