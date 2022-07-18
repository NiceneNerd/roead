use smartstring::alias::String;
mod parser;

#[derive(Debug, thiserror::Error)]
pub enum BymlError {
    #[error("Incorrect BYML node type: found `{0}`, expected `{1}`.")]
    TypeError(std::string::String, std::string::String),
    #[error(transparent)]
    BinaryRwError(#[from] binrw::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Error parsing BYML data: {0}")]
    ParseError(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Byml {
    String(String),
    Binary(Vec<u8>),
    Array(Vec<Byml>),
    Hash(im::OrdMap<String, Byml>),
    Bool(bool),
    Int(i32),
    Float(f32),
    UInt(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
    Null,
}
