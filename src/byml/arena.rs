//! Flat, fixed-capacity BYML readers that need no allocator at all.
//!
//! All nodes are stored in a plain store (`VecStore`, `ArrayStore`, or a
//! caller-provided `SliceStore`) and every payload (`&str`, `&[u8]`) is
//! borrowed from the input buffer. Nodes are referenced by [`NodeId`]
//! indices and linked into sibling lists, so a document is traversed with a
//! cursor/index API rather than owned handles.
//!
//! Both [`VecStore`] and [`SliceStore`] are backed by plain memory, so this
//! module compiles with `--no-default-features` (pure `core`) and works on
//! `no_std` targets without an allocator.
//!
//! The shared format walk lives in `super::reader` and is generic over the
//! [`BymlSink`](super::reader::BymlSink) trait; this module provides the
//! arena sink ([`ArenaSink`]) used by [`BymlArena::from_binary`].

use crate::{Error, Result};
use super::reader::{self as reader_mod, BymlSink, BYML_MAX_DEPTH, ContainerKind, Leaf};

/// A node index. `NodeId::none()` (`u32::MAX`) is the "no node" sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    /// The "no node" sentinel (`u32::MAX`).
    pub const fn none() -> Self {
        Self(u32::MAX)
    }

    /// Returns `true` if this is the "no node" sentinel.
    pub const fn is_none(self) -> bool {
        self.0 == u32::MAX
    }
}

impl From<u32> for NodeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// The key under which a node is stored in its parent container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key<'a> {
    /// No key: array children and the document root.
    None,
    /// A string key borrowed from the input buffer (map children).
    Str(&'a str),
    /// A u32 hash key (hash map / value hash map children).
    Hash(u32),
}

/// The payload of a node. Container kinds carry no payload data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Payload<'a> {
    String(&'a str),
    BinaryData(&'a [u8]),
    FileData(&'a [u8]),
    Array,
    Map,
    HashMap,
    ValueHashMap,
    Bool(bool),
    I32(i32),
    Float(f32),
    U32(u32),
    I64(i64),
    U64(u64),
    Double(f64),
    Null,
}

/// A single node in a [`BymlArena`].
///
/// The arena owns the storage; `payload`, `key`, and `aux` describe the
/// node itself, while `first_child` / `next_sibling` / `child_count`
/// describe its position in the container's child list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node<'a> {
    /// The node's payload.
    pub payload: Payload<'a>,
    /// The key under which the node is stored in its parent.
    pub key: Key<'a>,
    /// Auxiliary value: the u32 of a [`ValueHashMap`](Payload::ValueHashMap)
    /// entry; `0` otherwise.
    pub aux: u32,
    /// Index of the first child, or [`NodeId::none`] if childless.
    pub first_child: NodeId,
    /// Index of the next sibling, or [`NodeId::none`] if this is the last.
    pub next_sibling: NodeId,
    /// Number of children (containers only).
    pub child_count: u32,
}

impl<'a> Node<'a> {
    /// The name of this node's payload kind.
    pub fn kind(&self) -> &'static str {
        match self.payload {
            Payload::String(_) => "String",
            Payload::BinaryData(_) => "Binary",
            Payload::FileData(_) => "File",
            Payload::Array => "Array",
            Payload::Map => "Map",
            Payload::HashMap => "HashMap",
            Payload::ValueHashMap => "ValueHashMap",
            Payload::Bool(_) => "Bool",
            Payload::I32(_) => "I32",
            Payload::Float(_) => "Float",
            Payload::U32(_) => "U32",
            Payload::I64(_) => "I64",
            Payload::U64(_) => "U64",
            Payload::Double(_) => "Double",
            Payload::Null => "Null",
        }
    }

    /// Returns `true` for containers (`Array`, `Map`, `HashMap`,
    /// `ValueHashMap`).
    pub fn is_container(&self) -> bool {
        matches!(
            self.payload,
            Payload::Array | Payload::Map | Payload::HashMap | Payload::ValueHashMap
        )
    }

    /// The number of children of a container node.
    pub const fn child_count(&self) -> u32 {
        self.child_count
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> Result<&'a str> {
        match self.payload {
            Payload::String(s) => Ok(s),
            _ => Err(Error::TypeError(self.kind(), "String")),
        }
    }

    /// Get a reference to the inner byte slice.
    pub fn as_binary_data(&self) -> Result<&'a [u8]> {
        match self.payload {
            Payload::BinaryData(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "BinaryData")),
        }
    }

    /// Get a reference to the inner byte slice.
    pub fn as_file_data(&self) -> Result<&'a [u8]> {
        match self.payload {
            Payload::FileData(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "FileData")),
        }
    }

    /// Get a reference to the inner bool value.
    pub fn as_bool(&self) -> Result<bool> {
        match self.payload {
            Payload::Bool(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "Bool")),
        }
    }

    /// Get a reference to the inner i32 value.
    pub fn as_i32(&self) -> Result<i32> {
        match self.payload {
            Payload::I32(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "I32")),
        }
    }

    /// Get a reference to the inner u32 value.
    pub fn as_u32(&self) -> Result<u32> {
        match self.payload {
            Payload::U32(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "U32")),
        }
    }

    /// Get a reference to the inner i64 value.
    pub fn as_i64(&self) -> Result<i64> {
        match self.payload {
            Payload::I64(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "I64")),
        }
    }

    /// Get a reference to the inner u64 value.
    pub fn as_u64(&self) -> Result<u64> {
        match self.payload {
            Payload::U64(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "U64")),
        }
    }

    /// Get a reference to the inner f32 value.
    pub fn as_float(&self) -> Result<f32> {
        match self.payload {
            Payload::Float(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "Float")),
        }
    }

    /// Get a reference to the inner f64 value.
    pub fn as_double(&self) -> Result<f64> {
        match self.payload {
            Payload::Double(v) => Ok(v),
            _ => Err(Error::TypeError(self.kind(), "Double")),
        }
    }

    /// Returns `true` for null nodes.
    pub fn is_null(&self) -> bool {
        matches!(self.payload, Payload::Null)
    }
}

/// Backing store for a [`BymlArena`].
///
/// Implementations decide how node storage is managed: [`VecStore`]
/// grows like a `Vec`, [`ArrayStore`] uses a fixed-size array, and
/// [`SliceStore`] uses caller-provided memory. None of them require an
/// allocator beyond what the caller provides.
///
/// # Safety
///
/// Implementors must uphold the invariant that slots `[0..len)` are
/// initialized and slots `[len..capacity)` are uninitialized. `get` and
/// `get_mut` must only hand out references to initialized slots, and a slot
/// must be written exactly once (before it is read).
pub trait ArenaStore<'a> {
    /// Number of initialized nodes.
    fn len(&self) -> usize;

    /// Total number of slots (initialized or not).
    #[allow(dead_code)] // used by `ArrayStore`/`SliceStore` callers; kept for the trait contract
    fn capacity(&self) -> usize;

    /// Returns `true` if the store contains no nodes.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Append a node and return its index. Returns
    /// [`Error::CapacityExceeded`] when the store is full.
    fn push(&mut self, node: Node<'a>) -> Result<NodeId>;

    /// Get a shared reference to a node, or `None` if out of range.
    fn get(&self, id: NodeId) -> Option<&Node<'a>>;

    /// Get a mutable reference to a node, or `None` if out of range.
    ///
    /// Callers must not hold two live mutable borrows at once.
    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<'a>>;

    /// Allocate the next node id, checking for u32 overflow.
    fn next_id(&self) -> Result<NodeId> {
        u32::try_from(self.len())
            .map(NodeId)
            .map_err(|_| Error::CapacityExceeded(self.len()))
    }

    /// Discard all nodes, retaining capacity.
    fn clear(&mut self);
}

/// A growable store backed by `alloc::vec::Vec`.
///
/// [`VecStore`] has no fixed capacity: it grows like a `Vec` and is bounded
/// only by available memory, so [`push`](ArenaStore::push) never returns
/// [`Error::CapacityExceeded`] in practice (only if a node count ever
/// exceeded `u32::MAX`, which is unreachable for creatable stores).
///
/// TODO(step 4a): a hard capacity could be added here if callers need one;
/// `with_capacity(0)` currently means "unbounded", not "capacity 0".
#[cfg(feature = "alloc")]
pub struct VecStore<'a> {
    nodes: Vec<Node<'a>>,
}

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
impl<'a> Default for VecStore<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl<'a> VecStore<'a> {
    /// Create an empty, unbounded store.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Create an empty store with pre-allocated capacity (a hint only; the
    /// store still grows as needed).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> ArenaStore<'a> for VecStore<'a> {
    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    fn push(&mut self, node: Node<'a>) -> Result<NodeId> {
        let id = Self::next_id(self)?;
        self.nodes.push(node);
        Ok(id)
    }

    fn get(&self, id: NodeId) -> Option<&Node<'a>> {
        self.nodes.get(id.0 as usize)
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<'a>> {
        self.nodes.get_mut(id.0 as usize)
    }

    fn clear(&mut self) {
        self.nodes.clear();
    }
}

/// A fixed-capacity store backed by a stack- or statically-allocated array.
pub struct ArrayStore<'a, const N: usize> {
    slots: [core::mem::MaybeUninit<Node<'a>>; N],
    len: usize,
}

impl<'a, const N: usize> Default for ArrayStore<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const N: usize> ArrayStore<'a, N> {
    /// Create an empty store with room for `N` nodes.
    pub fn new() -> Self {
        Self {
            slots: [core::mem::MaybeUninit::uninit(); N],
            len: 0,
        }
    }
}

impl<'a, const N: usize> ArenaStore<'a> for ArrayStore<'a, N> {
    fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        N
    }

    fn push(&mut self, node: Node<'a>) -> Result<NodeId> {
        if self.len >= N {
            return Err(Error::CapacityExceeded(N));
        }
        let id = Self::next_id(self)?;
        // `MaybeUninit::write` is a safe fn; the unsafe invariant upheld
        // here is: `self.len < N` (slot in bounds and uninitialized), and
        // the slot is written exactly once before `len` is bumped, so it
        // joins the initialized prefix `[0..len)`.
        self.slots[self.len].write(node);
        self.len += 1;
        Ok(id)
    }

    fn get(&self, id: NodeId) -> Option<&Node<'a>> {
        let i = id.0 as usize;
        if i >= self.len {
            return None;
        }
        // SAFETY: `i < self.len`, so slot `i` has been written exactly
        // once and not yet dropped (nodes are never removed).
        unsafe { Some(self.slots[i].assume_init_ref()) }
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<'a>> {
        let i = id.0 as usize;
        if i >= self.len {
            return None;
        }
        // SAFETY: `i < self.len`, so slot `i` is initialized; the caller
        // must not hold another live borrow of this arena (see the trait
        // safety docs).
        unsafe { Some(self.slots[i].assume_init_mut()) }
    }

    fn clear(&mut self) {
        // Slots below `len` are abandoned, not dropped: `Node` is plain
        // copy data, so resetting `len` re-joins them to the
        // uninitialized region.
        self.len = 0;
    }
}

/// A store backed by a caller-provided mutable slice.
///
/// `SliceStore` never allocates: the caller supplies the backing memory
/// (e.g. a stack array or a statically allocated buffer), making it suitable
/// for `no_std` embedded contexts.
pub struct SliceStore<'s, 'a> {
    slots: &'s mut [core::mem::MaybeUninit<Node<'a>>],
    len: usize,
}

impl<'s, 'a> SliceStore<'s, 'a> {
    /// Create an empty store backed by `slots`.
    pub fn new(slots: &'s mut [core::mem::MaybeUninit<Node<'a>>]) -> Self {
        Self { slots, len: 0 }
    }
}

impl<'s, 'a> ArenaStore<'a> for SliceStore<'s, 'a> {
    fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        self.slots.len()
    }

    fn push(&mut self, node: Node<'a>) -> Result<NodeId> {
        if self.len >= self.slots.len() {
            return Err(Error::CapacityExceeded(self.slots.len()));
        }
        let id = Self::next_id(self)?;
        // `MaybeUninit::write` is a safe fn; the invariant upheld here is:
        // `self.len < self.slots.len()` (slot in bounds and uninitialized),
        // the slot is written exactly once, then `len` is bumped so it
        // joins the initialized prefix `[0..len)`.
        self.slots[self.len].write(node);
        self.len += 1;
        Ok(id)
    }

    fn get(&self, id: NodeId) -> Option<&Node<'a>> {
        let i = id.0 as usize;
        if i >= self.len {
            return None;
        }
        // SAFETY: `i < self.len`, so slot `i` is initialized.
        unsafe { Some(&*self.slots[i].as_ptr()) }
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<'a>> {
        let i = id.0 as usize;
        if i >= self.len {
            return None;
        }
        // SAFETY: `i < self.len`, so slot `i` is initialized.
        unsafe { Some(&mut *self.slots[i].as_mut_ptr()) }
    }

    fn clear(&mut self) {
        self.len = 0;
    }
}

/// A flat BYML document stored in `S`, with all payloads borrowed from the
/// input buffer.
///
/// The lifetime `'a` is carried by the store (`S: ArenaStore<'a>`), which
/// borrows the document's payloads; `BymlArena` itself adds no lifetime
/// parameter.
pub struct BymlArena<S> {
    store: S,
    root: Option<NodeId>,
}

impl<'a, S: ArenaStore<'a> + Default> BymlArena<S> {
    /// Read a document from binary data, borrowing all strings and
    /// payloads from `data`.
    ///
    /// **Note**: Yaz0-compressed input (`"Yaz0"` magic) is not
    /// auto-decompressed here. Decompress first with
    /// [`crate::yaz0::decompress_if`] (feature `yaz0`) and call
    /// `BymlArena::from_binary(cow.as_ref())`.
    ///
    /// Returns [`Error::CapacityExceeded`] if the document does not fit in
    /// `S`. This is available for stores that can be created without
    /// caller-provided memory ([`VecStore`], [`ArrayStore`]); use
    /// [`from_binary_in`](BymlArena::from_binary_in) for [`SliceStore`].
    ///
    /// # Stack use
    ///
    /// Constructing the store and returning the arena moves the whole
    /// inline store by value a few times, which debug builds do not elide.
    /// For large [`ArrayStore`] capacities (or very small stacks), prefer
    /// creating the arena with [`new`](BymlArena::new) and parsing with
    /// [`parse_in_place`](BymlArena::parse_in_place) (no large moves), or
    /// use [`from_binary_in`](BymlArena::from_binary_in) with a
    /// [`SliceStore`].
    pub fn from_binary(data: &'a [u8]) -> Result<Self> {
        let mut this = Self {
            store: S::default(),
            root: None,
        };
        this.parse_in_place(data)?;
        Ok(this)
    }

    /// Create an empty arena over a default-constructed store.
    pub fn new() -> Self {
        Self {
            store: S::default(),
            root: None,
        }
    }

    /// Parse `data` into this arena, discarding any previous contents.
    ///
    /// All strings and payloads are borrowed from `data`; nothing is
    /// allocated and the store is never moved. Returns
    /// [`Error::CapacityExceeded`] if the document does not fit in the
    /// store.
    /// On error the arena is left empty (`root()` returns `None`).
    pub fn parse_in_place(&mut self, data: &'a [u8]) -> Result<()> {
        self.store.clear();
        self.root = None;
        let root = {
            let mut sink = ArenaSink {
                store: &mut self.store,
                stack: [(NodeId::none(), None); BYML_MAX_DEPTH],
                depth: 0,
            };
            reader_mod::parse_byml(data, &mut sink)?
        };
        self.root = Some(root);
        Ok(())
    }
}

impl<'s, 'a> BymlArena<SliceStore<'s, 'a>> {
    /// Read a document into caller-provided `slots`, borrowing all strings
    /// and payloads from `data`. No allocation occurs.
    ///
    /// Returns [`Error::CapacityExceeded`] if the document does not fit in
    /// `slots`.
    pub fn from_binary_in(
        data: &'a [u8],
        slots: &'s mut [core::mem::MaybeUninit<Node<'a>>],
    ) -> Result<Self> {
        let mut store = SliceStore::new(slots);
        let root = {
            let mut sink = ArenaSink {
                store: &mut store,
                stack: [(NodeId::none(), None); BYML_MAX_DEPTH],
                depth: 0,
            };
            reader_mod::parse_byml(data, &mut sink)?
        };
        Ok(BymlArena {
            store,
            root: Some(root),
        })
    }
}

impl<'a, S: ArenaStore<'a> + Default> Default for BymlArena<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S: ArenaStore<'a>> BymlArena<S> {
    /// The root node id, or `None` for an empty document (root offset 0,
    /// which this reader represents as a single Null node).
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Get a shared reference to a node by id.
    ///
    /// The returned reference is borrowed from the arena and lives as long
    /// as the `&self` borrow.
    pub fn node(&self, id: NodeId) -> Option<&Node<'a>> {
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
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = (Key<'a>, NodeId)> + '_ {
        let first = self.node(id).and_then(|node| {
            (!node.first_child.is_none()).then_some(node.first_child)
        });
        core::iter::successors(first, move |current: &NodeId| {
            let node = self.node(*current)?;
            let next = node.next_sibling;
            (!next.is_none()).then_some(next)
        })
        .filter_map(move |id| self.node(id).map(|node| (node.key, id)))
    }

    /// Find a string-keyed child of `container` by linear scan.
    pub fn get(&self, id: NodeId, key: &str) -> Option<NodeId> {
        self.children(id)
            .find(|(k, _)| matches!(k, Key::Str(s) if *s == key))
            .map(|(_, id)| id)
    }

    /// Find a u32-hash-keyed child of `container` by linear scan.
    pub fn get_hash(&self, id: NodeId, hash: u32) -> Option<NodeId> {
        self.children(id)
            .find(|(k, _)| matches!(k, Key::Hash(h) if *h == hash))
            .map(|(_, id)| id)
    }

    /// Get the child of `container` at `index` (in document order).
    pub fn child(&self, id: NodeId, index: usize) -> Option<NodeId> {
        self.children(id).nth(index).map(|(_, id)| id)
    }
}

/// A fixed-capacity arena backed by an [`ArrayStore`].
pub type BymlStack<'a, const N: usize> = BymlArena<ArrayStore<'a, N>>;

/// The sink used by [`BymlArena::from_binary`].
struct ArenaSink<'b, S> {
    store: &'b mut S,
    /// One entry per open container: the container's node id and the id of
    /// the last child appended to it. `stack[self.depth - 1]` is the
    /// innermost open container while `depth > 0`.
    stack: [(NodeId, Option<NodeId>); BYML_MAX_DEPTH],
    /// Number of open containers; the next `begin_container` writes to
    /// `stack[self.depth]`.
    depth: usize,
}

impl<'b, 'a, S: ArenaStore<'a>> ArenaSink<'b, S> {
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

impl<'b, 'a, S: ArenaStore<'a>> BymlSink<'a> for ArenaSink<'b, S> {
    type Node = NodeId;

    fn leaf(&mut self, payload: Leaf<'a>, key: Key<'a>, aux: u32) -> Result<NodeId> {
        let payload = match payload {
            Leaf::String(s) => Payload::String(s),
            Leaf::BinaryData(b) => Payload::BinaryData(b),
            Leaf::FileData(b) => Payload::FileData(b),
            Leaf::Bool(b) => Payload::Bool(b),
            Leaf::I32(i) => Payload::I32(i),
            Leaf::Float(f) => Payload::Float(f),
            Leaf::U32(u) => Payload::U32(u),
            Leaf::I64(i) => Payload::I64(i),
            Leaf::U64(u) => Payload::U64(u),
            Leaf::Double(d) => Payload::Double(d),
            Leaf::Null => Payload::Null,
        };
        let node = Node {
            payload,
            key,
            aux,
            first_child: NodeId::none(),
            next_sibling: NodeId::none(),
            child_count: 0,
        };
        // Children are linked to their parent by `add_child`, which the
        // walk calls after every child; a leaf at the top of the document
        // (e.g. a Null root) has no parent and is not linked.
        self.store.push(node)
    }

    fn begin_container(
        &mut self,
        kind: ContainerKind,
        key: Key<'a>,
        aux: u32,
    ) -> Result<()> {
        debug_assert!(self.depth < BYML_MAX_DEPTH);
        let payload = match kind {
            ContainerKind::Array => Payload::Array,
            ContainerKind::Map => Payload::Map,
            ContainerKind::HashMap => Payload::HashMap,
            ContainerKind::ValueHashMap => Payload::ValueHashMap,
        };
        let node = Node {
            payload,
            key,
            aux,
            first_child: NodeId::none(),
            next_sibling: NodeId::none(),
            child_count: 0,
        };
        let id = self.store.push(node)?;
        self.stack[self.depth] = (id, None);
        self.depth += 1;
        Ok(())
    }

    fn add_child(&mut self, child: NodeId, _key: Key<'a>, _aux: u32) -> Result<()> {
        // The child's key/aux were stored when the child node was created
        // (by `leaf` or `begin_container`); here we only link siblings.
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
