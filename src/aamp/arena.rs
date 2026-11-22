//! Flat, fixed-capacity AAMP readers that need no allocator at all.
//!
//! All nodes are stored in a plain store ([`OwnedStore`](crate::storage::OwnedStore),
//! [`FixedStore`](crate::storage::FixedStore), or a caller-provided
//! [`BorrowedStore`](crate::storage::BorrowedStore)) and every payload
//! (`&str`, `&[u8]`) is borrowed from the input buffer. Nodes are referenced
//! by [`NodeId`] indices and linked into sibling lists, so a document is
//! traversed with a cursor/index API rather than owned handles.
//!
//! This module compiles with `--no-default-features` (pure `core`) and works
//! on `no_std` targets without an allocator.
//!
//! The shared format walk lives in `super::reader` and is generic over the
//! [`AampSink`](super::reader::AampSink) trait; this module provides the
//! arena sink ([`ArenaSink`]) used by [`ParameterIOArena::from_binary`].

use super::Name;
use super::reader::{
    self as reader_mod, AAMP_MAX_DEPTH, AampSink, BufferRef, CurveRef, ParameterRef,
};
use crate::storage;
use crate::storage::Store;
use crate::types::{Color, Quat, Vector2f, Vector3f, Vector4f};
use crate::{Error, Result};

pub use storage::NodeId;

/// A fixed-capacity arena backed by a [`FixedStore`](crate::storage::FixedStore).
pub type ParameterIOStack<'a, const N: usize> =
    ParameterIOArena<'a, storage::FixedStore<AampNode<'a>, N>>;

/// The payload of a node in a [`ParameterIOArena`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AampPayload<'a> {
    /// A parameter list container.
    List,
    /// A parameter object container.
    Object,
    /// Boolean value.
    Bool(bool),
    /// 32-bit float.
    F32(f32),
    /// 32-bit signed integer.
    I32(i32),
    /// 32-bit unsigned integer.
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

impl<'a> From<ParameterRef<'a>> for AampPayload<'a> {
    fn from(parameter: ParameterRef<'a>) -> Self {
        match parameter {
            ParameterRef::Bool(b) => AampPayload::Bool(b),
            ParameterRef::F32(f) => AampPayload::F32(f),
            ParameterRef::I32(i) => AampPayload::I32(i),
            ParameterRef::U32(u) => AampPayload::U32(u),
            ParameterRef::Vec2(v) => AampPayload::Vec2(v),
            ParameterRef::Vec3(v) => AampPayload::Vec3(v),
            ParameterRef::Vec4(v) => AampPayload::Vec4(v),
            ParameterRef::Color(c) => AampPayload::Color(c),
            ParameterRef::Quat(q) => AampPayload::Quat(q),
            ParameterRef::String32(s) => AampPayload::String32(s),
            ParameterRef::String64(s) => AampPayload::String64(s),
            ParameterRef::String256(s) => AampPayload::String256(s),
            ParameterRef::StringRef(s) => AampPayload::StringRef(s),
            ParameterRef::Curve1(c) => AampPayload::Curve1(c),
            ParameterRef::Curve2(c) => AampPayload::Curve2(c),
            ParameterRef::Curve3(c) => AampPayload::Curve3(c),
            ParameterRef::Curve4(c) => AampPayload::Curve4(c),
            ParameterRef::BufferInt(b) => AampPayload::BufferInt(b),
            ParameterRef::BufferF32(b) => AampPayload::BufferF32(b),
            ParameterRef::BufferU32(b) => AampPayload::BufferU32(b),
            ParameterRef::BufferBinary(b) => AampPayload::BufferBinary(b),
        }
    }
}

/// A single node in a [`ParameterIOArena`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AampNode<'a> {
    /// The node's payload.
    pub payload: AampPayload<'a>,
    /// The name of this node (parameter, object or list name hash).
    pub name: Name,
    /// Index of the first child, or [`NodeId::none`] if childless.
    pub first_child: NodeId,
    /// Index of the next sibling, or [`NodeId::none`] if this is the last.
    pub next_sibling: NodeId,
    /// Number of children (containers only).
    pub child_count: u32,
}

impl<'a> AampNode<'a> {
    /// The name of this node's payload kind.
    pub fn kind(&self) -> &'static str {
        match self.payload {
            AampPayload::List => "List",
            AampPayload::Object => "Object",
            AampPayload::Bool(_) => "Bool",
            AampPayload::F32(_) => "F32",
            AampPayload::I32(_) => "I32",
            AampPayload::U32(_) => "U32",
            AampPayload::Vec2(_) => "Vec2",
            AampPayload::Vec3(_) => "Vec3",
            AampPayload::Vec4(_) => "Vec4",
            AampPayload::Color(_) => "Color",
            AampPayload::Quat(_) => "Quat",
            AampPayload::String32(_) => "String32",
            AampPayload::String64(_) => "String64",
            AampPayload::String256(_) => "String256",
            AampPayload::StringRef(_) => "StringRef",
            AampPayload::Curve1(_) => "Curve1",
            AampPayload::Curve2(_) => "Curve2",
            AampPayload::Curve3(_) => "Curve3",
            AampPayload::Curve4(_) => "Curve4",
            AampPayload::BufferInt(_) => "BufferInt",
            AampPayload::BufferF32(_) => "BufferF32",
            AampPayload::BufferU32(_) => "BufferU32",
            AampPayload::BufferBinary(_) => "BufferBinary",
        }
    }

    /// Returns `true` for containers ([`AampPayload::List`],
    /// [`AampPayload::Object`]).
    pub fn is_container(&self) -> bool {
        matches!(self.payload, AampPayload::List | AampPayload::Object)
    }

    /// The number of children of a container node.
    pub const fn child_count(&self) -> u32 {
        self.child_count
    }
}

/// A flat AAMP parameter IO stored in `S`, with all payloads borrowed from the
/// input buffer.
pub struct ParameterIOArena<'a, S> {
    store: S,
    root: NodeId,
    version: u32,
    data_type: &'a str,
}

impl<'a, S: Store<AampNode<'a>> + Default> ParameterIOArena<'a, S> {
    /// Read a parameter archive from binary data, borrowing all strings and
    /// payloads from `data`.
    ///
    /// **Note**: Yaz0-compressed input (`"Yaz0"` magic) is not
    /// auto-decompressed here. Decompress first with
    /// [`crate::yaz0::decompress_if`] (feature `yaz0`) and call
    /// `ParameterIOArena::from_binary(cow.as_ref())`.
    ///
    /// Returns [`Error::CapacityExceeded`] if the document does not fit in
    /// `S`. This is available for stores that can be created without
    /// caller-provided memory ([`OwnedStore`](crate::storage::OwnedStore),
    /// [`FixedStore`](crate::storage::FixedStore)); use
    /// [`from_binary_in`](ParameterIOArena::from_binary_in) for
    /// [`BorrowedStore`](crate::storage::BorrowedStore).
    ///
    /// # Stack use
    ///
    /// Constructing the store and returning the arena moves the whole
    /// inline store by value a few times, which debug builds do not elide.
    /// For large [`FixedStore`](crate::storage::FixedStore) capacities (or
    /// very small stacks), prefer creating the arena with
    /// [`new`](ParameterIOArena::new) and parsing with
    /// [`parse_in_place`](ParameterIOArena::parse_in_place) (no large
    /// moves), or use [`from_binary_in`](ParameterIOArena::from_binary_in)
    /// with a [`BorrowedStore`](crate::storage::BorrowedStore).
    pub fn from_binary(data: &'a [u8]) -> Result<Self> {
        let mut this = Self {
            store: S::default(),
            root: NodeId::none(),
            version: 0,
            data_type: "",
        };
        this.parse_in_place(data)?;
        Ok(this)
    }

    /// Create an empty arena over a default-constructed store.
    pub fn new() -> Self {
        Self {
            store: S::default(),
            root: NodeId::none(),
            version: 0,
            data_type: "",
        }
    }

    /// Parse `data` into this arena, discarding any previous contents.
    ///
    /// All strings and payloads are borrowed from `data`; nothing is
    /// allocated and the store is never moved. Returns
    /// [`Error::CapacityExceeded`] if the document does not fit in the
    /// store.
    /// On error the arena is left empty: `root()` returns `None` and the
    /// header values are reset.
    pub fn parse_in_place(&mut self, data: &'a [u8]) -> Result<()> {
        self.store.clear();
        self.root = NodeId::none();
        self.version = 0;
        self.data_type = "";
        let (root, version, data_type) = {
            let mut sink = ArenaSink {
                store: &mut self.store,
                stack: [(NodeId::none(), None); AAMP_MAX_DEPTH + 1],
                depth: 0,
            };
            reader_mod::parse_aamp(data, &mut sink)?
        };
        self.root = root;
        self.version = version;
        self.data_type = data_type;
        Ok(())
    }
}

impl<'s, 'a> ParameterIOArena<'a, storage::BorrowedStore<'s, AampNode<'a>>> {
    /// Read a parameter archive into caller-provided `slots`, borrowing all
    /// strings and payloads from `data`. No allocation occurs.
    ///
    /// Returns [`Error::CapacityExceeded`] if the document does not fit in
    /// `slots`.
    pub fn from_binary_in(
        data: &'a [u8],
        slots: &'s mut [core::mem::MaybeUninit<AampNode<'a>>],
    ) -> Result<Self> {
        let mut store = storage::BorrowedStore::new(slots);
        let (root, version, data_type) = {
            let mut sink = ArenaSink {
                store: &mut store,
                stack: [(NodeId::none(), None); AAMP_MAX_DEPTH + 1],
                depth: 0,
            };
            reader_mod::parse_aamp(data, &mut sink)?
        };
        Ok(ParameterIOArena {
            store,
            root,
            version,
            data_type,
        })
    }
}

impl<'a, S: Store<AampNode<'a>> + Default> Default for ParameterIOArena<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> core::fmt::Debug for ParameterIOArena<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ParameterIOArena")
            .field("root", &self.root)
            .field("version", &self.version)
            .field("data_type", &self.data_type)
            .field("store", &"..")
            .finish()
    }
}

impl<'a, S: Store<AampNode<'a>>> ParameterIOArena<'a, S> {
    /// The root node id, or `None` if the arena is empty (never parsed, or
    /// the last parse failed).
    pub fn root(&self) -> Option<NodeId> {
        if self.root.is_none() { None } else { Some(self.root) }
    }

    /// The data version from the header (not the AAMP format version).
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The data type string from the header (e.g. `"xml"`).
    pub fn data_type(&self) -> &'a str {
        self.data_type
    }

    /// Get a shared reference to a node by id.
    ///
    /// The returned reference is borrowed from the arena and lives as long
    /// as the `&self` borrow.
    pub fn node(&self, id: NodeId) -> Option<&AampNode<'a>> {
        self.store.get(id)
    }

    /// Number of nodes in the arena.
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Returns `true` if the arena contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.store.len() == 0
    }

    /// Iterate over the children of `container` in document order.
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = (Name, NodeId)> + '_ {
        let first = self
            .node(id)
            .and_then(|node| (!node.first_child.is_none()).then_some(node.first_child));
        core::iter::successors(first, move |current: &NodeId| {
            let node = self.node(*current)?;
            let next = node.next_sibling;
            (!next.is_none()).then_some(next)
        })
        .filter_map(move |id| self.node(id).map(|node| (node.name, id)))
    }

    /// Find a child of `container` by name by linear scan.
    pub fn get<N: Into<Name>>(&self, id: NodeId, name: N) -> Option<NodeId> {
        let name = name.into();
        self.children(id)
            .find(move |(n, _)| *n == name)
            .map(|(_, id)| id)
    }
}

/// The sink used by [`ParameterIOArena::from_binary`].
struct ArenaSink<'b, S> {
    store: &'b mut S,
    /// One entry per open container: the container's node id and the id of
    /// the last child appended to it. `stack[self.depth - 1]` is the
    /// innermost open container while `depth > 0`.
    stack: [(NodeId, Option<NodeId>); AAMP_MAX_DEPTH + 1],
    /// Number of open containers; the next `begin_container` writes to
    /// `stack[self.depth]`.
    depth: usize,
}

impl<'b, 'a, S: Store<AampNode<'a>>> ArenaSink<'b, S> {
    fn link_child(&mut self, child: NodeId) -> Result<()> {
        debug_assert!(self.depth >= 1);
        let len = self.store.len();
        let (container, last) = self.stack[self.depth - 1];
        match last {
            Some(last_id) => {
                self.store
                    .get_mut(last_id)
                    .ok_or(Error::IndexOutOfRange {
                        index: last_id.0 as usize,
                        len,
                    })?
                    .next_sibling = child;
            }
            None => {
                self.store
                    .get_mut(container)
                    .ok_or(Error::IndexOutOfRange {
                        index: container.0 as usize,
                        len,
                    })?
                    .first_child = child;
            }
        }
        self.store
            .get_mut(container)
            .ok_or(Error::IndexOutOfRange {
                index: container.0 as usize,
                len,
            })?
            .child_count += 1;
        self.stack[self.depth - 1] = (container, Some(child));
        Ok(())
    }
}

impl<'b, 'a, S: Store<AampNode<'a>>> AampSink<'a> for ArenaSink<'b, S> {
    type Node = NodeId;

    fn leaf(&mut self, parameter: ParameterRef<'a>, key: Name) -> Result<NodeId> {
        let node = AampNode {
            payload: parameter.into(),
            name: key,
            first_child: NodeId::none(),
            next_sibling: NodeId::none(),
            child_count: 0,
        };
        // Children are linked to their parent by `add_child`, which the walk
        // calls after every child; a leaf at the top of the document has no
        // parent and is not linked.
        self.store.push(node)
    }

    fn begin_list(&mut self, name: Name) -> Result<()> {
        let node = AampNode {
            payload: AampPayload::List,
            name,
            first_child: NodeId::none(),
            next_sibling: NodeId::none(),
            child_count: 0,
        };
        let id = self.store.push(node)?;
        debug_assert!(self.depth <= AAMP_MAX_DEPTH);
        self.stack[self.depth] = (id, None);
        self.depth += 1;
        Ok(())
    }

    fn begin_object(&mut self, name: Name) -> Result<()> {
        let node = AampNode {
            payload: AampPayload::Object,
            name,
            first_child: NodeId::none(),
            next_sibling: NodeId::none(),
            child_count: 0,
        };
        let id = self.store.push(node)?;
        debug_assert!(self.depth <= AAMP_MAX_DEPTH);
        self.stack[self.depth] = (id, None);
        self.depth += 1;
        Ok(())
    }

    fn add_list(&mut self, child: NodeId, _key: Name) -> Result<()> {
        debug_assert!(self.depth >= 1);
        let _ = self.stack[self.depth - 1];
        self.link_child(child)
    }

    fn add_object(&mut self, child: NodeId, _key: Name) -> Result<()> {
        debug_assert!(self.depth >= 1);
        let _ = self.stack[self.depth - 1];
        self.link_child(child)
    }

    fn add_param(&mut self, child: NodeId, _key: Name) -> Result<()> {
        debug_assert!(self.depth >= 1);
        let _ = self.stack[self.depth - 1];
        self.link_child(child)
    }

    fn end_container(&mut self) -> Result<NodeId> {
        debug_assert!(self.depth >= 1);
        self.depth -= 1;
        let (container, _) = self.stack[self.depth];
        Ok(container)
    }
}
