//! Shared BYML format walk and the [`BymlRef`] borrowed tree view.
//!
//! The walk in this module is core-only: it borrows all payloads
//! (`&str`, `&[u8]`) from the input buffer and never allocates. It is
//! generic over the [`BymlSink`] trait, so the same format logic drives
//! both the alloc-backed [`BymlRef`] tree and the core-only
//! [`crate::byml::arena::BymlArena`] flat arena.
//!
//! `BymlRef` itself requires the `alloc` feature (its containers own
//! `Vec`s and hash maps); the walk does not.

use crate::{Endian, Error, Result, TinyString};
use super::arena::Key;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use smartstring::alias::String;

#[cfg(feature = "alloc")]
use super::{Byml, BymlIndex};

/// Maximum depth of nested containers the walk will descend into.
///
/// Bounds recursion on malformed or cyclic input: a container offset that
/// points at an ancestor loops until this cap trips (the arena trips its
/// capacity error first).
pub(crate) const BYML_MAX_DEPTH: usize = 128;

/// BYML node type tags (the low nibble distinguishes value nodes from
/// container/string-table nodes; see `NodeType` in `owned.rs`).
///
/// The walk defines its own byte-tag constants so that it stays
/// independent of the binrw derives used by the owned parser.
const TAG_STRING_TABLE: u8 = 0xC2;
const TAG_ARRAY: u8 = 0xC0;
const TAG_MAP: u8 = 0xC1;
const TAG_HASH_MAP: u8 = 0x20;
const TAG_VALUE_HASH_MAP: u8 = 0x21;
const TAG_STRING: u8 = 0xA0;
const TAG_BINARY: u8 = 0xA1;
const TAG_FILE: u8 = 0xA2;
const TAG_BOOL: u8 = 0xD0;
const TAG_I32: u8 = 0xD1;
const TAG_FLOAT: u8 = 0xD2;
const TAG_U32: u8 = 0xD3;
const TAG_I64: u8 = 0xD4;
const TAG_U64: u8 = 0xD5;
const TAG_DOUBLE: u8 = 0xD6;
const TAG_NULL: u8 = 0xFF;

const fn is_container_tag(node_type: u8) -> bool {
    matches!(
        node_type,
        TAG_ARRAY | TAG_MAP | TAG_HASH_MAP | TAG_VALUE_HASH_MAP
    )
}

/// Round `value` up to a multiple of `size` (size must be non-zero).
#[inline(always)]
const fn align(value: u32, size: u32) -> u32 {
    value + (size - value % size) % size
}

/// A leaf (non-container) payload borrowed from the input buffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Leaf<'a> {
    String(&'a str),
    BinaryData(&'a [u8]),
    FileData(&'a [u8]),
    Bool(bool),
    I32(i32),
    Float(f32),
    U32(u32),
    I64(i64),
    U64(u64),
    Double(f64),
    Null,
}

/// The kind of a container node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContainerKind {
    Array,
    Map,
    HashMap,
    ValueHashMap,
}

/// Sink for the shared BYML format walk.
///
/// The walk calls [`begin_container`](BymlSink::begin_container) before
/// descending into a container, [`add_child`](BymlSink::add_child) for
/// every child (in document order), and
/// [`end_container`](BymlSink::end_container) when the container is
/// complete. Leaves are reported with [`leaf`](BymlSink::leaf).
pub(crate) trait BymlSink<'a> {
    /// The node type produced by this sink (e.g. [`BymlRef`] or [`NodeId`](crate::byml::arena::NodeId)).
    type Node;

    /// Report a leaf node. `key` and `aux` are the key and auxiliary value
    /// under which the leaf is stored in its parent (for [`ValueHashMap`]
    /// entries `aux` is the entry's u32 value; otherwise `0`).
    fn leaf(&mut self, payload: Leaf<'a>, key: Key<'a>, aux: u32) -> Result<Self::Node>;

    /// Begin a container node. Children are reported with `add_child`
    /// until [`end_container`](BymlSink::end_container) completes it.
    fn begin_container(&mut self, kind: ContainerKind, key: Key<'a>, aux: u32) -> Result<()>;

    /// Add a completed child node to the innermost open container.
    fn add_child(&mut self, node: Self::Node, key: Key<'a>, aux: u32) -> Result<()>;

    /// End the innermost container and return its node.
    fn end_container(&mut self) -> Result<Self::Node>;
}

/// A BYML document walker over `&'a [u8]`.
///
/// All reads are bounds-checked (reporting [`Error::InsufficientData`])
/// and integers/floats are decoded from raw bytes (`from_le_bytes` /
/// `from_be_bytes`), so no unaligned or endian-dependent references are
/// ever created. Both `BY` (big endian) and `YB` (little endian) headers
/// are supported.
struct Walk<'a> {
    data: &'a [u8],
    endian: Endian,
    hash_key_table_offset: u32,
    string_table_offset: u32,
    root_node_offset: u32,
    hash_key_table_size: u32,
    string_table_size: u32,
}

impl<'a> Walk<'a> {
    /// Parse the header and locate the string / hash key tables.
    fn new(data: &'a [u8]) -> Result<Self> {
        if data.len() < 0x10 {
            return Err(Error::InsufficientData(data.len(), 0x10));
        }
        let endian = match &data[..2] {
            b"BY" => Endian::Big,
            b"YB" => Endian::Little,
            _ => {
                return Err(Error::BadMagic(
                    TinyString::from_bytes(&data[..2]),
                    "BY",
                ))
            }
        };
        let mut walk = Self {
            data,
            endian,
            hash_key_table_offset: 0,
            string_table_offset: 0,
            root_node_offset: 0,
            hash_key_table_size: 0,
            string_table_size: 0,
        };
        let version = walk.read_u16_at(2)?;
        if !(1..8).contains(&version) {
            return Err(Error::InvalidData("Unsupported BYML version (1-7 only)"));
        }
        walk.hash_key_table_offset = walk.read_u32_at(4)?;
        walk.string_table_offset = walk.read_u32_at(8)?;
        walk.root_node_offset = walk.read_u32_at(12)?;
        walk.hash_key_table_size = if walk.hash_key_table_offset == 0 {
            0
        } else {
            walk.table_size(walk.hash_key_table_offset)?
        };
        walk.string_table_size = if walk.string_table_offset == 0 {
            0
        } else {
            walk.table_size(walk.string_table_offset)?
        };
        Ok(walk)
    }

    /// Read the size (u24 entry count) of a string table or hash key
    /// table node at `offset`.
    fn table_size(&self, offset: u32) -> Result<u32> {
        let node_type = self.read_u8_at(offset as usize)?;
        if node_type != TAG_STRING_TABLE {
            return Err(Error::InvalidData(
                "Incorrect BYML node type: expected `string table`",
            ));
        }
        self.read_u24_at(offset as usize + 1)
    }

    /// Resolve a string table index to a `&'a str` borrowed from `data`.
    ///
    /// The writer stores `count + 1` offsets (the last one points past the
    /// final string), so `max_len` is bounded even for the last entry.
    fn resolve_table_string(&self, base: u32, size: u32, index: u32) -> Result<&'a str> {
        if index >= size {
            return Err(Error::InvalidData("Invalid string table entry index"));
        }
        let base = base as usize;
        let off_pos = self.off(base as u32, 4 + 4 * index)? as usize;
        let offset = self.read_u32_at(off_pos)? as usize;
        let next_offset = self.read_u32_at(off_pos + 4)? as usize;
        let max_len = next_offset.saturating_sub(offset);
        let start = base
            .checked_add(offset)
            .ok_or(Error::InsufficientData(self.data.len(), base))?;
        let bytes = self
            .data
            .get(start..)
            .ok_or(Error::InsufficientData(self.data.len(), start))?;
        let end = max_len.min(bytes.len());
        let mut len = 0;
        while len < end && bytes[len] != 0 {
            len += 1;
        }
        // `Utf8Error` converts into [`Error::InvalidUtf8`] via `From`.
        core::str::from_utf8(&bytes[..len]).map_err(Into::into)
    }

    /// Resolve an index into the string table.
    fn resolve_string(&self, index: u32) -> Result<&'a str> {
        self.resolve_table_string(self.string_table_offset, self.string_table_size, index)
    }

    /// Resolve an index into the hash key table.
    fn resolve_hash_key(&self, index: u32) -> Result<&'a str> {
        self.resolve_table_string(self.hash_key_table_offset, self.hash_key_table_size, index)
    }

    /// `base + extra`, checked for overflow (offsets are u32 values read
    /// from the document).
    fn off(&self, base: u32, extra: u32) -> Result<u32> {
        base.checked_add(extra)
            .ok_or(Error::InsufficientData(self.data.len(), base as usize))
    }

    /// Read a single byte at `pos`.
    fn read_u8_at(&self, pos: usize) -> Result<u8> {
        self.data
            .get(pos)
            .copied()
            .ok_or(Error::InsufficientData(self.data.len(), pos + 1))
    }

    /// Read a u16 at `pos` in the document's endianness.
    fn read_u16_at(&self, pos: usize) -> Result<u16> {
        let b = self.read_bytes_at(pos, 2)?;
        Ok(match self.endian {
            Endian::Little => u16::from_le_bytes([b[0], b[1]]),
            Endian::Big => u16::from_be_bytes([b[0], b[1]]),
        })
    }

    /// Read a u32 at `pos` in the document's endianness.
    fn read_u32_at(&self, pos: usize) -> Result<u32> {
        let b = self.read_bytes_at(pos, 4)?;
        Ok(match self.endian {
            Endian::Little => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            Endian::Big => u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
        })
    }

    /// Read a u64 at `pos` in the document's endianness.
    fn read_u64_at(&self, pos: usize) -> Result<u64> {
        let b = self.read_bytes_at(pos, 8)?;
        Ok(match self.endian {
            Endian::Little => u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
            Endian::Big => u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
        })
    }

    /// Read a u24 at `pos` in the document's endianness.
    fn read_u24_at(&self, pos: usize) -> Result<u32> {
        let b = self.read_bytes_at(pos, 3)?;
        Ok(match self.endian {
            Endian::Little => u32::from(b[0]) | u32::from(b[1]) << 8 | u32::from(b[2]) << 16,
            Endian::Big => u32::from(b[2]) | u32::from(b[1]) << 8 | u32::from(b[0]) << 16,
        })
    }

    /// Read `len` bytes at `pos`, bounds-checked.
    fn read_bytes_at(&self, pos: usize, len: usize) -> Result<&'a [u8]> {
        let end = pos
            .checked_add(len)
            .ok_or(Error::InsufficientData(self.data.len(), pos))?;
        self.data
            .get(pos..end)
            .ok_or(Error::InsufficientData(self.data.len(), end))
    }
}

/// Run the shared format walk over `data`, reporting into `sink`.
///
/// Returns the root node produced by the sink (a Null leaf when the root
/// offset is 0).
pub(crate) fn parse_byml<'a, Sink: BymlSink<'a>>(
    data: &'a [u8],
    sink: &mut Sink,
) -> Result<Sink::Node> {
    let mut walk = Walk::new(data)?;
    match walk.root_node_offset {
        0 => sink.leaf(Leaf::Null, Key::None, 0),
        root => walk_container(&mut walk, sink, root, Key::None, 0, 1),
    }
}

/// Descend into a container node at `offset` (depth counts this container;
/// the root container is depth 1).
fn walk_container<'a, Sink: BymlSink<'a>>(
    walk: &mut Walk<'a>,
    sink: &mut Sink,
    offset: u32,
    key: Key<'a>,
    aux: u32,
    depth: usize,
) -> Result<Sink::Node> {
    if depth > BYML_MAX_DEPTH {
        return Err(Error::InvalidData(
            "BYML container nesting exceeds maximum depth",
        ));
    }
    let node_type = walk.read_u8_at(offset as usize)?;
    let kind = match node_type {
        TAG_ARRAY => ContainerKind::Array,
        TAG_MAP => ContainerKind::Map,
        TAG_HASH_MAP => ContainerKind::HashMap,
        TAG_VALUE_HASH_MAP => ContainerKind::ValueHashMap,
        _ => {
            return Err(Error::InvalidData("Invalid BYML container node type"));
        }
    };
    let size = walk.read_u24_at(offset as usize + 1)?;

    sink.begin_container(kind, key, aux)?;
    match kind {
        ContainerKind::Array => {
            // `size` is a u24, so `4 * i` cannot overflow u32.
            let values_offset = walk.off(offset, 4 + align(size, 4))?;
            for i in 0..size {
                let tag_pos = walk.off(offset, 4 + i)?;
                let ct = walk.read_u8_at(tag_pos as usize)?;
                let val_pos = walk.off(values_offset, 4 * i)?;
                let child = walk_entry(walk, sink, val_pos, ct, Key::None, 0, depth + 1)?;
                sink.add_child(child, Key::None, 0)?;
            }
        }
        ContainerKind::Map => {
            for i in 0..size {
                let entry = walk.off(offset, 4 + 8 * i)?;
                let name_idx = walk.read_u24_at(entry as usize)?;
                let ct = walk.read_u8_at(walk.off(entry, 3)? as usize)?;
                let key = Key::Str(walk.resolve_hash_key(name_idx)?);
                let child = walk_entry(walk, sink, walk.off(entry, 4)?, ct, key, 0, depth + 1)?;
                sink.add_child(child, key, 0)?;
            }
        }
        ContainerKind::HashMap => {
            let types_offset = walk.off(offset, 4 + 8 * size)?;
            for i in 0..size {
                let entry = walk.off(offset, 4 + 8 * i)?;
                let hash = walk.read_u32_at(entry as usize)?;
                let ct = walk.read_u8_at(walk.off(types_offset, i)? as usize)?;
                let child =
                    walk_entry(walk, sink, walk.off(entry, 4)?, ct, Key::Hash(hash), 0, depth + 1)?;
                sink.add_child(child, Key::Hash(hash), 0)?;
            }
        }
        ContainerKind::ValueHashMap => {
            let types_offset = walk.off(offset, 4 + 12 * size)?;
            for i in 0..size {
                let entry = walk.off(offset, 4 + 12 * i)?;
                let hash = walk.read_u32_at(walk.off(entry, 4)? as usize)?;
                let aux = walk.read_u32_at(walk.off(entry, 8)? as usize)?;
                let ct = walk.read_u8_at(walk.off(types_offset, i)? as usize)?;
                let child = walk_entry(walk, sink, entry, ct, Key::Hash(hash), aux, depth + 1)?;
                sink.add_child(child, Key::Hash(hash), aux)?;
            }
        }
    }

    let node = sink.end_container()?;
    Ok(node)
}

/// Walk a value entry: read the raw u32 at `pos`, then dispatch to a
/// container or a leaf according to `node_type`.
///
/// `depth` is this entry's own container level (the root container is
/// level 1).
fn walk_entry<'a, Sink: BymlSink<'a>>(
    walk: &mut Walk<'a>,
    sink: &mut Sink,
    pos: u32,
    node_type: u8,
    key: Key<'a>,
    aux: u32,
    depth: usize,
) -> Result<Sink::Node> {
    let raw = walk.read_u32_at(pos as usize)?;
    if is_container_tag(node_type) {
        walk_container(walk, sink, raw, key, aux, depth)
    } else {
        walk_leaf(walk, sink, node_type, raw, key, aux)
    }
}

/// Decode a leaf value from its raw u32 entry.
fn walk_leaf<'a, Sink: BymlSink<'a>>(
    walk: &mut Walk<'a>,
    sink: &mut Sink,
    node_type: u8,
    raw: u32,
    key: Key<'a>,
    aux: u32,
) -> Result<Sink::Node> {
    let payload = match node_type {
        TAG_STRING => Leaf::String(walk.resolve_string(raw)?),
        TAG_BINARY => {
            let size = walk.read_u32_at(raw as usize)? as usize;
            let start = walk.off(raw, 4)?;
            Leaf::BinaryData(walk.read_bytes_at(start as usize, size)?)
        }
        TAG_FILE => {
            let size = walk.read_u32_at(raw as usize)? as usize;
            let _unknown = walk.read_u32_at(walk.off(raw, 4)? as usize)?;
            let start = walk.off(raw, 8)?;
            Leaf::FileData(walk.read_bytes_at(start as usize, size)?)
        }
        TAG_BOOL => Leaf::Bool(raw != 0),
        TAG_I32 => Leaf::I32(raw as i32),
        TAG_U32 => Leaf::U32(raw),
        TAG_FLOAT => Leaf::Float(f32::from_bits(raw)),
        TAG_I64 => Leaf::I64(walk.read_u64_at(raw as usize)? as i64),
        TAG_U64 => Leaf::U64(walk.read_u64_at(raw as usize)?),
        TAG_DOUBLE => Leaf::Double(f64::from_bits(walk.read_u64_at(raw as usize)?)),
        TAG_NULL => Leaf::Null,
        _ => return Err(Error::InvalidData("Invalid BYML value node type")),
    };
    sink.leaf(payload, key, aux)
}

// ---------------------------------------------------------------------------
// BymlRef: alloc-backed, enum-shaped borrowed view
// ---------------------------------------------------------------------------

/// An enum-shaped, zero-copy view of a BYML document.
///
/// All strings and binary payloads are borrowed (`&str`, `&[u8]`) from
/// the input buffer passed to [`BymlRef::from_binary`]; only the tree
/// structure (arrays and maps) is allocated.
///
/// **Note**: unlike [`Byml::from_binary`](crate::byml::Byml::from_binary),
/// `from_binary` does not auto-decompress Yaz0-compressed input. Decompress
/// first with [`crate::yaz0::decompress_if`] (feature `yaz0`), which
/// returns a [`Cow`](alloc::borrow::Cow), and call
/// `BymlRef::from_binary(cow.as_ref())`.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum BymlRef<'a> {
    /// String value.
    String(&'a str),
    /// Binary data (not used in BOTW).
    BinaryData(&'a [u8]),
    /// File data.
    FileData(&'a [u8]),
    /// Array of BYML nodes.
    Array(Vec<BymlRef<'a>>),
    /// Hash map of BYML nodes with string keys.
    Map(crate::FxHashMap<&'a str, BymlRef<'a>>),
    /// Hash map of BYML nodes with u32 keys.
    HashMap(crate::FxHashMap<u32, BymlRef<'a>>),
    /// Hash map of BYML nodes with u32 keys and additional value.
    ValueHashMap(crate::FxHashMap<u32, (BymlRef<'a>, u32)>),
    /// Boolean value.
    Bool(bool),
    /// 32-bit signed integer.
    I32(i32),
    /// 32-bit float.
    Float(f32),
    /// 32-bit unsigned integer.
    U32(u32),
    /// 64-bit signed integer.
    I64(i64),
    /// 64-bit unsigned integer.
    U64(u64),
    /// 64-bit float.
    Double(f64),
    /// Null value.
    Null,
}

#[cfg(feature = "alloc")]
impl<'a> BymlRef<'a> {
    /// The name of this node type.
    pub fn type_name(&self) -> &'static str {
        match self {
            BymlRef::String(_) => "String",
            BymlRef::BinaryData(_) => "Binary",
            BymlRef::FileData(_) => "File",
            BymlRef::Array(_) => "Array",
            BymlRef::Map(_) => "Map",
            BymlRef::HashMap(_) => "HashMap",
            BymlRef::ValueHashMap(_) => "ValueHashMap",
            BymlRef::Bool(_) => "Bool",
            BymlRef::I32(_) => "I32",
            BymlRef::Float(_) => "Float",
            BymlRef::U32(_) => "U32",
            BymlRef::I64(_) => "I64",
            BymlRef::U64(_) => "U64",
            BymlRef::Double(_) => "Double",
            BymlRef::Null => "Null",
        }
    }

    /// Checks if the BYML node is a null node.
    pub fn is_null(&self) -> bool {
        matches!(self, BymlRef::Null)
    }

    /// Get a reference to the inner bool value.
    pub fn as_bool(&self) -> Result<bool> {
        if let BymlRef::Bool(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "Bool"))
        }
    }

    /// Get a reference to the inner i32 value.
    pub fn as_i32(&self) -> Result<i32> {
        if let BymlRef::I32(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "I32"))
        }
    }

    /// Get a reference to the inner u32 value.
    pub fn as_u32(&self) -> Result<u32> {
        if let BymlRef::U32(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "U32"))
        }
    }

    /// Get a reference to the inner i64 value.
    pub fn as_i64(&self) -> Result<i64> {
        if let BymlRef::I64(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "I64"))
        }
    }

    /// Get a reference to the inner u64 value.
    pub fn as_u64(&self) -> Result<u64> {
        if let BymlRef::U64(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "U64"))
        }
    }

    /// Get a reference to the inner f32 value.
    pub fn as_float(&self) -> Result<f32> {
        if let BymlRef::Float(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "Float"))
        }
    }

    /// Get a reference to the inner f64 value.
    pub fn as_double(&self) -> Result<f64> {
        if let BymlRef::Double(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "Double"))
        }
    }

    /// Get a reference to the inner string value.
    pub fn as_str(&self) -> Result<&'a str> {
        if let BymlRef::String(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "String"))
        }
    }

    /// Get a reference to the inner byte slice.
    pub fn as_binary_data(&self) -> Result<&'a [u8]> {
        if let BymlRef::BinaryData(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "BinaryData"))
        }
    }

    /// Get a reference to the inner byte slice.
    pub fn as_file_data(&self) -> Result<&'a [u8]> {
        if let BymlRef::FileData(v) = self {
            Ok(*v)
        } else {
            Err(Error::TypeError(self.type_name(), "FileData"))
        }
    }

    /// Get a reference to the inner array of BYML nodes.
    pub fn as_array(&self) -> Result<&[BymlRef<'a>]> {
        if let BymlRef::Array(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Array"))
        }
    }

    /// Get a reference to the inner string-keyed hash map of BYML nodes.
    pub fn as_map(&self) -> Result<&crate::FxHashMap<&'a str, BymlRef<'a>>> {
        if let BymlRef::Map(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "Map"))
        }
    }

    /// Get a reference to the inner u32-keyed hash map of BYML nodes.
    pub fn as_hash_map(&self) -> Result<&crate::FxHashMap<u32, BymlRef<'a>>> {
        if let BymlRef::HashMap(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "HashMap"))
        }
    }

    /// Get a reference to the inner u32-keyed hash map of BYML nodes.
    pub fn as_value_hash_map(&self) -> Result<&crate::FxHashMap<u32, (BymlRef<'a>, u32)>>
    {
        if let BymlRef::ValueHashMap(v) = self {
            Ok(v)
        } else {
            Err(Error::TypeError(self.type_name(), "ValueHashMap"))
        }
    }

    /// Read a document from binary data, borrowing all strings and
    /// payloads from `data`.
    ///
    /// **Note**: Yaz0-compressed input (`"Yaz0"` magic) is not
    /// auto-decompressed here (unlike [`Byml::from_binary`]). Decompress
    /// first, e.g. with [`crate::yaz0::decompress_if`] (feature `yaz0`),
    /// and call `BymlRef::from_binary(cow.as_ref())`.
    pub fn from_binary(data: &'a [u8]) -> Result<Self> {
        let mut sink = BymlRefSink { stack: Vec::new() };
        parse_byml(data, &mut sink)
    }

    /// Convert this borrowed view into an owned [`Byml`].
    pub fn to_owned(&self) -> Byml {
        match self {
            BymlRef::String(s) => Byml::String(String::from(*s)),
            BymlRef::BinaryData(d) => Byml::BinaryData(d.to_vec()),
            BymlRef::FileData(d) => Byml::FileData(d.to_vec()),
            BymlRef::Array(a) => Byml::Array(a.iter().map(BymlRef::to_owned).collect()),
            BymlRef::Map(m) => Byml::Map(
                m.iter()
                    .map(|(k, v)| (String::from(*k), v.to_owned()))
                    .collect(),
            ),
            BymlRef::HashMap(m) => {
                Byml::HashMap(m.iter().map(|(k, v)| (*k, v.to_owned())).collect())
            }
            BymlRef::ValueHashMap(m) => Byml::ValueHashMap(
                m.iter()
                    .map(|(k, (v, aux))| (*k, (v.to_owned(), *aux)))
                    .collect(),
            ),
            BymlRef::Bool(b) => Byml::Bool(*b),
            BymlRef::I32(i) => Byml::I32(*i),
            BymlRef::Float(f) => Byml::Float(*f),
            BymlRef::U32(u) => Byml::U32(*u),
            BymlRef::I64(i) => Byml::I64(*i),
            BymlRef::U64(u) => Byml::U64(*u),
            BymlRef::Double(d) => Byml::Double(*d),
            BymlRef::Null => Byml::Null,
        }
    }

    fn from_leaf(payload: Leaf<'a>) -> Self {
        match payload {
            Leaf::String(s) => BymlRef::String(s),
            Leaf::BinaryData(d) => BymlRef::BinaryData(d),
            Leaf::FileData(d) => BymlRef::FileData(d),
            Leaf::Bool(b) => BymlRef::Bool(b),
            Leaf::I32(i) => BymlRef::I32(i),
            Leaf::Float(f) => BymlRef::Float(f),
            Leaf::U32(u) => BymlRef::U32(u),
            Leaf::I64(i) => BymlRef::I64(i),
            Leaf::U64(u) => BymlRef::U64(u),
            Leaf::Double(d) => BymlRef::Double(d),
            Leaf::Null => BymlRef::Null,
        }
    }
}

#[cfg(feature = "alloc")]
impl PartialEq for BymlRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BymlRef::String(s1), BymlRef::String(s2)) => s1 == s2,
            (BymlRef::BinaryData(d1), BymlRef::BinaryData(d2)) => d1 == d2,
            (BymlRef::FileData(d1), BymlRef::FileData(d2)) => d1 == d2,
            (BymlRef::Array(a1), BymlRef::Array(a2)) => a1 == a2,
            (BymlRef::Map(h1), BymlRef::Map(h2)) => h1 == h2,
            (BymlRef::HashMap(h1), BymlRef::HashMap(h2)) => h1 == h2,
            (
                BymlRef::ValueHashMap(h1),
                BymlRef::ValueHashMap(h2),
            ) => h1 == h2,
            (BymlRef::Bool(b1), BymlRef::Bool(b2)) => b1 == b2,
            (BymlRef::I32(i1), BymlRef::I32(i2)) => i1 == i2,
            (BymlRef::Float(f1), BymlRef::Float(f2)) => almost::equal(*f1, *f2),
            (BymlRef::U32(u1), BymlRef::U32(u2)) => u1 == u2,
            (BymlRef::I64(i1), BymlRef::I64(i2)) => i1 == i2,
            (BymlRef::U64(u1), BymlRef::U64(u2)) => u1 == u2,
            (BymlRef::Double(d1), BymlRef::Double(d2)) => almost::equal(*d1, *d2),
            (BymlRef::Null, BymlRef::Null) => true,
            _ => false,
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a, I: Into<BymlIndex<'a>>> core::ops::Index<I> for BymlRef<'a> {
    type Output = BymlRef<'a>;

    fn index(&self, index: I) -> &Self::Output {
        match (self, index.into()) {
            (BymlRef::Array(a), BymlIndex::ArrayIdx(i)) => &a[i],
            (BymlRef::Map(h), BymlIndex::StringIdx(k)) => &h[k],
            (BymlRef::HashMap(h), BymlIndex::HashIdx(i)) => &h[&i],
            (BymlRef::ValueHashMap(h), BymlIndex::HashIdx(i)) => &h[&i].0,
            _ => panic!("Wrong index type or node type."),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a Byml> for BymlRef<'a> {
    fn from(byaml: &'a Byml) -> Self {
        match byaml {
            Byml::String(s) => BymlRef::String(s.as_str()),
            Byml::BinaryData(d) => BymlRef::BinaryData(d.as_slice()),
            Byml::FileData(d) => BymlRef::FileData(d.as_slice()),
            Byml::Array(a) => BymlRef::Array(a.iter().map(From::from).collect()),
            Byml::Map(m) => BymlRef::Map(
                m.iter()
                    .map(|(k, v)| (k.as_str(), v.into()))
                    .collect(),
            ),
            Byml::HashMap(m) => {
                BymlRef::HashMap(m.iter().map(|(k, v)| (*k, v.into())).collect())
            }
            Byml::ValueHashMap(m) => BymlRef::ValueHashMap(
                m.iter()
                    .map(|(k, (v, aux))| (*k, (v.into(), *aux)))
                    .collect(),
            ),
            Byml::Bool(b) => BymlRef::Bool(*b),
            Byml::I32(i) => BymlRef::I32(*i),
            Byml::Float(f) => BymlRef::Float(*f),
            Byml::U32(u) => BymlRef::U32(*u),
            Byml::I64(i) => BymlRef::I64(*i),
            Byml::U64(u) => BymlRef::U64(*u),
            Byml::Double(d) => BymlRef::Double(*d),
            Byml::Null => BymlRef::Null,
        }
    }
}

/// Sink building a [`BymlRef`] tree.
#[cfg(feature = "alloc")]
struct BymlRefSink<'a> {
    stack: Vec<Frame<'a>>,
}

#[cfg(feature = "alloc")]
impl<'a> BymlSink<'a> for BymlRefSink<'a> {
    type Node = BymlRef<'a>;

    fn leaf(&mut self, payload: Leaf<'a>, _key: Key<'a>, _aux: u32) -> Result<BymlRef<'a>> {
        Ok(BymlRef::from_leaf(payload))
    }

    fn begin_container(
        &mut self,
        kind: ContainerKind,
        _key: Key<'a>,
        _aux: u32,
    ) -> Result<()> {
        self.stack.push(match kind {
            ContainerKind::Array => Frame::Array(Vec::new()),
            ContainerKind::Map => Frame::Map(crate::FxHashMap::default()),
            ContainerKind::HashMap => Frame::HashMap(crate::FxHashMap::default()),
            ContainerKind::ValueHashMap => Frame::ValueHashMap(crate::FxHashMap::default()),
        });
        Ok(())
    }

    fn add_child(
        &mut self,
        node: BymlRef<'a>,
        key: Key<'a>,
        aux: u32,
    ) -> Result<()> {
        let frame = self
            .stack
            .last_mut()
            .expect("BymlRefSink::add_child called outside a container");
        match frame {
            Frame::Array(children) => {
                children.push(node);
            }
            Frame::Map(map) => match key {
                Key::Str(s) => {
                    map.insert(s, node);
                }
                _ => {
                    return Err(Error::InvalidData(
                        "BYML map child has a non-string key",
                    ))
                }
            },
            Frame::HashMap(map) => match key {
                Key::Hash(h) => {
                    map.insert(h, node);
                }
                _ => {
                    return Err(Error::InvalidData(
                        "BYML hash map child has a non-hash key",
                    ))
                }
            },
            Frame::ValueHashMap(map) => match key {
                Key::Hash(h) => {
                    map.insert(h, (node, aux));
                }
                _ => {
                    return Err(Error::InvalidData(
                        "BYML value hash map child has a non-hash key",
                    ))
                }
            },
        }
        Ok(())
    }

    fn end_container(&mut self) -> Result<BymlRef<'a>> {
        let frame = self
            .stack
            .pop()
            .expect("BymlRefSink::end_container called outside a container");
        Ok(match frame {
            Frame::Array(children) => BymlRef::Array(children),
            Frame::Map(map) => BymlRef::Map(map),
            Frame::HashMap(map) => BymlRef::HashMap(map),
            Frame::ValueHashMap(map) => BymlRef::ValueHashMap(map),
        })
    }
}

/// One container under construction in a [`BymlRefSink`].
#[cfg(feature = "alloc")]
enum Frame<'a> {
    Array(Vec<BymlRef<'a>>),
    Map(crate::FxHashMap<&'a str, BymlRef<'a>>),
    HashMap(crate::FxHashMap<u32, BymlRef<'a>>),
    ValueHashMap(crate::FxHashMap<u32, (BymlRef<'a>, u32)>),
}
