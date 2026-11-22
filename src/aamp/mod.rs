//! Port of the `oead::aamp` module.
//!
//! Only version 2, little endian and UTF-8 binary parameter archives are
//! supported. All parameter types including buffers are supported.
//! The YAML output is compatible with the pure Python aamp library.
//!
//! The module contains three layers:
//! - The owned tree model (`owned`): [`ParameterIO`], [`ParameterList`],
//!   [`ParameterObject`] and [`Parameter`] require `alloc` and are produced
//!   by the binrw-based `parser` and `writer`.
//! - `reader` contains the shared AAMP format walk. It is core-only so
//!   that it can be used without an allocator.
//! - [`arena`] is core-only and provides flat, fixed-capacity AAMP readers
//!   ([`ParameterIOArena`]) that borrow all payloads from the input buffer
//!   and work with zero features.
//!
//! With `std` + `yaml`, the YAML layer (`text` for the owned model,
//! `ref_text` for the borrowed readers) emits byte-identical YAML from
//! either representation: `ParameterIO::to_text`, `ParameterIORef::to_text`
//! and `ParameterIOArena::to_text`.
//!
//! The main type is the [`ParameterIO`], which will usually be constructed
//! from binary data of a YAML document. Some sample usage:
//!
//! Without `alloc`, the core-only [`ParameterIOArena`] works with zero
//! features:
//! ```rust
//! # use roead::{Error, storage::FixedStore};
//! # use roead::aamp::{AampNode, Name, hash_name, ParameterIOArena};
//! # fn doctest() -> Result<(), Error> {
//! // A minimal valid AAMP archive: header, "xml" data type, empty param_root.
//! let mut data = [0u8; 0x40];
//! data[0..4].copy_from_slice(b"AAMP");
//! data[4..8].copy_from_slice(&2u32.to_le_bytes());   // version
//! data[8..12].copy_from_slice(&3u32.to_le_bytes());  // flags: LE + UTF-8
//! data[24..28].copy_from_slice(&4u32.to_le_bytes()); // pio_offset: root at 0x34
//! data[48..52].copy_from_slice(b"xml\0");            // data_type at 0x30
//! data[52..56].copy_from_slice(&hash_name("param_root").to_le_bytes());
//! let arena = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&data)?;
//! let root = arena.root().expect("root node");
//! assert_eq!(arena.node(root).unwrap().name, Name(hash_name("param_root")));
//! # Ok(())
//! # }
//! ```
//!
//! All parameter map structures ([`ParameterObject`], [`ParameterObjectMap`],
//! [`ParameterListMap`]) can take either a name or a hash for key-based
//! operations, and likewise can be indexed by the same. As usual, indexing into
//! a non-existent key will panic.

pub mod arena;
#[cfg(feature = "aamp-names")]
mod names;
#[cfg(feature = "alloc")]
mod owned;
#[cfg(feature = "alloc")]
mod parser;
mod reader;
#[cfg(feature = "yaml")]
mod ref_text;
#[cfg(feature = "yaml")]
mod text;
#[cfg(feature = "alloc")]
mod writer;

use crate::types::FixedSafeString;

#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "binrw", binrw::binrw, brw(repr = u8))]
#[repr(u8)]
pub(crate) enum Type {
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

/// Parameter structure name. This is a wrapper around a CRC32 hash.
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "binrw", binrw::binrw, brw(little))]
pub struct Name(pub u32);

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

impl<const N: usize> From<FixedSafeString<N>> for Name {
    fn from(f: FixedSafeString<N>) -> Self {
        Name(hash_name(f.as_str()))
    }
}

impl core::borrow::Borrow<u32> for Name {
    fn borrow(&self) -> &u32 {
        &self.0
    }
}

impl AsRef<u32> for Name {
    fn as_ref(&self) -> &u32 {
        &self.0
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

#[cfg(feature = "alloc")]
impl From<alloc::string::String> for Name {
    fn from(s: alloc::string::String) -> Self {
        Name(hash_name(&s))
    }
}

#[cfg(feature = "alloc")]
impl From<&alloc::string::String> for Name {
    fn from(s: &alloc::string::String) -> Self {
        Name(hash_name(s))
    }
}

#[cfg(feature = "aamp-names")]
impl core::fmt::Display for Name {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match names::get_default_name_table().get_name(self.0, 0, 0) {
            Some(name) => name.fmt(f),
            None => self.0.fmt(f),
        }
    }
}

/// Key of the root parameter list: the CRC32 hash of `"param_root"`.
const ROOT_KEY: Name = Name::from_str("param_root");

// The owned tree model is `alloc`-backed.
#[cfg(feature = "alloc")]
pub use owned::{
    Parameter, ParameterIO, ParameterList, ParameterListMap, ParameterListing, ParameterObject,
    ParameterObjectMap, lists, objs, params,
};

// The borrowed tree view. `BufferRef`, `CurveRef` and `ParameterRef` are
// core-only; the ref tree types require `alloc`.
pub use reader::{BufferRef, CurveRef, ParameterRef};
#[cfg(feature = "alloc")]
pub use reader::{ParameterIORef, ParameterListRef, ParameterObjectRef};

// The flat arena reader is core-only.
pub use arena::{AampNode, AampPayload, NodeId, ParameterIOArena, ParameterIOStack};

#[cfg(feature = "aamp-names")]
pub use names::{NameTable, get_default_name_table};
