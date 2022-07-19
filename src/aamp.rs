use crate::types::*;
use decorum::R32;
use enum_as_inner::EnumAsInner;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use smartstring::alias::String;

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumAsInner)]
pub enum Parameter {
    Bool(bool),
    Int(i32),
    Float(R32),
    StringRef(String),
    String32(FixedSafeString<32>),
    String64(FixedSafeString<64>),
    String256(FixedSafeString<256>),
}
