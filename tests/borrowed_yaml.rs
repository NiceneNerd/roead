//! Tests for YAML emission directly from the zero-copy BYML readers
//! ([`BymlRef`] and [`BymlArena`]).
//!
//! Output is byte-identical to the owned emitter ([`Byml::to_text`]).
//! Compiled only with the `yaml` and `byml` features.

#![cfg(all(feature = "yaml", feature = "byml"))]

use std::borrow::Cow;
use std::fs;
use std::panic;
use std::path::PathBuf;

use roead::Endian;
use roead::Error;
use roead::byml::arena::*;
use roead::byml::{Byml, BymlRef};

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

/// Decompress Yaz0 input when the `yaz0` feature is on; otherwise pass
/// through. `D-3_Dynamic.byml` is Yaz0-compressed.
fn prepared(data: &[u8]) -> Cow<'_, [u8]> {
    #[cfg(feature = "yaz0")]
    let data = roead::yaz0::decompress_if(data);
    #[cfg(not(feature = "yaz0"))]
    let data = Cow::Borrowed(data);
    data
}

/// Parse `data` into a `BorrowedStore` (growing the backing buffer until it
/// fits) and return its YAML text. A single `BorrowedStore` parse.
fn parse_slice_text(data: &[u8]) -> String {
    let count = BymlArena::<OwnedStore<Node<'_>>>::from_binary(data).unwrap().len();
    let mut buffer: Vec<Node<'_>> = Vec::with_capacity(count);
    let slots = buffer.spare_capacity_mut();
    let arena = BymlArena::<BorrowedStore<'_, Node<'_>>>::from_binary_in(data, slots).unwrap();
    arena.to_text()
}

// ---------------------------------------------------------------------------
// 1. Byte-identity against the owned emitter, across the corpus
// ---------------------------------------------------------------------------

#[test]
fn byte_identity_corpus() {
    for path in corpus_files() {
        let name = path.file_name().unwrap().to_str().unwrap();
        let file = load(name);
        if name == "D-3_Dynamic.byml" && !cfg!(feature = "yaz0") {
            continue;
        }
        let data = prepared(&file);
        let expected = Byml::from_binary(data.as_ref()).unwrap().to_text();

        assert_eq!(
            BymlRef::from_binary(data.as_ref()).unwrap().to_text(),
            expected,
            "BymlRef byte-identity failed for {name}"
        );
        assert_eq!(
            BymlArena::<OwnedStore<Node<'_>>>::from_binary(data.as_ref())
                .unwrap()
                .to_text(),
            expected,
            "OwnedStore byte-identity failed for {name}"
        );
        // BymlStack has a fixed 8192-node capacity: documents that do not
        // fit are the only expected failure. Parse in place: constructing an
        // 8192-node inline store and moving it by value would overflow small
        // stacks in debug builds (moves are not elided there).
        let mut stack = BymlStack::<8192>::new();
        match stack.parse_in_place(data.as_ref()) {
            Ok(()) => assert_eq!(
                stack.to_text(),
                expected,
                "BymlStack byte-identity failed for {name}"
            ),
            Err(Error::CapacityExceeded(_)) => {}
            Err(err) => panic!("unexpected parse error for {name}: {err}"),
        }
        assert_eq!(
            parse_slice_text(data.as_ref()),
            expected,
            "BorrowedStore byte-identity failed for {name}"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Direct fixture check
// ---------------------------------------------------------------------------

#[test]
fn fixture_a1_dynamic() {
    let file = load("A-1_Dynamic.byml");
    let data = prepared(&file);
    // Normalize CRLF in case the checkout converted line endings despite
    // .gitattributes (e.g. a pre-existing clone); the emitter always uses LF.
    let expected =
        fs::read_to_string("test/fixtures/byml/A-1_Dynamic.byml.yml.expected")
            .unwrap()
            .replace("\r\n", "\n");
    assert_eq!(
        BymlRef::from_binary(data.as_ref()).unwrap().to_text(),
        expected
    );
    assert_eq!(
        BymlArena::<OwnedStore<Node<'_>>>::from_binary(data.as_ref())
            .unwrap()
            .to_text(),
        expected
    );
}

// ---------------------------------------------------------------------------
// 3. Roundtrip through the owned parser
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_corpus() {
    for path in corpus_files() {
        let name = path.file_name().unwrap().to_str().unwrap();
        let file = load(name);
        if name == "D-3_Dynamic.byml" && !cfg!(feature = "yaz0") {
            continue;
        }
        let data = prepared(&file);
        let ref_text = BymlRef::from_binary(data.as_ref()).unwrap().to_text();
        let roundtripped = Byml::from_text(&ref_text).unwrap();
        let original = Byml::from_binary(data.as_ref()).unwrap();
        assert_eq!(roundtripped, original, "roundtrip failed for {name}");
    }
}

// ---------------------------------------------------------------------------
// 4. Null root and scalar-root panic
// ---------------------------------------------------------------------------

fn null_doc(root_offset: u32, endian: Endian) -> Vec<u8> {
    let mut out = match endian {
        Endian::Big => b"BY".to_vec(),
        Endian::Little => b"YB".to_vec(),
    };
    out.extend_from_slice(&match endian {
        Endian::Big => (2u16).to_be_bytes(),
        Endian::Little => (2u16).to_le_bytes(),
    });
    for v in [0u32, 0, root_offset] {
        out.extend_from_slice(&match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        });
    }
    out
}

#[test]
fn null_root_is_null() {
    // Root offset 0: both endians, all three readers.
    for endian in [Endian::Big, Endian::Little] {
        let doc = null_doc(0, endian);
        assert_eq!(BymlRef::from_binary(&doc).unwrap().to_text(), "null");
        assert_eq!(
            BymlArena::<OwnedStore<Node<'_>>>::from_binary(&doc)
                .unwrap()
                .to_text(),
            "null"
        );
        assert_eq!(parse_slice_text(&doc), "null");
    }
}

fn panic_message(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
    let err = panic::catch_unwind(panic::AssertUnwindSafe(f)).unwrap_err();
    err.downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| err.downcast_ref::<String>().cloned())
        .unwrap_or_default()
}

#[test]
fn scalar_root_panics() {
    // `BymlRef::to_text` panics on a scalar root, matching the owned emitter.
    let msg = panic_message(|| {
        let _ = BymlRef::I32(42).to_text();
    });
    assert_eq!(msg, "BYML must be container or null to serialize");
}

// ---------------------------------------------------------------------------
// 5. Duplicate map keys collapse like the owned map (last value wins)
// ---------------------------------------------------------------------------

#[test]
fn arena_duplicate_map_keys_match_owned() {
    // The owned map overwrites on duplicate insert. The arena emitter must
    // collapse to the last value before deciding whether to use flow style:
    // ten raw entries can therefore still produce an inline one-entry map.
    for count in [2u32, 10] {
        let hash_key_table = 0x10 + 4 + 8 * count;
        let mut doc = b"BY".to_vec();
        doc.extend_from_slice(&2u16.to_be_bytes()); // version
        doc.extend_from_slice(&hash_key_table.to_be_bytes());
        doc.extend_from_slice(&0u32.to_be_bytes()); // no string table
        doc.extend_from_slice(&0x10u32.to_be_bytes()); // root at 0x10
        doc.push(0xC1); // Map
        doc.extend_from_slice(&count.to_be_bytes()[1..]);
        for value in 0..count {
            doc.extend_from_slice(&0u32.to_be_bytes()[1..]); // name index 0
            doc.push(0xD1); // I32
            doc.extend_from_slice(&value.to_be_bytes());
        }
        // String table: tag 0xC2, size 1, then relative start/end offsets.
        doc.extend_from_slice(&[0xC2, 0x00, 0x00, 0x01]);
        doc.extend_from_slice(&0x0Cu32.to_be_bytes());
        doc.extend_from_slice(&0x10u32.to_be_bytes());
        doc.extend_from_slice(b"key\0");

        let expected = Byml::from_binary(&doc).unwrap().to_text();
        assert_eq!(
            BymlRef::from_binary(&doc).unwrap().to_text(),
            expected,
            "BymlRef duplicate-key mismatch with {count} raw entries"
        );
        assert_eq!(
            BymlArena::<OwnedStore<Node<'_>>>::from_binary(&doc)
                .unwrap()
                .to_text(),
            expected,
            "arena duplicate-key mismatch with {count} raw entries"
        );
        assert_eq!(parse_slice_text(&doc), expected);
    }
}

// ---------------------------------------------------------------------------
// 6. 100 KiB pseudorandom BinaryData: identical owned vs borrowed
// ---------------------------------------------------------------------------

#[test]
fn binary_data_large_identical() {
    let mut data = Vec::with_capacity(100 * 1024);
    let mut state: u64 = 0x1234_5678_9ABC_DEF0;
    for _ in 0..100 * 1024 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        data.push(((state >> 33) & 0xff) as u8);
    }

    // Wrap in an array so the owned writer (which requires a container root)
    // can serialize it to binary.
    let owned = Byml::Array(vec![Byml::BinaryData(data.clone())]);
    let owned_bytes = owned.to_binary(Endian::Little);
    let owned_text = Byml::from_binary(&owned_bytes).unwrap().to_text();

    let borrowed = BymlRef::from_binary(&owned_bytes).unwrap().to_text();
    assert_eq!(borrowed, owned_text);

    let arena = BymlArena::<OwnedStore<Node<'_>>>::from_binary(&owned_bytes)
        .unwrap()
        .to_text();
    assert_eq!(arena, owned_text);
}

// ---------------------------------------------------------------------------
// 7. BE fixtures: to_text equals the LE parse's output
// ---------------------------------------------------------------------------

#[test]
fn be_fixtures_match_le() {
    for entry in fs::read_dir("test/fixtures/byml").unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.ends_with(".be.bin.expected") {
            continue;
        }
        let be_data = fs::read(&path).unwrap();
        // Compare the BE parse's text to the owned parse of the same bytes.
        let be_text = BymlRef::from_binary(&be_data).unwrap().to_text();
        let be_owned = Byml::from_binary(&be_data).unwrap().to_text();
        assert_eq!(be_text, be_owned, "BE byte-identity failed for {name}");

        // The corresponding LE fixture (same document, little endian).
        let le_name = name.replace(".be.bin.expected", ".le.bin.expected");
        let le_path = path.with_file_name(&le_name);
        if le_path.exists() {
            let le_data = fs::read(&le_path).unwrap();
            let le_text = Byml::from_binary(&le_data).unwrap().to_text();
            assert_eq!(be_text, le_text, "BE vs LE text mismatch for {name}");
        }
    }
}

// ---------------------------------------------------------------------------
// 8. D-3_Dynamic (Yaz0) when yaz0 is on
// ---------------------------------------------------------------------------

#[test]
fn yaz0_d3_dynamic() {
    #[cfg(feature = "yaz0")]
    {
        let data = load("D-3_Dynamic.byml");
        let decompressed = roead::yaz0::decompress_if(&data);
        let expected = Byml::from_binary(decompressed.as_ref()).unwrap().to_text();
        assert_eq!(
            BymlRef::from_binary(decompressed.as_ref())
                .unwrap()
                .to_text(),
            expected
        );
        assert_eq!(
            BymlArena::<OwnedStore<Node<'_>>>::from_binary(decompressed.as_ref())
                .unwrap()
                .to_text(),
            expected
        );
    }
    #[cfg(not(feature = "yaz0"))]
    {
        // Without yaz0 the document cannot be parsed; just ensure the test
        // compiles and passes vacuously.
    }
}

// ---------------------------------------------------------------------------
// 9. Stack arenas: parse_in_place, store reuse, and small from_binary
// ---------------------------------------------------------------------------

/// A tiny corpus file that fits a 1024-node stack arena.
const TINY_BYML: &str = "Mrg_01e57204_MrgD100_B4-B3-B2-1A90E17A.bcett.byml";

#[test]
fn stack_parse_in_place_reuse_and_from_binary() {
    let tiny_raw = load(TINY_BYML);
    let tiny = prepared(&tiny_raw);
    let tiny_expected = Byml::from_binary(tiny.as_ref()).unwrap().to_text();
    let other_raw = load("LevelSensor.byml");
    let other = prepared(&other_raw);
    let other_expected = Byml::from_binary(other.as_ref()).unwrap().to_text();

    // `parse_in_place` moves nothing, so an inline capacity that fits both
    // documents is safe on a default-size test thread. (`LevelSensor.byml`
    // needs ~2.6k nodes; the tiny file needs 66.)
    let mut stack = BymlStack::<4096>::new();
    stack.parse_in_place(tiny.as_ref()).unwrap();
    assert_eq!(stack.to_text(), tiny_expected);
    assert!(stack.root().is_some());

    // Re-parsing a different document into the same arena discards the
    // previous contents (`Store::clear`).
    stack.parse_in_place(other.as_ref()).unwrap();
    assert_eq!(stack.to_text(), other_expected);

    // A failed reparse leaves the arena empty.
    let bad = b"NOPE".to_vec();
    assert!(stack.parse_in_place(&bad).is_err());
    assert!(stack.root().is_none());
    assert_eq!(stack.len(), 0);

    // A fresh arena is empty too.
    assert!(BymlStack::<8>::new().root().is_none());

    // `from_binary` remains available for small inline capacities.
    let from_bin = BymlStack::<1024>::from_binary(tiny.as_ref()).unwrap();
    assert_eq!(from_bin.to_text(), tiny_expected);
}
