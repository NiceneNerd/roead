//! Tests for YAML emission directly from the zero-copy AAMP readers
//! ([`ParameterIORef`] and [`ParameterIOArena`]).
//!
//! Output is byte-identical to the owned emitter
//! ([`ParameterIO::to_text`](roead::aamp::ParameterIO::to_text)).
//! Compiled only with the `yaml` and `aamp` features.

#![cfg(all(feature = "yaml", feature = "aamp"))]

use std::fs;
use std::path::{Path, PathBuf};

use roead::Error;
use roead::aamp::arena::{AampNode, ParameterIOArena, ParameterIOStack};
use roead::aamp::{Name, ParameterIO, ParameterIORef, hash_name};
use roead::storage::{BorrowedStore, OwnedStore};

/// Every non-YAML binary file under `test/aamp/`, recursively, sorted.
fn aamp_corpus_files() -> Vec<PathBuf> {
    fn add(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                add(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) != Some("yml") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    add(Path::new("test/aamp"), &mut out);
    out.sort();
    out
}

/// Read an AAMP corpus file.
fn aamp_load(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap()
}

#[test]
fn aamp_byte_identity_corpus() {
    for path in aamp_corpus_files() {
        let data = aamp_load(&path);
        let expected = ParameterIO::from_binary(&data).unwrap().to_text();

        assert_eq!(
            ParameterIORef::from_binary(&data).unwrap().to_text(),
            expected,
            "ParameterIORef byte-identity failed for {}",
            path.display()
        );
        assert_eq!(
            ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&data)
                .unwrap()
                .to_text(),
            expected,
            "OwnedStore byte-identity failed for {}",
            path.display()
        );

        // A caller-owned `BorrowedStore`, sized from a counting parse.
        let count = ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&data)
            .unwrap()
            .len();
        let mut buffer: Vec<AampNode<'_>> = Vec::with_capacity(count);
        let arena =
            ParameterIOArena::<BorrowedStore<'_, AampNode<'_>>>::from_binary_in(
                &data,
                buffer.spare_capacity_mut(),
            )
            .unwrap();
        assert_eq!(
            arena.to_text(),
            expected,
            "BorrowedStore byte-identity failed for {}",
            path.display()
        );

        // `ParameterIOStack` has a fixed 4096-node capacity: documents that
        // do not fit are the only expected failure. Parsed in place so the
        // inline store is never moved by value.
        let mut stack = ParameterIOStack::<4096>::new();
        match stack.parse_in_place(&data) {
            Ok(()) => assert_eq!(
                stack.to_text(),
                expected,
                "ParameterIOStack byte-identity failed for {}",
                path.display()
            ),
            Err(Error::CapacityExceeded(_)) => {}
            Err(err) => panic!("unexpected parse error for {}: {err}", path.display()),
        }
    }
}

// ---------------------------------------------------------------------------
// Direct fixture check
// ---------------------------------------------------------------------------

#[test]
fn aamp_fixture_engine_rom_horse_link() {
    let data = aamp_load(Path::new("test/aamp/EngineRom_Horse_Link.bphysics"));
    let expected =
        fs::read_to_string("test/fixtures/aamp/EngineRom_Horse_Link.bphysics.yml.expected")
            .unwrap();
    assert_eq!(
        ParameterIORef::from_binary(&data).unwrap().to_text(),
        expected
    );
    assert_eq!(
        ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&data)
            .unwrap()
            .to_text(),
        expected
    );
}

// ---------------------------------------------------------------------------
// Roundtrip through the owned YAML parser
// ---------------------------------------------------------------------------

#[test]
fn aamp_roundtrip_corpus() {
    for path in aamp_corpus_files() {
        let data = aamp_load(&path);
        let ref_text = ParameterIORef::from_binary(&data).unwrap().to_text();
        let roundtripped = ParameterIO::from_text(&ref_text).unwrap();
        let original = ParameterIO::from_binary(&data).unwrap();
        assert_eq!(
            roundtripped,
            original,
            "roundtrip failed for {}",
            path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// Stack arena reuse
// ---------------------------------------------------------------------------

#[test]
fn aamp_stack_parse_in_place_reuse() {
    // Both documents are a few hundred bytes and fit the 4096-node store.
    let first = aamp_load(Path::new("test/aamp/Horse.blifecondition"));
    let second = aamp_load(Path::new("test/aamp/AS/Horse_Link_Surprised.bas"));
    let first_expected = ParameterIO::from_binary(&first).unwrap().to_text();
    let second_expected = ParameterIO::from_binary(&second).unwrap().to_text();

    let mut stack = ParameterIOStack::<4096>::new();
    stack.parse_in_place(&first).unwrap();
    assert_eq!(stack.to_text(), first_expected);

    // Re-parsing discards the previous document (`Store::clear`).
    stack.parse_in_place(&second).unwrap();
    assert_eq!(stack.to_text(), second_expected);
}

#[test]
fn aamp_failed_parse_leaves_empty_arena() {
    let good = aamp_load(Path::new("test/aamp/Horse.blifecondition"));
    let mut stack = ParameterIOStack::<4096>::new();
    stack.parse_in_place(&good).unwrap();
    assert!(stack.root().is_some());

    // A failed reparse resets the arena: no stale root or header values.
    let bad = b"NOPE".to_vec();
    assert!(stack.parse_in_place(&bad).is_err());
    assert!(stack.root().is_none());
    assert_eq!(stack.version(), 0);
    assert_eq!(stack.data_type(), "");
    assert_eq!(stack.len(), 0);

    // A fresh arena is empty too.
    let fresh = ParameterIOStack::<8>::new();
    assert!(fresh.root().is_none());
}

// ---------------------------------------------------------------------------
// Minimal archive, header values, malformed input
// ---------------------------------------------------------------------------

/// A minimal valid AAMP archive: header, `"xml"` data type at 0x30 and an
/// empty root list named `param_root` at 0x34.
fn aamp_minimal_root_bytes() -> Vec<u8> {
    let mut out = b"AAMP".to_vec();
    out.extend_from_slice(&2u32.to_le_bytes()); // format version
    out.extend_from_slice(&3u32.to_le_bytes()); // flags: LE + UTF-8
    out.extend_from_slice(&0u32.to_le_bytes()); // file size (unused)
    out.extend_from_slice(&7u32.to_le_bytes()); // pio_version
    out.extend_from_slice(&4u32.to_le_bytes()); // pio_offset: root at 0x34
    out.extend_from_slice(&[0u8; 24]); // counts and section sizes
    out.extend_from_slice(b"xml\0"); // data_type at 0x30
    out.extend_from_slice(&hash_name("param_root").to_le_bytes());
    out.extend_from_slice(&[0u8; 8]); // lists_rel, list_count, objects_rel, object_count
    out
}

#[test]
fn aamp_minimal_archive() {
    let data = aamp_minimal_root_bytes();
    let owned = ParameterIO::from_binary(&data).unwrap();
    assert_eq!(owned.version, 7);
    assert_eq!(owned.data_type, "xml");

    let expected = owned.to_text();
    assert_eq!(
        ParameterIORef::from_binary(&data).unwrap().to_text(),
        expected
    );
    let arena = ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&data).unwrap();
    assert_eq!(arena.version(), 7);
    assert_eq!(arena.data_type(), "xml");
    assert_eq!(arena.to_text(), expected);
}

#[test]
fn aamp_malformed_input_errors() {
    let bad_magic = b"NOPE".to_vec();
    assert!(ParameterIO::from_binary(&bad_magic).is_err());
    assert!(ParameterIORef::from_binary(&bad_magic).is_err());
    assert!(ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&bad_magic).is_err());

    // Truncated header.
    let truncated = b"AAMP".to_vec();
    assert!(ParameterIO::from_binary(&truncated).is_err());
    assert!(ParameterIORef::from_binary(&truncated).is_err());
    assert!(ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&truncated).is_err());

    // Unsupported format version.
    let mut bad_version = aamp_minimal_root_bytes();
    bad_version[4..8].copy_from_slice(&3u32.to_le_bytes());
    assert!(ParameterIO::from_binary(&bad_version).is_err());
    assert!(ParameterIORef::from_binary(&bad_version).is_err());
    assert!(ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&bad_version).is_err());
}

/// Root list -> object `"Obj"` -> two params both named `"Dup"`, an `Int`
/// with values 1 then 2.
fn aamp_duplicate_param_bytes() -> Vec<u8> {
    let mut bytes = aamp_minimal_root_bytes();
    // Root entry at 0x34: objects table at 0x34 + 4*4 = 0x44.
    bytes[0x34 + 8..0x34 + 10].copy_from_slice(&(4u16).to_le_bytes()); // objects_rel
    bytes[0x34 + 10..0x34 + 12].copy_from_slice(&[1u8, 0]); // object_count
    // Object entry "Obj" at 0x44: params table at 0x44 + 4*4 = 0x54.
    bytes.resize(0x54, 0);
    bytes[0x44..0x48].copy_from_slice(&hash_name("Obj").to_le_bytes());
    bytes[0x48..0x4a].copy_from_slice(&(4u16).to_le_bytes()); // params_rel
    bytes[0x4a..0x4c].copy_from_slice(&[2u8, 0]); // param_count
    // Param entries at 0x54 and 0x5c; data at 0x64 and 0x68.
    bytes.resize(0x6c, 0);
    let dup = hash_name("Dup").to_le_bytes();
    bytes[0x54..0x58].copy_from_slice(&dup);
    bytes[0x58..0x5b].copy_from_slice(&[4u8, 0, 0]); // data_rel = 4 -> 0x64
    bytes[0x5b] = 2; // Int
    bytes[0x5c..0x60].copy_from_slice(&dup);
    bytes[0x60..0x63].copy_from_slice(&[3u8, 0, 0]); // data_rel = 3 -> 0x68
    bytes[0x63] = 2; // Int
    bytes[0x64..0x68].copy_from_slice(&1i32.to_le_bytes());
    bytes[0x68..0x6c].copy_from_slice(&2i32.to_le_bytes());
    bytes
}

#[test]
fn aamp_duplicate_names_match_owned() {
    // The owned maps collapse duplicate names (first position, last value);
    // the borrowed emitters must do the same.
    let data = aamp_duplicate_param_bytes();
    let expected = ParameterIO::from_binary(&data).unwrap().to_text();
    assert_eq!(
        ParameterIORef::from_binary(&data).unwrap().to_text(),
        expected
    );
    assert_eq!(
        ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&data)
            .unwrap()
            .to_text(),
        expected
    );
}

/// Root list with an object and a list both named `"Same"`, each also
/// duplicated within its own namespace: objects `"Same"` (param `p` = 1,
/// then `p` = 2) and lists `"Same"` (object `"Inner"` with `v` = 1, then
/// `"Inner2"` with `v` = 2). Objects and lists are separate namespaces in
/// the owned model, so the shared name must survive; duplicates collapse
/// within each namespace (first position, last value).
fn aamp_mixed_names_bytes() -> Vec<u8> {
    let mut bytes = aamp_minimal_root_bytes();
    bytes.resize(0xE0, 0);
    let name = |s: &str| hash_name(s).to_le_bytes();

    // Root at 0x34: lists table at 0x40 (rel 3, count 2), objects table at
    // 0x58 (rel 9, count 2).
    bytes[0x38..0x3A].copy_from_slice(&3u16.to_le_bytes());
    bytes[0x3A..0x3C].copy_from_slice(&2u16.to_le_bytes());
    bytes[0x3C..0x3E].copy_from_slice(&9u16.to_le_bytes());
    bytes[0x3E..0x40].copy_from_slice(&2u16.to_le_bytes());

    // List entries "Same" at 0x40 and 0x4C, each with one object child.
    bytes[0x40..0x44].copy_from_slice(&name("Same"));
    bytes[0x48..0x4A].copy_from_slice(&20u16.to_le_bytes()); // objects_rel -> 0x90
    bytes[0x4A..0x4C].copy_from_slice(&1u16.to_le_bytes()); // object_count
    bytes[0x4C..0x50].copy_from_slice(&name("Same"));
    bytes[0x54..0x56].copy_from_slice(&19u16.to_le_bytes()); // objects_rel -> 0x98
    bytes[0x56..0x58].copy_from_slice(&1u16.to_le_bytes()); // object_count

    // Object entries "Same" (root's objects table) at 0x58 and 0x60, each
    // with one Int param.
    bytes[0x58..0x5C].copy_from_slice(&name("Same"));
    bytes[0x5C..0x5E].copy_from_slice(&26u16.to_le_bytes()); // params_rel -> 0xC0
    bytes[0x5E..0x60].copy_from_slice(&1u16.to_le_bytes()); // param_count
    bytes[0x60..0x64].copy_from_slice(&name("Same"));
    bytes[0x64..0x66].copy_from_slice(&26u16.to_le_bytes()); // params_rel -> 0xC8
    bytes[0x66..0x68].copy_from_slice(&1u16.to_le_bytes()); // param_count

    // The two lists' object children: "Inner" at 0x90, "Inner2" at 0x98.
    bytes[0x90..0x94].copy_from_slice(&name("Inner"));
    bytes[0x94..0x96].copy_from_slice(&8u16.to_le_bytes()); // params_rel -> 0xB0
    bytes[0x96..0x98].copy_from_slice(&1u16.to_le_bytes());
    bytes[0x98..0x9C].copy_from_slice(&name("Inner2"));
    bytes[0x9C..0x9E].copy_from_slice(&8u16.to_le_bytes()); // params_rel -> 0xB8
    bytes[0x9E..0xA0].copy_from_slice(&1u16.to_le_bytes());

    // Param entries: "v" at 0xB0 (data 0xD0), "v" at 0xB8 (data 0xD4),
    // "p" at 0xC0 (data 0xD8), "p" at 0xC8 (data 0xDC).
    for (entry, key, rel, data) in [
        (0xB0u32, "v", 8u8, 0xD0u32),
        (0xB8, "v", 7, 0xD4),
        (0xC0, "p", 6, 0xD8),
        (0xC8, "p", 5, 0xDC),
    ] {
        let e = entry as usize;
        bytes[e..e + 4].copy_from_slice(&name(key));
        bytes[e + 4..e + 7].copy_from_slice(&[rel, 0, 0]); // data_rel
        bytes[e + 7] = 2; // Int
        let d = data as usize;
        let value = if key == "v" { d - 0xD0 + 1 } else { d - 0xD8 + 1 };
        bytes[d..d + 4].copy_from_slice(&(value as i32).to_le_bytes());
    }
    bytes
}

#[test]
fn aamp_namespaces_dedup_match_owned() {
    let data = aamp_mixed_names_bytes();
    let owned = ParameterIO::from_binary(&data).unwrap();

    // The shared name survives in both namespaces; duplicates collapsed to
    // one entry each, last value winning.
    let same = Name(hash_name("Same"));
    let root = &owned.param_root;
    assert_eq!(root.objects.len(), 1);
    assert_eq!(root.lists.len(), 1);
    assert!(root.objects.get(same).is_some());
    assert!(root.lists.get(same).is_some());

    let expected = owned.to_text();
    assert_eq!(
        ParameterIORef::from_binary(&data).unwrap().to_text(),
        expected
    );
    assert_eq!(
        ParameterIOArena::<OwnedStore<AampNode<'_>>>::from_binary(&data)
            .unwrap()
            .to_text(),
        expected
    );
}
