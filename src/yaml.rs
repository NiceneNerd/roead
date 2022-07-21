pub(crate) enum TagBasedType {
    Bool,
    Str,
    Int,
    Float,
    Null,
}

enum Scalar {
    Null,
    Bool(bool),
    Int(u64),
    Float(f64),
    String(String),
}

#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum YamlError {
    #[error(transparent)]
    ParseError(#[from] yaml_peg::parser::PError),
}
