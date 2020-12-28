use crate::{ffi, Endian};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SarcError {
    #[error("Invalid SARC magic")]
    MagicError,
    #[error("Not enough data for valid SARC, expected >40 bytes, found {0}")]
    InsufficientDataError(usize),
    #[error("oead could not parse SARC")]
    OeadError(#[from] cxx::Exception),
}

type Result<T> = std::result::Result<T, SarcError>;

pub struct Sarc(cxx::UniquePtr<ffi::Sarc>);

impl std::fmt::Debug for Sarc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sarc")
            .field("data_offset", &self.data_offset())
            .field("alignment", &self.guess_min_alignment())
            .field("endian", &self.endian())
            .field("files", &self.list_filenames())
            .finish()
    }
}

impl PartialEq for Sarc {
    fn eq(&self, other: &Sarc) -> bool {
        self.0.files_eq(&other.0)
    }
}

impl Sarc {
    pub fn len(&self) -> usize {
        self.0.num_files() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.0.num_files() == 0
    }

    pub fn files(&self) -> impl Iterator<Item = (&str, &[u8])> {
        (0..self.len())
            .into_iter()
            .filter_map(move |i| self.get_file_by_index(i))
    }

    pub fn list_filenames(&self) -> Vec<&str> {
        (0..self.len())
            .into_iter()
            .filter_map(|i| self.0.idx_file_name(i as u16).ok())
            .collect()
    }

    pub fn get_file_data(&self, name: &str) -> Option<&[u8]> {
        self.0.get_file_data(name).ok()
    }

    pub fn get_file_by_index(&self, idx: usize) -> Option<(&str, &[u8])> {
        if idx >= self.len() {
            return None;
        }
        let name = self.0.idx_file_name(idx as u16);
        let data = self.0.idx_file_data(idx as u16);
        if let Ok(name) = name {
            Some((name, data.unwrap()))
        } else {
            None
        }
    }

    pub fn endian(&self) -> Endian {
        if self.0.big_endian() {
            Endian::Big
        } else {
            Endian::Little
        }
    }

    pub fn data_offset(&self) -> usize {
        self.0.get_offset() as usize
    }

    pub fn guess_min_alignment(&self) -> usize {
        self.0.guess_align()
    }

    pub fn read(data: &[u8]) -> Result<Sarc> {
        if data.len() < 40 {
            return Err(SarcError::InsufficientDataError(data.len()));
        }
        if &data[0..4] != b"SARC" {
            return Err(SarcError::MagicError);
        };
        Ok(Sarc(ffi::sarc_from_binary(data)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::{sarc, Endian};

    #[test]
    fn read_sarc() {
        let data = std::fs::read("test/Enemy_Lynel_Dark.bactorpack").unwrap();
        let sarc = sarc::Sarc::read(data.as_slice()).unwrap();
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
        dbg!(sarc);
    }

    #[test]
    fn sarc_eq() {
        let data = std::fs::read("test/Enemy_Lynel_Dark.bactorpack").unwrap();
        let data2 = std::fs::read("test/Enemy_Lynel_Dark.bactorpack").unwrap();
        let sarc = sarc::Sarc::read(&data).unwrap();
        let sarc2 = sarc::Sarc::read(&data2).unwrap();
        assert_eq!(sarc, sarc2)
    }
}
