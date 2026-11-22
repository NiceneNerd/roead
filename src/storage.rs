//! Generic node stores shared by the flat arena readers
//! ([`crate::byml::arena`] and [`crate::aamp::arena`]).
//!
//! All stores hold nodes of type `T` in a contiguous, never-merged backing
//! and hand out [`NodeId`] indices. Nodes are written exactly once and never
//! removed or moved, so references into a stable prefix of the store remain
//! valid for the lifetime of the borrow.
//!
//! This module is core-only: none of the stores require an allocator beyond
//! what the caller provides.

use crate::{Error, Result};

/// A node index into a [`Store`]. `NodeId::none()` (`u32::MAX`) is the
/// "no node" sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

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

/// Backing store for nodes of type `T`.
///
/// Implementations decide how node storage is managed: [`OwnedStore`] grows
/// like a `Vec`, [`FixedStore`] uses a fixed-size array, and
/// [`BorrowedStore`] uses caller-provided memory. None of them require an
/// allocator beyond what the caller provides.
///
/// # Safety
///
/// Implementors must uphold the invariant that slots `[0..len)` are
/// initialized and slots `[len..capacity)` are uninitialized. `get` and
/// `get_mut` must only hand out references to initialized slots, and a slot
/// must be written exactly once (before it is read).
pub trait Store<T> {
    /// Number of initialized nodes.
    fn len(&self) -> usize;

    /// Total number of slots (initialized or not).
    #[allow(dead_code)] // used by `FixedStore`/`BorrowedStore` callers; kept for the trait contract
    fn capacity(&self) -> usize;

    /// Returns `true` if the store contains no nodes.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Append a node and return its index. Returns
    /// [`Error::CapacityExceeded`] when the store is full.
    fn push(&mut self, node: T) -> Result<NodeId>;

    /// Discard all nodes, retaining capacity.
    fn clear(&mut self);

    /// Get a shared reference to a node, or `None` if out of range.
    fn get(&self, id: NodeId) -> Option<&T>;

    /// Get a mutable reference to a node, or `None` if out of range.
    ///
    /// Callers must not hold two live mutable borrows at once.
    fn get_mut(&mut self, id: NodeId) -> Option<&mut T>;

    /// Allocate the next node id, checking for `u32` overflow.
    fn next_id(&self) -> Result<NodeId> {
        u32::try_from(self.len())
            .map(NodeId)
            .map_err(|_| Error::CapacityExceeded(self.len()))
    }
}

/// A growable store backed by `alloc::vec::Vec`.
///
/// [`OwnedStore`] has no fixed capacity: it grows like a `Vec` and is bounded
/// only by available memory, so [`push`](Store::push) never returns
/// [`Error::CapacityExceeded`] in practice (only if a node count ever
/// exceeded `u32::MAX`, which is unreachable for creatable stores).
#[cfg(feature = "alloc")]
pub struct OwnedStore<T> {
    nodes: Vec<T>,
}

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
impl<T> Default for OwnedStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl<T> OwnedStore<T> {
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
impl<T> Store<T> for OwnedStore<T> {
    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    fn push(&mut self, node: T) -> Result<NodeId> {
        let id = Self::next_id(self)?;
        self.nodes.push(node);
        Ok(id)
    }

    fn clear(&mut self) {
        self.nodes.clear();
    }

    fn get(&self, id: NodeId) -> Option<&T> {
        self.nodes.get(id.0 as usize)
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.nodes.get_mut(id.0 as usize)
    }
}

/// A fixed-capacity store backed by a stack- or statically-allocated array.
pub struct FixedStore<T, const N: usize> {
    slots: [core::mem::MaybeUninit<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> Default for FixedStore<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> FixedStore<T, N> {
    /// Create an empty store with room for `N` nodes.
    pub fn new() -> Self {
        Self {
            slots: [core::mem::MaybeUninit::uninit(); N],
            len: 0,
        }
    }
}

impl<T, const N: usize> Store<T> for FixedStore<T, N> {
    fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        N
    }

    fn push(&mut self, node: T) -> Result<NodeId> {
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

    fn clear(&mut self) {
        // Slots below `len` are simply abandoned: `T` is never dropped
        // (nodes are plain data), so resetting `len` re-joins them to the
        // uninitialized region.
        self.len = 0;
    }

    fn get(&self, id: NodeId) -> Option<&T> {
        let i = id.0 as usize;
        if i >= self.len {
            return None;
        }
        // SAFETY: `i < self.len`, so slot `i` has been written exactly
        // once and not yet dropped (nodes are never removed).
        unsafe { Some(self.slots[i].assume_init_ref()) }
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let i = id.0 as usize;
        if i >= self.len {
            return None;
        }
        // SAFETY: `i < self.len`, so slot `i` is initialized; the caller
        // must not hold another live borrow of this arena (see the trait
        // safety docs).
        unsafe { Some(self.slots[i].assume_init_mut()) }
    }
}

/// A store backed by a caller-provided mutable slice.
///
/// [`BorrowedStore`] never allocates: the caller supplies the backing memory
/// (e.g. a stack array or a statically allocated buffer), making it suitable
/// for `no_std` embedded contexts.
pub struct BorrowedStore<'s, T> {
    slots: &'s mut [core::mem::MaybeUninit<T>],
    len: usize,
}

impl<'s, T> BorrowedStore<'s, T> {
    /// Create an empty store backed by `slots`.
    pub fn new(slots: &'s mut [core::mem::MaybeUninit<T>]) -> Self {
        Self { slots, len: 0 }
    }
}

impl<'s, T> Store<T> for BorrowedStore<'s, T> {
    fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        self.slots.len()
    }

    fn push(&mut self, node: T) -> Result<NodeId> {
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

    fn clear(&mut self) {
        self.len = 0;
    }

    fn get(&self, id: NodeId) -> Option<&T> {
        let i = id.0 as usize;
        if i >= self.len {
            return None;
        }
        // SAFETY: `i < self.len`, so slot `i` is initialized.
        unsafe { Some(&*self.slots[i].as_ptr()) }
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        let i = id.0 as usize;
        if i >= self.len {
            return None;
        }
        // SAFETY: `i < self.len`, so slot `i` is initialized.
        unsafe { Some(&mut *self.slots[i].as_mut_ptr()) }
    }
}
