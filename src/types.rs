#![allow(clippy::derive_hash_xor_eq)]
//! Miscellaneous needful oead types.
use decorum::R32;
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
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FixedSafeString<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> std::fmt::Debug for FixedSafeString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<const N: usize> FixedSafeString<N> {
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
        (&mut data[..len]).copy_from_slice(&s.as_bytes()[..len]);
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

#[cfg(feature = "smartstring")]
impl<const N: usize> From<smartstring::alias::String> for FixedSafeString<N> {
    fn from(s: smartstring::alias::String) -> Self {
        s.as_str().into()
    }
}

#[cfg(feature = "smartstring")]
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
    type Args = ();
    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        _: &binrw::ReadOptions,
        _: Self::Args,
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
#[derive(Debug, Clone, Copy, Eq, Hash)]
pub struct Vector2f {
    pub x: R32,
    pub y: R32,
}

#[cfg(feature = "almost")]
impl PartialEq for Vector2f {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(*self.x.as_ref(), *other.x.as_ref())
            && almost::equal(*self.y.as_ref(), *other.y.as_ref())
    }
}

#[cfg(feature = "binrw")]
const _: () = {
    impl binrw::BinRead for Vector2f {
        type Args = ();
        fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            opts: &binrw::ReadOptions,
            _: Self::Args,
        ) -> binrw::BinResult<Self> {
            Ok(Self {
                x: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                y: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
            })
        }
    }

    impl binrw::BinWrite for Vector2f {
        type Args = ();
        fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            opts: &binrw::WriteOptions,
            _: Self::Args,
        ) -> binrw::BinResult<()> {
            self.x
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.y
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            Ok(())
        }
    }
};

/// 3D vector.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, Hash)]
pub struct Vector3f {
    pub x: R32,
    pub y: R32,
    pub z: R32,
}

#[cfg(feature = "almost")]
impl PartialEq for Vector3f {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(*self.x.as_ref(), *other.x.as_ref())
            && almost::equal(*self.y.as_ref(), *other.y.as_ref())
            && almost::equal(*self.z.as_ref(), *other.z.as_ref())
    }
}

#[cfg(feature = "binrw")]
const _: () = {
    impl binrw::BinRead for Vector3f {
        type Args = ();
        fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            opts: &binrw::ReadOptions,
            _: Self::Args,
        ) -> binrw::BinResult<Self> {
            Ok(Self {
                x: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                y: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                z: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
            })
        }
    }

    impl binrw::BinWrite for Vector3f {
        type Args = ();
        fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            opts: &binrw::WriteOptions,
            _: Self::Args,
        ) -> binrw::BinResult<()> {
            self.x
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.y
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.z
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            Ok(())
        }
    }
};

/// 4D vector.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, Hash)]
pub struct Vector4f {
    pub x: R32,
    pub y: R32,
    pub z: R32,
    pub t: R32,
}

#[cfg(feature = "almost")]
impl PartialEq for Vector4f {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(*self.x.as_ref(), *other.x.as_ref())
            && almost::equal(*self.y.as_ref(), *other.y.as_ref())
            && almost::equal(*self.z.as_ref(), *other.z.as_ref())
            && almost::equal(*self.t.as_ref(), *other.t.as_ref())
    }
}

#[cfg(feature = "binrw")]
const _: () = {
    impl binrw::BinRead for Vector4f {
        type Args = ();
        fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            opts: &binrw::ReadOptions,
            _: Self::Args,
        ) -> binrw::BinResult<Self> {
            Ok(Self {
                x: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                y: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                z: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                t: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
            })
        }
    }

    impl binrw::BinWrite for Vector4f {
        type Args = ();
        fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            opts: &binrw::WriteOptions,
            _: Self::Args,
        ) -> binrw::BinResult<()> {
            self.x
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.y
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.z
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.t
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            Ok(())
        }
    }
};

/// Quaternion.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, Hash)]
pub struct Quat {
    pub a: R32,
    pub b: R32,
    pub c: R32,
    pub d: R32,
}

#[cfg(feature = "almost")]
impl PartialEq for Quat {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(*self.a.as_ref(), *other.a.as_ref())
            && almost::equal(*self.b.as_ref(), *other.b.as_ref())
            && almost::equal(*self.c.as_ref(), *other.c.as_ref())
            && almost::equal(*self.d.as_ref(), *other.d.as_ref())
    }
}

#[cfg(feature = "binrw")]
const _: () = {
    impl binrw::BinRead for Quat {
        type Args = ();
        fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            opts: &binrw::ReadOptions,
            _: Self::Args,
        ) -> binrw::BinResult<Self> {
            Ok(Self {
                a: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                b: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                c: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                d: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
            })
        }
    }

    impl binrw::BinWrite for Quat {
        type Args = ();
        fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            opts: &binrw::WriteOptions,
            _: Self::Args,
        ) -> binrw::BinResult<()> {
            self.a
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.b
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.c
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.d
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            Ok(())
        }
    }
};

/// RGBA color (Red/Green/Blue/Alpha).
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, Hash)]
pub struct Color {
    pub r: R32,
    pub g: R32,
    pub b: R32,
    pub a: R32,
}

#[cfg(feature = "almost")]
impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        almost::equal(*self.r.as_ref(), *other.r.as_ref())
            && almost::equal(*self.g.as_ref(), *other.g.as_ref())
            && almost::equal(*self.b.as_ref(), *other.b.as_ref())
            && almost::equal(*self.a.as_ref(), *other.a.as_ref())
    }
}

#[cfg(feature = "binrw")]
const _: () = {
    impl binrw::BinRead for Color {
        type Args = ();
        fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            opts: &binrw::ReadOptions,
            _: Self::Args,
        ) -> binrw::BinResult<Self> {
            Ok(Self {
                r: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                g: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                b: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
                a: f32::from_le_bytes(<[u8; 4]>::read_options(reader, opts, ())?).into(),
            })
        }
    }

    impl binrw::BinWrite for Color {
        type Args = ();
        fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            opts: &binrw::WriteOptions,
            _: Self::Args,
        ) -> binrw::BinResult<()> {
            self.r
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.g
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.b
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            self.a
                .as_ref()
                .to_le_bytes()
                .write_options(writer, opts, ())?;
            Ok(())
        }
    }
};

/// Curve (`sead::hostio::curve*`)
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, Copy, Eq, Hash)]
pub struct Curve {
    pub a: u32,
    pub b: u32,
    pub floats: [R32; 30],
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
                .all(|(a, b)| almost::equal(*a.as_ref(), *b.as_ref()))
    }
}

#[cfg(feature = "binrw")]
const _: () = {
    impl binrw::BinRead for Curve {
        type Args = ();
        fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            options: &binrw::ReadOptions,
            _: Self::Args,
        ) -> binrw::BinResult<Self> {
            let mut curve = Self {
                a: u32::read_options(reader, options, ())?,
                b: u32::read_options(reader, options, ())?,
                floats: Default::default(),
            };
            for i in 0..30 {
                curve.floats[i] = f32::read_options(reader, options, ())?.into();
            }
            Ok(curve)
        }
    }

    impl binrw::BinWrite for Curve {
        type Args = ();
        fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            options: &binrw::WriteOptions,
            _: Self::Args,
        ) -> binrw::BinResult<()> {
            self.a.write_options(writer, options, ())?;
            self.b.write_options(writer, options, ())?;
            for i in 0..30 {
                (*self.floats[i].as_ref() as u32).write_options(writer, options, ())?;
            }
            Ok(())
        }
    }
};
