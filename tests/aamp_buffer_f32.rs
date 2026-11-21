//! Documents the (fixed) `Type::BufferF32` offset bug in
//! `src/aamp/parser.rs` (`parse_parameter`).
//!
//! For every buffer type except `BufferF32`, the parser calls
//! `read_buffer::<T>(data_offset)`, which reads the element count from
//! `data_offset - 4` — where the writer (`src/aamp/writer.rs`,
//! `write_parameter_data` / `write_buffer`) stores it, immediately followed
//! by the elements at `data_offset`, with
//! `data_offset = param_offset + data_rel_offset * 4`.
//!
//! For `BufferF32` the parser previously called `read_float_buffer(offset)`
//! with the *parameter entry offset*, reading the count from
//! `param_offset - 4` — i.e. 4 bytes before the parameter entry — while the
//! floats themselves were still consumed from `data_offset`. v2 step 4b
//! changed the call to pass `data_offset`, so the count is read from
//! `data_offset - 4` like every other buffer type.
//!
//!   * `test_buffer_f32_ground_truth` decodes the bytes the writer produces
//!     straight from the file layout and asserts the writer's output is
//!     exactly what was requested (ground truth). Untouched.
//!   * `test_buffer_f32_current_parse_behavior` now asserts that parsing
//!     succeeds with the correct values `[1.0, 2.0, 3.0, 4.0, 5.0]`; the
//!     bug was fixed in v2 (step 4b), so this test was flipped accordingly.
//!   * `test_buffer_f32_roundtrip_byte_equality` asserts that a
//!     `BufferF32` parameter round-trips through `to_binary` ->
//!     `from_binary` -> `to_binary` byte-for-byte.
//!   * `test_aamp_corpus_buffer_f32_scan` scans `test/aamp/` for
//!     `BufferF32` parameters (none are present).

#![cfg(feature = "aamp")]

#[cfg(feature = "alloc")]
mod alloc_tests {
    use std::io::Read;

    use roead::aamp::*;

    /// `Type::BufferF32` discriminant (see `src/aamp/mod.rs`).
    const BUFFER_F32_TAG: u8 = 14;

    const FLOATS: [f32; 5] = [1.0, 2.0, 3.0, 4.0, 5.0];

    fn build_pio() -> ParameterIO {
        ParameterIO::new().with_object(
            "TestObj",
            params!("TestFloatBuffer" => Parameter::BufferF32(FLOATS.to_vec())),
        )
    }

    fn read_u16_le(bytes: &[u8], off: usize) -> u16 {
        u16::from_le_bytes(bytes[off..off + 2].try_into().unwrap())
    }

    fn read_u32_le(bytes: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap())
    }

    /// Ground truth: walk the produced archive straight from the file layout
    /// (header at 0, `pio_offset` at 0x14, root list at 0x30 + `pio_offset`,
    /// objects, params) and check the count at `data_offset - 4` and the
    /// floats at `data_offset`.
    #[test]
    fn test_buffer_f32_ground_truth_matches_writer() {
        let pio = build_pio();
        let bytes = pio.to_binary();

        // ResHeader: 4-byte "AAMP" magic, then fields; `pio_offset` (relative to
        // 0x30) is at file offset 0x14.
        let pio_offset = read_u32_le(&bytes, 0x14) as usize;
        let list_off = 0x30 + pio_offset;

        // ResParameterList: name u32, lists_rel_offset u16, list_count u16,
        // objects_rel_offset u16, object_count u16.
        let objects_rel = read_u16_le(&bytes, list_off + 8);
        let object_count = read_u16_le(&bytes, list_off + 10);
        let object_off = list_off + objects_rel as usize * 4;

        let mut found = 0;
        for i in 0..object_count as usize {
            let ooff = object_off + 8 * i;
            let params_rel = read_u16_le(&bytes, ooff + 4);
            let param_count = read_u16_le(&bytes, ooff + 6);
            let param_off = ooff + params_rel as usize * 4;

            for j in 0..param_count as usize {
                let poff = param_off + 8 * j;
                // ResParameter: name u32, data_rel_offset u24, type u8.
                let data_rel = (bytes[poff + 4] as u32)
                    | ((bytes[poff + 5] as u32) << 8)
                    | ((bytes[poff + 6] as u32) << 16);
                let tag = bytes[poff + 7];
                if tag != BUFFER_F32_TAG {
                    continue;
                }
                found += 1;
                let data_off = poff + data_rel as usize * 4;

                // Ground truth from the file layout.
                let count = read_u32_le(&bytes, data_off - 4);
                assert_eq!(
                    count,
                    FLOATS.len() as u32,
                    "ground-truth count at data_offset - 4 (data_offset = {data_off})"
                );
                let mut actual = Vec::with_capacity(count as usize);
                for k in 0..count as usize {
                    actual.push(f32::from_le_bytes(
                        bytes[data_off + 4 * k..data_off + 4 * k + 4]
                            .try_into()
                            .unwrap(),
                    ));
                }
                assert_eq!(
                    actual, FLOATS,
                    "ground-truth floats at data_offset {data_off}"
                );
            }
        }
        assert_eq!(
            found, 1,
            "expected exactly one BufferF32 parameter in the archive"
        );
    }

    /// Pins the parse behavior of `ParameterIO::from_binary` on an archive
    /// containing a single `BufferF32` parameter.
    #[test]
    fn test_buffer_f32_current_parse_behavior() {
        let bytes = build_pio().to_binary();

        match ParameterIO::from_binary(&bytes) {
            Ok(pio) => {
                let found = walk_buffer_f32(&pio.param_root);
                match found {
                    None => panic!("parsed OK but no BufferF32 parameter found"),
                    Some(values) => {
                        assert_eq!(
                            values, FLOATS,
                            "parsed BufferF32 values differ from ground truth"
                        );
                    }
                }
            }
            Err(err) => panic!(
                "expected BufferF32 parse to succeed (bug fixed in v2, step 4b), got: {err:?}"
            ),
        }
    }

    /// Regression test: a `BufferF32` parameter must round-trip through
    /// `to_binary()` -> `from_binary()` -> `to_binary()` byte-for-byte.
    #[test]
    fn test_buffer_f32_roundtrip_byte_equality() {
        let pio = build_pio();
        let first = pio.to_binary();
        let reparsed = ParameterIO::from_binary(&first).unwrap();
        let second = reparsed.to_binary();
        assert_eq!(first, second, "BufferF32 roundtrip changed the bytes");
    }

    /// Collects `Parameter::BufferF32` values from a parsed parameter tree.
    fn walk_buffer_f32(list: &ParameterList) -> Option<Vec<f32>> {
        for (_name, obj) in &list.objects.0 {
            for (_name, param) in &obj.0 {
                if let Parameter::BufferF32(values) = param {
                    return Some(values.clone());
                }
            }
        }
        for (_name, sub) in &list.lists.0 {
            if let Some(values) = walk_buffer_f32(sub) {
                return Some(values);
            }
        }
        None
    }

    /// Scans every binary file under `test/aamp/` and reports how many
    /// contain `Parameter::BufferF32`. Asserts what is currently true: zero
    /// occurrences, which is why the existing corpus roundtrip tests never
    /// exercise the previously buggy parse path.
    #[test]
    fn test_aamp_corpus_buffer_f32_scan() {
        let mut buffer_f32_count = 0usize;
        let mut files_with_buffer_f32 = Vec::new();
        let mut parsed_files = 0usize;
        let mut parse_errors = Vec::new();

        scan_aamp_dir(
            "test/aamp",
            &mut |path, data| match ParameterIO::from_binary(data) {
                Ok(pio) => {
                    parsed_files += 1;
                    let n = count_buffer_f32(&pio.param_root);
                    if n > 0 {
                        buffer_f32_count += n;
                        files_with_buffer_f32.push(path.to_string());
                    }
                }
                Err(err) => parse_errors.push((path.to_string(), err.to_string())),
            },
        );

        println!(
            "AAMP corpus scan: {parsed_files} files parsed, \
             {buffer_f32_count} BufferF32 parameter(s) in {} file(s), \
             {} parse error(s)",
            files_with_buffer_f32.len(),
            parse_errors.len()
        );
        for (path, err) in &parse_errors {
            println!("  parse error in {path}: {err}");
        }
        for path in &files_with_buffer_f32 {
            println!("  BufferF32 in {path}");
        }

        assert_eq!(
            buffer_f32_count, 0,
            "expected no BufferF32 parameters in test/aamp binaries"
        );
    }

    fn count_buffer_f32(list: &ParameterList) -> usize {
        let mut n = 0;
        for (_name, obj) in &list.objects.0 {
            for (_name, param) in &obj.0 {
                if matches!(param, Parameter::BufferF32(_)) {
                    n += 1;
                }
            }
        }
        for (_name, sub) in &list.lists.0 {
            n += count_buffer_f32(sub);
        }
        n
    }

    fn scan_aamp_dir<F: FnMut(&str, Vec<u8>)>(dir: &str, f: &mut F) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                scan_aamp_dir(path.to_str().unwrap(), f);
            } else if path.extension().and_then(|s| s.to_str()) != Some("yml") {
                let mut data = Vec::new();
                std::fs::File::open(&path)
                    .unwrap()
                    .read_to_end(&mut data)
                    .unwrap();
                f(path.to_str().unwrap(), data);
            }
        }
    }
}
