//! Tests for YAML emission directly from the zero-copy BYML readers.
//!
//! Output is byte-identical to the owned emitter ([`Byml::to_text`]).
//! Compiled only with the `yaml` feature (which implies `byml` + `alloc`).

#![cfg(feature = "yaml")]

use std::borrow::Cow;
use std::fs;
use std::panic;
use std::path::{Path, PathBuf};

use roead::Endian;
use roead::aamp::arena::{AampNode, ParameterIOArena, ParameterIOStack};
use roead::aamp::{ParameterIO, ParameterIORef};
use roead::byml::arena::*;
use roead::byml::{Byml, BymlRef};
use roead::storage::{BorrowedStore, OwnedStore};

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
        // don't fit. Parse in place: constructing an 8192-node inline store
        // and moving it by value would overflow small stacks in debug
        // builds (moves are not elided there).
        let mut stack = BymlStack::<8192>::new();
        if stack.parse_in_place(data.as_ref()).is_ok() {
            assert_eq!(
                stack.to_text(),
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

// ---------------------------------------------------------------------------
// 8. Stack arenas: parse_in_place, store reuse, and small from_binary
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

    // Re-parsing a different document into the same arena discards the
    // previous contents (`Store::clear`).
    stack.parse_in_place(other.as_ref()).unwrap();
    assert_eq!(stack.to_text(), other_expected);

    // `from_binary` remains available for small inline capacities.
    let from_bin = BymlStack::<1024>::from_binary(tiny.as_ref()).unwrap();
    assert_eq!(from_bin.to_text(), tiny_expected);
}

// ---------------------------------------------------------------------------
// AAMP: byte-identity against the owned emitter, across the corpus
// ---------------------------------------------------------------------------

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

        // `ParameterIOStack` has a fixed 4096-node capacity: skip documents
        // that do not fit. Parsed in place so the inline store is never moved
        // by value.
        let mut stack = ParameterIOStack::<4096>::new();
        if stack.parse_in_place(&data).is_ok() {
            assert_eq!(
                stack.to_text(),
                expected,
                "ParameterIOStack byte-identity failed for {}",
                path.display()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AAMP: direct fixture check
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
// AAMP: roundtrip through the owned YAML parser
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
// AAMP: stack arena reuse
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

// ---------------------------------------------------------------------------
// AAMP: minimal archive, header values, malformed input
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
    out.extend_from_slice(&roead::aamp::hash_name("param_root").to_le_bytes());
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
    bytes[0x44..0x48].copy_from_slice(&roead::aamp::hash_name("Obj").to_le_bytes());
    bytes[0x48..0x4a].copy_from_slice(&(4u16).to_le_bytes()); // params_rel
    bytes[0x4a..0x4c].copy_from_slice(&[2u8, 0]); // param_count
    // Param entries at 0x54 and 0x5c; data at 0x64 and 0x68.
    bytes.resize(0x6c, 0);
    let dup = roead::aamp::hash_name("Dup").to_le_bytes();
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
