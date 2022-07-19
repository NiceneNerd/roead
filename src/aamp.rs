mod parser;
use crate::types::*;
use decorum::R32;
use enum_as_inner::EnumAsInner;
use indexmap::IndexMap;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use smartstring::alias::String;

#[derive(Debug, thiserror::Error)]
pub enum AampError {
    #[error("{0}")]
    InvalidData(&'static str),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    BinaryRWError(#[from] binrw::Error),
}

type ParameterStructureMap<V> =
    IndexMap<Name, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

/// CRC hash function matching that used in BOTW.
#[inline]
pub const fn hash_name(name: &str) -> u32 {
    let mut crc = 0xFFFFFFFF;
    let mut i = 0;
    while i < name.len() {
        crc ^= name.as_bytes()[i] as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        i += 1;
    }
    !crc
}

/// A convenient macro for hashing AAMP names. This can help ensure they are
/// hashed at compile time in contexts where the compiler may not otherwise
/// realize it is an option.
///
/// # Example
/// ```rust
/// # use roead::aamp::{Name, h};
/// # struct Pio;
/// # impl Pio {
/// #    fn list<I: Into<Name>>(&self, name: I) {}
/// # }
/// # let pio = Pio;
/// pio.list(h!("LinkTargets"));
#[macro_export]
macro_rules! h {
    ($name:expr) => {
        ::roead::aamp::hash_name($name)
    };
}

#[cfg(test)]
#[test]
fn check_hasher() {
    const HASHED: u32 = hash_name("The Abolition of Man");
    const HASH: u32 = 0x41afa934;
    assert_eq!(HASHED, HASH);
}

#[derive(Debug)]
#[binrw::binrw]
#[repr(u8)]
#[brw(repr = u8)]
enum Type {
    Bool = 0,
    F32,
    Int,
    Vec2,
    Vec3,
    Vec4,
    Color,
    String32,
    String64,
    Curve1,
    Curve2,
    Curve3,
    Curve4,
    BufferInt,
    BufferF32,
    String256,
    Quat,
    U32,
    BufferU32,
    BufferBinary,
    StringRef,
}

/// Parameter.
///
/// Note that unlike `agl::utl::Parameter` the name is not stored as part of
/// the parameter class in order to make the parameter logic simpler and more
/// efficient.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumAsInner)]
pub enum Parameter {
    Bool(bool),
    F32(R32),
    Int(i32),
    Vec2(Vector2f),
    Vec3(Vector3f),
    Vec4(Vector4f),
    Color(Color),
    String32(FixedSafeString<32>),
    String64(FixedSafeString<64>),
    Curve1([Curve; 1]),
    Curve2([Curve; 2]),
    Curve3([Curve; 3]),
    Curve4([Curve; 4]),
    BufferInt(Vec<i32>),
    BufferF32(Vec<R32>),
    String256(FixedSafeString<256>),
    Quat(Quat),
    U32(u32),
    BufferU32(Vec<u32>),
    BufferBinary(Vec<u8>),
    StringRef(String),
}

impl Parameter {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Parameter::String32(s) => Some(s.as_str()),
            Parameter::String64(s) => Some(s.as_str()),
            Parameter::String256(s) => Some(s.as_str()),
            Parameter::StringRef(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

/// Parameter structure name. This is a wrapper around a CRC32 hash.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[binrw::binrw]
#[brw(little)]
pub struct Name(u32);

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Name(hash_name(s))
    }
}

impl From<u32> for Name {
    fn from(u: u32) -> Self {
        Name(u)
    }
}

impl Name {
    /// The CRC32 hash of the name.
    pub fn hash(&self) -> u32 {
        self.0
    }

    /// Const function to construct from a string.
    pub const fn from_str(s: &str) -> Self {
        Name(hash_name(s))
    }
}

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterObject(pub ParameterStructureMap<Parameter>);

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterObjectMap(pub ParameterStructureMap<ParameterObject>);

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterListMap(pub ParameterStructureMap<ParameterList>);

pub trait ParameterListing {
    fn lists(&self) -> &ParameterListMap;
    fn lists_mut(&mut self) -> &mut ParameterListMap;
    fn list<N: Into<Name>>(&self, name: N) -> Option<&ParameterList>;
    fn objects(&self) -> &ParameterObjectMap;
    fn objects_mut(&mut self) -> &mut ParameterObjectMap;
    fn object<N: Into<Name>>(&self, name: N) -> Option<&ParameterObject>;
}

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterList {
    pub objects: ParameterObjectMap,
    pub lists: ParameterListMap,
}

const ROOT_KEY: Name = Name::from_str("param_root");

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterIO {
    pub version: u32,
    pub data_type: String,
    pub param_root: ParameterList,
}
