//! Port of the `oead::byml` module.
//!
//! The BYML module is compiled whenever the `byml` feature is on. It
//! contains three layers:
//!
//! - The [owned tree model][`Byml`] (`owned`), the binrw-based
//!   [`parser`], and the [`writer`] require `alloc` and produce/consume
//!   owned [`Byml`] values.
//! - [`reader`] contains the shared BYML format walk. It is core-only so
//!   that it can be used without an allocator.
//! - [`arena`] is core-only and provides flat, fixed-capacity BYML readers
//!   ([`BymlArena`]) that borrow all payloads from the input buffer and
//!   work with zero features.
//!
//! With `std` + `yaml`, the YAML layer (`text` for the owned model,
//! `ref_text` for the borrowed readers) emits byte-identical YAML from
//! either representation: `Byml::to_text`, `BymlRef::to_text` and
//! `BymlArena::to_text`.
//!
//! Without `alloc`, [`BymlArena`] borrows all payloads from the input
//! buffer and needs no allocator at all:
//! ```
//! # use roead::{byml::arena::*, Endian};
//! # fn doctest() -> Result<(), roead::Error> {
//! // A little-endian array of one `Bool(true)` node.
//! let data = [
//!     0x59, 0x42, 0x02, 0x00,   // "YB", version 2
//!     0x00, 0x00, 0x00, 0x00,   // hash key table offset 0
//!     0x00, 0x00, 0x00, 0x00,   // string table offset 0
//!     0x10, 0x00, 0x00, 0x00,   // root node offset 0x10
//!     0xC0, 0x01, 0x00, 0x00,   // Array, size 1
//!     0xD0, 0x00, 0x00, 0x00,   // child tag: Bool, padding
//!     0x01, 0x00, 0x00, 0x00,   // value: true
//! ];
//! let arena = BymlArena::<ArrayStore<8>>::from_binary(&data)?;
//! let root = arena.root().expect("root node");
//! let child = arena.child(root, 0).expect("first child");
//! assert_eq!(arena.node(child).unwrap().as_bool()?, true);
//! # Ok(())
//! # }
//! ```
//! Most of the node types are fairly self-explanatory. Arrays are implemented
//! as `Vec<Byml>`, and maps as `FxHashMap<String, Byml>`. The new v7 hash maps
//! are `FxHashMap<u32, Byml>` and `FxHashMap<u32, (Byml, u32)>`.
//!
//! For convenience, an owned `Byml` *known* to be an array or map can be
//! indexed. **Panics if the node has the wrong type, the index has the wrong
//! type, or the index is not found**.

//! Zero-copy arena readers are documented on [`arena::BymlArena`].

#[cfg(all(test, feature = "alloc"))]
static FILES: &[&str] = &[
    "A-1_Dynamic",
    "D-3_Dynamic",
    "EventInfo.product",
    "GameROMPlayer",
    "LevelSensor",
    "MainFieldLocation",
    "MainFieldStatic",
    "Preset0_Field",
    "ActorInfo.product",
    "ElectricGenerator.Nin_NX_NVN.esetb",
    "USen",
    "Mrg_01e57204_MrgD100_B4-B3-B2-1A90E17A.bcett",
];

#[cfg(feature = "yaml")]
mod text;

#[cfg(feature = "yaml")]
mod ref_text;

#[cfg(feature = "alloc")]
mod owned;
mod reader;
#[cfg(feature = "alloc")]
mod parser;
#[cfg(feature = "alloc")]
mod writer;

pub mod arena;

#[cfg(feature = "alloc")]
pub(crate) use owned::{NodeType, is_container_type, is_valid_version};

#[cfg(feature = "alloc")]
pub use owned::{Byml, Map, HashMap, ValueHashMap, BymlIndex};
// The `map!`/`array!` macros are defined in `owned` (with `#[macro_export]`,
// so they live at the crate root); re-export them at `byml::map` /
// `byml::array` so `use roead::byml::*; map!()` still works.
#[cfg(feature = "alloc")]
pub use crate::{map, array};

#[cfg(feature = "alloc")]
pub use reader::BymlRef;

pub use arena::{NodeId, Key, Payload, Node, BymlArena, BymlStack, ArrayStore, SliceStore};
#[cfg(feature = "alloc")]
pub use arena::VecStore;
