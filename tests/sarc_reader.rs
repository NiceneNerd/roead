//! Corpus + malformed-input tests for the alloc-free SARC reader.
//!
//! The reader itself is core-only (`--no-default-features --features sarc`
//! compiles it with zero alloc), but this integration test links the std test
//! harness so it can read fixtures from disk. Parts that only need `core` are
//! gated on `cfg(feature = "std")` for I/O and left as plain `fn`s for the
//! alloc-free path where possible.

#![cfg(feature = "sarc")]

use roead::Error;
use roead::sarc::Sarc;

#[cfg(feature = "std")]
fn corpus_files() -> Vec<std::path::PathBuf> {
    use std::ffi::OsStr;
    use std::fs;
    fs::read_dir("test/sarc")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension() == Some(OsStr::new("sarc")) || p.extension().is_none())
        .collect()
}

#[cfg(feature = "std")]
#[test]
fn corpus_parse_roundtrip() {
    let mut count = 0usize;
    for path in corpus_files() {
        let data = std::fs::read(&path).unwrap();
        let sarc = Sarc::new(&data).unwrap();
        let len = sarc.len();
        count += 1;
        // Every file listed by the iterator must be retrievable by name.
        for file in sarc.files() {
            let name = file.name().expect("corpus files are named");
            let data_again = sarc.get_data(name).expect("file data retrievable");
            assert_eq!(data_again, file.data(), "{}", name);
            let file_again = sarc.get(name).expect("file retrievable");
            assert_eq!(file_again.data(), file.data());
        }
        // Alignment guess must be a power of two.
        let al = sarc.guess_min_alignment();
        assert!(al.is_power_of_two(), "{} alignment {}", path.display(), al);
        println!("{} files={} alignment={}", path.display(), len, al);
    }
    assert!(count > 0, "expected at least one corpus SARC");
}

#[cfg(feature = "std")]
#[test]
fn spot_check_dungeon119() {
    let data = std::fs::read("test/sarc/Dungeon119.pack").unwrap();
    let sarc = Sarc::new(&data).unwrap();
    assert_eq!(sarc.len(), 10);
    assert_eq!(sarc.endian(), roead::Endian::Big);
    assert_eq!(sarc.guess_min_alignment(), 4);
    for name in [
        "NavMesh/CDungeon/Dungeon119/Dungeon119.shknm2",
        "Map/CDungeon/Dungeon119/Dungeon119_Static.smubin",
        "Model/DgnMrgPrt_Dungeon119.Tex2.sbfres",
    ] {
        sarc.get(name).unwrap_or_else(|| panic!("missing {name}"));
    }
}

// ---- alloc-free core path: hand-built minimal SARC (empty archive, big endian) ----
//
// Bytes (all little-endian): SARC | header_size=0x0014 | BOM=0xFEFF
// | file_size=0x28 | data_offset=0x28 | version=0x0100 | reserved=0
// | SFAT | header_size=0x000C | num_files=0 | hash_multiplier=0x65
// | SFNT | header_size=0x0008 | reserved=0
// Total = 0x28 = 40 bytes, zero files.
const MINIMAL_LE: &[u8] = &[
    0x53, 0x41, 0x52, 0x43, // SARC
    0x14, 0x00, // header_size = 0x0014
    0xFF, 0xFE, // BOM = 0xFEFF -> Little
    0x28, 0x00, 0x00, 0x00, // file_size = 0x28
    0x28, 0x00, 0x00, 0x00, // data_offset = 0x28
    0x00, 0x01, // version = 0x0100
    0x00, 0x00, // reserved
    0x53, 0x46, 0x41, 0x54, // SFAT
    0x0C, 0x00, // header_size = 0x000C
    0x00, 0x00, // num_files = 0
    0x65, 0x00, 0x00, 0x00, // hash_multiplier = 0x65
    0x53, 0x46, 0x4E, 0x54, // SFNT
    0x08, 0x00, // header_size = 0x0008
    0x00, 0x00, // reserved
];

#[test]
fn minimal_sarc_parses() {
    let sarc = Sarc::new(MINIMAL_LE).unwrap();
    assert_eq!(sarc.len(), 0);
    assert!(sarc.is_empty());
    assert_eq!(sarc.endian(), roead::Endian::Little);
    assert_eq!(sarc.data_offset(), 0x28);
    assert!(sarc.files().next().is_none());
}

// ---- malformed inputs ----

#[test]
fn truncated_header() {
    let err = Sarc::new(b"SARC\x00\x14").unwrap_err();
    assert!(matches!(err, Error::InsufficientData(..)));
}

#[test]
fn bad_sarc_magic() {
    let err = Sarc::new(b"XXXX\x00\x14\xfe\xff\x00\x00\x00\x28\x00\x00\x00\x28\x01\x00\x00\x00")
        .unwrap_err();
    assert!(matches!(err, Error::BadMagic(..)));
}

#[test]
fn bad_bom() {
    let err = Sarc::new(b"SARC\x00\x14\x00\x00\x00\x00\x00\x28\x00\x00\x00\x28\x01\x00\x00\x00")
        .unwrap_err();
    assert!(matches!(err, Error::InvalidData(_)));
}

#[test]
fn wrong_version() {
    let err = Sarc::new(b"SARC\x00\x14\xfe\xff\x00\x00\x00\x28\x00\x00\x00\x28\x02\x00\x00\x00")
        .unwrap_err();
    assert!(matches!(err, Error::InvalidData(_)));
}

#[test]
fn wrong_header_size() {
    // header_size field at offset 4 set to 0x000C instead of 0x0014
    let err = Sarc::new(b"SARC\x00\x0c\xfe\xff\x00\x00\x00\x28\x00\x00\x00\x28\x01\x00\x00\x00")
        .unwrap_err();
    assert!(matches!(err, Error::InvalidData(_)));
}

#[test]
fn wrong_sfat_size() {
    let mut buf = MINIMAL_LE.to_vec();
    // SFAT header_size at offset 0x18 set to 0x0008
    buf[0x18] = 0x08;
    let err = Sarc::new(&buf).unwrap_err();
    assert!(matches!(err, Error::InvalidData(_)));
}

#[test]
fn wrong_sfnt_size() {
    let mut buf = MINIMAL_LE.to_vec();
    // SFNT header_size at offset 0x24 set to 0x0004 (LE low byte)
    buf[0x24] = 0x04;
    let err = Sarc::new(&buf).unwrap_err();
    assert!(matches!(err, Error::InvalidData(_)));
}

#[test]
fn too_many_files() {
    let mut buf = MINIMAL_LE.to_vec();
    // num_files at offset 0x1A = 0xC000 (>>0xE != 0), little-endian bytes 00 C0
    buf[0x1A] = 0x00;
    buf[0x1B] = 0xC0;
    let err = Sarc::new(&buf).unwrap_err();
    assert!(matches!(err, Error::TooManyFiles(..)));
}

#[test]
fn index_out_of_range() {
    let sarc = Sarc::new(MINIMAL_LE).unwrap();
    let err = sarc.file_at(0).unwrap_err();
    assert!(matches!(err, Error::IndexOutOfRange { .. }));
}

#[cfg(feature = "std")]
#[test]
fn data_range_out_of_bounds() {
    // num_files=1 whose single entry claims a data range past the end of the
    // archive. Layout: SARC(0x14) + SFAT(0x0C) + 1 entry(0x10) + SFNT(0x08).
    let mut buf = vec![0u8; 0x38];
    buf[0..4].copy_from_slice(b"SARC");
    buf[4..6].copy_from_slice(&0x0014u16.to_le_bytes()); // header_size
    buf[6..8].copy_from_slice(&0xFEFFu16.to_le_bytes()); // BOM Little
    buf[8..12].copy_from_slice(&0x38u32.to_le_bytes()); // file_size
    buf[12..16].copy_from_slice(&0x38u32.to_le_bytes()); // data_offset
    buf[16..18].copy_from_slice(&0x0100u16.to_le_bytes()); // version
    buf[0x14..0x18].copy_from_slice(b"SFAT");
    buf[0x18..0x1a].copy_from_slice(&0x000Cu16.to_le_bytes()); // fat header_size
    buf[0x1a..0x1c].copy_from_slice(&1u16.to_le_bytes()); // num_files
    buf[0x1c..0x20].copy_from_slice(&0x65u32.to_le_bytes()); // hash_multiplier
    // entry[0] at 0x20: data range past end
    buf[0x20..0x24].copy_from_slice(&0u32.to_le_bytes()); // name_hash
    buf[0x24..0x28].copy_from_slice(&0u32.to_le_bytes()); // rel_name_opt_offset
    buf[0x28..0x2c].copy_from_slice(&0x00FF_FFFFu32.to_le_bytes()); // data_begin
    buf[0x2c..0x30].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // data_end
    buf[0x30..0x34].copy_from_slice(b"SFNT");
    buf[0x34..0x36].copy_from_slice(&0x0008u16.to_le_bytes());
    let sarc = Sarc::new(&buf).unwrap();
    assert_eq!(sarc.len(), 1);
    assert!(sarc.file_at(0).is_err());
}

// ---- are_files_equal + File equality (fast path) ----

#[cfg(feature = "std")]
#[test]
fn are_files_equal_and_file_eq() {
    let data = std::fs::read("test/sarc/test.sarc").unwrap();
    let a = Sarc::new(&data).unwrap();
    let b = Sarc::new(&data).unwrap();
    assert!(Sarc::are_files_equal(&a, &b));

    let f1 = a.get("blyt/BootUp_00.bflyt").unwrap();
    let f2 = b.get("blyt/BootUp_00.bflyt").unwrap();
    assert_eq!(f1, f2);
    assert_eq!(f1.index(), f2.index());
    assert_eq!(f1.data(), f2.data());
    assert_eq!(f1.name(), f2.name());

    // Same data, different index -> not equal.
    let f_first = a.files().next().unwrap();
    let f_last = a.files().last().unwrap();
    if f_first.index() != f_last.index() && f_first.data() == f_last.data() {
        assert_ne!(f_first, f_last);
    }
}
