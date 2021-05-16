use crate::ffi;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Yaz0Error {
    #[error("Invalid yaz0 magic, expected \"Yaz0\", found {0}")]
    MagicError(String),
    #[error("Invalid compression level, expected 6-9, found {0}")]
    InvalidLevelError(u8),
    #[error("oead could not compress or decompress")]
    OeadError(#[from] cxx::Exception),
}

type Result<T> = std::result::Result<T, Yaz0Error>;

pub fn decompress<B: AsRef<[u8]>>(data: B) -> Result<Vec<u8>> {
    if &data.as_ref()[0..4] != b"Yaz0" {
        return Err(Yaz0Error::MagicError(
            String::from_utf8_lossy(&data.as_ref()[0..4]).to_string(),
        ));
    }
    Ok(ffi::decompress(data.as_ref())?)
}

pub fn compress<B: AsRef<[u8]>>(data: B) -> Vec<u8> {
    ffi::compress(data.as_ref(), 7)
}

pub fn compress_with_level<B: AsRef<[u8]>>(data: B, level: u8) -> Result<Vec<u8>> {
    if !(6..=9).contains(&level) {
        return Err(Yaz0Error::InvalidLevelError(level));
    }
    Ok(ffi::compress(data.as_ref(), level))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decompress_test() {
        let data = std::fs::read("test/Cargo.stoml").unwrap();
        let contents = String::from_utf8(decompress(&data).unwrap()).unwrap();
        let decomp = std::fs::read_to_string("Cargo.toml").unwrap();
        assert_eq!(&contents[0..9], "[package]");
        assert_eq!(&contents, &decomp);
    }

    #[test]
    fn compress_test() {
        let data = std::fs::read("Cargo.toml").unwrap();
        let meta = std::fs::metadata("test/Cargo.stoml").unwrap();
        let comp = compress(&data);
        assert_eq!(comp.len(), meta.len() as usize);
        std::fs::write("test/Cargo.stoml", &comp).unwrap();
    }
}
