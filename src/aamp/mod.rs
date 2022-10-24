//! Port of the `oead::aamp` module.
//!
//! Only version 2, little endian and UTF-8 binary parameter archives are
//! supported. All parameter types including buffers are supported.  
//! The YAML output is compatible with the pure Python aamp library.
//!
//! The main type is the `ParameterIO`, which will usually be constructed
//! from binary data of a YAML document. Some sample usage:
//! ```
//! # use roead::aamp::*;
//! # fn doctest() -> std::result::Result<(), Box<dyn std::error::Error>> {
//! let data = std::fs::read("test/aamp/Lizalfos.bphysics")?;
//! let pio = ParameterIO::from_binary(&data)?; // Parse AAMP from binary data
//! // A parameter IO is to an extent interchangeable with its root list.
//! for (hash, list) in pio.lists().iter() {
//!     // Do stuff with lists
//! }
//! if let Some(demo_obj) = pio.object("DemoAIActionIdx") { // Access a parameter object
//!     for (hash, parameter) in demo_obj.iter() {
//!         // Do stuff with parameters
//!     }
//! }
//! // Dumps YAML representation to a String
//! // let yaml_dump: String = pio.to_text();
//! # Ok(())
//! # }
//! ```
//!
//! All parameter map structures ([`ParameterObject`], [`ParameterObjectMap`],
//! [`ParameterListMap`]) can take either a name or a hash for key-based
//! operations, and likewise can be indexed by the same. As usual, indexing into
//! a non-existent key will panic.
mod names;
mod parser;
#[cfg(feature = "yaml")]
mod text;
mod writer;
use crate::{types::*, util::u24, Error, Result};
use binrw::binrw;
use indexmap::IndexMap;
pub use names::{get_default_name_table, NameTable};
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use smartstring::alias::String;

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
        $crate::aamp::hash_name($name)
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
#[derive(Debug, Clone)]
pub enum Parameter {
    /// Boolean.
    Bool(bool),
    /// Float.
    F32(f32),
    /// Int.
    Int(i32),
    /// 2D vector.
    Vec2(Vector2f),
    /// 3D vector.
    Vec3(Vector3f),
    /// 4D vector.
    Vec4(Vector4f),
    /// Color.
    Color(Color),
    /// String (max length 32 bytes).
    String32(FixedSafeString<32>),
    /// String (max length 64 bytes).
    String64(FixedSafeString<64>),
    /// A single curve.
    Curve1([Curve; 1]),
    /// Two curves.
    Curve2([Curve; 2]),
    /// Three curves.
    Curve3([Curve; 3]),
    /// Four curves.
    Curve4([Curve; 4]),
    /// Buffer of signed ints.
    BufferInt(Vec<i32>),
    /// Buffer of floats.
    BufferF32(Vec<f32>),
    /// String (max length 256 bytes).
    String256(FixedSafeString<256>),
    /// Quaternion.
    Quat(Quat),
    /// Unsigned int.
    U32(u32),
    /// Buffer of unsigned ints.
    BufferU32(Vec<u32>),
    /// Buffer of binary data.
    BufferBinary(Vec<u8>),
    /// String (no length limit).
    StringRef(String),
}

impl Parameter {
    fn type_name(&self) -> String {
        match self {
            Parameter::Bool(_) => "Bool".into(),
            Parameter::F32(_) => "F32".into(),
            Parameter::Int(_) => "Int".into(),
            Parameter::Vec2(_) => "Vec2".into(),
            Parameter::Vec3(_) => "Vec3".into(),
            Parameter::Vec4(_) => "Vec4".into(),
            Parameter::Color(_) => "Color".into(),
            Parameter::String32(_) => "String32".into(),
            Parameter::String64(_) => "String64".into(),
            Parameter::Curve1(_) => "Curve1".into(),
            Parameter::Curve2(_) => "Curve2".into(),
            Parameter::Curve3(_) => "Curve3".into(),
            Parameter::Curve4(_) => "Curve4".into(),
            Parameter::BufferInt(_) => "BufferInt".into(),
            Parameter::BufferF32(_) => "BufferF32".into(),
            Parameter::String256(_) => "String256".into(),
            Parameter::Quat(_) => "Quat".into(),
            Parameter::U32(_) => "U32".into(),
            Parameter::BufferU32(_) => "BufferU32".into(),
            Parameter::BufferBinary(_) => "BufferBinary".into(),
            Parameter::StringRef(_) => "StringRef".into(),
        }
    }

    /// Get the inner bool value.
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Parameter::Bool(b) => Ok(*b),
            _ => Err(Error::TypeError(self.type_name(), "Bool")),
        }
    }

    /// Get a mutable reference to the inner bool.
    pub fn as_mut_bool(&mut self) -> Result<&mut bool> {
        match self {
            Parameter::Bool(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "bool")),
        }
    }

    /// Extract the inner bool value.
    pub fn into_bool(self) -> Result<bool> {
        match self {
            Parameter::Bool(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "bool")),
        }
    }

    /// Get the inner f32 value.
    pub fn as_f32(&self) -> Result<f32> {
        match self {
            Parameter::F32(f) => Ok(*f),
            _ => Err(Error::TypeError(self.type_name(), "F32")),
        }
    }

    /// Get a mutable reference to the inner f32.
    pub fn as_mut_f32(&mut self) -> Result<&mut f32> {
        match self {
            Parameter::F32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "f32")),
        }
    }

    /// Extract the inner f32 value.
    pub fn into_f32(self) -> Result<f32> {
        match self {
            Parameter::F32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "f32")),
        }
    }

    /// Get the inner i32 value.
    pub fn as_int(&self) -> Result<i32> {
        match self {
            Parameter::Int(i) => Ok(*i),
            _ => Err(Error::TypeError(self.type_name(), "Int")),
        }
    }

    /// Get a mutable reference to the inner i32.
    pub fn as_mut_int(&mut self) -> Result<&mut i32> {
        match self {
            Parameter::Int(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "i32")),
        }
    }

    /// Extract the inner i32 value.
    pub fn into_int(self) -> Result<i32> {
        match self {
            Parameter::Int(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "i32")),
        }
    }

    /// Get the inner Vector2f value.
    pub fn as_vec2(&self) -> Result<&Vector2f> {
        match self {
            Parameter::Vec2(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Vec2")),
        }
    }

    /// Get a mutable reference to the inner Vector2f.
    pub fn as_mut_vec2(&mut self) -> Result<&mut Vector2f> {
        match self {
            Parameter::Vec2(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "vec2")),
        }
    }

    /// Extract the inner Vector2f value.
    pub fn into_vec2(self) -> Result<Vector2f> {
        match self {
            Parameter::Vec2(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "vec2")),
        }
    }

    /// Get the inner Vector3f value.
    pub fn as_vec3(&self) -> Result<&Vector3f> {
        match self {
            Parameter::Vec3(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Vec3")),
        }
    }

    /// Get a mutable reference to the inner Vector3f.
    pub fn as_mut_vec3(&mut self) -> Result<&mut Vector3f> {
        match self {
            Parameter::Vec3(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "vec3")),
        }
    }

    /// Extract the inner Vector3f value.
    pub fn into_vec3(self) -> Result<Vector3f> {
        match self {
            Parameter::Vec3(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "vec3")),
        }
    }

    /// Get the inner Vector4f value.
    pub fn as_vec4(&self) -> Result<&Vector4f> {
        match self {
            Parameter::Vec4(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Vec4")),
        }
    }

    /// Get a mutable reference to the inner Vector4f.
    pub fn as_mut_vec4(&mut self) -> Result<&mut Vector4f> {
        match self {
            Parameter::Vec4(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "vec4")),
        }
    }

    /// Extract the inner Vector4f value.
    pub fn into_vec4(self) -> Result<Vector4f> {
        match self {
            Parameter::Vec4(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "vec4")),
        }
    }

    /// Get the inner BufferF32 value.
    pub fn as_buffer_f32(&self) -> Result<&[f32]> {
        match self {
            Parameter::BufferF32(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "BufferF32")),
        }
    }

    /// Get a mutable reference to the inner BufferF32.
    pub fn as_mut_buffer_f32(&mut self) -> Result<&mut Vec<f32>> {
        match self {
            Parameter::BufferF32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Vec<f32>")),
        }
    }

    /// Extract the inner BufferF32 value.
    pub fn into_buffer_f32(self) -> Result<Vec<f32>> {
        match self {
            Parameter::BufferF32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Vec<f32>")),
        }
    }

    /// Get the inner BufferI32 value.
    pub fn as_buffer_int(&self) -> Result<&[i32]> {
        match self {
            Parameter::BufferInt(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "BufferI32")),
        }
    }

    /// Get a mutable reference to the inner BufferI32.
    pub fn as_mut_buffer_int(&mut self) -> Result<&mut Vec<i32>> {
        match self {
            Parameter::BufferInt(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Vec<i32>")),
        }
    }

    /// Extract the inner BufferI32 value.
    pub fn into_buffer_int(self) -> Result<Vec<i32>> {
        match self {
            Parameter::BufferInt(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Vec<i32>")),
        }
    }

    /// Get the inner color value.
    pub fn as_color(&self) -> Result<&Color> {
        match self {
            Parameter::Color(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Color")),
        }
    }

    /// Get a mutable reference to the inner color.
    pub fn as_mut_color(&mut self) -> Result<&mut Color> {
        match self {
            Parameter::Color(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "color")),
        }
    }

    /// Extract the inner color value.
    pub fn into_color(self) -> Result<Color> {
        match self {
            Parameter::Color(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "color")),
        }
    }

    /// Get the inner String32 value.
    pub fn as_string32(&self) -> Result<&FixedSafeString<32>> {
        match self {
            Parameter::String32(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "String32")),
        }
    }

    /// Get a mutable reference to the inner String32.
    pub fn as_mut_string32(&mut self) -> Result<&mut FixedSafeString<32>> {
        match self {
            Parameter::String32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "FixedSafeString<32>")),
        }
    }

    /// Extract the inner String32 value.
    pub fn into_string32(self) -> Result<FixedSafeString<32>> {
        match self {
            Parameter::String32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "FixedSafeString<32>")),
        }
    }

    /// Get the inner String64 value.
    pub fn as_string64(&self) -> Result<&FixedSafeString<64>> {
        match self {
            Parameter::String64(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "String64")),
        }
    }

    /// Get a mutable reference to the inner String64.
    pub fn as_mut_string64(&mut self) -> Result<&mut FixedSafeString<64>> {
        match self {
            Parameter::String64(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "FixedSafeString<64>")),
        }
    }

    /// Extract the inner String64 value.
    pub fn into_string64(self) -> Result<FixedSafeString<64>> {
        match self {
            Parameter::String64(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "FixedSafeString<64>")),
        }
    }

    /// Get the inner String256 value.
    pub fn as_string256(&self) -> Result<&FixedSafeString<256>> {
        match self {
            Parameter::String256(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "String256")),
        }
    }

    /// Get a mutable reference to the inner String256.
    pub fn as_mut_string256(&mut self) -> Result<&mut FixedSafeString<256>> {
        match self {
            Parameter::String256(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "FixedSafeString<256>")),
        }
    }

    /// Extract the inner String256 value.
    pub fn into_string256(self) -> Result<FixedSafeString<256>> {
        match self {
            Parameter::String256(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "FixedSafeString<256>")),
        }
    }

    /// Get the inner Curve1 value.
    pub fn as_curve1(&self) -> Result<&[Curve; 1]> {
        match self {
            Parameter::Curve1(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Curve1")),
        }
    }

    /// Get a mutable reference to the inner Curve1.
    pub fn as_mut_curve1(&mut self) -> Result<&mut [Curve; 1]> {
        match self {
            Parameter::Curve1(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "[Curve; 1]")),
        }
    }

    /// Extract the inner Curve1 value.
    pub fn into_curve1(self) -> Result<[Curve; 1]> {
        match self {
            Parameter::Curve1(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "[Curve; 1]")),
        }
    }

    /// Get the inner Curve2 value.
    pub fn as_curve2(&self) -> Result<&[Curve; 2]> {
        match self {
            Parameter::Curve2(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Curve2")),
        }
    }

    /// Get a mutable reference to the inner Curve2.
    pub fn as_mut_curve2(&mut self) -> Result<&mut [Curve; 2]> {
        match self {
            Parameter::Curve2(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "[Curve; 2]")),
        }
    }

    /// Extract the inner Curve2 value.
    pub fn into_curve2(self) -> Result<[Curve; 2]> {
        match self {
            Parameter::Curve2(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "[Curve; 2]")),
        }
    }

    /// Get the inner Curve3 value.
    pub fn as_curve3(&self) -> Result<&[Curve; 3]> {
        match self {
            Parameter::Curve3(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Curve3")),
        }
    }

    /// Get a mutable reference to the inner Curve3.
    pub fn as_mut_curve3(&mut self) -> Result<&mut [Curve; 3]> {
        match self {
            Parameter::Curve3(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "[Curve; 3]")),
        }
    }

    /// Extract the inner Curve3 value.
    pub fn into_curve3(self) -> Result<[Curve; 3]> {
        match self {
            Parameter::Curve3(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "[Curve; 3]")),
        }
    }

    /// Get the inner Curve4 value.
    pub fn as_curve4(&self) -> Result<&[Curve; 4]> {
        match self {
            Parameter::Curve4(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Curve4")),
        }
    }

    /// Get a mutable reference to the inner Curve4.
    pub fn as_mut_curve4(&mut self) -> Result<&mut [Curve; 4]> {
        match self {
            Parameter::Curve4(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "[Curve; 4]")),
        }
    }

    /// Extract the inner Curve4 value.
    pub fn into_curve4(self) -> Result<[Curve; 4]> {
        match self {
            Parameter::Curve4(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "[Curve; 4]")),
        }
    }

    /// Get the inner Quat value.
    pub fn as_quat(&self) -> Result<&Quat> {
        match self {
            Parameter::Quat(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Quat")),
        }
    }

    /// Get a mutable reference to the inner Quat.
    pub fn as_mut_quat(&mut self) -> Result<&mut Quat> {
        match self {
            Parameter::Quat(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Quat")),
        }
    }

    /// Extract the inner Quat value.
    pub fn into_quat(self) -> Result<Quat> {
        match self {
            Parameter::Quat(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Quat")),
        }
    }

    /// Get the inner u32 value.
    pub fn as_u32(&self) -> Result<u32> {
        match self {
            Parameter::U32(v) => Ok(*v),
            _ => Err(Error::TypeError(self.type_name(), "u32")),
        }
    }

    /// Get a mutable reference to the inner u32.
    pub fn as_mut_u32(&mut self) -> Result<&mut u32> {
        match self {
            Parameter::U32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "u32")),
        }
    }

    /// Extract the inner u32 value.
    pub fn into_u32(self) -> Result<u32> {
        match self {
            Parameter::U32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "u32")),
        }
    }

    /// Get the inner u32 buffer value.
    pub fn as_buffer_u32(&self) -> Result<&[u32]> {
        match self {
            Parameter::BufferU32(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "BufferU32")),
        }
    }

    /// Get a mutable reference to the inner u32 buffer.
    pub fn as_mut_buffer_u32(&mut self) -> Result<&mut [u32]> {
        match self {
            Parameter::BufferU32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Vec<u32>")),
        }
    }

    /// Extract the inner u32 buffer value.
    pub fn into_buffer_u32(self) -> Result<Vec<u32>> {
        match self {
            Parameter::BufferU32(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Vec<u32>")),
        }
    }

    /// Get the inner binary buffer value.
    pub fn as_buffer_binary(&self) -> Result<&[u8]> {
        match self {
            Parameter::BufferBinary(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "BufferBinary")),
        }
    }

    /// Get a mutable reference to the inner binary buffer.
    pub fn as_mut_buffer_binary(&mut self) -> Result<&mut [u8]> {
        match self {
            Parameter::BufferBinary(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Vec<u8>")),
        }
    }

    /// Extract the inner binary buffer value.
    pub fn into_buffer_binary(self) -> Result<Vec<u8>> {
        match self {
            Parameter::BufferBinary(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "Vec<u8>")),
        }
    }

    /// Get the inner string value.
    pub fn as_string_ref(&self) -> Result<&str> {
        match self {
            Parameter::StringRef(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "String")),
        }
    }

    /// Get a mutable reference to the inner string.
    pub fn as_mut_string_ref(&mut self) -> Result<&mut str> {
        match self {
            Parameter::StringRef(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "string")),
        }
    }

    /// Extract the inner string value.
    pub fn into_string_ref(self) -> Result<String> {
        match self {
            Parameter::StringRef(value) => Ok(value),
            _ => Err(Error::TypeError(self.type_name(), "string")),
        }
    }
}

impl From<bool> for Parameter {
    fn from(value: bool) -> Self {
        Parameter::Bool(value)
    }
}

impl TryFrom<Parameter> for bool {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Bool(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<f32> for Parameter {
    fn from(value: f32) -> Self {
        Parameter::F32(value)
    }
}

impl TryFrom<Parameter> for f32 {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::F32(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<i32> for Parameter {
    fn from(value: i32) -> Self {
        Parameter::Int(value)
    }
}

impl TryFrom<Parameter> for i32 {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Int(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Vector2f> for Parameter {
    fn from(value: Vector2f) -> Self {
        Parameter::Vec2(value)
    }
}

impl TryFrom<Parameter> for Vector2f {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Vec2(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Vector3f> for Parameter {
    fn from(value: Vector3f) -> Self {
        Parameter::Vec3(value)
    }
}

impl TryFrom<Parameter> for Vector3f {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Vec3(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Vector4f> for Parameter {
    fn from(value: Vector4f) -> Self {
        Parameter::Vec4(value)
    }
}

impl TryFrom<Parameter> for Vector4f {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Vec4(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Color> for Parameter {
    fn from(value: Color) -> Self {
        Parameter::Color(value)
    }
}

impl TryFrom<Parameter> for Color {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Color(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<FixedSafeString<32>> for Parameter {
    fn from(value: FixedSafeString<32>) -> Self {
        Parameter::String32(value)
    }
}

impl TryFrom<Parameter> for FixedSafeString<32> {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::String32(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<FixedSafeString<64>> for Parameter {
    fn from(value: FixedSafeString<64>) -> Self {
        Parameter::String64(value)
    }
}

impl TryFrom<Parameter> for FixedSafeString<64> {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::String64(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<[Curve; 1]> for Parameter {
    fn from(value: [Curve; 1]) -> Self {
        Parameter::Curve1(value)
    }
}

impl TryFrom<Parameter> for [Curve; 1] {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Curve1(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<[Curve; 2]> for Parameter {
    fn from(value: [Curve; 2]) -> Self {
        Parameter::Curve2(value)
    }
}

impl TryFrom<Parameter> for [Curve; 2] {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Curve2(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<[Curve; 3]> for Parameter {
    fn from(value: [Curve; 3]) -> Self {
        Parameter::Curve3(value)
    }
}

impl TryFrom<Parameter> for [Curve; 3] {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Curve3(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<[Curve; 4]> for Parameter {
    fn from(value: [Curve; 4]) -> Self {
        Parameter::Curve4(value)
    }
}

impl TryFrom<Parameter> for [Curve; 4] {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Curve4(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Vec<i32>> for Parameter {
    fn from(value: Vec<i32>) -> Self {
        Parameter::BufferInt(value)
    }
}

impl TryFrom<Parameter> for Vec<i32> {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::BufferInt(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Vec<f32>> for Parameter {
    fn from(value: Vec<f32>) -> Self {
        Parameter::BufferF32(value)
    }
}

impl TryFrom<Parameter> for Vec<f32> {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::BufferF32(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<FixedSafeString<256>> for Parameter {
    fn from(value: FixedSafeString<256>) -> Self {
        Parameter::String256(value)
    }
}

impl TryFrom<Parameter> for FixedSafeString<256> {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::String256(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Quat> for Parameter {
    fn from(value: Quat) -> Self {
        Parameter::Quat(value)
    }
}

impl TryFrom<Parameter> for Quat {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::Quat(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<u32> for Parameter {
    fn from(value: u32) -> Self {
        Parameter::U32(value)
    }
}

impl TryFrom<Parameter> for u32 {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::U32(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Vec<u32>> for Parameter {
    fn from(value: Vec<u32>) -> Self {
        Parameter::BufferU32(value)
    }
}

impl TryFrom<Parameter> for Vec<u32> {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::BufferU32(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<Vec<u8>> for Parameter {
    fn from(value: Vec<u8>) -> Self {
        Parameter::BufferBinary(value)
    }
}

impl TryFrom<Parameter> for Vec<u8> {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::BufferBinary(v) => Ok(v),
            _ => Err(value),
        }
    }
}

impl From<String> for Parameter {
    fn from(value: String) -> Self {
        Parameter::StringRef(value)
    }
}

impl TryFrom<Parameter> for String {
    type Error = Parameter;

    fn try_from(value: Parameter) -> std::result::Result<Self, Self::Error> {
        match value {
            Parameter::StringRef(v) => Ok(v),
            Parameter::String32(v) => Ok(v.into()),
            Parameter::String64(v) => Ok(v.into()),
            Parameter::String256(v) => Ok(v.into()),
            _ => Err(value),
        }
    }
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

    /// Returns a string slice if the parameter is any string type.
    pub fn as_str(&self) -> Result<&str> {
        match self {
            Parameter::String32(s) => Ok(s.as_str()),
            Parameter::String64(s) => Ok(s.as_str()),
            Parameter::String256(s) => Ok(s.as_str()),
            Parameter::StringRef(s) => Ok(s.as_str()),
            _ => Err(Error::TypeError(self.type_name(), "a string type")),
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

impl From<String> for Name {
    fn from(s: String) -> Self {
        Name(hash_name(&s))
    }
}

impl From<&String> for Name {
    fn from(s: &String) -> Self {
        Name(hash_name(s))
    }
}

impl From<std::string::String> for Name {
    fn from(s: std::string::String) -> Self {
        Name(hash_name(&s))
    }
}

impl From<&std::string::String> for Name {
    fn from(s: &std::string::String) -> Self {
        Name(hash_name(s))
    }
}

impl<const N: usize> From<FixedSafeString<N>> for Name {
    fn from(f: FixedSafeString<N>) -> Self {
        Name(hash_name(f.as_str()))
    }
}

impl From<u32> for Name {
    fn from(u: u32) -> Self {
        Name(u)
    }
}

impl std::borrow::Borrow<u32> for Name {
    fn borrow(&self) -> &u32 {
        &self.0
    }
}

impl AsRef<u32> for Name {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match names::get_default_name_table().get_name(self.0, 0, 0) {
            Some(name) => name.fmt(f),
            None => self.0.fmt(f),
        }
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

macro_rules! impl_map_wrapper {
    ($type:tt, $valtype:tt) => {
        impl $type {
            /// Return the number of entries.
            #[inline(always)]
            pub fn len(&self) -> usize {
                self.0.len()
            }

            /// Returns `true` if the map is empty.
            #[inline(always)]
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            /// Insert a new entry.
            #[inline(always)]
            pub fn insert<N: Into<Name>>(&mut self, key: N, value: $valtype) {
                self.0.insert(key.into(), value);
            }

            /// Insert multiple entries from an iterator.
            #[inline(always)]
            pub fn extend<I: IntoIterator<Item = (Name, $valtype)>>(&mut self, iter: I) {
                self.0.extend(iter);
            }

            /// Get an entry value by name or hash.
            #[inline(always)]
            pub fn get<N: Into<Name>>(&self, key: N) -> Option<&$valtype> {
                self.0.get(&key.into())
            }

            /// Get an entry value mutably by name or hash.
            #[inline(always)]
            pub fn get_mut<N: Into<Name>>(&mut self, key: N) -> Option<&mut $valtype> {
                self.0.get_mut(&key.into())
            }

            /// Get a full entry by name or hash.
            #[inline(always)]
            pub fn entry<N: Into<Name>>(&mut self, key: N) -> indexmap::map::Entry<Name, $valtype> {
                self.0.entry(key.into())
            }

            /// Iterate entries.
            #[inline(always)]
            pub fn iter(&self) -> impl Iterator<Item = (&Name, &$valtype)> {
                self.0.iter()
            }

            /// Iterate entries mutably.
            #[inline(always)]
            pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Name, &mut $valtype)> {
                self.0.iter_mut()
            }

            #[cfg(feature = "yaml")]
            /// Iterate entries by name (this is potentially rather expensive,
            /// as the name for each hash must be looked up from the default
            /// name table). It returns a [`Result`](`std::result::Result`) for
            /// each name, with the found name as the success value or the hash
            /// as the error value.
            ///
            /// This is only available with the `yaml` feature.
            #[inline(always)]
            pub fn iter_by_name(
                &self,
            ) -> impl Iterator<
                Item = (
                    std::result::Result<&'static std::borrow::Cow<'static, str>, u32>,
                    &$valtype,
                ),
            > {
                let table = crate::aamp::get_default_name_table();
                self.0.iter().map(|(hash, val)| {
                    (
                        match table.get_name(hash.0, 0, 0) {
                            Some(name) => Ok(name),
                            None => Err(hash.0),
                        },
                        val,
                    )
                })
            }
        }

        impl<N: Into<Name>> FromIterator<(N, $valtype)> for $type {
            fn from_iter<T: IntoIterator<Item = (N, $valtype)>>(iter: T) -> Self {
                Self(iter.into_iter().map(|(k, v)| (k.into(), v)).collect())
            }
        }

        impl<N: Into<Name>> std::ops::Index<N> for $type {
            type Output = $valtype;
            fn index(&self, name: N) -> &$valtype {
                self.0.get(&name.into()).expect("Index out of bounds")
            }
        }

        impl<N: Into<Name>> std::ops::IndexMut<N> for $type {
            fn index_mut(&mut self, name: N) -> &mut $valtype {
                self.0.get_mut(&name.into()).expect("Index out of bounds")
            }
        }
    };
}

/// [`Parameter`] object. This is essentially a dictionary of parameters.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterObject(pub ParameterStructureMap<Parameter>);
impl_map_wrapper!(ParameterObject, Parameter);

impl ParameterObject {
    /// Create a new empty parameter object.
    pub fn new() -> Self {
        Default::default()
    }

    /// Builder-like method to add a new parameter.
    pub fn with_parameter<N: Into<Name>>(
        mut self,
        name: N,
        parameter: Parameter,
    ) -> ParameterObject {
        self.0.insert(name.into(), parameter);
        self
    }

    /// Builder-like method to add multiple parameters from an iterator.
    pub fn with_parameters<N: Into<Name>, I: IntoIterator<Item = (N, Parameter)>>(
        mut self,
        iter: I,
    ) -> ParameterObject {
        self.0.extend(iter.into_iter().map(|(k, v)| (k.into(), v)));
        self
    }
}

/// Newtype map of parameter objects.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterObjectMap(pub ParameterStructureMap<ParameterObject>);
impl_map_wrapper!(ParameterObjectMap, ParameterObject);

/// Newtype map of parameter lists.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterListMap(pub ParameterStructureMap<ParameterList>);
impl_map_wrapper!(ParameterListMap, ParameterList);

/// Trait abstracting over [`ParameterList`] and [`ParameterIO`]. Useful since
/// a parameter IO is all but interchangeable with the root list.
pub trait ParameterListing {
    /// Returns a map of parameter lists.
    fn lists(&self) -> &ParameterListMap;
    /// Returns a mutable map of parameter lists.
    fn lists_mut(&mut self) -> &mut ParameterListMap;
    /// Get a parameter list by name or hash.
    fn list<N: Into<Name>>(&self, name: N) -> Option<&ParameterList> {
        self.lists().get(name.into())
    }
    /// Get a mutable reference to a parameter list by name or hash.
    fn list_mut<N: Into<Name>>(&mut self, name: N) -> Option<&mut ParameterList> {
        self.lists_mut().get_mut(name.into())
    }
    /// Set a parameter list by name or hash.
    fn set_list<N: Into<Name>>(&mut self, name: N, list: ParameterList) {
        self.lists_mut().insert(name.into(), list);
    }
    /// Returns a map of parameter objects.
    fn objects(&self) -> &ParameterObjectMap;
    /// Returns a mutable map of parameter objects.
    fn objects_mut(&mut self) -> &mut ParameterObjectMap;
    /// Get a parameter object by name or hash.
    fn object<N: Into<Name>>(&self, name: N) -> Option<&ParameterObject> {
        self.objects().get(name.into())
    }
    /// Get a mutable reference to a parameter object by name or hash.
    fn object_mut<N: Into<Name>>(&mut self, name: N) -> Option<&mut ParameterObject> {
        self.objects_mut().get_mut(name.into())
    }
    /// Set a parameter object by name or hash.
    fn set_object<N: Into<Name>>(&mut self, name: N, object: ParameterObject) {
        self.objects_mut().insert(name.into(), object);
    }
}

/// [`Parameter`] list. This is essentially a dictionary of parameter objects
/// and a dictionary of parameter lists.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterList {
    /// Map of child parameter objects.
    pub objects: ParameterObjectMap,
    /// Map of child parameter lists.
    pub lists: ParameterListMap,
}

impl ParameterListing for ParameterList {
    #[inline(always)]
    fn lists(&self) -> &ParameterListMap {
        &self.lists
    }

    #[inline(always)]
    fn lists_mut(&mut self) -> &mut ParameterListMap {
        &mut self.lists
    }

    #[inline(always)]
    fn objects(&self) -> &ParameterObjectMap {
        &self.objects
    }

    #[inline(always)]
    fn objects_mut(&mut self) -> &mut ParameterObjectMap {
        &mut self.objects
    }
}

impl ParameterList {
    /// Create a new empty parameter list.
    pub fn new() -> Self {
        Default::default()
    }

    /// Builder-like method to add a new parameter object.
    pub fn with_object<N: Into<Name>>(mut self, name: N, object: ParameterObject) -> ParameterList {
        self.objects.insert(name.into(), object);
        self
    }

    /// Builder-like method to add multiple parameter objects from an iterator.
    pub fn with_objects<N: Into<Name>, I: IntoIterator<Item = (N, ParameterObject)>>(
        mut self,
        iter: I,
    ) -> ParameterList {
        self.objects
            .extend(iter.into_iter().map(|(k, v)| (k.into(), v)));
        self
    }

    /// Builder-like method to add a new parameter list.
    pub fn with_list<N: Into<Name>>(mut self, name: N, list: ParameterList) -> ParameterList {
        self.lists.insert(name.into(), list);
        self
    }

    /// Builder-like method to add multiple parameter lists from an iterator.
    pub fn with_lists<N: Into<Name>, I: IntoIterator<Item = (N, ParameterList)>>(
        mut self,
        iter: I,
    ) -> ParameterList {
        self.lists
            .extend(iter.into_iter().map(|(k, v)| (k.into(), v)));
        self
    }
}

const ROOT_KEY: Name = Name::from_str("param_root");

/// [`Parameter`] IO. This is the root parameter list and the only structure
/// that can be serialized to or deserialized from a binary parameter archive.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterIO {
    /// Data version (not the AAMP format version). Typically 0.
    pub version: u32,
    /// Data type identifier. Typically “xml”.
    pub data_type: String,
    /// Root parameter list.
    pub param_root: ParameterList,
}

impl ParameterListing for ParameterIO {
    fn lists(&self) -> &ParameterListMap {
        &self.param_root.lists
    }

    fn lists_mut(&mut self) -> &mut ParameterListMap {
        &mut self.param_root.lists
    }

    fn objects(&self) -> &ParameterObjectMap {
        &self.param_root.objects
    }

    fn objects_mut(&mut self) -> &mut ParameterObjectMap {
        &mut self.param_root.objects
    }
}

impl ParameterIO {
    /// Create a new empty parameter IO.
    pub fn new() -> Self {
        Self {
            version: 0,
            data_type: "xml".into(),
            param_root: Default::default(),
        }
    }

    /// Builder-like method to set the data type.
    pub fn with_data_type(mut self, data_type: impl Into<String>) -> ParameterIO {
        self.data_type = data_type.into();
        self
    }

    /// Builder-like method to set the data version.
    pub fn with_version(mut self, version: u32) -> ParameterIO {
        self.version = version;
        self
    }

    /// Builder-like method to add a new parameter object.
    pub fn with_object<N: Into<Name>>(mut self, name: N, object: ParameterObject) -> ParameterIO {
        self.param_root.objects.insert(name.into(), object);
        self
    }

    /// Builder-like method to add multiple parameter objects from an iterator.
    pub fn with_objects<N: Into<Name>, I: IntoIterator<Item = (N, ParameterObject)>>(
        mut self,
        iter: I,
    ) -> ParameterIO {
        self.param_root
            .objects
            .extend(iter.into_iter().map(|(k, v)| (k.into(), v)));
        self
    }

    /// Builder-like method to add a new parameter list.
    pub fn with_list<N: Into<Name>>(mut self, name: N, list: ParameterList) -> ParameterIO {
        self.param_root.lists.insert(name.into(), list);
        self
    }

    /// Builder-like method to add multiple parameter lists from an iterator.
    pub fn with_lists<N: Into<Name>, I: IntoIterator<Item = (N, ParameterList)>>(
        mut self,
        iter: I,
    ) -> ParameterIO {
        self.param_root
            .lists
            .extend(iter.into_iter().map(|(k, v)| (k.into(), v)));
        self
    }

    /// Builder-like method to set the root parameter list.
    pub fn with_root(mut self, list: ParameterList) -> ParameterIO {
        self.param_root = list;
        self
    }
}
