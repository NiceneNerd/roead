#![allow(clippy::derived_hash_with_manual_eq)]
//! Miscellaneous needful oead types.
// use decorum::f32;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

/// A string class with its own inline, fixed-size storage.
///
/// In sead, this is actually a derived class of `sead::BufferedSafeString`
/// which is in turn derived from `sead::SafeString`. Since the latter is
/// essentially a `{vptr, const char* cstr}` pair and the former is a
/// `std::string_view`, we will not bother implementing those base classes.
///
/// **Note:** Any string that is too long to be stored in a `FixedSafeString`
/// is truncated.
#[cfg_attr(
    feature = "with-serde",
    derive(Serialize, Deserialize),
    serde(from = "std::string::String", into = "std::string::String")
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct FixedSafeString<const N: usize> {
    data: [u8; N],
    len:  usize,
}

impl<const N: usize> Default for FixedSafeString<N> {
    fn default() -> Self {
        Self {
            data: [0; N],
            len:  0,
        }
    }
}

impl<const N: usize> std::fmt::Debug for FixedSafeString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<const N: usize> FixedSafeString<N> {
    /// Extracts a string slice from the owned string.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl<const N: usize> std::ops::Deref for FixedSafeString<N> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl<const N: usize> std::ops::DerefMut for FixedSafeString<N> {
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut()
    }
}

impl<const N: usize> AsRef<str> for FixedSafeString<N> {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.data[..self.len]) }
    }
}

impl<const N: usize> AsMut<str> for FixedSafeString<N> {
    fn as_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.data[..self.len]) }
    }
}

impl<const N: usize> From<&str> for FixedSafeString<N> {
    fn from(s: &str) -> Self {
        let mut data = [0; N];
        let len = std::cmp::min(N, s.len());
        data[..len].copy_from_slice(&s.as_bytes()[..len]);
        Self { data, len }
    }
}

impl<const N: usize> From<FixedSafeString<N>> for String {
    fn from(s: FixedSafeString<N>) -> Self {
        s.as_ref().to_owned()
    }
}

impl<const N: usize> From<String> for FixedSafeString<N> {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl<const N: usize> std::borrow::Borrow<str> for FixedSafeString<N> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl<const N: usize> From<smartstring::alias::String> for FixedSafeString<N> {
    fn from(s: smartstring::alias::String) -> Self {
        s.as_str().into()
    }
}

impl<const N: usize> From<FixedSafeString<N>> for smartstring::alias::String {
    fn from(s: FixedSafeString<N>) -> Self {
        s.as_ref().into()
    }
}

impl<const N: usize> std::fmt::Display for FixedSafeString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

#[cfg(feature = "binrw")]
impl<const N: usize> binrw::BinRead for FixedSafeString<N> {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _: binrw::Endian,
        _: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let mut data = [0; N];
        let mut c = u8::read(reader)?;
        let mut len = 0;
        while c != 0 && len < N {
            data[len] = c;
            len += 1;
            c = u8::read(reader)?;
        }
        Ok(Self { data, len })
    }
}

/// 2D vector.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "binrw", binrw::binrw)]
pub struct Vector2f {
    pub x: f32,
    pub y: f32,
}

#[cfg(feature = "almost")]
impl PartialEq for Vector2f {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(self.x, other.x) && almost::equal(self.y, other.y)
    }
}

impl std::hash::Hash for Vector2f {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.x.to_bits()).hash(state);
        (self.y.to_bits()).hash(state);
    }
}

/// 3D vector.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "binrw", binrw::binrw)]
pub struct Vector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[cfg(feature = "almost")]
impl PartialEq for Vector3f {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(self.x, other.x)
            && almost::equal(self.y, other.y)
            && almost::equal(self.z, other.z)
    }
}

impl std::hash::Hash for Vector3f {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        b"f".hash(state);
        (self.x.to_bits()).hash(state);
        b"f".hash(state);
        (self.y.to_bits()).hash(state);
        b"f".hash(state);
        (self.z.to_bits()).hash(state);
    }
}

/// 4D vector.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "binrw", binrw::binrw)]
pub struct Vector4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub t: f32,
}

#[cfg(feature = "almost")]
impl PartialEq for Vector4f {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(self.x, other.x)
            && almost::equal(self.y, other.y)
            && almost::equal(self.z, other.z)
            && almost::equal(self.t, other.t)
    }
}

impl std::hash::Hash for Vector4f {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        b"f".hash(state);
        (self.x.to_bits()).hash(state);
        b"f".hash(state);
        (self.y.to_bits()).hash(state);
        b"f".hash(state);
        (self.z.to_bits()).hash(state);
        b"f".hash(state);
        (self.t.to_bits()).hash(state);
    }
}

/// Quaternion.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "binrw", binrw::binrw)]
pub struct Quat {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
}

#[cfg(feature = "almost")]
impl PartialEq for Quat {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(self.a, other.a)
            && almost::equal(self.b, other.b)
            && almost::equal(self.c, other.c)
            && almost::equal(self.d, other.d)
    }
}

impl std::hash::Hash for Quat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        b"f".hash(state);
        (self.a.to_bits()).hash(state);
        b"f".hash(state);
        (self.b.to_bits()).hash(state);
        b"f".hash(state);
        (self.c.to_bits()).hash(state);
        b"f".hash(state);
        (self.d.to_bits()).hash(state);
    }
}

/// RGBA color (Red/Green/Blue/Alpha).
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "binrw", binrw::binrw)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[cfg(feature = "almost")]
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(self.r, other.r)
            && almost::equal(self.g, other.g)
            && almost::equal(self.b, other.b)
            && almost::equal(self.a, other.a)
    }
}

impl std::hash::Hash for Color {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        b"f".hash(state);
        (self.r.to_bits()).hash(state);
        b"f".hash(state);
        (self.g.to_bits()).hash(state);
        b"f".hash(state);
        (self.b.to_bits()).hash(state);
        b"f".hash(state);
        (self.a.to_bits()).hash(state);
    }
}

/// Curve (`sead::hostio::curve*`)
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "binrw", binrw::binrw)]
pub struct Curve {
    pub a: u32,
    pub b: u32,
    pub floats: [f32; 30],
}

#[cfg(feature = "almost")]
impl PartialEq for Curve {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a
            && self.b == other.b
            && self
                .floats
                .iter()
                .zip(other.floats.iter())
                .all(|(a, b)| almost::equal(*a, *b))
    }
}

impl std::hash::Hash for Curve {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.a.hash(state);
        self.b.hash(state);
        for f in &self.floats {
            b"f".hash(state);
            (f.to_bits()).hash(state);
        }
    }
}
