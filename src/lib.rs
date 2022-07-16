// #![deny(missing_docs)]
//! TODO: Docs
#[cfg(feature = "sarc")]
pub mod sarc;
pub mod types;
#[cfg(feature = "yaz0")]
pub mod yaz0;

/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incorrect magic: found `{0}`, expected `{1}`.")]
    InvalidMagic(String, &'static str),
    #[error("Data too short: found {0:#x} bytes, expected >= {1:#x}.")]
    InsufficientData(usize, usize),
    #[cfg(feature = "yaz0")]
    #[error(transparent)]
    Yaz0Error(#[from] yaz0::Yaz0Error),
    #[error("{0}")]
    Any(String),
}

type Result<T> = std::result::Result<T, Error>;

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            #[cfg(feature = "yaz0")]
            Error::Yaz0Error(e) => Error::Any(join_str::jstr!("Yaz0Error: {&e.to_string()}")),
            other => other.clone(),
        }
    }
}
