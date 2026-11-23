//! Tests for the zero-copy BYML readers ([`BymlRef`] and [`BymlArena`]).
//!
//! This file is compiled whenever the `byml` feature is on. Tests that
//! allocate ([`BymlRef`], `to_owned`, comparisons against the owned [`Byml`])
//! are additionally gated on `feature = "alloc"`, so the pure core/arena
//! subset runs under `--no-default-features --features byml`.

#![cfg(feature = "byml")]

use core::mem::MaybeUninit;
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use roead::byml::arena::*;
#[cfg(feature = "alloc")]
use roead::byml::Byml;
#[cfg(feature = "alloc")]
use roead::byml::BymlRef;
use roead::{Endian, Error};

/// Path of a file inside `test/byml/`.
fn byml_file(name: &str) -> PathBuf {
    PathBuf::from("test/byml").join(name)
}

/// Read a `test/byml/` file (panics on IO error; tests run from the crate
/// root).
fn load(name: &str) -> Vec<u8> {
    fs::read(byml_file(name)).unwrap()
}

/// Every `*.byml` file in `test/byml/`, sorted.
fn corpus_files() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir("test/byml")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("byml"))
        .collect();
    out.sort();
    out
}

/// Build a BYML header (16 bytes). Offset fields are u32 in the document's
/// endianness.
fn header(endian: Endian, version: u16, hkt: u32, st: u32, root: u32) -> Vec<u8> {
    let mut out = match endian {
        Endian::Big => b"BY".to_vec(),
        Endian::Little => b"YB".to_vec(),
    };
    out.extend_from_slice(&match endian {
        Endian::Big => version.to_be_bytes(),
        Endian::Little => version.to_le_bytes(),
    });
    for v in [hkt, st, root] {
        out.extend_from_slice(&match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        });
    }
    out
}

/// Decompress Yaz0 input when the `yaz0` and `alloc` features are on
/// (`decompress_if` allocates); otherwise pass through. `D-3_Dynamic.byml`
/// is Yaz0-compressed, so tests skip it unless `yaz0_can_decompress()`.
fn prepared(data: &[u8]) -> Cow<'_, [u8]> {
    // `#[cfg]` on `let` bindings: the Yaz0 branch is not type-checked
    // without the feature, so `roead::yaz0` need not exist.
    #[cfg(all(feature = "yaz0", feature = "alloc"))]
    let data = roead::yaz0::decompress_if(data);
    #[cfg(not(all(feature = "yaz0", feature = "alloc")))]
    let data = Cow::Borrowed(data);
    data
}

/// Whether `D-3_Dynamic.byml` can be decompressed in this build.
fn yaz0_can_decompress() -> bool {
    cfg!(all(feature = "yaz0", feature = "alloc"))
}

fn parse_arena<'a, S: Store<Node<'a>> + Default>(data: &'a [u8]) -> BymlArena<S> {
    BymlArena::from_binary(data).unwrap()
}

/// Parse and return the error, panicking if the document unexpectedly
/// parses successfully. Avoids `Result::unwrap_err`, which requires
/// `BymlArena<T>: Debug` (unimplemented for fixed-capacity stores).
fn expect_err<'a, S: Store<Node<'a>> + Default>(data: &'a [u8]) -> Error {
    match BymlArena::<S>::from_binary(data) {
        Err(err) => err,
        Ok(_) => panic!("expected a parse error"),
    }
}

fn expect_err_in<'a, 's>(
    data: &'a [u8],
    slots: &'s mut [MaybeUninit<Node<'a>>],
) -> Error {
    match BymlArena::<BorrowedStore<'s, Node<'a>>>::from_binary_in(data, slots) {
        Err(err) => err,
        Ok(_) => panic!("expected a parse error"),
    }
}

// ---------------------------------------------------------------------------
// Core-only arena tests (no allocator)
// ---------------------------------------------------------------------------

#[test]
fn arena_generic_numeric_accessors() {
    let node = Node {
        payload: Payload::I32(-3),
        key: Key::None,
        aux: 0,
        first_child: NodeId::none(),
        next_sibling: NodeId::none(),
        child_count: 0,
    };
    assert_eq!(node.as_int::<i64>().unwrap(), -3);
    assert_eq!(node.as_int::<u8>().unwrap(), 253); // `as`-semantics wrap
    assert_eq!(node.as_num::<f32>().unwrap(), -3.0);

    let double = Node {
        payload: Payload::Double(2.5),
        ..node
    };
    assert_eq!(double.as_num::<f64>().unwrap(), 2.5);
    assert_eq!(double.as_num::<f32>().unwrap(), 2.5);
    assert!(matches!(
        double.as_int::<i32>(),
        Err(Error::TypeError(..))
    ));

    let string = Node {
        payload: Payload::String("1"),
        ..node
    };
    assert!(matches!(
        string.as_num::<f32>(),
        Err(Error::TypeError(..))
    ));
}

#[test]
fn arena_smoke() {
    // A little-endian array of [Bool(true), I32(42)].
    let mut doc = header(Endian::Little, 2, 0, 0, 0x10);
    doc.push(0xC0); // Array
    doc.extend_from_slice(&2u32.to_le_bytes()[..3]); // size = 2 (u24, LE)
    doc.push(0xD0); // child 0: Bool
    doc.push(0xD1); // child 1: I32
    // align(2, 4) == 4 padding bytes, then the two u32 values.
    doc.resize(0x10 + 4 + 4, 0);
    doc.extend_from_slice(&1u32.to_le_bytes());
    doc.extend_from_slice(&42u32.to_le_bytes());

    let arena = parse_arena::<FixedStore<Node, 8>>(&doc);
    let root = arena.root().expect("root node");
    assert_eq!(arena.len(), 3);
    assert!(!arena.is_empty());

    let children: Vec<_> = arena.children(root).collect();
    assert_eq!(
        children,
        [
            (Key::None, NodeId::from(1)),
            (Key::None, NodeId::from(2)),
        ]
    );
    assert_eq!(arena.child(root, 0), Some(NodeId::from(1)));
    assert_eq!(arena.child(root, 1), Some(NodeId::from(2)));
    assert_eq!(arena.child(root, 2), None);
    assert_eq!(arena.get(root, "anything"), None);
    assert_eq!(arena.get_hash(root, 42), None);

    let bool_node = arena.node(NodeId::from(1)).expect("node 1");
    assert!(!bool_node.is_container());
    assert_eq!(bool_node.kind(), "Bool");
    assert!(bool_node.as_bool().unwrap());
    let i32_node = arena.node(NodeId::from(2)).expect("node 2");
    assert_eq!(i32_node.as_i32().unwrap(), 42);
}

#[test]
fn arena_null_root() {
    // Root offset 0: an empty document reads as a single Null node.
    let doc = header(Endian::Big, 2, 0, 0, 0);
    let arena = parse_arena::<FixedStore<Node, 4>>(&doc);
    let root = arena.root().expect("root node");
    assert_eq!(arena.len(), 1);
    assert_eq!(arena.node(root).unwrap().payload, Payload::Null);
    assert!(arena.node(root).unwrap().is_null());
    assert_eq!(arena.children(root).count(), 0);
}

#[test]
fn arena_equivalence_across_corpus() {
    for path in corpus_files() {
        let name = path.file_name().unwrap().to_str().unwrap();
        if name == "D-3_Dynamic.byml" && !yaz0_can_decompress() {
            continue;
        }
        let mut data = load(name);
        if name == "D-3_Dynamic.byml" {
            // Replace owned bytes with the decompressed payload for parsing.
            data = match prepared(&data) {
                Cow::Owned(owned) => owned,
                Cow::Borrowed(_) => continue,
            };
        }

        // Reference arena: a heap-allocated slice store, grown until the
        // document fits. Core-only: no allocator beyond the test harness.
        let mut buffer: Vec<Node<'_>> = Vec::with_capacity(1024);
        let ref_arena = loop {
            let slots = buffer.spare_capacity_mut();
            match BymlArena::<BorrowedStore<'_, Node<'_>>>::from_binary_in(&data, slots) {
                Ok(arena) => break arena,
                Err(Error::CapacityExceeded(_)) => buffer.reserve_exact(buffer.capacity() * 2),
                Err(err) => panic!("unexpected error: {err}"),
            }
        };
        let node_count = ref_arena.len();

        // A fixed-capacity array store, when the document is small enough
        // to keep its backing array on the stack.
        if node_count <= 4096 {
            let arr_arena = parse_arena::<FixedStore<Node, 4096>>(&data);
            assert_arena_equal(&ref_arena, &arr_arena);
            assert_eq!(node_count, arr_arena.len());
        }

        // A second slice store with exactly the needed capacity.
        let mut buffer: Vec<Node<'_>> = Vec::with_capacity(node_count);
        let slots = buffer.spare_capacity_mut();
        let slice_arena =
            BymlArena::<BorrowedStore<'_, Node<'_>>>::from_binary_in(&data, slots).unwrap();
        assert_arena_equal(&ref_arena, &slice_arena);
        assert_eq!(node_count, slice_arena.len());

        // Sanity: the ActorInfo map has 7934 actors.
        if name == "ActorInfo.product.byml" {
            let actors = ref_arena
                .root()
                .and_then(|root| ref_arena.get(root, "Actors"))
                .expect("Actors entry");
            assert_eq!(ref_arena.node(actors).unwrap().child_count(), 7934);
        }
    }
}

// ---------------------------------------------------------------------------
// Payload and traversal comparison helpers
// ---------------------------------------------------------------------------

fn assert_payload_eq(a: &Payload<'_>, b: &Payload<'_>) {
    match (a, b) {
        (Payload::String(s1), Payload::String(s2)) => assert_eq!(*s1, *s2),
        (Payload::BinaryData(d1), Payload::BinaryData(d2)) => assert_eq!(*d1, *d2),
        (Payload::FileData(d1), Payload::FileData(d2)) => assert_eq!(*d1, *d2),
        (Payload::Bool(v1), Payload::Bool(v2)) => assert_eq!(*v1, *v2),
        (Payload::I32(v1), Payload::I32(v2)) => assert_eq!(*v1, *v2),
        (Payload::U32(v1), Payload::U32(v2)) => assert_eq!(*v1, *v2),
        (Payload::I64(v1), Payload::I64(v2)) => assert_eq!(*v1, *v2),
        (Payload::U64(v1), Payload::U64(v2)) => assert_eq!(*v1, *v2),
        (Payload::Float(v1), Payload::Float(v2)) => assert_eq!((*v1).to_bits(), (*v2).to_bits()),
        (Payload::Double(v1), Payload::Double(v2)) => assert_eq!((*v1).to_bits(), (*v2).to_bits()),
        (Payload::Array, Payload::Array)
        | (Payload::Map, Payload::Map)
        | (Payload::HashMap, Payload::HashMap)
        | (Payload::ValueHashMap, Payload::ValueHashMap)
        | (Payload::Null, Payload::Null) => {}
        _ => panic!("payload mismatch: {a:?} vs {b:?}"),
    }
}

fn assert_arena_equal<'a, 'b, S1: Store<Node<'a>>, S2: Store<Node<'b>>>(
    a: &BymlArena<S1>,
    b: &BymlArena<S2>,
) {
    match (a.root(), b.root()) {
        (None, None) => {}
        (Some(id1), Some(id2)) => {
            // Iterative worklist walk: recursion could overflow the stack on
            // deeply nested documents.
            let mut stack: Vec<(NodeId, NodeId)> = vec![(id1, id2)];
            while let Some((id1, id2)) = stack.pop() {
                let n1 = a
                    .node(id1)
                    .unwrap_or_else(|| panic!("node {id1:?} missing from first arena"));
                let n2 = b
                    .node(id2)
                    .unwrap_or_else(|| panic!("node {id2:?} missing from second arena"));
                assert_eq!(n1.kind(), n2.kind(), "kind mismatch between arenas");
                assert_payload_eq(&n1.payload, &n2.payload);
                assert_eq!(
                    n1.child_count(),
                    n2.child_count(),
                    "child count mismatch between arenas"
                );
                let c1: Vec<_> = a.children(id1).collect();
                let c2: Vec<_> = b.children(id2).collect();
                assert_eq!(c1.len(), c2.len(), "children count mismatch between arenas");
                for ((key1, child1), (key2, child2)) in c1.iter().zip(c2.iter()) {
                    assert_eq!(key1, key2, "child key mismatch between arenas");
                    stack.push((*child1, *child2));
                }
            }
        }
        _ => panic!("root presence mismatch between arenas"),
    }
}

// ---------------------------------------------------------------------------
// Capacity and malformed-input tests (alloc-free errors, no panics)
// ---------------------------------------------------------------------------

/// A little-endian array of `n` I32 children (0..n).
fn array_of_n_le(n: u32) -> Vec<u8> {
    let mut doc = header(Endian::Little, 2, 0, 0, 0x10);
    doc.push(0xC0);
    doc.extend_from_slice(&n.to_le_bytes()[..3]); // size (u24, LE)
    doc.extend(std::iter::repeat_n(0xD1, n as usize)); // I32 children
    let values_offset = 0x10usize + 4 + align_usize(n, 4) as usize;
    doc.resize(values_offset, 0);
    for i in 0..n {
        doc.extend_from_slice(&i.to_le_bytes());
    }
    doc
}

fn align_usize(value: u32, size: u32) -> u32 {
    value + (size - value % size) % size
}

#[test]
fn capacity_exceeded() {
    let doc = array_of_n_le(9); // 10 nodes total (container + 9 children)

    // Fixed-capacity stores overflow.
    let err = expect_err::<FixedStore<Node, 8>>(&doc);
    assert!(matches!(err, Error::CapacityExceeded(8)));

    let mut slots = [MaybeUninit::<Node>::uninit(); 8];
    let err = expect_err_in(&doc, &mut slots);
    assert!(matches!(err, Error::CapacityExceeded(8)));

    // With 10 slots the document fits.
    let mut slots = [MaybeUninit::uninit(); 10];
    let arena = BymlArena::<BorrowedStore<'_, Node<'_>>>::from_binary_in(&doc, &mut slots).unwrap();
    assert_eq!(arena.len(), 10);
}

#[test]
fn malformed_inputs() {
    // Truncated header (< 0x10 bytes).
    let err = expect_err::<FixedStore<Node, 64>>(b"BY");
    assert!(matches!(err, Error::InsufficientData(_, _)));

    // Bad magic.
    let mut bad = [0u8; 16];
    bad[..2].copy_from_slice(b"XX");
    let err = expect_err::<FixedStore<Node, 64>>(&bad);
    assert!(matches!(err, Error::BadMagic(_, _)));

    // Unsupported version 8.
    let ver8 = header(Endian::Big, 8, 0, 0, 0);
    let err = expect_err::<FixedStore<Node, 64>>(&ver8);
    assert!(matches!(err, Error::InvalidData(_)));
    assert_eq!(err.to_string(), "Unsupported BYML version (1-7 only)");

    // Root node offset beyond EOF.
    let mut oob = header(Endian::Big, 2, 0, 0, 0x1000);
    oob.truncate(16);
    let err = expect_err::<FixedStore<Node, 64>>(&oob);
    assert!(matches!(err, Error::InsufficientData(_, _)));

    // String table index out of range: hash key table with 0 entries, root
    // map entry referencing name index 0.
    let mut st_oor = header(Endian::Big, 2, 0x10, 0, 0x14);
    st_oor.extend_from_slice(&[0xC2, 0x00, 0x00, 0x00]); // hash key table (a StringTable node), size 0
    st_oor.extend_from_slice(&[0xC1, 0x01, 0x00, 0x00]); // Map, size 1
    // entry: name_idx=0, type=String, value=0
    st_oor.extend_from_slice(&[0x00, 0x00, 0x00, 0xA0, 0x00, 0x00, 0x00, 0x00]);
    let err = expect_err::<FixedStore<Node, 64>>(&st_oor);
    assert!(matches!(
        err,
        Error::InvalidData("Invalid string table entry index")
    ));

    // Invalid UTF-8 in the string table: root array holds one String child
    // referencing string table index 0, whose bytes are not valid UTF-8.
    let mut bad_utf8 = header(Endian::Big, 2, 0, 0x1C, 0x30);
    bad_utf8.resize(0x1C, 0);
    bad_utf8.extend_from_slice(&[0xC2, 0x00, 0x00, 0x01]); // 0x1C: string table, size 1 (BE u24)
    bad_utf8.extend_from_slice(&0x0Cu32.to_be_bytes()); // 0x20: off[0] = 0x0C (BE)
    bad_utf8.extend_from_slice(&0x0Fu32.to_be_bytes()); // 0x24: off[1] = 0x0F (max_len 3, BE)
    bad_utf8.extend_from_slice(&[0xFF, 0xFE, 0x00]); // 0x28: invalid UTF-8, NUL-terminated
    bad_utf8.resize(0x30, 0);
    bad_utf8.extend_from_slice(&[0xC0, 0x00, 0x00, 0x01]); // 0x30: Array, size 1 (BE u24)
    bad_utf8.push(0xA0); // 0x34: child type tag: String
    bad_utf8.resize(0x3C, 0);
    bad_utf8.extend_from_slice(&0u32.to_be_bytes()); // 0x3C: raw = string index 0
    let err = expect_err::<FixedStore<Node, 64>>(&bad_utf8);
    assert!(matches!(err, Error::InvalidUtf8(_)));

    // A string payload offset beyond EOF must error rather than panic.
    let mut bad_string_offset = bad_utf8.clone();
    bad_string_offset[0x20..0x24].copy_from_slice(&0x1000u32.to_be_bytes());
    bad_string_offset[0x24..0x28].copy_from_slice(&0x1001u32.to_be_bytes());
    let err = expect_err::<FixedStore<Node, 64>>(&bad_string_offset);
    assert!(matches!(err, Error::InsufficientData(_, _)));

    // Binary payload extending past EOF.
    let mut bin_past = header(Endian::Big, 2, 0, 0, 0x10);
    bin_past.push(0xC0); // Array
    bin_past.extend_from_slice(&1u32.to_be_bytes()[1..]); // size 1 (u24 BE)
    bin_past.push(0xA1); // child: BinaryData
    bin_past.resize(0x20, 0);
    bin_past.extend_from_slice(&0x20u32.to_be_bytes()); // value: offset 0x20
    bin_past.extend_from_slice(&0x100u32.to_be_bytes()); // size 0x100 at 0x20 (data would run past EOF)
    let err = expect_err::<FixedStore<Node, 64>>(&bin_past);
    assert!(matches!(err, Error::InsufficientData(_, _)));
}

#[test]
fn self_referential_container() {
    // Array of size 1 whose sole child (a container) points back at itself.
    let mut doc = header(Endian::Big, 2, 0, 0, 0x10);
    doc.push(0xC0); // Array
    doc.extend_from_slice(&1u32.to_be_bytes()[1..]); // size 1 (u24 BE)
    doc.push(0xC0); // child type tag: container (Array)
    doc.resize(0x18, 0); // values_offset = 0x10 + 4 + align(1, 4) = 0x18
    doc.extend_from_slice(&0x10u32.to_be_bytes()); // child container offset = 0x10 (self)

    // The arena trips its capacity error first.
    let err = expect_err::<FixedStore<Node, 8>>(&doc);
    assert!(matches!(err, Error::CapacityExceeded(8)));
}

// ---------------------------------------------------------------------------
// Alloc-backed tests (BymlRef, owned model comparison)
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
mod alloc_tests {
    use super::*;

    #[test]
    fn generic_numeric_accessors() {
        assert_eq!(BymlRef::U32(7).as_int::<i64>().unwrap(), 7);
        assert_eq!(BymlRef::I32(-3).as_int::<u8>().unwrap(), 253);
        assert_eq!(BymlRef::I64(9).as_num::<f32>().unwrap(), 9.0);
        assert_eq!(BymlRef::Float(1.5).as_num::<f64>().unwrap(), 1.5);
        assert_eq!(BymlRef::Double(2.5).as_num::<f32>().unwrap(), 2.5);
        assert!(matches!(
            BymlRef::Bool(true).as_int::<i32>(),
            Err(Error::TypeError(..))
        ));
        assert!(matches!(
            BymlRef::String("1").as_num::<f32>(),
            Err(Error::TypeError(..))
        ));
    }

    #[test]
    fn cross_equality_corpus() {
        for path in corpus_files() {
            let name = path.file_name().unwrap().to_str().unwrap();
            if name == "D-3_Dynamic.byml" && !yaz0_can_decompress() {
                continue;
            }
            let owned = load(name);
            let data = prepared(&owned);
            let r = BymlRef::from_binary(data.as_ref()).unwrap();
            let o = Byml::from_binary(data.as_ref()).unwrap();
            assert_eq!(r.to_owned(), o, "cross-equality failed for {name}");
        }
    }

    #[test]
    fn cross_equality_be_fixtures() {
        for entry in fs::read_dir("test/fixtures/byml").unwrap() {
            let path = entry.unwrap().path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.ends_with(".be.bin.expected") {
                continue;
            }
            let data = fs::read(&path).unwrap();
            let r = BymlRef::from_binary(&data).unwrap();
            let o = Byml::from_binary(&data).unwrap();
            assert_eq!(r.to_owned(), o, "cross-equality failed for {name}");
        }
    }

    #[test]
    fn actor_info_accessors() {
        let data = load("ActorInfo.product.byml");
        let doc = BymlRef::from_binary(&data).unwrap();
        assert_eq!(doc["Actors"].as_array().unwrap().len(), 7934);

        let owned = Byml::from_binary(&data).unwrap();
        let name = doc["Actors"][0]["name"].as_str().unwrap();
        let owned_name = owned["Actors"][0]["name"].as_string().unwrap();
        assert!(!name.is_empty());
        assert_eq!(name, owned_name.as_str());

        // Index with a u32 acts as a hash index (wrong type here -> panic
        // domain, so just check the accessor results).
        let actors = doc.as_map().unwrap();
        let actors_node = actors.get("Actors").unwrap();
        assert_eq!(actors_node.as_array().unwrap().len(), 7934);
    }

    #[test]
    fn from_owned_roundtrip() {
        let data = load("ActorInfo.product.byml");
        let owned = Byml::from_binary(&data).unwrap();
        let borrowed: BymlRef<'_> = (&owned).into();
        assert_eq!(borrowed.to_owned(), owned);
        assert_eq!(borrowed.type_name(), "Map");
        assert!(!borrowed.is_null());
    }

    #[test]
    fn synthetic_type_coverage() {
        fn case(doc: roead::byml::Byml, endian: Endian, version: u16) -> Vec<u8> {
            doc.to_binary_with_version(endian, version)
        }

        let mut cases: Vec<(Endian, u16, roead::byml::Byml)> = Vec::new();
        let values: Vec<roead::byml::Byml> = vec![
            roead::byml::Byml::BinaryData(vec![0xDE, 0xAD, 0xBE, 0xEF]),
            roead::byml::Byml::FileData(vec![0x01, 0x02, 0x03]),
            roead::byml::Byml::I64(i64::MIN),
            roead::byml::Byml::U64(u64::MAX),
            roead::byml::Byml::Double(1.25),
        ];
        for endian in [Endian::Big, Endian::Little] {
            cases.push((endian, 2, roead::byml::Byml::Array(values.clone())));
        }

        // v7 HashMap and ValueHashMap (with the aux value).
        let mut hash_map = roead::byml::HashMap::default();
        hash_map.insert(0xDEADBEEF, roead::byml::Byml::I32(42));
        for endian in [Endian::Big, Endian::Little] {
            cases.push((endian, 7, roead::byml::Byml::HashMap(hash_map.clone())));
        }
        let mut value_hash_map = roead::byml::ValueHashMap::default();
        value_hash_map.insert(0x1234, (roead::byml::Byml::Double(2.5), 99u32));
        for endian in [Endian::Big, Endian::Little] {
            cases.push((endian, 7, roead::byml::Byml::ValueHashMap(
                value_hash_map.clone(),
            )));
        }

        for (endian, version, doc) in cases {
            let bytes = case(doc, endian, version);
            let r = BymlRef::from_binary(&bytes).unwrap();
            let o = Byml::from_binary(&bytes).unwrap();
            assert_eq!(r.to_owned(), o, "synthetic case {endian:?} v{version}");

            let arena = parse_arena::<OwnedStore<Node<'_>>>(&bytes);
            let root = arena.root().expect("root");
            assert_ref_matches_arena(&r, &arena, root);
        }
    }

    #[test]
    fn depth_cap() {
        fn deep(depth: u32) -> roead::byml::Byml {
            if depth == 0 {
                roead::byml::Byml::Null
            } else {
                roead::byml::Byml::Array(vec![deep(depth - 1)])
            }
        }

        // The root container is level 1, so 128 nested containers are
        // accepted and the 129th is rejected.
        let ok = deep(128).to_binary(Endian::Little);
        BymlRef::from_binary(&ok).expect("128 containers should parse");
        let arena = parse_arena::<OwnedStore<Node<'_>>>(&ok);
        assert_eq!(arena.len(), 129); // 128 containers plus the null leaf

        let err = BymlRef::from_binary(&deep(129).to_binary(Endian::Little)).unwrap_err();
        assert!(matches!(err, Error::InvalidData(_)));
        assert_eq!(
            err.to_string(),
            "BYML container nesting exceeds maximum depth"
        );
    }

    #[test]
    fn self_referential_container_bymlref() {
        let mut doc = header(Endian::Big, 2, 0, 0, 0x10);
        doc.push(0xC0);
        doc.extend_from_slice(&1u32.to_be_bytes()[1..]);
        doc.push(0xC0);
        doc.resize(0x18, 0); // values_offset = 0x10 + 4 + align(1, 4) = 0x18
        doc.extend_from_slice(&0x10u32.to_be_bytes());

        let err = BymlRef::from_binary(&doc).unwrap_err();
        assert!(matches!(err, Error::InvalidData(_)));
        assert_eq!(
            err.to_string(),
            "BYML container nesting exceeds maximum depth"
        );
    }

    #[test]
    fn truncated_header_is_not_a_panic() {
        // An outright empty buffer must not panic: the walk bounds-checks
        // every read.
        let err = BymlRef::from_binary(&[]).unwrap_err();
        assert!(matches!(err, Error::InsufficientData(_, _)));
    }

    #[test]
    fn malformed_inputs_bymlref() {
        let err = BymlRef::from_binary(b"BY").unwrap_err();
        assert!(matches!(err, Error::InsufficientData(_, _)));

        let mut bad = [0u8; 16];
        bad[..2].copy_from_slice(b"XX");
        let err = BymlRef::from_binary(&bad).unwrap_err();
        assert!(matches!(err, Error::BadMagic(..)));

        let ver8 = header(Endian::Big, 8, 0, 0, 0);
        let err = BymlRef::from_binary(&ver8).unwrap_err();
        assert!(matches!(err, Error::InvalidData(_)));

        let doc = header(Endian::Little, 2, 0, 0, 0);
        let err = BymlRef::from_binary(&doc).unwrap();
        assert!(err.is_null());
    }

    #[test]
    fn arena_store_capacities() {
        let vs: OwnedStore<Node<'_>> = OwnedStore::new();
        assert_eq!(vs.capacity(), 0); // hint only; OwnedStore grows
        let vs: OwnedStore<Node<'_>> = OwnedStore::with_capacity(64);
        assert_eq!(vs.capacity(), 64);
        assert_eq!(FixedStore::<Node, 8>::new().capacity(), 8);
        let mut slots = [MaybeUninit::<Node>::uninit(); 8];
        assert_eq!(BorrowedStore::new(&mut slots).capacity(), 8);

        // OwnedStore is unbounded: a 10-node document fits.
        let doc = super::array_of_n_le(9);
        let arena = BymlArena::<OwnedStore<Node<'_>>>::from_binary(&doc).unwrap();
        assert_eq!(arena.len(), 10);
    }

    fn assert_ref_matches_arena(
        ref_node: &BymlRef<'_>,
        arena: &BymlArena<OwnedStore<Node<'_>>>,
        id: NodeId,
    ) {
        let node = arena
            .node(id)
            .unwrap_or_else(|| panic!("node {id:?} missing from arena"));
        match (ref_node, &node.payload) {
            (BymlRef::String(s), Payload::String(s2)) => assert_eq!(s, s2),
            (BymlRef::BinaryData(d), Payload::BinaryData(d2)) => assert_eq!(d, d2),
            (BymlRef::FileData(d), Payload::FileData(d2)) => assert_eq!(d, d2),
            (BymlRef::Bool(b), Payload::Bool(b2)) => assert_eq!(b, b2),
            (BymlRef::I32(i), Payload::I32(i2)) => assert_eq!(i, i2),
            (BymlRef::U32(u), Payload::U32(u2)) => assert_eq!(u, u2),
            (BymlRef::I64(i), Payload::I64(i2)) => assert_eq!(i, i2),
            (BymlRef::U64(u), Payload::U64(u2)) => assert_eq!(u, u2),
            (BymlRef::Float(f), Payload::Float(f2)) => assert_eq!(f.to_bits(), f2.to_bits()),
            (BymlRef::Double(d), Payload::Double(d2)) => assert_eq!(d.to_bits(), d2.to_bits()),
            (BymlRef::Null, Payload::Null) => {}
            (BymlRef::Array(children), Payload::Array) => {
                assert_eq!(children.len(), node.child_count() as usize);
                let arena_children: Vec<_> = arena.children(id).collect();
                assert_eq!(arena_children.len(), children.len());
                for ((key, child_id), ref_child) in
                    arena_children.iter().zip(children.iter())
                {
                    assert_eq!(*key, Key::None);
                    assert_ref_matches_arena(ref_child, arena, *child_id);
                }
            }
            (BymlRef::Map(map), Payload::Map) => {
                assert_eq!(map.len(), node.child_count() as usize);
                for (key, ref_child) in map {
                    let child_id = arena
                        .get(id, key)
                        .unwrap_or_else(|| panic!("map key {key:?} missing in arena"));
                    assert_ref_matches_arena(ref_child, arena, child_id);
                }
                let str_children: Vec<_> = arena
                    .children(id)
                    .filter_map(|(k, _)| match k {
                        Key::Str(s) => Some(s),
                        _ => None,
                    })
                    .collect();
                assert_eq!(str_children.len(), map.len());
            }
            (BymlRef::HashMap(map), Payload::HashMap) => {
                assert_eq!(map.len(), node.child_count() as usize);
                for (hash, ref_child) in map {
                    let child_id = arena
                        .get_hash(id, *hash)
                        .unwrap_or_else(|| panic!("hash {hash} missing in arena"));
                    assert_ref_matches_arena(ref_child, arena, child_id);
                }
            }
            (BymlRef::ValueHashMap(map), Payload::ValueHashMap) => {
                assert_eq!(map.len(), node.child_count() as usize);
                for (hash, (ref_child, ref_aux)) in map {
                    let child_id = arena
                        .get_hash(id, *hash)
                        .unwrap_or_else(|| panic!("hash {hash} missing in arena"));
                    let child = arena.node(child_id).unwrap();
                    assert_ref_matches_arena(ref_child, arena, child_id);
                    assert_eq!(child.aux, *ref_aux, "aux mismatch for hash {hash}");
                }
            }
            _ => panic!(
                "type mismatch: {} vs {}",
                ref_node.type_name(),
                node.kind()
            ),
        }
    }
}

// Hand-crafted map/hash-map docs for key-lookup tests.

fn map_with_string_key_le() -> Vec<u8> {
    // Header: root map at 0x30; hash key table at 0x10 with one entry "abc".
    let mut doc = header(Endian::Little, 2, 0x10, 0, 0x30);
    doc.extend_from_slice(&[0xC2, 0x01, 0x00, 0x00]); // hash key table, size 1
    doc.extend_from_slice(&[0x0C, 0x00, 0x00, 0x00]); // off[0] = 0x0C
    doc.extend_from_slice(&[0x0F, 0x00, 0x00, 0x00]); // off[1] = 0x0F
    doc.resize(0x1C, 0);
    doc.extend_from_slice(b"abc\0"); // key string at 0x1C..0x20
    doc.resize(0x30, 0);
    doc.extend_from_slice(&[0xC1, 0x01, 0x00, 0x00]); // Map, size 1
    // entry: name_idx=0, type=I32, value=42
    doc.extend_from_slice(&[0x00, 0x00, 0x00, 0xD1, 0x2A, 0x00, 0x00, 0x00]);
    doc
}

fn hash_map_le() -> Vec<u8> {
    // HashMap: one entry hash=0xDEADBEEF -> I32(7).
    let mut doc = header(Endian::Little, 2, 0, 0, 0x10);
    doc.extend_from_slice(&[0x20, 0x01, 0x00, 0x00]); // HashMap, size 1
    // entry: hash at 0x14, value u32 at 0x18
    doc.extend_from_slice(&0xDEADBEEFu32.to_le_bytes());
    doc.extend_from_slice(&7u32.to_le_bytes());
    // type tag at types_offset = 0x10 + 4 + 8 = 0x1C
    doc.resize(0x1C, 0);
    doc.push(0xD1); // I32
    doc
}

fn value_hash_map_le() -> Vec<u8> {
    // ValueHashMap: one entry value=I32(8), hash=0x1234, aux=55.
    let mut doc = header(Endian::Little, 2, 0, 0, 0x10);
    doc.extend_from_slice(&[0x21, 0x01, 0x00, 0x00]); // ValueHashMap, size 1
    // entry: value u32 at 0x14, hash at 0x18, aux at 0x1C
    doc.extend_from_slice(&8u32.to_le_bytes());
    doc.extend_from_slice(&0x1234u32.to_le_bytes());
    doc.extend_from_slice(&55u32.to_le_bytes());
    // type tag at types_offset = 0x10 + 4 + 12 = 0x20
    doc.resize(0x20, 0);
    doc.push(0xD1); // I32
    doc
}

#[test]
fn arena_key_lookups() {
    let doc = map_with_string_key_le();
    let arena = parse_arena::<FixedStore<Node, 64>>(&doc);
    let root = arena.root().expect("root");
    assert_eq!(arena.node(root).unwrap().kind(), "Map");
    assert_eq!(arena.node(root).unwrap().child_count(), 1);

    let child = arena.get(root, "abc").expect("key abc");
    assert_eq!(arena.node(child).unwrap().as_i32().unwrap(), 42);
    assert_eq!(arena.get(root, "abd"), None);

    let kids: Vec<_> = arena.children(root).collect();
    assert_eq!(kids.len(), 1);
    assert_eq!(kids[0].0, Key::Str("abc"));

    let hdoc = hash_map_le();
    let h = parse_arena::<FixedStore<Node, 64>>(&hdoc);
    let root = h.root().expect("root");
    assert_eq!(h.node(root).unwrap().kind(), "HashMap");
    let child = h.get_hash(root, 0xDEADBEEF).expect("hash");
    assert_eq!(h.node(child).unwrap().as_i32().unwrap(), 7);
    assert_eq!(h.get_hash(root, 1), None);

    let vdoc = value_hash_map_le();
    let v = parse_arena::<FixedStore<Node, 64>>(&vdoc);
    let root = v.root().expect("root");
    assert_eq!(v.node(root).unwrap().kind(), "ValueHashMap");
    let child = v.get_hash(root, 0x1234).expect("hash");
    let node = v.node(child).unwrap();
    assert_eq!(node.as_i32().unwrap(), 8);
    assert_eq!(node.aux, 55);
    assert_eq!(v.get_hash(root, 1), None);
}
