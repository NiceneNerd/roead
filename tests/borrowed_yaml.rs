//! Tests for YAML emission directly from the zero-copy BYML readers.
//!
//! Output is byte-identical to the owned emitter ([`Byml::to_text`]).
//! Compiled only with the `yaml` feature (which implies `byml` + `alloc`).

#![cfg(feature = "yaml")]

use std::borrow::Cow;
use std::fs;
use std::panic;
use std::path::PathBuf;

use roead::Endian;
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

/// Parse `data` into a `SliceStore`, growing the backing buffer until it
/// fits.
/// Parse `data` into a `SliceStore` (growing the backing buffer until it
/// fits) and return its YAML text. A single `SliceStore` parse.
fn parse_slice_text(data: &[u8]) -> String {
    let count = BymlArena::<VecStore<'_>>::from_binary(data).unwrap().len();
    let mut buffer: Vec<Node<'_>> = Vec::with_capacity(count);
    let slots = buffer.spare_capacity_mut();
    let arena = BymlArena::<SliceStore<'_, '_>>::from_binary_in(data, slots).unwrap();
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
            BymlArena::<VecStore<'_>>::from_binary(data.as_ref())
                .unwrap()
                .to_text(),
            expected,
            "VecStore byte-identity failed for {name}"
        );
        // BymlStack has a fixed 8192-node capacity: skip documents that
        // don't fit.
        if let Ok(stack_arena) = BymlArena::<ArrayStore<8192>>::from_binary(data.as_ref()) {
            assert_eq!(
                stack_arena.to_text(),
                expected,
                "BymlStack byte-identity failed for {name}"
            );
        }
        assert_eq!(
            parse_slice_text(data.as_ref()),
            expected,
            "SliceStore byte-identity failed for {name}"
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
    let expected = fs::read_to_string("test/fixtures/byml/A-1_Dynamic.byml.yml.expected").unwrap();
    assert_eq!(
        BymlRef::from_binary(data.as_ref()).unwrap().to_text(),
        expected
    );
    assert_eq!(
        BymlArena::<VecStore<'_>>::from_binary(data.as_ref())
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
            BymlArena::<VecStore<'_>>::from_binary(&doc)
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
        .or_else(|| err.downcast_ref::<String>().map(|s| s.clone()))
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
// 100 KiB pseudorandom BinaryData: identical owned vs borrowed
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

    let arena = BymlArena::<VecStore<'_>>::from_binary(&owned_bytes)
        .unwrap()
        .to_text();
    assert_eq!(arena, owned_text);
}

// ---------------------------------------------------------------------------
// BE fixtures: to_text equals the LE parse's output
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
        // Strip the trailing ".be.bin.expected" -> ".byml" to find the LE
        // source fixture, if present; otherwise build the LE counterpart by
        // reparsing the BE bytes and re-emitting little endian via the
        // owned writer (round-trip). Simplest robust approach: compare the
        // BE parse's text to the owned LE parse of the same logical doc.
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
// D-3_Dynamic (Yaz0) when yaz0 is on
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
            BymlArena::<VecStore<'_>>::from_binary(decompressed.as_ref())
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
