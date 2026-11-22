#![cfg(feature = "std")]

//! Checks that the checked-in AGLEnv alignment table
//! (`src/sarc/aglenv.rs`) matches `data/aglenv_file_info.json`.
//!
//! The JSON is scanned by hand (no serde_json / JSON crate): objects are split
//! by brace depth and the `"ext"`, `"bext"` and `"align"` fields are located by
//! key name within each object. The file's field order is fixed
//! (id, i0, ext, bext, s, align, system, desc) but the scan is keyed by name so
//! it is tolerant of reordering.
//!
//! With `UPDATE_AGLENV=1` the table file is rewritten instead of asserting;
//! then run `cargo fmt` to keep it formatted.

use std::path::Path;

use roead::sarc::agl_env_alignment_requirements;

const JSON_REL: &str = "data/aglenv_file_info.json";
const TABLE_REL: &str = "src/sarc/aglenv.rs";

fn manifest_dir() -> String {
    std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set")
}

fn json_path() -> String {
    Path::new(&manifest_dir())
        .join(JSON_REL)
        .to_string_lossy()
        .into_owned()
}

fn table_path() -> String {
    Path::new(&manifest_dir())
        .join(TABLE_REL)
        .to_string_lossy()
        .into_owned()
}

/// Scan one `{...}` object for the `ext`, `bext` and `align` fields.
fn scan_object(obj: &str) -> Option<(&str, &str, i32)> {
    let ext = scan_str_field(obj, "ext")?;
    let bext = scan_str_field(obj, "bext")?;
    let align = scan_int_field(obj, "align")?;
    Some((ext, bext, align))
}

/// Extract the string value of `"field": "..."` from an object.
fn scan_str_field<'a>(obj: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{}\"", field);
    let key_start = obj.find(&key)?;
    let after_key = &obj[key_start + key.len()..];
    let colon = after_key.find(':')?;
    let rest = after_key[colon + 1..].trim_start();
    let s = rest.strip_prefix('"')?;
    let end = s.find('"')?;
    Some(&s[..end])
}

/// Extract the integer value of `"field": <int>` from an object.
fn scan_int_field(obj: &str, field: &str) -> Option<i32> {
    let key = format!("\"{}\"", field);
    let key_start = obj.find(&key)?;
    let after_key = &obj[key_start + key.len()..];
    let colon = after_key.find(':')?;
    let rest = after_key[colon + 1..].trim_start();
    let end = rest
        .find(',')
        .or_else(|| rest.find('}'))
        .unwrap_or(rest.len());
    rest[..end].trim().parse::<i32>().ok()
}

/// Split the top-level JSON array into its `{...}` objects by brace depth and
/// flatten each entry with `align >= 0` to `(ext, align)` then `(bext, align)`,
/// in object order.
fn scan_json(json: &str) -> Vec<(&str, usize)> {
    let mut out: Vec<(&str, usize)> = Vec::new();
    let mut depth: u32 = 0;
    let mut obj_start: Option<usize> = None;
    for (i, ch) in json.char_indices() {
        match ch {
            '[' | '{' => {
                depth += 1;
                if depth == 2 {
                    obj_start = Some(i);
                }
            }
            '}' => {
                if depth == 2 {
                    if let Some(start) = obj_start.take() {
                        if let Some((ext, bext, align)) = scan_object(&json[start..=i]) {
                            if align >= 0 {
                                out.push((ext, align as usize));
                                out.push((bext, align as usize));
                            }
                        }
                    }
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    out
}

fn table_header() -> String {
    "//! Generated from `data/aglenv_file_info.json` — do not edit by hand.\n\
     //! Regenerate/verify with `UPDATE_AGLENV=1 cargo test --all-features --test sarc_aglenv`.\n\
     /// `(extension, data alignment)` requirements flattened in JSON entry order:\n\
     /// each source entry contributes `(ext, align)` then `(bext, align)`, entries\n\
     /// with negative alignment excluded.\n\
     pub(crate) static AGLENV_ALIGNMENT_REQUIREMENTS: &[(&str, usize)] = &[\n"
        .to_string()
}

fn write_table(pairs: &[(&str, usize)]) {
    let mut content = table_header();
    for (ext, align) in pairs {
        content.push_str(&format!("    (\"{}\", {}),\n", ext, align));
    }
    content.push_str("];\n");
    std::fs::write(&table_path(), content).expect("failed to write aglenv table file");
    println!(
        "Wrote {}. Regeneration complete — now run: cargo fmt",
        TABLE_REL
    );
}

#[test]
fn agl_env_table_matches_json() {
    let json = std::fs::read_to_string(&json_path()).expect("failed to read aglenv json");
    let expected = scan_json(&json);

    if std::env::var("UPDATE_AGLENV").is_ok() {
        write_table(&expected);
        return;
    }

    let actual: Vec<(&str, usize)> = agl_env_alignment_requirements().to_vec();

    assert_eq!(
        actual.len(),
        expected.len(),
        "AGLENV table length ({}) != JSON entry count ({}); \
         regenerate with UPDATE_AGLENV=1 cargo test --all-features --test sarc_aglenv",
        actual.len(),
        expected.len()
    );
    for (i, ((a_ext, a_align), (e_ext, e_align))) in actual.iter().zip(expected.iter()).enumerate()
    {
        assert_eq!(
            (a_ext, a_align),
            (e_ext, e_align),
            "AGLENV table mismatch at index {i}: table ({a_ext:?}, {a_align}) != \
             json ({e_ext:?}, {e_align}); regenerate with \
             UPDATE_AGLENV=1 cargo test --all-features --test sarc_aglenv"
        );
    }
}
