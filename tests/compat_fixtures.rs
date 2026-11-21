//! Compatibility ("golden fixture") tests for roead v2.0, step 1.
//!
//! These tests capture the CURRENT parse/serialize behavior of the library as
//! committed byte-for-byte expected artifacts under `test/fixtures/`, so that
//! the v2.0 restructure (no_std, zero-copy readers, feature split) can verify
//! every later step against them.
//!
//! ## Regeneration
//!
//! Set the environment variable `UPDATE_FIXTURES=1` to rewrite the expected
//! artifact files and `test/fixtures/manifest.json` instead of comparing
//! against them. Regenerate with `--all-features` so every artifact kind
//! (including YAML) is produced:
//!
//!     UPDATE_FIXTURES=1 cargo test --test compat_fixtures --all-features
//!
//! In normal (non-update) runs each artifact is regenerated with the current
//! API and compared byte-for-byte with its committed `.expected` file. On
//! mismatch a diff showing the first differing byte/line is printed and the
//! test fails.
//!
//! Fixtures are cfg-gated: BYML tests require `byml`, AAMP tests require
//! `aamp`, and YAML (`.yml`) artifact comparisons additionally require
//! `yaml`. Binary artifact comparisons do not require `yaml`.

// ---------------------------------------------------------------------------
// cfg-free harness: fixture table, hashing, diffing, manifest emission.
// ---------------------------------------------------------------------------

mod harness {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    /// SHA-256 (FIPS 180-4) implemented in-tree so the fixture harness has no
    /// extra dependencies (Cargo.toml is intentionally unchanged in this step).
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        let mut h: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];
        let k: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];

        let mut msg = data.to_vec();
        msg.push(0x80);
        while msg.len() % 64 != 56 {
            msg.push(0);
        }
        msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());

        for chunk in msg.chunks(64) {
            let mut w = [0u32; 64];
            for i in 0..16 {
                w[i] = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }
            let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
                (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let temp1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(k[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }
            h[0] = h[0].wrapping_add(a);
            h[1] = h[1].wrapping_add(b);
            h[2] = h[2].wrapping_add(c);
            h[3] = h[3].wrapping_add(d);
            h[4] = h[4].wrapping_add(e);
            h[5] = h[5].wrapping_add(f);
            h[6] = h[6].wrapping_add(g);
            h[7] = h[7].wrapping_add(hh);
        }
        let mut out = [0u8; 32];
        for (i, v) in h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
        }
        out
    }

    pub fn sha256_hex(hash: &[u8; 32]) -> String {
        let mut s = String::with_capacity(64);
        for b in hash {
            s.push_str(&format!("{b:02x}"));
        }
        s
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Format {
        Byml,
        Aamp,
    }

    impl Format {
        pub fn dir(self) -> &'static str {
            match self {
                Format::Byml => "test/fixtures/byml",
                Format::Aamp => "test/fixtures/aamp",
            }
        }
    }

    /// One fixture: a corpus file plus the artifacts it must produce.
    pub struct Fixture {
        pub source: &'static str,
        pub format: Format,
        /// `false` means the source is >= 64 KiB: no byte snapshots are
        /// committed; only `source_sha256`/`source_size` are recorded in the
        /// manifest and a parse-smoke check runs.
        pub snapshot: bool,
        /// Feature that must be enabled for the parse-smoke check to run
        /// (e.g. Yaz0 decompression), or `None`.
        pub parse_requires: Option<&'static str>,
        pub artifacts: &'static [&'static str], // artifact kinds, e.g. &["yml", "le.bin", "be.bin"]
    }

    /// Basename of a source path (everything after the final `/`).
    pub fn basename(path: &str) -> &str {
        Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
    }

    /// Committed path of an artifact, e.g.
    /// `test/fixtures/byml/A-1_Dynamic.byml.yml.expected`.
    pub fn artifact_path(format: Format, basename: &str, kind: &str) -> String {
        format!("{}/{basename}.{kind}.expected", format.dir())
    }

    /// All fixtures, in the order they are tested.
    pub const FIXTURES: &[Fixture] = &[
        // --- BYML snapshots (< 64 KiB each) ---
        Fixture {
            source: "test/byml/A-1_Dynamic.byml",
            format: Format::Byml,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "le.bin", "be.bin"],
        },
        Fixture {
            source: "test/byml/LevelSensor.byml",
            format: Format::Byml,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "le.bin", "be.bin"],
        },
        Fixture {
            source: "test/byml/MainFieldLocation.byml",
            format: Format::Byml,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "le.bin", "be.bin"],
        },
        Fixture {
            source: "test/byml/ElectricGenerator.Nin_NX_NVN.esetb.byml",
            format: Format::Byml,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "le.bin", "be.bin"],
        },
        Fixture {
            source: "test/byml/Preset0_Field.byml",
            format: Format::Byml,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "le.bin", "be.bin"],
        },
        Fixture {
            source: "test/byml/Mrg_01e57204_MrgD100_B4-B3-B2-1A90E17A.bcett.byml",
            format: Format::Byml,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "le.bin", "be.bin"],
        },
        // --- BYML manifest-only (>= 64 KiB; no byte snapshots) ---
        Fixture {
            source: "test/byml/USen.byml",
            format: Format::Byml,
            snapshot: false,
            parse_requires: None,
            artifacts: &[],
        },
        Fixture {
            source: "test/byml/J-8_Dynamic.bcett.byml",
            format: Format::Byml,
            snapshot: false,
            parse_requires: None,
            artifacts: &[],
        },
        Fixture {
            source: "test/byml/EventInfo.product.byml",
            format: Format::Byml,
            snapshot: false,
            parse_requires: None,
            artifacts: &[],
        },
        Fixture {
            source: "test/byml/GameROMPlayer.byml",
            format: Format::Byml,
            snapshot: false,
            parse_requires: None,
            artifacts: &[],
        },
        Fixture {
            source: "test/byml/ActorInfo.product.byml",
            format: Format::Byml,
            snapshot: false,
            parse_requires: None,
            artifacts: &[],
        },
        Fixture {
            source: "test/byml/D-3_Dynamic.byml",
            format: Format::Byml,
            snapshot: false,
            parse_requires: Some("yaz0"), // Yaz0-compressed; unparsable without `yaz0`
            artifacts: &[],
        },
        // --- AAMP snapshots (< 64 KiB each) ---
        Fixture {
            source: "test/aamp/d_frequentuse.baglblm",
            format: Format::Aamp,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "bin"],
        },
        Fixture {
            source: "test/aamp/actor_auto_capture.baglfila",
            format: Format::Aamp,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "bin"],
        },
        Fixture {
            source: "test/aamp/Enemy_Lizalfos_Electric.bchemical",
            format: Format::Aamp,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "bin"],
        },
        Fixture {
            source: "test/aamp/common.bagllmap",
            format: Format::Aamp,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "bin"],
        },
        Fixture {
            source: "test/aamp/EngineRom_Horse_Link.bphysics",
            format: Format::Aamp,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "bin"],
        },
        Fixture {
            source: "test/aamp/AIProgram/ShelterFromRainTag.baiprog",
            format: Format::Aamp,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "bin"],
        },
        Fixture {
            source: "test/aamp/Demo000_2.bdemo",
            format: Format::Aamp,
            snapshot: true,
            parse_requires: None,
            artifacts: &["yml", "bin"],
        },
    ];

    pub fn update_mode() -> bool {
        std::env::var("UPDATE_FIXTURES").ok().as_deref() == Some("1")
    }

    /// Read `test/fixtures/manifest.json` if present.
    pub fn read_manifest() -> Option<String> {
        fs::read_to_string("test/fixtures/manifest.json").ok()
    }

    /// Soft consistency check for the committed manifest: every fixture's
    /// source and every committed artifact path must appear in the manifest
    /// text.
    pub fn check_manifest_presence() {
        let Some(manifest) = read_manifest() else {
            return;
        };
        for fixture in FIXTURES {
            let source = basename(fixture.source);
            assert!(
                manifest.contains(source),
                "manifest.json does not mention source {source}"
            );
            for kind in fixture.artifacts {
                let path = artifact_path(fixture.format, basename(fixture.source), kind);
                assert!(
                    manifest.contains(&path),
                    "manifest.json does not mention artifact {path}"
                );
            }
        }
    }

    /// Byte comparison with a helpful diff on mismatch. Panics with the diff.
    pub fn compare_bytes(label: &str, expected: &[u8], actual: &[u8]) {
        if expected == actual {
            return;
        }
        let len_exp = expected.len();
        let len_act = actual.len();
        let mut first = None;
        for (i, (a, b)) in expected.iter().zip(actual.iter()).enumerate() {
            if a != b {
                first = Some(i);
                break;
            }
        }
        let first = first.unwrap_or_else(|| len_exp.min(len_act));
        let start = first.saturating_sub(8);
        let end = (first + 16).min(len_exp.max(len_act));
        let exp_slice = &expected[start..end.min(len_exp)];
        let act_slice = &actual[start..end.min(len_act)];
        panic!(
            "{label} mismatch: first differing byte at byte {first} \
             (expected length {len_exp}, actual length {len_act})\n  expected: {:02x?}\n  actual:   {:02x?}",
            exp_slice, act_slice
        );
    }

    /// Text comparison: reports the first differing line, then the byte diff.
    pub fn compare_text(label: &str, expected: &[u8], actual: &[u8]) {
        let exp_text = String::from_utf8_lossy(expected);
        let act_text = String::from_utf8_lossy(actual);
        for (n, (le, la)) in exp_text.lines().zip(act_text.lines()).enumerate() {
            if le != la {
                eprintln!("{label}: first differing line {n}:\n  expected: {le}\n  actual:   {la}");
                break;
            }
        }
        compare_bytes(label, expected, actual);
    }

    /// Write one artifact in update mode and record its size/sha256.
    pub fn write_artifact(
        format: Format,
        basename: &str,
        kind: &str,
        bytes: &[u8],
    ) -> (String, u64, String) {
        let path = artifact_path(format, basename, kind);
        fs::create_dir_all(Path::new(&path).parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        let hash = sha256(bytes);
        (path, bytes.len() as u64, sha256_hex(&hash))
    }

    /// Escape a string for emission as a JSON string literal.
    fn json_escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 4);
        for ch in s.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // Minimal JSON value emitted by the update path (no serde_json dependency).
    // -----------------------------------------------------------------------

    #[derive(Clone, Debug)]
    pub enum JValue {
        Null,
        Bool(bool),
        U64(u64),
        String(String),
        Map(BTreeMap<String, JValue>),
    }

    impl From<&str> for JValue {
        fn from(s: &str) -> Self {
            JValue::String(s.to_owned())
        }
    }

    impl From<u64> for JValue {
        fn from(v: u64) -> Self {
            JValue::U64(v)
        }
    }

    impl JValue {
        fn emit(&self, out: &mut String) {
            match self {
                JValue::Null => out.push_str("null"),
                JValue::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
                JValue::U64(v) => out.push_str(&v.to_string()),
                JValue::String(s) => {
                    out.push('"');
                    for ch in s.chars() {
                        match ch {
                            '"' => out.push_str("\\\""),
                            '\\' => out.push_str("\\\\"),
                            '\n' => out.push_str("\\n"),
                            '\r' => out.push_str("\\r"),
                            '\t' => out.push_str("\\t"),
                            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                            c => out.push(c),
                        }
                    }
                    out.push('"');
                }
                JValue::Map(items) => {
                    out.push('{');
                    for (i, (k, v)) in items.iter().enumerate() {
                        if i > 0 {
                            out.push(',');
                        }
                        JValue::String(k.clone()).emit(out);
                        out.push(':');
                        v.emit(out);
                    }
                    out.push('}');
                }
            }
        }
    }

    pub fn header_endian(data: &[u8]) -> Option<&'static str> {
        if data.len() < 2 {
            return None;
        }
        match &data[..2] {
            b"BY" => Some("big"),
            b"YB" => Some("little"),
            _ => None,
        }
    }

    pub fn header_version(data: &[u8]) -> Option<u64> {
        if data.len() < 4 {
            return None;
        }
        Some(u16::from_le_bytes([data[2], data[3]]) as u64)
    }

    pub struct ManifestArtifact {
        pub path: String,
        pub kind: String,
        pub size: u64,
        pub sha256: String,
    }

    pub struct ManifestEntry {
        pub source: String,
        pub format: String,
        pub source_size: u64,
        pub source_sha256: String,
        pub snapshot: bool,
        pub parse_requires: Option<String>,
        pub summary: JValue,
        pub artifacts: Vec<ManifestArtifact>,
    }

    impl ManifestEntry {
        fn emit(&self, out: &mut String) {
            out.push_str("    {\n");
            out.push_str(&format!(
                "      \"source\": \"{}\",\n",
                json_escape(&self.source)
            ));
            out.push_str(&format!("      \"format\": \"{}\",\n", self.format));
            out.push_str(&format!("      \"source_size\": {},\n", self.source_size));
            out.push_str(&format!(
                "      \"source_sha256\": \"{}\",\n",
                self.source_sha256
            ));
            out.push_str(&format!("      \"snapshot\": {},\n", self.snapshot));
            match &self.parse_requires {
                Some(f) => out.push_str(&format!("      \"parse_requires\": \"{f}\",\n")),
                None => out.push_str("      \"parse_requires\": null,\n"),
            }
            out.push_str("      \"summary\": ");
            self.summary.emit(out);
            out.push_str(",\n      \"artifacts\": [");
            for (i, a) in self.artifacts.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str("\n        {");
                out.push_str(&format!(
                    "\"path\": \"{}\", \"kind\": \"{}\", \"size\": {}, \"sha256\": \"{}\"",
                    json_escape(&a.path),
                    a.kind,
                    a.size,
                    a.sha256
                ));
                out.push('}');
            }
            if self.artifacts.is_empty() {
                out.push_str("]");
            } else {
                out.push_str("\n      ]");
            }
            out.push_str("\n    }");
        }
    }

    /// Write (or merge into) `test/fixtures/manifest.json`. Entries for
    /// sources already present in the manifest are replaced; entries written
    /// earlier by another feature-gated test module are preserved so the
    /// manifest is complete after all fixture tests have run.
    pub fn write_manifest(entries: &[ManifestEntry]) {
        fs::create_dir_all("test/fixtures").unwrap();
        let new_sources: Vec<&str> = entries.iter().map(|e| e.source.as_str()).collect();
        let mut kept_blocks: Vec<String> = Vec::new();
        if let Ok(text) = fs::read_to_string("test/fixtures/manifest.json") {
            let mut block = String::new();
            let mut in_block = false;
            let mut source: Option<String> = None;
            for line in text.lines() {
                if line.starts_with("    {") && !in_block {
                    in_block = true;
                    block.clear();
                    source = None;
                    block.push_str(line);
                    block.push('\n');
                    continue;
                }
                if in_block {
                    block.push_str(line);
                    block.push('\n');
                    if source.is_none() {
                        if let Some(rest) = line.strip_prefix("      \"source\": \"") {
                            source = Some(rest.split('"').next().unwrap().to_string());
                        }
                    }
                    if line.starts_with("    }") {
                        if source
                            .as_ref()
                            .map(|s| !new_sources.contains(&s.as_str()))
                            .unwrap_or(true)
                        {
                            kept_blocks.push(std::mem::take(&mut block));
                        }
                        in_block = false;
                    }
                }
            }
        }
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(
            "  \"roead_compatibility_fixtures\": \"step 1 golden fixtures for parse/serialize behavior\",\n",
        );
        out.push_str(
            "  \"note\": \"Regenerate with UPDATE_FIXTURES=1. Byte-for-byte authority is the committed .expected files.\",\n",
        );
        out.push_str("  \"entries\": [\n");
        let mut first = true;
        for block in &kept_blocks {
            if !first {
                out.push_str(",\n");
            }
            first = false;
            out.push_str(block);
            out.push('\n');
        }
        for (_i, entry) in entries.iter().enumerate() {
            if !first {
                out.push_str(",\n");
            }
            first = false;
            entry.emit(&mut out);
            out.push('\n');
        }
        out.push_str("  ]\n}\n");
        fs::write("test/fixtures/manifest.json", &out).unwrap();
    }

    /// Structural summary of a parsed BYML document (human-reviewable manifest).
    pub fn byml_summary(doc: &roead::byml::Byml) -> JValue {
        let (root_kind, entry_count) = match doc {
            roead::byml::Byml::Array(v) => ("Array", v.len()),
            roead::byml::Byml::Map(v) => ("Map", v.len()),
            roead::byml::Byml::HashMap(v) => ("HashMap", v.len()),
            roead::byml::Byml::ValueHashMap(v) => ("ValueHashMap", v.len()),
            _ => ("Scalar/Null", 0),
        };
        JValue::Map(BTreeMap::from([
            ("root_kind".into(), JValue::String(root_kind.into())),
            ("entry_count".into(), JValue::U64(entry_count as u64)),
        ]))
    }

    /// Structural summary of parameter types in an AAMP tree.
    pub fn aamp_summary(version: u32, pio: &roead::aamp::ParameterIO) -> JValue {
        let mut counts: BTreeMap<&'static str, u64> = BTreeMap::new();
        fn walk(list: &roead::aamp::ParameterList, counts: &mut BTreeMap<&'static str, u64>) {
            for (_name, obj) in &list.objects.0 {
                for (_name, param) in &obj.0 {
                    *counts.entry(type_name(param)).or_insert(0) += 1;
                }
            }
            for (_name, sub) in &list.lists.0 {
                walk(sub, counts);
            }
        }
        fn type_name(param: &roead::aamp::Parameter) -> &'static str {
            match param {
                roead::aamp::Parameter::Bool(_) => "Bool",
                roead::aamp::Parameter::F32(_) => "F32",
                roead::aamp::Parameter::I32(_) => "Int",
                roead::aamp::Parameter::Vec2(_) => "Vec2",
                roead::aamp::Parameter::Vec3(_) => "Vec3",
                roead::aamp::Parameter::Vec4(_) => "Vec4",
                roead::aamp::Parameter::Color(_) => "Color",
                roead::aamp::Parameter::String32(_) => "String32",
                roead::aamp::Parameter::String64(_) => "String64",
                roead::aamp::Parameter::String256(_) => "String256",
                roead::aamp::Parameter::StringRef(_) => "StringRef",
                roead::aamp::Parameter::Curve1(_) => "Curve1",
                roead::aamp::Parameter::Curve2(_) => "Curve2",
                roead::aamp::Parameter::Curve3(_) => "Curve3",
                roead::aamp::Parameter::Curve4(_) => "Curve4",
                roead::aamp::Parameter::BufferInt(_) => "BufferInt",
                roead::aamp::Parameter::BufferF32(_) => "BufferF32",
                roead::aamp::Parameter::Quat(_) => "Quat",
                roead::aamp::Parameter::U32(_) => "U32",
                roead::aamp::Parameter::BufferU32(_) => "BufferU32",
                roead::aamp::Parameter::BufferBinary(_) => "BufferBinary",
            }
        }
        walk(&pio.param_root, &mut counts);
        let mut map = BTreeMap::new();
        map.insert("pio_version".into(), JValue::U64(version as u64));
        map.insert(
            "param_types".into(),
            JValue::Map(
                counts
                    .into_iter()
                    .map(|(k, v)| (k.into(), JValue::U64(v)))
                    .collect(),
            ),
        );
        JValue::Map(map)
    }
}

#[cfg(feature = "byml")]
mod byml_tests {
    use std::collections::BTreeMap;

    use super::harness::*;

    #[test]
    fn byml_fixtures() {
        let mut entries = Vec::new();
        for fixture in FIXTURES.iter().filter(|f| f.format == Format::Byml) {
            let base = basename(fixture.source);

            // Yaz0-compressed files cannot be parsed without the `yaz0`
            // feature (`cfg!` requires a literal feature name; the only
            // `parse_requires` value in use is "yaz0").
            if fixture.parse_requires.is_some() && !cfg!(feature = "yaz0") {
                let req = fixture.parse_requires.unwrap();
                entries.push(ManifestEntry {
                    source: fixture.source.into(),
                    format: "byml".into(),
                    source_size: std::fs::metadata(fixture.source)
                        .map(|m| m.len())
                        .unwrap_or(0),
                    source_sha256: "skipped (feature not enabled)".to_string(),
                    snapshot: false,
                    parse_requires: Some(req.into()),
                    summary: JValue::Map(BTreeMap::from([
                        ("skipped".into(), JValue::Bool(true)),
                        (
                            "reason".into(),
                            JValue::String(format!("feature `{req}` not enabled")),
                        ),
                    ])),
                    artifacts: vec![],
                });
                continue;
            }

            let data = std::fs::read(fixture.source)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", fixture.source));
            let doc = roead::byml::Byml::from_binary(&data)
                .unwrap_or_else(|e| panic!("failed to parse BYML {}: {e}", fixture.source));

            let summary = JValue::Map(BTreeMap::from([
                (
                    "version".into(),
                    match header_version(&data) {
                        Some(v) => JValue::U64(v),
                        None => JValue::Null,
                    },
                ),
                (
                    "endian".into(),
                    header_endian(&data).unwrap_or("unknown").into(),
                ),
                ("summary".into(), byml_summary(&doc)),
            ]));

            let mut artifacts = Vec::new();
            if fixture.snapshot {
                for kind in fixture.artifacts {
                    let path = artifact_path(Format::Byml, base, kind);
                    let Some(bytes) = produce_byml(&doc, kind) else {
                        continue;
                    };
                    if update_mode() {
                        let (path, size, sha256) = write_artifact(Format::Byml, base, kind, &bytes);
                        artifacts.push(ManifestArtifact {
                            path,
                            kind: (*kind).to_string(),
                            size,
                            sha256,
                        });
                    } else {
                        let committed = std::fs::read(&path).unwrap_or_else(|e| {
                            panic!(
                                "committed artifact {path} missing (run with UPDATE_FIXTURES=1 --all-features): {e}"
                            )
                        });
                        if *kind == "yml" {
                            compare_text(&path, &committed, &bytes);
                        } else {
                            compare_bytes(&path, &committed, &bytes);
                        }
                    }
                }
            }

            entries.push(ManifestEntry {
                source: fixture.source.into(),
                format: "byml".into(),
                source_size: data.len() as u64,
                source_sha256: sha256_hex(&sha256(&data)),
                snapshot: fixture.snapshot,
                parse_requires: fixture.parse_requires.map(|s| s.into()),
                summary,
                artifacts,
            });
        }

        if update_mode() {
            write_manifest(&entries);
        }
    }

    fn produce_byml(doc: &roead::byml::Byml, kind: &str) -> Option<Vec<u8>> {
        match kind {
            "yml" => {
                #[cfg(feature = "yaml")]
                {
                    Some(doc.to_text().into_bytes())
                }
                #[cfg(not(feature = "yaml"))]
                {
                    None
                }
            }
            "le.bin" => Some(doc.to_binary(roead::Endian::Little)),
            "be.bin" => Some(doc.to_binary(roead::Endian::Big)),
            _ => unreachable!("unexpected byml artifact kind {kind}"),
        }
    }
}

#[cfg(feature = "aamp")]
mod aamp_tests {
    use super::harness::*;

    #[test]
    fn aamp_fixtures() {
        let mut entries = Vec::new();
        for fixture in FIXTURES.iter().filter(|f| f.format == Format::Aamp) {
            let base = basename(fixture.source);
            let data = std::fs::read(fixture.source)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", fixture.source));
            let pio = roead::aamp::ParameterIO::from_binary(&data)
                .unwrap_or_else(|e| panic!("failed to parse AAMP {}: {e}", fixture.source));

            let summary = aamp_summary(pio.version, &pio);

            let mut artifacts = Vec::new();
            if fixture.snapshot {
                for kind in fixture.artifacts {
                    let path = artifact_path(Format::Aamp, base, kind);
                    let Some(bytes) = produce_aamp(&pio, kind) else {
                        continue;
                    };
                    if update_mode() {
                        let (path, size, sha256) = write_artifact(Format::Aamp, base, kind, &bytes);
                        artifacts.push(ManifestArtifact {
                            path,
                            kind: (*kind).to_string(),
                            size,
                            sha256,
                        });
                    } else {
                        let committed = std::fs::read(&path).unwrap_or_else(|e| {
                            panic!(
                                "committed artifact {path} missing (run with UPDATE_FIXTURES=1 --all-features): {e}"
                            )
                        });
                        if *kind == "yml" {
                            compare_text(&path, &committed, &bytes);
                        } else {
                            compare_bytes(&path, &committed, &bytes);
                        }
                    }
                }
            }

            entries.push(ManifestEntry {
                source: fixture.source.into(),
                format: "aamp".into(),
                source_size: data.len() as u64,
                source_sha256: sha256_hex(&sha256(&data)),
                snapshot: fixture.snapshot,
                parse_requires: fixture.parse_requires.map(|s| s.into()),
                summary,
                artifacts,
            });
        }

        if update_mode() {
            write_manifest(&entries);
        }
    }

    fn produce_aamp(pio: &roead::aamp::ParameterIO, kind: &str) -> Option<Vec<u8>> {
        match kind {
            "yml" => {
                #[cfg(feature = "yaml")]
                {
                    Some(pio.to_text().into_bytes())
                }
                #[cfg(not(feature = "yaml"))]
                {
                    None
                }
            }
            "bin" => Some(pio.to_binary()),
            _ => unreachable!("unexpected aamp artifact kind {kind}"),
        }
    }
}

// Manifest presence check is cfg-free and runs under every feature set.
#[test]
fn manifest_presence() {
    harness::check_manifest_presence();
}
