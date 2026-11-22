//! Shared AAMP format walk and the [`ParameterIORef`] borrowed tree view.
//!
//! The walk in this module is core-only: it borrows all payloads
//! (`&str`, `&[u8]`) from the input buffer and never allocates. It is
//! generic over the [`AampSink`] trait, so the same format logic drives
//! both the alloc-backed [`ParameterIORef`] tree and the core-only
//! [`crate::aamp::arena::ParameterIOArena`] flat arena.
//!
//! Only version 2, little endian and UTF-8 binary parameter archives are
//! supported. All reads are bounds-checked (reporting
//! [`Error::InsufficientData`]) and integers/floats are decoded from raw
//! bytes (`from_le_bytes`), so no unaligned or endian-dependent references
//! are ever created.

use crate::{Error, Result, TinyString, types::*};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

use super::{Name, ROOT_KEY, Type};

/// Maximum depth of nested containers the walk will descend into.
///
/// Bounds recursion on malformed input: a container offset that points at an
/// ancestor loops until this cap trips (the arena trips its capacity error
/// first).
pub(crate) const AAMP_MAX_DEPTH: usize = 128;

/// A borrowed buffer element slice.
///
/// All integers and floats are little endian, matching the AAMP format
/// (little endian is required by the header flag).
#[derive(Debug, Clone, Copy)]
pub struct BufferRef<'a> {
    bytes: &'a [u8],
}

/// A borrowed sequence of 128-byte curve blocks.
#[derive(Debug, Clone, Copy)]
pub struct CurveRef<'a> {
    bytes: &'a [u8],
}

const CURVE_BLOCK_SIZE: usize = 128;
const CURVE_FLOAT_COUNT: usize = 30;

impl<'a> BufferRef<'a> {
    /// Wraps `bytes` as a buffer view. `pub` so tests can exercise alignment
    /// errors directly.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Returns the underlying byte slice.
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Number of `i32` elements (`bytes.len() / 4`).
    pub fn len_i32(&self) -> usize {
        self.bytes.len() / 4
    }

    /// Number of `f32` elements (`bytes.len() / 4`).
    pub fn len_f32(&self) -> usize {
        self.bytes.len() / 4
    }

    /// Number of `u32` elements (`bytes.len() / 4`).
    pub fn len_u32(&self) -> usize {
        self.bytes.len() / 4
    }

    /// Iterates the elements as little-endian `i32` values. Trailing bytes
    /// (if the slice length is not a multiple of 4) are ignored.
    pub fn iter_i32(&self) -> impl Iterator<Item = i32> + '_ {
        self.bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Iterates the elements as little-endian `f32` values.
    pub fn iter_f32(&self) -> impl Iterator<Item = f32> + '_ {
        self.bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Iterates the elements as little-endian `u32` values.
    pub fn iter_u32(&self) -> impl Iterator<Item = u32> + '_ {
        self.bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Returns the elements as a little-endian `&[i32]` slice.
    ///
    /// Only available on little-endian targets (the AAMP format is
    /// little-endian only). Returns [`Error::InvalidData`] when the slice is
    /// not aligned to 4.
    #[cfg(target_endian = "little")]
    pub fn as_i32(&self) -> Result<&'a [i32]> {
        if self.bytes.as_ptr().align_offset(4) != 0 {
            return Err(Error::InvalidData("AAMP buffer data is not aligned to 4"));
        }
        // SAFETY: the pointer is 4-byte aligned (checked above), the slice
        // holds `self.bytes.len() / 4` initialized `i32` values (the byte
        // slice itself is fully initialized), and the returned borrow is tied
        // to `self`.
        unsafe {
            Ok(core::slice::from_raw_parts(
                self.bytes.as_ptr().cast(),
                self.bytes.len() / 4,
            ))
        }
    }

    /// Returns the elements as a little-endian `&[f32]` slice.
    #[cfg(target_endian = "little")]
    pub fn as_f32(&self) -> Result<&'a [f32]> {
        if self.bytes.as_ptr().align_offset(4) != 0 {
            return Err(Error::InvalidData("AAMP buffer data is not aligned to 4"));
        }
        // SAFETY: the pointer is 4-byte aligned (checked above), the slice
        // holds `self.bytes.len() / 4` initialized `f32` values, and the
        // returned borrow is tied to `self`.
        unsafe {
            Ok(core::slice::from_raw_parts(
                self.bytes.as_ptr().cast(),
                self.bytes.len() / 4,
            ))
        }
    }

    /// Returns the elements as a little-endian `&[u32]` slice.
    #[cfg(target_endian = "little")]
    pub fn as_u32(&self) -> Result<&'a [u32]> {
        if self.bytes.as_ptr().align_offset(4) != 0 {
            return Err(Error::InvalidData("AAMP buffer data is not aligned to 4"));
        }
        // SAFETY: the pointer is 4-byte aligned (checked above), the slice
        // holds `self.bytes.len() / 4` initialized `u32` values, and the
        // returned borrow is tied to `self`.
        unsafe {
            Ok(core::slice::from_raw_parts(
                self.bytes.as_ptr().cast(),
                self.bytes.len() / 4,
            ))
        }
    }
}

impl<'a> PartialEq for BufferRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<'a> CurveRef<'a> {
    /// Wraps `bytes` as a curve block sequence. The length should be a
    /// multiple of 128.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Number of 128-byte curve blocks (`bytes.len() / 128`).
    pub fn len(&self) -> usize {
        self.bytes.len() / CURVE_BLOCK_SIZE
    }

    /// Returns `true` if there are no complete curve blocks.
    pub fn is_empty(&self) -> bool {
        self.bytes.len() < CURVE_BLOCK_SIZE
    }

    /// Decodes the curve at `index`, or `None` if out of range.
    pub fn curve(&self, index: usize) -> Option<Curve> {
        self.iter().nth(index)
    }

    /// Iterates the decoded curves.
    pub fn iter(&self) -> impl Iterator<Item = Curve> + '_ {
        self.bytes
            .as_chunks::<CURVE_BLOCK_SIZE>()
            .0
            .iter()
            .map(|block| {
                let a = u32::from_le_bytes([block[0], block[1], block[2], block[3]]);
                let b = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
                let mut floats = [0.0f32; CURVE_FLOAT_COUNT];
                let mut i = 0;
                while i < CURVE_FLOAT_COUNT {
                    let o = 8 + i * 4;
                    floats[i] =
                        f32::from_le_bytes([block[o], block[o + 1], block[o + 2], block[o + 3]]);
                    i += 1;
                }
                Curve { a, b, floats }
            })
    }
}

impl<'a> PartialEq for CurveRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

/// A borrowed parameter value.
///
/// Unlike the owned [`Parameter`](crate::aamp::Parameter), buffers are
/// borrowed byte slices ([`BufferRef`]) and curves are borrowed block
/// sequences ([`CurveRef`]) instead of owned vectors. Strings are `&str`
/// borrowed from the input buffer instead of [`FixedSafeString`]s.
#[derive(Debug, Clone, Copy)]
pub enum ParameterRef<'a> {
    /// Boolean.
    Bool(bool),
    /// Float.
    F32(f32),
    /// Int.
    I32(i32),
    /// Unsigned int.
    U32(u32),
    /// 2D vector.
    Vec2(Vector2f),
    /// 3D vector.
    Vec3(Vector3f),
    /// 4D vector.
    Vec4(Vector4f),
    /// Color.
    Color(Color),
    /// Quaternion.
    Quat(Quat),
    /// String (max length 32 bytes).
    String32(&'a str),
    /// String (max length 64 bytes).
    String64(&'a str),
    /// String (max length 256 bytes).
    String256(&'a str),
    /// String (no length limit).
    StringRef(&'a str),
    /// A single curve.
    Curve1(CurveRef<'a>),
    /// Two curves.
    Curve2(CurveRef<'a>),
    /// Three curves.
    Curve3(CurveRef<'a>),
    /// Four curves.
    Curve4(CurveRef<'a>),
    /// Buffer of signed ints.
    BufferInt(BufferRef<'a>),
    /// Buffer of floats.
    BufferF32(BufferRef<'a>),
    /// Buffer of unsigned ints.
    BufferU32(BufferRef<'a>),
    /// Buffer of binary data.
    BufferBinary(&'a [u8]),
}

impl<'a> ParameterRef<'a> {
    /// The name of this parameter type.
    pub fn type_name(&self) -> &'static str {
        match self {
            ParameterRef::Bool(_) => "Bool",
            ParameterRef::F32(_) => "F32",
            ParameterRef::I32(_) => "I32",
            ParameterRef::U32(_) => "U32",
            ParameterRef::Vec2(_) => "Vec2",
            ParameterRef::Vec3(_) => "Vec3",
            ParameterRef::Vec4(_) => "Vec4",
            ParameterRef::Color(_) => "Color",
            ParameterRef::Quat(_) => "Quat",
            ParameterRef::String32(_) => "String32",
            ParameterRef::String64(_) => "String64",
            ParameterRef::String256(_) => "String256",
            ParameterRef::StringRef(_) => "StringRef",
            ParameterRef::Curve1(_) => "Curve1",
            ParameterRef::Curve2(_) => "Curve2",
            ParameterRef::Curve3(_) => "Curve3",
            ParameterRef::Curve4(_) => "Curve4",
            ParameterRef::BufferInt(_) => "BufferInt",
            ParameterRef::BufferF32(_) => "BufferF32",
            ParameterRef::BufferU32(_) => "BufferU32",
            ParameterRef::BufferBinary(_) => "BufferBinary",
        }
    }

    /// Get the inner unsigned int value.
    pub fn as_u32(&self) -> Result<u32> {
        match self {
            ParameterRef::U32(u) => Ok(*u),
            _ => Err(Error::TypeError(self.type_name(), "u32")),
        }
    }

    /// Get the inner bool value.
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            ParameterRef::Bool(b) => Ok(*b),
            _ => Err(Error::TypeError(self.type_name(), "Bool")),
        }
    }

    /// Get the inner f32 value.
    pub fn as_f32(&self) -> Result<f32> {
        match self {
            ParameterRef::F32(f) => Ok(*f),
            _ => Err(Error::TypeError(self.type_name(), "F32")),
        }
    }

    /// Get the inner i32 value.
    pub fn as_i32(&self) -> Result<i32> {
        match self {
            ParameterRef::I32(i) => Ok(*i),
            _ => Err(Error::TypeError(self.type_name(), "Int")),
        }
    }

    /// Get the inner Vector2f value.
    pub fn as_vec2(&self) -> Result<&Vector2f> {
        match self {
            ParameterRef::Vec2(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Vec2")),
        }
    }

    /// Get the inner Vector3f value.
    pub fn as_vec3(&self) -> Result<&Vector3f> {
        match self {
            ParameterRef::Vec3(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Vec3")),
        }
    }

    /// Get the inner Vector4f value.
    pub fn as_vec4(&self) -> Result<&Vector4f> {
        match self {
            ParameterRef::Vec4(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Vec4")),
        }
    }

    /// Get the inner color value.
    pub fn as_color(&self) -> Result<&Color> {
        match self {
            ParameterRef::Color(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Color")),
        }
    }

    /// Get the inner quat value.
    pub fn as_quat(&self) -> Result<&Quat> {
        match self {
            ParameterRef::Quat(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Quat")),
        }
    }

    /// Get the inner String32 value.
    pub fn as_string32(&self) -> Result<&'a str> {
        match self {
            ParameterRef::String32(s) => Ok(*s),
            _ => Err(Error::TypeError(self.type_name(), "String32")),
        }
    }

    /// Get the inner String64 value.
    pub fn as_string64(&self) -> Result<&'a str> {
        match self {
            ParameterRef::String64(s) => Ok(*s),
            _ => Err(Error::TypeError(self.type_name(), "String64")),
        }
    }

    /// Get the inner String256 value.
    pub fn as_string256(&self) -> Result<&'a str> {
        match self {
            ParameterRef::String256(s) => Ok(*s),
            _ => Err(Error::TypeError(self.type_name(), "String256")),
        }
    }

    /// Get the inner StringRef value.
    pub fn as_string_ref(&self) -> Result<&'a str> {
        match self {
            ParameterRef::StringRef(s) => Ok(*s),
            _ => Err(Error::TypeError(self.type_name(), "StringRef")),
        }
    }

    /// Get the value from any string type.
    pub fn as_str(&self) -> Result<&'a str> {
        match self {
            ParameterRef::String32(s)
            | ParameterRef::String64(s)
            | ParameterRef::String256(s)
            | ParameterRef::StringRef(s) => Ok(*s),
            _ => Err(Error::TypeError(self.type_name(), "a string type")),
        }
    }

    /// Get the inner BufferInt value.
    pub fn as_buffer_int(&self) -> Result<&BufferRef<'a>> {
        match self {
            ParameterRef::BufferInt(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "BufferInt")),
        }
    }

    /// Get the inner BufferF32 value.
    pub fn as_buffer_f32(&self) -> Result<&BufferRef<'a>> {
        match self {
            ParameterRef::BufferF32(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "BufferF32")),
        }
    }

    /// Get the inner BufferU32 value.
    pub fn as_buffer_u32(&self) -> Result<&BufferRef<'a>> {
        match self {
            ParameterRef::BufferU32(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "BufferU32")),
        }
    }

    /// Get the inner BufferBinary value.
    pub fn as_buffer_binary(&self) -> Result<&'a [u8]> {
        match self {
            ParameterRef::BufferBinary(v) => Ok(*v),
            _ => Err(Error::TypeError(self.type_name(), "BufferBinary")),
        }
    }

    /// Get the inner Curve1 value.
    pub fn as_curve1(&self) -> Result<&CurveRef<'a>> {
        match self {
            ParameterRef::Curve1(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Curve1")),
        }
    }

    /// Get the inner Curve2 value.
    pub fn as_curve2(&self) -> Result<&CurveRef<'a>> {
        match self {
            ParameterRef::Curve2(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Curve2")),
        }
    }

    /// Get the inner Curve3 value.
    pub fn as_curve3(&self) -> Result<&CurveRef<'a>> {
        match self {
            ParameterRef::Curve3(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Curve3")),
        }
    }

    /// Get the inner Curve4 value.
    pub fn as_curve4(&self) -> Result<&CurveRef<'a>> {
        match self {
            ParameterRef::Curve4(v) => Ok(v),
            _ => Err(Error::TypeError(self.type_name(), "Curve4")),
        }
    }

    /// Converts this borrowed parameter into an owned
    /// [`Parameter`](super::owned::Parameter).
    #[cfg(feature = "alloc")]
    pub fn to_owned(&self) -> super::owned::Parameter {
        match self {
            ParameterRef::Bool(b) => super::owned::Parameter::Bool(*b),
            ParameterRef::F32(f) => super::owned::Parameter::F32(*f),
            ParameterRef::I32(i) => super::owned::Parameter::I32(*i),
            ParameterRef::U32(u) => super::owned::Parameter::U32(*u),
            ParameterRef::Vec2(v) => super::owned::Parameter::Vec2(*v),
            ParameterRef::Vec3(v) => super::owned::Parameter::Vec3(*v),
            ParameterRef::Vec4(v) => super::owned::Parameter::Vec4(*v),
            ParameterRef::Color(c) => super::owned::Parameter::Color(*c),
            ParameterRef::Quat(q) => super::owned::Parameter::Quat(*q),
            ParameterRef::String32(s) => {
                super::owned::Parameter::String32(FixedSafeString::from(*s))
            }
            ParameterRef::String64(s) => {
                super::owned::Parameter::String64(Box::new(FixedSafeString::from(*s)))
            }
            ParameterRef::String256(s) => {
                super::owned::Parameter::String256(Box::new(FixedSafeString::from(*s)))
            }
            ParameterRef::StringRef(s) => {
                super::owned::Parameter::StringRef(smartstring::alias::String::from(*s))
            }
            ParameterRef::Curve1(c) => {
                super::owned::Parameter::Curve1(Box::new([c.curve(0).unwrap_or_default()]))
            }
            ParameterRef::Curve2(c) => super::owned::Parameter::Curve2(Box::new([
                c.curve(0).unwrap_or_default(),
                c.curve(1).unwrap_or_default(),
            ])),
            ParameterRef::Curve3(c) => super::owned::Parameter::Curve3(Box::new([
                c.curve(0).unwrap_or_default(),
                c.curve(1).unwrap_or_default(),
                c.curve(2).unwrap_or_default(),
            ])),
            ParameterRef::Curve4(c) => super::owned::Parameter::Curve4(Box::new([
                c.curve(0).unwrap_or_default(),
                c.curve(1).unwrap_or_default(),
                c.curve(2).unwrap_or_default(),
                c.curve(3).unwrap_or_default(),
            ])),
            ParameterRef::BufferInt(b) => {
                super::owned::Parameter::BufferInt(b.iter_i32().collect())
            }
            ParameterRef::BufferF32(b) => {
                super::owned::Parameter::BufferF32(b.iter_f32().collect())
            }
            ParameterRef::BufferU32(b) => {
                super::owned::Parameter::BufferU32(b.iter_u32().collect())
            }
            ParameterRef::BufferBinary(b) => super::owned::Parameter::BufferBinary(b.to_vec()),
        }
    }
}

impl<'a> PartialEq for ParameterRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::F32(a), Self::F32(b)) => almost::equal(*a, *b),
            (Self::I32(a), Self::I32(b)) => a == b,
            (Self::U32(a), Self::U32(b)) => a == b,
            (Self::Vec2(a), Self::Vec2(b)) => a == b,
            (Self::Vec3(a), Self::Vec3(b)) => a == b,
            (Self::Vec4(a), Self::Vec4(b)) => a == b,
            (Self::Color(a), Self::Color(b)) => a == b,
            (Self::Quat(a), Self::Quat(b)) => a == b,
            (Self::String32(a), Self::String32(b)) => a == b,
            (Self::String64(a), Self::String64(b)) => a == b,
            (Self::String256(a), Self::String256(b)) => a == b,
            (Self::StringRef(a), Self::StringRef(b)) => a == b,
            (Self::Curve1(a), Self::Curve1(b)) => a == b,
            (Self::Curve2(a), Self::Curve2(b)) => a == b,
            (Self::Curve3(a), Self::Curve3(b)) => a == b,
            (Self::Curve4(a), Self::Curve4(b)) => a == b,
            (Self::BufferInt(a), Self::BufferInt(b)) => a == b,
            (Self::BufferF32(a), Self::BufferF32(b)) => a == b,
            (Self::BufferU32(a), Self::BufferU32(b)) => a == b,
            (Self::BufferBinary(a), Self::BufferBinary(b)) => a == b,
            _ => false,
        }
    }
}

/// A parameter list borrowed from the input buffer.
///
/// This requires the `alloc` feature because its containers are owned
/// hash maps; the format walk itself is core-only.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct ParameterListRef<'a> {
    /// Map of child parameter objects.
    pub objects: indexmap::IndexMap<
        Name,
        ParameterObjectRef<'a>,
        core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    >,
    /// Map of child parameter lists.
    pub lists: indexmap::IndexMap<
        Name,
        ParameterListRef<'a>,
        core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    >,
}

/// A parameter object borrowed from the input buffer.
///
/// This requires the `alloc` feature because its container is an owned
/// hash map; the format walk itself is core-only.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct ParameterObjectRef<'a>(
    pub  indexmap::IndexMap<
        Name,
        ParameterRef<'a>,
        core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    >,
);

/// The parameter IO borrowed from the input buffer: the version, data type
/// string and root parameter list.
///
/// This requires the `alloc` feature because its root list owns hash maps;
/// the format walk itself is core-only.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct ParameterIORef<'a> {
    /// Data version (not the AAMP format version). Typically 0.
    pub version: u32,
    /// Data type identifier. Typically “xml”.
    pub data_type: &'a str,
    /// Root parameter list.
    pub param_root: ParameterListRef<'a>,
}

#[cfg(feature = "alloc")]
impl<'a> ParameterListRef<'a> {
    /// Returns a map of parameter lists.
    pub fn lists(
        &self,
    ) -> &indexmap::IndexMap<
        Name,
        ParameterListRef<'a>,
        core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    > {
        &self.lists
    }

    /// Returns a map of parameter objects.
    pub fn objects(
        &self,
    ) -> &indexmap::IndexMap<
        Name,
        ParameterObjectRef<'a>,
        core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    > {
        &self.objects
    }

    /// Get a parameter list by name or hash.
    pub fn list<N: Into<Name>>(&self, name: N) -> Option<&ParameterListRef<'a>> {
        self.lists.get(&name.into())
    }

    /// Get a parameter object by name or hash.
    pub fn object<N: Into<Name>>(&self, name: N) -> Option<&ParameterObjectRef<'a>> {
        self.objects.get(&name.into())
    }

    /// Converts this borrowed list into an owned [`ParameterList`].
    pub fn to_owned(&self) -> super::owned::ParameterList {
        super::owned::ParameterList {
            objects: self
                .objects
                .iter()
                .map(|(name, object)| (*name, object.to_owned()))
                .collect(),
            lists: self
                .lists
                .iter()
                .map(|(name, list)| (*name, list.to_owned()))
                .collect(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> ParameterObjectRef<'a> {
    /// Get a parameter by name or hash.
    pub fn get<N: Into<Name>>(&self, key: N) -> Option<&ParameterRef<'a>> {
        self.0.get(&key.into())
    }

    /// Returns `true` if the map contains a parameter for `key`.
    pub fn contains_key<N: Into<Name>>(&self, key: N) -> bool {
        self.0.contains_key(&key.into())
    }

    /// Returns the number of parameters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the object contains no parameters.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate entries.
    pub fn iter(&self) -> impl Iterator<Item = (&Name, &ParameterRef<'a>)> + '_ {
        self.0.iter()
    }

    /// Converts this borrowed object into an owned [`ParameterObject`].
    pub fn to_owned(&self) -> super::owned::ParameterObject {
        super::owned::ParameterObject(
            self.0
                .iter()
                .map(|(name, parameter)| (*name, parameter.to_owned()))
                .collect(),
        )
    }
}

#[cfg(feature = "alloc")]
impl<'a, N: Into<Name>> core::ops::Index<N> for ParameterObjectRef<'a> {
    type Output = ParameterRef<'a>;

    fn index(&self, name: N) -> &ParameterRef<'a> {
        self.0.get(&name.into()).expect("Index out of bounds")
    }
}

/// Intermediate node kind produced by the walk while building a ref tree.
#[cfg(feature = "alloc")]
enum RefTreeNode<'a> {
    /// A completed parameter list.
    List(ParameterListRef<'a>),
    /// A completed parameter object.
    Object(ParameterObjectRef<'a>),
    /// A parameter leaf.
    Param(ParameterRef<'a>),
}

/// One container under construction in a [`RefSink`].
#[cfg(feature = "alloc")]
enum Frame<'a> {
    List {
        objects: indexmap::IndexMap<
            Name,
            ParameterObjectRef<'a>,
            core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
        >,
        lists: indexmap::IndexMap<
            Name,
            ParameterListRef<'a>,
            core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
        >,
    },
    Object(
        indexmap::IndexMap<
            Name,
            ParameterRef<'a>,
            core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
        >,
    ),
}

/// Sink building a [`ParameterIORef`] tree.
#[cfg(feature = "alloc")]
struct RefSink<'a> {
    stack: Vec<Frame<'a>>,
}

#[cfg(feature = "alloc")]
impl<'a> RefSink<'a> {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }
}

#[cfg(feature = "alloc")]
impl<'a> AampSink<'a> for RefSink<'a> {
    type Node = RefTreeNode<'a>;

    fn leaf(&mut self, parameter: ParameterRef<'a>, _key: Name) -> Result<RefTreeNode<'a>> {
        Ok(RefTreeNode::Param(parameter))
    }

    fn begin_list(&mut self, _name: Name) -> Result<()> {
        self.stack.push(Frame::List {
            objects: indexmap::IndexMap::default(),
            lists: indexmap::IndexMap::default(),
        });
        Ok(())
    }

    fn begin_object(&mut self, _name: Name) -> Result<()> {
        self.stack
            .push(Frame::Object(indexmap::IndexMap::default()));
        Ok(())
    }

    fn add_list(&mut self, node: RefTreeNode<'a>, key: Name) -> Result<()> {
        let frame = self
            .stack
            .last_mut()
            .expect("RefSink::add_list called outside a container");
        match (frame, node) {
            (Frame::List { lists, .. }, RefTreeNode::List(list)) => {
                lists.insert(key, list);
            }
            _ => return Err(Error::InvalidData("AAMP list child is not a list")),
        }
        Ok(())
    }

    fn add_object(&mut self, node: RefTreeNode<'a>, key: Name) -> Result<()> {
        let frame = self
            .stack
            .last_mut()
            .expect("RefSink::add_object called outside a container");
        match (frame, node) {
            (Frame::List { objects, .. }, RefTreeNode::Object(object)) => {
                objects.insert(key, object);
            }
            _ => return Err(Error::InvalidData("AAMP list child is not an object")),
        }
        Ok(())
    }

    fn add_param(&mut self, node: RefTreeNode<'a>, key: Name) -> Result<()> {
        let frame = self
            .stack
            .last_mut()
            .expect("RefSink::add_param called outside a container");
        match (frame, node) {
            (Frame::Object(map), RefTreeNode::Param(parameter)) => {
                map.insert(key, parameter);
            }
            _ => return Err(Error::InvalidData("AAMP object child is not a parameter")),
        }
        Ok(())
    }

    fn end_container(&mut self) -> Result<RefTreeNode<'a>> {
        let frame = self
            .stack
            .pop()
            .expect("RefSink::end_container called outside a container");
        Ok(match frame {
            Frame::List { objects, lists } => {
                RefTreeNode::List(ParameterListRef { objects, lists })
            }
            Frame::Object(map) => RefTreeNode::Object(ParameterObjectRef(map)),
        })
    }
}

#[cfg(feature = "alloc")]
impl<'a> ParameterIORef<'a> {
    /// Returns a map of parameter lists.
    pub fn lists(
        &self,
    ) -> &indexmap::IndexMap<
        Name,
        ParameterListRef<'a>,
        core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    > {
        &self.param_root.lists
    }

    /// Returns a map of parameter objects.
    pub fn objects(
        &self,
    ) -> &indexmap::IndexMap<
        Name,
        ParameterObjectRef<'a>,
        core::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    > {
        &self.param_root.objects
    }

    /// Get a parameter list by name or hash.
    pub fn list<N: Into<Name>>(&self, name: N) -> Option<&ParameterListRef<'a>> {
        self.param_root.lists.get(&name.into())
    }

    /// Get a parameter object by name or hash.
    pub fn object<N: Into<Name>>(&self, name: N) -> Option<&ParameterObjectRef<'a>> {
        self.param_root.objects.get(&name.into())
    }

    /// Read a parameter archive from binary data, borrowing all strings and
    /// payloads from `data`.
    ///
    /// **Note**: Yaz0-compressed input (`"Yaz0"` magic) is not
    /// auto-decompressed here. Decompress first with
    /// [`crate::yaz0::decompress_if`] (feature `yaz0`), which returns a
    /// [`Cow`](alloc::borrow::Cow), and call
    /// `ParameterIORef::from_binary(cow.as_ref())`.
    pub fn from_binary(data: &'a [u8]) -> Result<Self> {
        let mut walk = Walk::new(data)?;
        let version = walk.version();
        let data_type = walk.data_type();
        let mut sink = RefSink::new();
        let param_root = match walk.walk_root(&mut sink)? {
            RefTreeNode::List(list) => list,
            RefTreeNode::Object(_) | RefTreeNode::Param(_) => {
                unreachable!("root node is always a list")
            }
        };
        Ok(ParameterIORef {
            version,
            data_type,
            param_root,
        })
    }

    /// Converts this borrowed parameter IO into an owned [`ParameterIO`].
    pub fn to_owned(&self) -> super::owned::ParameterIO {
        super::owned::ParameterIO {
            version: self.version,
            data_type: self.data_type.into(),
            param_root: self.param_root.to_owned(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a super::owned::ParameterIO> for ParameterIORef<'a> {
    fn from(pio: &'a super::owned::ParameterIO) -> Self {
        ParameterIORef {
            version: pio.version,
            data_type: pio.data_type.as_str(),
            param_root: ParameterListRef::from(&pio.param_root),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a super::owned::ParameterList> for ParameterListRef<'a> {
    fn from(list: &'a super::owned::ParameterList) -> Self {
        ParameterListRef {
            objects: list
                .objects
                .iter()
                .map(|(name, object)| (*name, ParameterObjectRef::from(object)))
                .collect(),
            lists: list
                .lists
                .iter()
                .map(|(name, list)| (*name, ParameterListRef::from(list)))
                .collect(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a super::owned::ParameterObject> for ParameterObjectRef<'a> {
    fn from(object: &'a super::owned::ParameterObject) -> Self {
        ParameterObjectRef(
            object
                .0
                .iter()
                .map(|(name, parameter)| (*name, ParameterRef::from(parameter)))
                .collect(),
        )
    }
}

/// Views an owned parameter over its native storage.
///
/// Buffer and curve payloads are reinterpreted as raw bytes and decoded as
/// little-endian, so this assumes a little-endian host (AAMP is a
/// little-endian-only format).
#[cfg(feature = "alloc")]
impl<'a> From<&'a super::owned::Parameter> for ParameterRef<'a> {
    fn from(parameter: &'a super::owned::Parameter) -> Self {
        match parameter {
            super::owned::Parameter::Bool(b) => ParameterRef::Bool(*b),
            super::owned::Parameter::F32(f) => ParameterRef::F32(*f),
            super::owned::Parameter::I32(i) => ParameterRef::I32(*i),
            super::owned::Parameter::U32(u) => ParameterRef::U32(*u),
            super::owned::Parameter::Vec2(v) => ParameterRef::Vec2(*v),
            super::owned::Parameter::Vec3(v) => ParameterRef::Vec3(*v),
            super::owned::Parameter::Vec4(v) => ParameterRef::Vec4(*v),
            super::owned::Parameter::Color(c) => ParameterRef::Color(*c),
            super::owned::Parameter::Quat(q) => ParameterRef::Quat(*q),
            super::owned::Parameter::String32(s) => ParameterRef::String32(s.as_str()),
            super::owned::Parameter::String64(s) => ParameterRef::String64(s.as_str()),
            super::owned::Parameter::String256(s) => ParameterRef::String256(s.as_str()),
            super::owned::Parameter::StringRef(s) => ParameterRef::StringRef(s.as_str()),
            super::owned::Parameter::Curve1(v) => {
                ParameterRef::Curve1(CurveRef::new(curve_bytes(v)))
            }
            super::owned::Parameter::Curve2(v) => {
                ParameterRef::Curve2(CurveRef::new(curve_bytes(v)))
            }
            super::owned::Parameter::Curve3(v) => {
                ParameterRef::Curve3(CurveRef::new(curve_bytes(v)))
            }
            super::owned::Parameter::Curve4(v) => {
                ParameterRef::Curve4(CurveRef::new(curve_bytes(v)))
            }
            super::owned::Parameter::BufferInt(v) => {
                ParameterRef::BufferInt(BufferRef::new(byte_slice(v.as_slice())))
            }
            super::owned::Parameter::BufferF32(v) => {
                ParameterRef::BufferF32(BufferRef::new(byte_slice(v.as_slice())))
            }
            super::owned::Parameter::BufferU32(v) => {
                ParameterRef::BufferU32(BufferRef::new(byte_slice(v.as_slice())))
            }
            super::owned::Parameter::BufferBinary(v) => ParameterRef::BufferBinary(v.as_slice()),
        }
    }
}

/// Reinterprets `slice` as a byte slice with the same lifetime. The bytes
/// are later decoded as little-endian, so this assumes a little-endian host.
#[cfg(feature = "alloc")]
fn byte_slice<T>(slice: &[T]) -> &[u8] {
    // SAFETY: the slice's elements are initialized and the returned borrow is
    // tied to the input slice's lifetime.
    unsafe {
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            core::mem::size_of_val(slice),
        )
    }
}

/// Reinterprets `curves` (an array of `N` 128-byte `Curve` values) as a byte
/// slice with the same lifetime. `Curve` is assumed to be laid out as
/// `a: u32, b: u32, floats: [f32; 30]`, which matches the on-disk curve
/// block layout on a little-endian host.
#[cfg(feature = "alloc")]
fn curve_bytes<const N: usize>(curves: &[Curve; N]) -> &[u8] {
    // SAFETY: `Curve` is `#[repr(C)]` and 128 bytes with fields `a`, `b`,
    // `floats` laid out contiguously, matching the on-disk 128-byte curve
    // blocks; the returned borrow is tied to the input slice's lifetime.
    unsafe {
        core::slice::from_raw_parts(
            curves.as_ptr() as *const u8,
            core::mem::size_of_val(curves),
        )
    }
}
// Shared format walk
// ---------------------------------------------------------------------------

/// An AAMP document walker over `&'a [u8]`.
///
/// All reads are bounds-checked (reporting [`Error::InsufficientData`]) and
/// integers/floats are decoded from raw bytes (`from_le_bytes`), so no
/// unaligned or endian-dependent references are ever created. Only little
/// endian documents are supported (the header flag requires it).
struct Walk<'a> {
    data: &'a [u8],
    version: u32,
    data_type: &'a str,
    /// Offset of the root list (absolute file offset, i.e. `0x30 + pio_offset`).
    root_offset: u32,
}

impl<'a> Walk<'a> {
    /// Parses and validates the header and locates the data type string and
    /// the root list.
    fn new(data: &'a [u8]) -> Result<Self> {
        if data.len() < 0x30 {
            return Err(Error::InsufficientData(data.len(), 0x30));
        }
        if &data[..4] != b"AAMP" {
            return Err(Error::BadMagic(TinyString::from_bytes(&data[..4]), "AAMP"));
        }
        let format_version = Self::read_u32_at(data, 4)?;
        if format_version != 2 {
            return Err(Error::InvalidData(
                "Only version 2 parameter archives are supported",
            ));
        }
        let version = Self::read_u32_at(data, 0x10)?; // pio_version
        let flags = Self::read_u32_at(data, 8)?;
        if flags & 0x1 == 0 {
            return Err(Error::InvalidData(
                "Only little endian parameter archives are supported",
            ));
        }
        if flags & 0x2 == 0 {
            return Err(Error::InvalidData(
                "Only UTF-8 parameter archives are supported",
            ));
        }
        let pio_offset = Self::read_u32_at(data, 0x14)?;
        let root_offset = 0x30u32
            .checked_add(pio_offset)
            .ok_or(Error::InsufficientData(data.len(), 0x14))?;
        // `data_type` is a NUL-terminated string at file offset 0x30.
        let data_type = Self::read_null_string(data, 0x30, None)?;
        Ok(Self {
            data,
            version,
            data_type,
            root_offset,
        })
    }

    /// The data version from the header.
    fn version(&self) -> u32 {
        self.version
    }

    /// The data type string from file offset 0x30.
    fn data_type(&self) -> &'a str {
        self.data_type
    }

    /// `base + extra`, checked for overflow (offsets are u32 values read from
    /// the document).
    fn off(&self, base: u32, extra: u32) -> Result<u32> {
        base.checked_add(extra)
            .ok_or(Error::InsufficientData(self.data.len(), base as usize))
    }

    /// Read a u32 at `pos`, little endian.
    fn read_u32_at(data: &'a [u8], pos: usize) -> Result<u32> {
        let b = read_bytes_at(data, pos, 4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Read an i32 at `pos`, little endian.
    fn read_i32_at(data: &'a [u8], pos: usize) -> Result<i32> {
        let b = read_bytes_at(data, pos, 4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Read an f32 at `pos`, little endian.
    fn read_f32_at(data: &'a [u8], pos: usize) -> Result<f32> {
        let b = read_bytes_at(data, pos, 4)?;
        Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// Read a NUL-terminated string at `start`. When `max_len` is `Some`, the
    /// content (bytes before the NUL terminator) must not exceed it.
    fn read_null_string(data: &'a [u8], start: usize, max_len: Option<usize>) -> Result<&'a str> {
        let mut len = 0;
        loop {
            let pos = start
                .checked_add(len)
                .ok_or(Error::InsufficientData(data.len(), start))?;
            let byte = read_bytes_at(data, pos, 1)?[0];
            if byte == 0 {
                break;
            }
            len += 1;
            if let Some(max) = max_len && len > max {
                return Err(Error::InvalidData("AAMP string exceeds its maximum length"));
            }
        }
        core::str::from_utf8(&data[start..start + len]).map_err(Into::into)
    }
}

fn read_bytes_at(data: &[u8], pos: usize, len: usize) -> Result<&[u8]> {
    let end = pos
        .checked_add(len)
        .ok_or(Error::InsufficientData(data.len(), pos))?;
    data.get(pos..end)
        .ok_or(Error::InsufficientData(data.len(), end))
}

/// Runs the shared format walk over `data`, reporting into `sink`. Returns
/// the root node produced by the sink (the root list) together with the
/// header's data `version` and `data_type` string.
pub(crate) fn parse_aamp<'a, Sink: AampSink<'a>>(
    data: &'a [u8],
    sink: &mut Sink,
) -> Result<(Sink::Node, u32, &'a str)> {
    let mut walk = Walk::new(data)?;
    let version = walk.version();
    let data_type = walk.data_type();
    let node = walk.walk_root(sink)?;
    Ok((node, version, data_type))
}

/// Sink for the shared AAMP format walk.
///
/// The walk calls [`begin_list`](AampSink::begin_list) /
/// [`begin_object`](AampSink::begin_object) before descending into a
/// container, [`add_list`](AampSink::add_list) /
/// [`add_object`](AampSink::add_object) / [`add_param`](AampSink::add_param)
/// for every child (in document order), and
/// [`end_container`](AampSink::end_container) when the container is
/// complete. Leaves are reported with [`leaf`](AampSink::leaf).
pub(crate) trait AampSink<'a> {
    /// The node type produced by this sink (e.g. [`NodeId`](crate::storage::NodeId)).
    type Node;

    /// Report a leaf parameter value.
    fn leaf(&mut self, parameter: ParameterRef<'a>, key: Name) -> Result<Self::Node>;

    /// Begin a list container.
    fn begin_list(&mut self, name: Name) -> Result<()>;

    /// Begin an object container.
    fn begin_object(&mut self, name: Name) -> Result<()>;

    /// Add a completed child list to the innermost open container.
    fn add_list(&mut self, node: Self::Node, key: Name) -> Result<()>;

    /// Add a completed child object to the innermost open container.
    fn add_object(&mut self, node: Self::Node, key: Name) -> Result<()>;

    /// Add a completed parameter leaf to the innermost open object.
    fn add_param(&mut self, node: Self::Node, key: Name) -> Result<()>;

    /// End the innermost container and return its node.
    fn end_container(&mut self) -> Result<Self::Node>;
}

impl<'a> Walk<'a> {
    /// Walks the root list at `root_offset`. Its name must equal [`ROOT_KEY`].
    fn walk_root<Sink: AampSink<'a>>(&mut self, sink: &mut Sink) -> Result<Sink::Node> {
        let (name, node) = self.walk_list(self.root_offset, sink, 1)?;
        if name != ROOT_KEY {
            return Err(Error::InvalidData(
                "No param root found in parameter archive",
            ));
        }
        Ok(node)
    }

    /// Walks a list container at `offset`.
    ///
    /// Child list `i` is at `offset + lists_rel * 4 + 12 * i`; child object
    /// `i` is at `offset + objects_rel * 4 + 8 * i`.
    fn walk_list<Sink: AampSink<'a>>(
        &mut self,
        offset: u32,
        sink: &mut Sink,
        depth: usize,
    ) -> Result<(Name, Sink::Node)> {
        if depth > AAMP_MAX_DEPTH {
            return Err(Error::InvalidData(
                "AAMP container nesting exceeds maximum depth",
            ));
        }
        let entry = read_bytes_at(self.data, offset as usize, 12)?;
        let name = Name(u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]));
        let lists_rel = u32::from_le_bytes([entry[4], entry[5], 0, 0]);
        let list_count = u32::from_le_bytes([entry[6], entry[7], 0, 0]);
        let objects_rel = u32::from_le_bytes([entry[8], entry[9], 0, 0]);
        let object_count = u32::from_le_bytes([entry[10], entry[11], 0, 0]);

        sink.begin_list(name)?;
        let lists_base = self.off(offset, lists_rel * 4)?;
        for i in 0..list_count {
            let child_offset = self.off(lists_base, 12 * i)?;
            let (child_name, child) = self.walk_list(child_offset, sink, depth + 1)?;
            sink.add_list(child, child_name)?;
        }
        let objects_base = self.off(offset, objects_rel * 4)?;
        for i in 0..object_count {
            let child_offset = self.off(objects_base, 8 * i)?;
            let (child_name, child) = self.walk_object(child_offset, sink, depth + 1)?;
            sink.add_object(child, child_name)?;
        }
        let node = sink.end_container()?;
        Ok((name, node))
    }

    /// Walks an object container at `offset`.
    ///
    /// Parameter `i` is at `offset + params_rel * 4 + 8 * i`.
    fn walk_object<Sink: AampSink<'a>>(
        &mut self,
        offset: u32,
        sink: &mut Sink,
        depth: usize,
    ) -> Result<(Name, Sink::Node)> {
        if depth > AAMP_MAX_DEPTH {
            return Err(Error::InvalidData(
                "AAMP container nesting exceeds maximum depth",
            ));
        }
        let entry = read_bytes_at(self.data, offset as usize, 8)?;
        let name = Name(u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]));
        let params_rel = u32::from_le_bytes([entry[4], entry[5], 0, 0]);
        let param_count = u32::from_le_bytes([entry[6], entry[7], 0, 0]);

        sink.begin_object(name)?;
        let params_base = self.off(offset, params_rel * 4)?;
        for i in 0..param_count {
            let param_offset = self.off(params_base, 8 * i)?;
            let (param_name, parameter) = self.walk_parameter(param_offset)?;
            let node = sink.leaf(parameter, param_name)?;
            sink.add_param(node, param_name)?;
        }
        let node = sink.end_container()?;
        Ok((name, node))
    }

    /// Walks a parameter entry at `offset`.
    ///
    /// `data_offset = offset + data_rel_offset * 4`; for buffer types the
    /// element count is stored at `data_offset - 4`.
    fn walk_parameter(&mut self, offset: u32) -> Result<(Name, ParameterRef<'a>)> {
        let entry = read_bytes_at(self.data, offset as usize, 8)?;
        let name = Name(u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]));
        let data_rel = u32::from_le_bytes([entry[4], entry[5], entry[6], 0]);
        let type_tag = entry[7];
        let ty = match type_tag {
            0 => Type::Bool,
            1 => Type::F32,
            2 => Type::Int,
            3 => Type::Vec2,
            4 => Type::Vec3,
            5 => Type::Vec4,
            6 => Type::Color,
            7 => Type::String32,
            8 => Type::String64,
            9 => Type::Curve1,
            10 => Type::Curve2,
            11 => Type::Curve3,
            12 => Type::Curve4,
            13 => Type::BufferInt,
            14 => Type::BufferF32,
            15 => Type::String256,
            16 => Type::Quat,
            17 => Type::U32,
            18 => Type::BufferU32,
            19 => Type::BufferBinary,
            20 => Type::StringRef,
            _ => {
                return Err(Error::InvalidData("Invalid AAMP parameter type"));
            }
        };
        let data_offset = offset
            .checked_add(
                data_rel
                    .checked_mul(4)
                    .ok_or(Error::InvalidData("AAMP data relative offset overflows"))?,
            )
            .ok_or(Error::InsufficientData(self.data.len(), offset as usize))?;
        let value = match ty {
            Type::Bool => {
                ParameterRef::Bool(Self::read_u32_at(self.data, data_offset as usize)? != 0)
            }
            Type::F32 => ParameterRef::F32(Self::read_f32_at(self.data, data_offset as usize)?),
            Type::Int => ParameterRef::I32(Self::read_i32_at(self.data, data_offset as usize)?),
            Type::U32 => ParameterRef::U32(Self::read_u32_at(self.data, data_offset as usize)?),
            Type::Vec2 => ParameterRef::Vec2(Vector2f {
                x: Self::read_f32_at(self.data, data_offset as usize)?,
                y: Self::read_f32_at(self.data, self.off(data_offset, 4)? as usize)?,
            }),
            Type::Vec3 => ParameterRef::Vec3(Vector3f {
                x: Self::read_f32_at(self.data, data_offset as usize)?,
                y: Self::read_f32_at(self.data, self.off(data_offset, 4)? as usize)?,
                z: Self::read_f32_at(self.data, self.off(data_offset, 8)? as usize)?,
            }),
            Type::Vec4 => ParameterRef::Vec4(Vector4f {
                x: Self::read_f32_at(self.data, data_offset as usize)?,
                y: Self::read_f32_at(self.data, self.off(data_offset, 4)? as usize)?,
                z: Self::read_f32_at(self.data, self.off(data_offset, 8)? as usize)?,
                t: Self::read_f32_at(self.data, self.off(data_offset, 12)? as usize)?,
            }),
            Type::Color => ParameterRef::Color(Color {
                r: Self::read_f32_at(self.data, data_offset as usize)?,
                g: Self::read_f32_at(self.data, self.off(data_offset, 4)? as usize)?,
                b: Self::read_f32_at(self.data, self.off(data_offset, 8)? as usize)?,
                a: Self::read_f32_at(self.data, self.off(data_offset, 12)? as usize)?,
            }),
            Type::Quat => ParameterRef::Quat(Quat {
                a: Self::read_f32_at(self.data, data_offset as usize)?,
                b: Self::read_f32_at(self.data, self.off(data_offset, 4)? as usize)?,
                c: Self::read_f32_at(self.data, self.off(data_offset, 8)? as usize)?,
                d: Self::read_f32_at(self.data, self.off(data_offset, 12)? as usize)?,
            }),
            Type::String32 => ParameterRef::String32(Self::read_null_string(
                self.data,
                data_offset as usize,
                Some(32),
            )?),
            Type::String64 => ParameterRef::String64(Self::read_null_string(
                self.data,
                data_offset as usize,
                Some(64),
            )?),
            Type::String256 => ParameterRef::String256(Self::read_null_string(
                self.data,
                data_offset as usize,
                Some(256),
            )?),
            Type::StringRef => ParameterRef::StringRef(Self::read_null_string(
                self.data,
                data_offset as usize,
                None,
            )?),
            Type::Curve1 => ParameterRef::Curve1(CurveRef::new(read_bytes_at(
                self.data,
                data_offset as usize,
                128,
            )?)),
            Type::Curve2 => ParameterRef::Curve2(CurveRef::new(read_bytes_at(
                self.data,
                data_offset as usize,
                2 * 128,
            )?)),
            Type::Curve3 => ParameterRef::Curve3(CurveRef::new(read_bytes_at(
                self.data,
                data_offset as usize,
                3 * 128,
            )?)),
            Type::Curve4 => ParameterRef::Curve4(CurveRef::new(read_bytes_at(
                self.data,
                data_offset as usize,
                4 * 128,
            )?)),
            Type::BufferInt => {
                let count = self.buffer_count(data_offset)?;
                let bytes = read_bytes_at(self.data, data_offset as usize, count as usize * 4)?;
                ParameterRef::BufferInt(BufferRef::new(bytes))
            }
            Type::BufferF32 => {
                let count = self.buffer_count(data_offset)?;
                let bytes = read_bytes_at(self.data, data_offset as usize, count as usize * 4)?;
                ParameterRef::BufferF32(BufferRef::new(bytes))
            }
            Type::BufferU32 => {
                let count = self.buffer_count(data_offset)?;
                let bytes = read_bytes_at(self.data, data_offset as usize, count as usize * 4)?;
                ParameterRef::BufferU32(BufferRef::new(bytes))
            }
            Type::BufferBinary => {
                let count = self.buffer_count(data_offset)?;
                let bytes = read_bytes_at(self.data, data_offset as usize, count as usize)?;
                ParameterRef::BufferBinary(bytes)
            }
        };
        Ok((name, value))
    }

    /// Reads the element count stored 4 bytes before `data_offset`.
    fn buffer_count(&self, data_offset: u32) -> Result<u32> {
        let count_offset = data_offset
            .checked_sub(4)
            .ok_or(Error::InsufficientData(data_offset as usize, 4))?;
        Self::read_u32_at(self.data, count_offset as usize)
    }
}
