//! Tests for the zero-copy AAMP readers ([`ParameterIORef`] and
//! [`ParameterIOArena`]).
//!
//! Compiled whenever `feature = "aamp"` is on. Tests that allocate
//! ([`ParameterIORef`], `to_owned`, comparisons against the owned
//! [`ParameterIO`]) are additionally gated on `feature = "alloc"`, so the pure
//! core/arena subset runs under `--no-default-features --features aamp`.

#![cfg(feature = "aamp")]

use core::mem::MaybeUninit;

use roead::Error;
use roead::aamp::arena::*;
use roead::aamp::{self as aamp, AampNode, AampPayload, BufferRef, ParameterIOArena, hash_name};
#[cfg(feature = "alloc")]
use roead::aamp::{
    CurveRef, Name, ParameterIORef, ParameterListRef, ParameterObjectRef, ParameterRef,
};
use roead::storage::{BorrowedStore, FixedStore, Store};

/// The CRC32 hash of `"param_root"`.
const ROOT_KEY_HASH: u32 = hash_name("param_root");

/// Every non-yml binary file under `test/aamp/`, recursive.
fn corpus_files() -> Vec<std::path::PathBuf> {
    let mut out: Vec<std::path::PathBuf> = Vec::new();
    add_dir_files("test/aamp", &mut out);
    out.sort();
    out
}

fn add_dir_files(dir: &str, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            add_dir_files(path.to_str().unwrap(), out);
        } else if path.extension().and_then(|s| s.to_str()) != Some("yml") {
            out.push(path);
        }
    }
}

/// Builds a minimal valid AAMP archive: data type `"xml"` at 0x30 and an
/// empty root list named `param_root` at 0x34.
fn minimal_root_bytes() -> Vec<u8> {
    let mut out = b"AAMP".to_vec();
    out.extend_from_slice(&2u32.to_le_bytes()); // version
    out.extend_from_slice(&3u32.to_le_bytes()); // flags: LE + UTF-8
    out.extend_from_slice(&0u32.to_le_bytes()); // file_size (unused by the reader)
    out.extend_from_slice(&0u32.to_le_bytes()); // pio_version
    out.extend_from_slice(&4u32.to_le_bytes()); // pio_offset: root list at 0x34
    out.extend_from_slice(&[0u8; 24]); // counts and section sizes
    out.extend_from_slice(b"xml\0"); // data_type at 0x30
    // Root list entry at 0x34: name = param_root, no children.
    out.extend_from_slice(&ROOT_KEY_HASH.to_le_bytes());
    out.extend_from_slice(&[0u8; 8]); // lists_rel, list_count, objects_rel, object_count
    out
}

// ---------------------------------------------------------------------------
// Core-only tests (no allocator)
// ---------------------------------------------------------------------------

/// Builds the bytes of a root list with one object "Obj" holding one
/// `BufferInt` param "Buf" with values [7, 8] on top of `minimal_root_bytes`.
fn buffer_int_bytes() -> Vec<u8> {
    let mut bytes = minimal_root_bytes();
    // Root entry at 0x34: objects follow the entry (objects_rel = 4, i.e. at
    // offset 0x34 + 4*4 = 0x44).
    bytes[0x34 + 8..0x34 + 10].copy_from_slice(&(4u16).to_le_bytes()); // objects_rel = 4
    bytes[0x34 + 10..0x34 + 12].copy_from_slice(&[1u8, 0]); // object_count = 1
    // Object entry "Obj" at 0x44.
    bytes.resize(0x44 + 8, 0);
    bytes[0x44..0x48].copy_from_slice(&aamp::Name::from_str("Obj").hash().to_le_bytes());
    bytes[0x48..0x4a].copy_from_slice(&(4u16).to_le_bytes()); // params_rel = 4
    bytes[0x4a..0x4c].copy_from_slice(&[1u8, 0]); // param_count = 1
    // Param entry "Buf" at 0x54: data_rel = 3, type = BufferInt.
    bytes.resize(0x54 + 8, 0);
    bytes[0x54..0x58].copy_from_slice(&aamp::Name::from_str("Buf").hash().to_le_bytes());
    bytes[0x58..0x5b].copy_from_slice(&[3u8, 0, 0]); // data_rel = 3
    bytes[0x5b] = 13; // BufferInt
    // Element count at 0x5c (= data_offset - 4), elements at 0x60 (= data_offset).
    bytes.resize(0x68, 0);
    bytes[0x5c..0x60].copy_from_slice(&2u32.to_le_bytes());
    bytes[0x60..0x64].copy_from_slice(&7i32.to_le_bytes());
    bytes[0x64..0x68].copy_from_slice(&8i32.to_le_bytes());
    bytes
}

/// Compares two arenas' traversals recursively: node names, payloads and
/// child order.
fn assert_arena_traversal_eq<'a, S1: Store<AampNode<'a>>, S2: Store<AampNode<'a>>>(
    a: &ParameterIOArena<S1>,
    b: &ParameterIOArena<S2>,
    id1: NodeId,
    id2: NodeId,
) {
    let n1 = a.node(id1).expect("node missing from first arena");
    let n2 = b.node(id2).expect("node missing from second arena");
    assert_eq!(n1.name, n2.name, "node name mismatch between arenas");
    assert_payload_eq(&n1.payload, &n2.payload);
    assert_eq!(
        n1.child_count, n2.child_count,
        "child count mismatch between arenas"
    );
    let c1: Vec<_> = a.children(id1).collect();
    let c2: Vec<_> = b.children(id2).collect();
    assert_eq!(c1.len(), c2.len(), "children count mismatch between arenas");
    for ((name1, child1), (name2, child2)) in c1.iter().zip(c2.iter()) {
        assert_eq!(name1, name2, "child name mismatch between arenas");
        assert_arena_traversal_eq(a, b, *child1, *child2);
    }
}

fn assert_payload_eq(a: &AampPayload<'_>, b: &AampPayload<'_>) {
    match (a, b) {
        (AampPayload::List, AampPayload::List) | (AampPayload::Object, AampPayload::Object) => {}
        (AampPayload::Bool(v1), AampPayload::Bool(v2)) => assert_eq!(v1, v2),
        (AampPayload::F32(v1), AampPayload::F32(v2)) => assert_eq!(v1, v2),
        (AampPayload::I32(v1), AampPayload::I32(v2)) => assert_eq!(v1, v2),
        (AampPayload::U32(v1), AampPayload::U32(v2)) => assert_eq!(v1, v2),
        (AampPayload::Vec2(v1), AampPayload::Vec2(v2)) => assert_eq!(v1, v2),
        (AampPayload::Vec3(v1), AampPayload::Vec3(v2)) => assert_eq!(v1, v2),
        (AampPayload::Vec4(v1), AampPayload::Vec4(v2)) => assert_eq!(v1, v2),
        (AampPayload::Color(v1), AampPayload::Color(v2)) => assert_eq!(v1, v2),
        (AampPayload::Quat(v1), AampPayload::Quat(v2)) => assert_eq!(v1, v2),
        (AampPayload::String32(s1), AampPayload::String32(s2)) => assert_eq!(s1, s2),
        (AampPayload::String64(s1), AampPayload::String64(s2)) => assert_eq!(s1, s2),
        (AampPayload::String256(s1), AampPayload::String256(s2)) => assert_eq!(s1, s2),
        (AampPayload::StringRef(s1), AampPayload::StringRef(s2)) => assert_eq!(s1, s2),
        (AampPayload::Curve1(c1), AampPayload::Curve1(c2))
        | (AampPayload::Curve2(c1), AampPayload::Curve2(c2))
        | (AampPayload::Curve3(c1), AampPayload::Curve3(c2))
        | (AampPayload::Curve4(c1), AampPayload::Curve4(c2)) => {
            assert_eq!(c1.len(), c2.len());
            for (c1, c2) in c1.iter().zip(c2.iter()) {
                assert_eq!(c1.a, c2.a);
                assert_eq!(c1.b, c2.b);
                assert_eq!(c1.floats, c2.floats);
            }
        }
        (AampPayload::BufferInt(b1), AampPayload::BufferInt(b2))
        | (AampPayload::BufferF32(b1), AampPayload::BufferF32(b2))
        | (AampPayload::BufferU32(b1), AampPayload::BufferU32(b2)) => {
            assert_eq!(b1.bytes(), b2.bytes());
        }
        (AampPayload::BufferBinary(b1), AampPayload::BufferBinary(b2)) => {
            assert_eq!(b1, b2);
        }
        _ => panic!("payload mismatch: {a:?} vs {b:?}"),
    }
}

#[test]
fn arena_traversal_core() {
    let bytes = buffer_int_bytes();

    let fixed = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&bytes).unwrap();
    let mut slots = [MaybeUninit::uninit(); 64];
    let borrowed =
        ParameterIOArena::<BorrowedStore<'_, AampNode<'_>>>::from_binary_in(&bytes, &mut slots)
            .unwrap();

    assert_arena_traversal_eq(
        &fixed,
        &borrowed,
        fixed.root().unwrap(),
        borrowed.root().unwrap(),
    );

    // Spot-check the BufferInt payload.
    let root = fixed.root().unwrap();
    let (_, obj_id) = fixed.children(root).next().expect("object child");
    let (_, param_id) = fixed.children(obj_id).next().expect("parameter child");
    let node = fixed.node(param_id).expect("param node");
    match &node.payload {
        AampPayload::BufferInt(buf) => {
            assert_eq!(buf.len_i32(), 2);
            assert_eq!(buf.iter_i32().collect::<Vec<_>>(), vec![7, 8]);
        }
        other => panic!("expected BufferInt payload, got {other:?}"),
    }
}

#[test]
fn malformed_inputs_core() {
    // Truncated header (< 0x30 bytes).
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(b"AAMP").unwrap_err();
    assert!(matches!(err, Error::InsufficientData(_, _)));

    // Bad magic.
    let mut bad = [0u8; 0x30];
    bad[..4].copy_from_slice(b"XXXX");
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&bad).unwrap_err();
    assert!(matches!(err, Error::BadMagic(_, _)));

    // Version != 2.
    let mut v3 = minimal_root_bytes();
    v3[4..8].copy_from_slice(&3u32.to_le_bytes());
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&v3).unwrap_err();
    assert!(matches!(err, Error::InvalidData(_)));
    assert_eq!(
        err.to_string(),
        "Only version 2 parameter archives are supported"
    );

    // Little endian flag clear.
    let mut no_le = minimal_root_bytes();
    no_le[8..12].copy_from_slice(&2u32.to_le_bytes()); // flags = 2 (UTF-8 only)
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&no_le).unwrap_err();
    assert!(matches!(err, Error::InvalidData(_)));

    // UTF-8 flag clear.
    let mut no_utf8 = minimal_root_bytes();
    no_utf8[8..12].copy_from_slice(&1u32.to_le_bytes()); // flags = 1 (LE only)
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&no_utf8).unwrap_err();
    assert!(matches!(err, Error::InvalidData(_)));

    // Root list name != param_root.
    let mut wrong_root = minimal_root_bytes();
    wrong_root[0x34..0x38].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    let err =
        ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&wrong_root).unwrap_err();
    assert_eq!(err.to_string(), "No param root found in parameter archive");

    // List offset out of bounds.
    let mut oob_list = minimal_root_bytes();
    oob_list[0x34 + 4..0x34 + 6].copy_from_slice(&[0xFF, 0xFF]); // lists_rel = 0xFFFF
    oob_list[0x34 + 6..0x34 + 8].copy_from_slice(&[1u8, 0]); // list_count = 1
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&oob_list).unwrap_err();
    assert!(matches!(err, Error::InsufficientData(_, _)));

    // Buffer elements past EOF: count = 100 but only the count is present.
    let mut buf_oob = buffer_int_bytes();
    buf_oob[0x58..0x5C].copy_from_slice(&100u32.to_le_bytes());
    buf_oob.truncate(0x5C);
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&buf_oob).unwrap_err();
    assert!(matches!(err, Error::InsufficientData(_, _)));

    // Unterminated StringRef: point at a string without NUL before EOF.
    let mut sref = minimal_root_bytes();
    sref[0x34 + 8..0x34 + 10].copy_from_slice(&(4u16).to_le_bytes()); // objects_rel = 4 -> object at 0x44
    sref[0x34 + 10..0x34 + 12].copy_from_slice(&[1u8, 0]); // object_count = 1
    sref.resize(0x44 + 8, 0);
    sref[0x44..0x48].copy_from_slice(&aamp::Name::from_str("Obj").hash().to_le_bytes());
    sref[0x48..0x4a].copy_from_slice(&(4u16).to_le_bytes()); // params_rel = 4
    sref[0x4a..0x4c].copy_from_slice(&[1u8, 0]); // param_count = 1
    sref.resize(0x54 + 8, 0);
    sref[0x54..0x58].copy_from_slice(&aamp::Name::from_str("S").hash().to_le_bytes());
    sref[0x58..0x5b].copy_from_slice(&[2u8, 0, 0]); // data_rel = 2 -> data_offset = 0x5c
    sref[0x5b] = 20; // StringRef
    sref.resize(0x5c, 0); // keep the full 8-byte param entry
    sref.extend_from_slice(b"unterminated"); // no NUL before EOF
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&sref).unwrap_err();
    assert!(matches!(err, Error::InsufficientData(_, _)));

    // Invalid UTF-8 in data_type.
    let mut bad_utf8 = minimal_root_bytes();
    bad_utf8[0x30..0x33].copy_from_slice(b"\xff\xfe\0");
    let err = ParameterIOArena::<FixedStore<AampNode<'_>, 64>>::from_binary(&bad_utf8).unwrap_err();
    assert!(matches!(err, Error::InvalidUtf8(_)));
}

#[test]
fn buffer_ref_alignment_core() {
    // Forming a misaligned byte slice is safe for `u8`; `BufferRef` defers
    // the alignment check to the typed accessors.
    let misaligned = BufferRef::new(&[1u8, 2, 3, 4, 5, 6, 7, 8][1..5]);
    assert!(matches!(misaligned.as_i32(), Err(Error::InvalidData(_))));
}

// ---------------------------------------------------------------------------
// Alloc-backed tests
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
mod alloc_tests {
    use super::*;

    use roead::aamp::params;
    use roead::aamp::{Parameter, ParameterIO, ParameterList};
    use roead::storage::{BorrowedStore, FixedStore, OwnedStore};
    use roead::types::*;

    fn synthetic_pio() -> ParameterIO {
        ParameterIO::new().with_object(
            "Obj",
            params!(
                "Bool" => Parameter::Bool(true),
                "F32" => Parameter::F32(1.5),
                "I32" => Parameter::I32(-7),
                "U32" => Parameter::U32(42),
                "Vec2" => Parameter::Vec2(Vector2f { x: 0.25, y: -0.75 }),
                "Vec3" => Parameter::Vec3(Vector3f { x: 4.0, y: 5.0, z: 6.0 }),
                "Vec4" => Parameter::Vec4(Vector4f { x: 0.0, y: 1.0, z: 2.0, t: 3.0 }),
                "Color" => Parameter::Color(Color { r: 1.0, g: 0.0, b: 0.0, a: 0.5 }),
                "Quat" => Parameter::Quat(Quat { a: 1.0, b: 0.0, c: 0.0, d: 0.0 }),
                "String32" => Parameter::String32("s32".into()),
                "String64" => Parameter::String64(Box::new("s64".into())),
                "String256" => Parameter::String256(Box::new("s256".into())),
                "StringRef" => Parameter::StringRef("sref".into()),
                "Curve1" => Parameter::Curve1(Box::new([curve(7, 11)])),
                "Curve2" => Parameter::Curve2(Box::new([curve(1, 2), curve(3, 4)])),
                "Curve3" => Parameter::Curve3(Box::new([curve(1, 2), curve(3, 4), curve(5, 6)])),
                "Curve4" => Parameter::Curve4(Box::new([
                    curve(1, 2),
                    curve(3, 4),
                    curve(5, 6),
                    curve(7, 8),
                ])),
                "BufferInt" => Parameter::BufferInt(vec![-1, 0, 1]),
                "BufferF32" => Parameter::BufferF32(vec![1.0, 2.0, 3.0]),
                "BufferU32" => Parameter::BufferU32(vec![1, 2, 3]),
                "BufferBinary" => Parameter::BufferBinary(vec![9, 8, 7]),
            ),
        )
    }

    fn curve(a: u32, b: u32) -> Curve {
        Curve {
            a,
            b,
            floats: [0.25; 30],
        }
    }

    fn deep_list(depth: u32) -> ParameterList {
        if depth == 0 {
            ParameterList::new()
        } else {
            ParameterList::new().with_list("l", deep_list(depth - 1))
        }
    }

    /// Compares a borrowed ref tree against an arena traversal, recursively
    /// over objects and lists by name.
    fn assert_ref_matches_arena<'a, S: Store<AampNode<'a>>>(
        ref_list: &ParameterListRef<'a>,
        arena: &ParameterIOArena<S>,
        id: NodeId,
    ) {
        let node = arena.node(id).expect("node missing from arena");
        assert_eq!(node.payload, AampPayload::List);
        assert_eq!(
            node.child_count,
            (ref_list.objects.len() + ref_list.lists.len()) as u32
        );
        for (name, ref_obj) in ref_list.objects.iter() {
            let child = arena
                .get(id, name.clone())
                .unwrap_or_else(|| panic!("object {name:?} missing from arena"));
            assert_arena_object_eq(ref_obj, arena, child);
        }
        for (name, ref_list) in ref_list.lists.iter() {
            let child = arena
                .get(id, name.clone())
                .unwrap_or_else(|| panic!("list {name:?} missing from arena"));
            assert_ref_matches_arena(ref_list, arena, child);
        }
    }

    fn assert_arena_object_eq<'a, S: Store<AampNode<'a>>>(
        ref_obj: &ParameterObjectRef<'a>,
        arena: &ParameterIOArena<S>,
        id: NodeId,
    ) {
        let node = arena.node(id).expect("node missing from arena");
        assert_eq!(node.payload, AampPayload::Object);
        assert_eq!(node.child_count, ref_obj.len() as u32);
        for (name, ref_param) in ref_obj.iter() {
            let child = arena
                .get(id, *name)
                .unwrap_or_else(|| panic!("parameter {name:?} missing from arena"));
            assert_arena_param_eq(ref_param, arena, child);
        }
    }

    fn assert_arena_param_eq<'a, S: Store<AampNode<'a>>>(
        ref_param: &ParameterRef<'a>,
        arena: &ParameterIOArena<S>,
        id: NodeId,
    ) {
        let node = arena.node(id).expect("node missing from arena");
        assert_payload_eq(&ref_param_to_payload(ref_param), &node.payload);
    }

    /// Converts a [`ParameterRef`] into an [`AampPayload`] for comparison.
    fn ref_param_to_payload<'a>(p: &ParameterRef<'a>) -> AampPayload<'a> {
        match p {
            ParameterRef::Bool(b) => AampPayload::Bool(*b),
            ParameterRef::F32(f) => AampPayload::F32(*f),
            ParameterRef::I32(i) => AampPayload::I32(*i),
            ParameterRef::U32(u) => AampPayload::U32(*u),
            ParameterRef::Vec2(v) => AampPayload::Vec2(*v),
            ParameterRef::Vec3(v) => AampPayload::Vec3(*v),
            ParameterRef::Vec4(v) => AampPayload::Vec4(*v),
            ParameterRef::Color(c) => AampPayload::Color(*c),
            ParameterRef::Quat(q) => AampPayload::Quat(*q),
            ParameterRef::String32(s) => AampPayload::String32(*s),
            ParameterRef::String64(s) => AampPayload::String64(*s),
            ParameterRef::String256(s) => AampPayload::String256(*s),
            ParameterRef::StringRef(s) => AampPayload::StringRef(*s),
            ParameterRef::Curve1(c) => AampPayload::Curve1(*c),
            ParameterRef::Curve2(c) => AampPayload::Curve2(*c),
            ParameterRef::Curve3(c) => AampPayload::Curve3(*c),
            ParameterRef::Curve4(c) => AampPayload::Curve4(*c),
            ParameterRef::BufferInt(b) => AampPayload::BufferInt(*b),
            ParameterRef::BufferF32(b) => AampPayload::BufferF32(*b),
            ParameterRef::BufferU32(b) => AampPayload::BufferU32(*b),
            ParameterRef::BufferBinary(b) => AampPayload::BufferBinary(*b),
        }
    }

    #[test]
    fn cross_equality_corpus() {
        for path in corpus_files() {
            let data = std::fs::read(&path).unwrap();
            let name = path.file_name().unwrap().to_string_lossy();
            let ref_tree = ParameterIORef::from_binary(&data)
                .unwrap_or_else(|err| panic!("ref parse failed for {name}: {err}"));
            let owned = ParameterIO::from_binary(&data)
                .unwrap_or_else(|err| panic!("owned parse failed for {name}: {err}"));
            assert_eq!(
                ref_tree.to_owned(),
                owned,
                "cross-equality failed for {name}"
            );
        }
    }

    #[test]
    fn arena_equivalence_corpus() {
        for path in corpus_files() {
            let data = std::fs::read(&path).unwrap();
            let name = path.file_name().unwrap().to_string_lossy();
            let ref_tree = ParameterIORef::from_binary(&data)
                .unwrap_or_else(|err| panic!("ref parse failed for {name}: {err}"));

            let owned_arena = ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&data)
                .unwrap_or_else(|err| panic!("owned arena failed for {name}: {err}"));
            let root = owned_arena.root().unwrap();
            let node_count = owned_arena.len();
            if node_count <= 8192 {
                let fixed_arena =
                    ParameterIOArena::<FixedStore<AampNode<'_>, 8192>>::from_binary(&data)
                        .unwrap_or_else(|err| panic!("fixed arena failed for {name}: {err}"));
                assert_arena_traversal_eq(&owned_arena, &fixed_arena, root, root);
            }
            assert_ref_matches_arena(&ref_tree.param_root, &owned_arena, root);

            // A slice store grown until the document fits.
            let mut buffer: Vec<AampNode<'_>> = Vec::with_capacity(1024);
            let slice_arena = loop {
                let slots = buffer.spare_capacity_mut();
                match ParameterIOArena::<BorrowedStore<'_, AampNode<'_>>>::from_binary_in(
                    &data, slots,
                ) {
                    Ok(arena) => break arena,
                    Err(Error::CapacityExceeded(_)) => buffer.reserve_exact(buffer.capacity() * 2),
                    Err(err) => panic!("unexpected error: {err}"),
                }
            };
            assert_arena_traversal_eq(
                &owned_arena,
                &slice_arena,
                root,
                slice_arena.root().unwrap(),
            );
            assert_ref_matches_arena(&ref_tree.param_root, &slice_arena, root);
        }
    }

    #[test]
    fn synthetic_full_type_coverage() {
        let pio = synthetic_pio();
        let bytes = pio.to_binary();

        // Cross-equality with the owned model.
        let ref_tree = ParameterIORef::from_binary(&bytes).unwrap();
        assert_eq!(ref_tree.to_owned(), pio, "synthetic cross-equality");

        // From<&ParameterIO>.
        let ref_from_owned: ParameterIORef = (&pio).into();
        assert_eq!(ref_from_owned.to_owned(), pio, "from-owned cross-equality");

        let obj = ref_tree
            .param_root
            .object("Obj")
            .unwrap_or_else(|| panic!("synthetic object missing"));

        assert_eq!(obj.get("String32").unwrap().as_string32().unwrap(), "s32");
        assert_eq!(obj.get("String64").unwrap().as_string64().unwrap(), "s64");
        assert_eq!(
            obj.get("String256").unwrap().as_string256().unwrap(),
            "s256"
        );
        assert_eq!(
            obj.get("StringRef").unwrap().as_string_ref().unwrap(),
            "sref"
        );

        assert_eq!(obj.get("Bool").unwrap().as_bool().unwrap(), true);
        assert_eq!(obj.get("F32").unwrap().as_f32().unwrap(), 1.5);
        assert_eq!(obj.get("I32").unwrap().as_i32().unwrap(), -7);
        assert_eq!(obj.get("U32").unwrap().as_u32().unwrap(), 42);
        assert_eq!(
            *obj.get("Vec2").unwrap().as_vec2().unwrap(),
            Vector2f { x: 0.25, y: -0.75 }
        );
        assert_eq!(
            *obj.get("Vec3").unwrap().as_vec3().unwrap(),
            Vector3f {
                x: 4.0,
                y: 5.0,
                z: 6.0
            }
        );
        assert_eq!(
            *obj.get("Vec4").unwrap().as_vec4().unwrap(),
            Vector4f {
                x: 0.0,
                y: 1.0,
                z: 2.0,
                t: 3.0
            }
        );
        assert_eq!(
            *obj.get("Color").unwrap().as_color().unwrap(),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 0.5
            }
        );
        assert_eq!(
            *obj.get("Quat").unwrap().as_quat().unwrap(),
            Quat {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 0.0
            }
        );

        // Curves decode exactly.
        let curves = |name: &str| -> &CurveRef<'_> { obj.get(name).unwrap().as_curve1().unwrap() };
        let c = curves("Curve1");
        assert_eq!(c.len(), 1);
        let curve = c.curve(0).unwrap();
        assert_eq!(curve.a, 7);
        assert_eq!(curve.b, 11);
        assert_eq!(curve.floats, [0.25; 30]);
        assert_eq!(c.iter().collect::<Vec<_>>(), vec![curve]);

        let c2 = obj.get("Curve2").unwrap().as_curve2().unwrap();
        assert_eq!(c2.len(), 2);
        assert_eq!(c2.curve(0).unwrap().a, 1);
        assert_eq!(c2.curve(1).unwrap().b, 4);
        assert_eq!(c2.iter().count(), 2);

        // Buffers: iterators and typed slices.
        let bi = obj.get("BufferInt").unwrap().as_buffer_int().unwrap();
        assert_eq!(bi.iter_i32().collect::<Vec<_>>(), vec![-1, 0, 1]);
        let bf = obj.get("BufferF32").unwrap().as_buffer_f32().unwrap();
        assert_eq!(bf.iter_f32().collect::<Vec<_>>(), vec![1.0, 2.0, 3.0]);
        let bu = obj.get("BufferU32").unwrap().as_buffer_u32().unwrap();
        assert_eq!(bu.iter_u32().collect::<Vec<_>>(), vec![1, 2, 3]);
        let bb = obj.get("BufferBinary").unwrap().as_buffer_binary().unwrap();
        assert_eq!(bb, &[9, 8, 7]);

        #[cfg(target_endian = "little")]
        {
            assert_eq!(bf.as_f32().unwrap(), &[1.0, 2.0, 3.0]);
            assert_eq!(bi.as_i32().unwrap(), &[-1, 0, 1]);
            assert_eq!(bu.as_u32().unwrap(), &[1, 2, 3]);
        }

        // Arena equality.
        let owned_arena =
            ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&bytes).unwrap();
        let root = owned_arena.root().unwrap();
        let node_count = owned_arena.len();
        if node_count <= 8192 {
            let fixed_arena =
                ParameterIOArena::<FixedStore<AampNode<'_>, 8192>>::from_binary(&bytes)
                    .unwrap_or_else(|err| panic!("fixed arena failed: {err}"));
            assert_arena_traversal_eq(&owned_arena, &fixed_arena, root, root);
        }
        assert_ref_matches_arena(&ref_tree.param_root, &owned_arena, root);

        // BufferRef alignment: the parsed buffer lives at a 4-byte aligned
        // offset, so the typed slice accessor succeeds.
        let buf = obj.get("BufferInt").unwrap().as_buffer_int().unwrap();
        assert_eq!(buf.bytes().as_ptr().align_offset(4), 0);
        assert_eq!(buf.as_i32().unwrap(), &[-1, 0, 1]);
        // Deliberately misaligned slice: the typed accessor must reject it.
        let misaligned = BufferRef::new(&bytes[1..5]);
        assert!(matches!(misaligned.as_i32(), Err(Error::InvalidData(_))));
    }

    #[test]
    fn buffer_ref_alignment_alloc() {
        // Reuse the hand-built buffer (normally aligned heap memory).
        let bytes = buffer_int_bytes();
        let pio = ParameterIORef::from_binary(&bytes).unwrap();
        let buf = pio
            .param_root
            .object("Obj")
            .and_then(|o| o.get("Buf"))
            .and_then(|p| p.as_buffer_int().ok())
            .unwrap();
        assert_eq!(buf.iter_i32().collect::<Vec<_>>(), vec![7, 8]);
        #[cfg(target_endian = "little")]
        assert_eq!(buf.as_i32().unwrap(), &[7, 8]);
        let misaligned = BufferRef::new(&bytes[0x5D..0x60]);
        assert!(matches!(misaligned.as_i32(), Err(Error::InvalidData(_))));
    }

    #[test]
    fn depth_cap() {
        // >128-deep list nesting hits the depth cap.
        let pio = ParameterIO::new().with_list("l", deep_list(130));
        let bytes = pio.to_binary();

        let err = ParameterIORef::from_binary(&bytes).unwrap_err();
        assert!(matches!(err, Error::InvalidData(_)));
        assert_eq!(
            err.to_string(),
            "AAMP container nesting exceeds maximum depth"
        );

        // The arena either trips its depth check or its capacity first.
        let err = match ParameterIOArena::<FixedStore<AampNode<'_>, 4096>>::from_binary(&bytes) {
            Err(err) => err,
            Ok(_) => panic!("expected an error"),
        };
        assert!(matches!(
            err,
            Error::InvalidData(_) | Error::CapacityExceeded(_)
        ));

        // A tiny fixed store trips its capacity first.
        let err = match ParameterIOArena::<FixedStore<AampNode<'_>, 8>>::from_binary(&bytes) {
            Err(err) => err,
            Ok(_) => panic!("expected an error"),
        };
        assert!(matches!(err, Error::CapacityExceeded(_)));
    }

    #[test]
    fn accessor_parity() {
        let pio = synthetic_pio();
        let bytes = pio.to_binary();
        let ref_tree = ParameterIORef::from_binary(&bytes).unwrap();

        // Inherent accessors mirroring the owned API.
        assert_eq!(ref_tree.version, pio.version);
        assert_eq!(ref_tree.data_type, pio.data_type);
        assert_eq!(
            ref_tree.param_root.objects.len(),
            pio.param_root.objects.len()
        );
        assert_eq!(ref_tree.param_root.lists.len(), pio.param_root.lists.len());
        assert!(
            ref_tree
                .param_root
                .objects
                .contains_key(&Name::from_str("Obj"))
        );
        assert!(ref_tree.object("Obj").is_some());
        assert!(ref_tree.list("l").is_none());
        let obj = ref_tree.object("Obj").unwrap();
        assert_eq!(obj.len(), 21);
        assert!(obj.contains_key("BufferBinary"));
        assert_eq!(obj["Vec2"].type_name(), "Vec2");
        assert_eq!(obj["I32"].type_name(), "I32");
        assert_eq!(obj["F32"].type_name(), "F32");

        // Index by name and by hash.
        assert_eq!(obj[hash_name("Vec2")].type_name(), "Vec2");
        assert_eq!(obj["Vec2"].type_name(), "Vec2");

        // to_owned round trip.
        assert_eq!(ref_tree.to_owned(), pio);
    }
}
