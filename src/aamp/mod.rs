mod parser;
mod writer;
use crate::{types::*, util::u24};
use binrw::binrw;
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
    #[error("Invalid string")]
    BadString(#[from] std::str::Utf8Error),
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
/// # use roead::{h, aamp::{Name}};
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

#[derive(Debug)]
#[binrw]
#[brw(little, magic = b"AAMP")]
struct ResHeader {
    version: u32,     // 0x4
    flags: u32,       // 0x8
    file_size: u32,   // 0xC
    pio_version: u32, // 0x10
    /// Offset to parameter IO (relative to 0x30)
    pio_offset: u32, // 0x14
    /// Number of lists (including root)
    list_count: u32, // 0x18
    object_count: u32, // 0x1C
    param_count: u32, // 0x20
    data_section_size: u32, // 0x24
    string_section_size: u32, // 0x28
    unknown_section_size: u32, // 0x2C
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameter {
    name: Name,
    data_rel_offset: u24,
    type_: Type,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameterObj {
    name: Name,
    params_rel_offset: u16,
    param_count: u16,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameterList {
    name: Name,
    lists_rel_offset: u16,
    list_count: u16,
    objects_rel_offset: u16,
    object_count: u16,
}

/// Parameter.
///
/// Note that unlike `agl::utl::Parameter` the name is not stored as part of
/// the parameter class in order to make the parameter logic simpler and more
/// efficient.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[allow(clippy::derive_hash_xor_eq)]
#[derive(Debug, Clone, EnumAsInner)]
pub enum Parameter {
    Bool(bool),
    F32(f32),
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
    BufferF32(Vec<f32>),
    String256(FixedSafeString<256>),
    Quat(Quat),
    U32(u32),
    BufferU32(Vec<u32>),
    BufferBinary(Vec<u8>),
    StringRef(String),
}

impl std::hash::Hash for Parameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Parameter::Bool(b) => b.hash(state),
            Parameter::F32(f) => {
                b"f".hash(state);
                f.to_bits().hash(state)
            }
            Parameter::Int(i) => i.hash(state),
            Parameter::Vec2(v) => v.hash(state),
            Parameter::Vec3(v) => v.hash(state),
            Parameter::Vec4(v) => v.hash(state),
            Parameter::Color(c) => c.hash(state),
            Parameter::String32(s) => s.hash(state),
            Parameter::String64(s) => s.hash(state),
            Parameter::Curve1(c) => c.hash(state),
            Parameter::Curve2(c) => c.hash(state),
            Parameter::Curve3(c) => c.hash(state),
            Parameter::Curve4(c) => c.hash(state),
            Parameter::BufferInt(v) => v.hash(state),
            Parameter::BufferF32(v) => {
                for f in v {
                    b"f".hash(state);
                    f.to_bits().hash(state)
                }
            }
            Parameter::String256(s) => s.hash(state),
            Parameter::Quat(q) => q.hash(state),
            Parameter::U32(u) => u.hash(state),
            Parameter::BufferU32(v) => v.hash(state),
            Parameter::BufferBinary(v) => v.hash(state),
            Parameter::StringRef(s) => s.hash(state),
        }
    }
}

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::F32(a), Self::F32(b)) => almost::equal(*a, *b),
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Vec2(a), Self::Vec2(b)) => a == b,
            (Self::Vec3(a), Self::Vec3(b)) => a == b,
            (Self::Vec4(a), Self::Vec4(b)) => a == b,
            (Self::Color(a), Self::Color(b)) => a == b,
            (Self::String32(a), Self::String32(b)) => a == b,
            (Self::String64(a), Self::String64(b)) => a == b,
            (Self::Curve1(a), Self::Curve1(b)) => a == b,
            (Self::Curve2(a), Self::Curve2(b)) => a == b,
            (Self::Curve3(a), Self::Curve3(b)) => a == b,
            (Self::Curve4(a), Self::Curve4(b)) => a == b,
            (Self::BufferInt(a), Self::BufferInt(b)) => a == b,
            (Self::BufferF32(a), Self::BufferF32(b)) => a == b,
            (Self::String256(a), Self::String256(b)) => a == b,
            (Self::Quat(a), Self::Quat(b)) => a == b,
            (Self::U32(a), Self::U32(b)) => a == b,
            (Self::BufferU32(a), Self::BufferU32(b)) => a == b,
            (Self::BufferBinary(a), Self::BufferBinary(b)) => a == b,
            (Self::StringRef(a), Self::StringRef(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Parameter {}

impl Parameter {
    #[inline(always)]
    fn get_type(&self) -> Type {
        match self {
            Parameter::Bool(_) => Type::Bool,
            Parameter::F32(_) => Type::F32,
            Parameter::Int(_) => Type::Int,
            Parameter::Vec2(_) => Type::Vec2,
            Parameter::Vec3(_) => Type::Vec3,
            Parameter::Vec4(_) => Type::Vec4,
            Parameter::Color(_) => Type::Color,
            Parameter::String32(_) => Type::String32,
            Parameter::String64(_) => Type::String64,
            Parameter::Curve1(_) => Type::Curve1,
            Parameter::Curve2(_) => Type::Curve2,
            Parameter::Curve3(_) => Type::Curve3,
            Parameter::Curve4(_) => Type::Curve4,
            Parameter::BufferInt(_) => Type::BufferInt,
            Parameter::BufferF32(_) => Type::BufferF32,
            Parameter::String256(_) => Type::String256,
            Parameter::Quat(_) => Type::Quat,
            Parameter::U32(_) => Type::U32,
            Parameter::BufferU32(_) => Type::BufferU32,
            Parameter::BufferBinary(_) => Type::BufferBinary,
            Parameter::StringRef(_) => Type::StringRef,
        }
    }

    #[inline(always)]
    fn is_buffer_type(&self) -> bool {
        matches!(
            self,
            Parameter::BufferInt(_)
                | Parameter::BufferF32(_)
                | Parameter::BufferU32(_)
                | Parameter::BufferBinary(_)
        )
    }

    #[inline(always)]
    fn is_string_type(&self) -> bool {
        matches!(
            self,
            Parameter::String32(_)
                | Parameter::String64(_)
                | Parameter::String256(_)
                | Parameter::StringRef(_)
        )
    }

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl ParameterObject {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<N: Into<Name>> FromIterator<(N, Parameter)> for ParameterObject {
    fn from_iter<T: IntoIterator<Item = (N, Parameter)>>(iter: T) -> Self {
        Self(iter.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterObjectMap(pub ParameterStructureMap<ParameterObject>);

impl ParameterObjectMap {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<N: Into<Name>> FromIterator<(N, ParameterObject)> for ParameterObjectMap {
    fn from_iter<T: IntoIterator<Item = (N, ParameterObject)>>(iter: T) -> Self {
        Self(iter.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterListMap(pub ParameterStructureMap<ParameterList>);

impl ParameterListMap {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<N: Into<Name>> FromIterator<(N, ParameterList)> for ParameterListMap {
    fn from_iter<T: IntoIterator<Item = (N, ParameterList)>>(iter: T) -> Self {
        Self(iter.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

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
