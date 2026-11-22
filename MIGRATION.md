# Migrating from roead 1.x to 2.0

roead 2.0 is a `no_std`-first release. The owned (allocating) API keeps its
behavior and, for BYML and AAMP, its exact binary and YAML output, but a
number of types and signatures changed. This guide lists every breaking change
and what to write instead.

## Toolchain

- **MSRV is now 1.100** and the crate uses **edition 2024**. Upgrade the
  toolchain before upgrading roead.
- The optional `yaml` feature still needs a C++ toolchain because `ryml`
  builds C++ code; everything else is pure Rust.

## Feature model

`roead` builds as a pure `core` library with no features:

| Features | What you get |
| --- | --- |
| *(none)* | Pure `core` library: the error type and shared utilities. Each format module needs its own feature. |
| `alloc` | Everything above plus every allocating type and the owned BYML/AAMP parsers and writers. |
| `std` | `alloc` plus the standard library (implies `alloc`). |
| `byml`, `aamp`, `sarc`, `yaz0` | Enable the corresponding module. Each module self-gates its items, so e.g. `--no-default-features --features byml` compiles the core-only BYML walk and arena readers without an allocator. |
| `yaml` | Owned *and* borrowed BYML/AAMP ⇄ YAML. Implies `std` (it depends on `ryml`). |
| `with-serde` | `serde` derives for the owned types. `serde_json` is no longer a dependency. |

The default feature set is `std, alloc, aamp, byml, sarc, yaz0`.

`SarcWriter` is `std`-only; the SARC reader works with no allocator.

## Errors

- `Error` no longer implements `Clone` (the v1 implementation was a panicking
  stub; wrapping errors are not generally cloneable).
- `Error::BadMagic` now holds a `TinyString` (a copyable 16-byte inline
  string) instead of a `String`.
- `Error::TypeError` now holds two `&'static str`s instead of owned strings.
- `Error::InvalidUtf8` now wraps `core::str::Utf8Error`.
- `Error::InvalidDataD` and `Error::Any` require `alloc`; `Error::Io` requires
  `std`.
- `Result<T>` is now `core::result::Result<T, Error>`.

## SARC

```rust
// 1.x
let sarc = Sarc::new(bytes)?; // took Into<Cow<[u8]>>, owned the data,
                              // decompressed Yaz0 automatically
// 2.0
let decompressed = roead::yaz0::decompress_if(bytes);
let sarc = Sarc::new(decompressed.as_ref())?; // borrows &'a [u8]
```

- `Sarc::new` borrows the input and never allocates or owns it.
- Automatic Yaz0 decompression was removed; call
  `yaz0::decompress_if` (or `yaz0::decompress_into`) first.
- `Endian` no longer derives `binrw` traits. Use `Endian::from_bom` to
  interpret a SARC big-endian marker.
- `File` equality now compares `name`, `data`, and `index` (O(file)) instead
  of the whole backing archive (O(files × size)).

## Yaz0

- The crate has **zero dependencies** (`yaz0 = []`).
- `decompress_into`, `decompress_unchecked`, and header parsing are
  core-only.
- `decompress` and `decompress_if` require `alloc`; `compress_if` requires
  `std`.

## AAMP

- The owned parser/writer (`ParameterIO`) require `alloc` rather than `std`
  and behave as before.
- `aamp::arena::ParameterIOArena` gained a data lifetime parameter so it can
  retain the header's data version and type:

  ```rust
  // 2.0
  let arena: ParameterIOArena<'a, S> = ParameterIOArena::from_binary(data)?;
  arena.version();   // header data version
  arena.data_type(); // e.g. "xml"
  ```

- `ParameterIORef` (borrowed tree) and `ParameterIOArena` (flat, no allocator)
  are new. `BufferRef` and `CurveRef` expose buffer/curve payloads without
  allocating.

## BYML

- `BymlRef` (borrowed tree) and `byml::arena::{BymlArena, BymlStack}` (flat
  readers over the shared `storage::{OwnedStore, FixedStore, BorrowedStore}`
  stores) are new.
- `BymlArena::from_binary` requires a `Default` store; use
  `BymlArena::new()` + `parse_in_place(data)` for large inline stores, or
  `from_binary_in(data, slots)` with caller-provided memory.
- Container nesting is capped at 128 levels (`BYML_MAX_DEPTH`,
  `AAMP_MAX_DEPTH`).

## YAML

`to_text` is now also available directly on the borrowed readers:
`BymlRef::to_text`, `BymlArena::to_text`, `ParameterIORef::to_text`, and
`ParameterIOArena::to_text`. Their output is byte-identical to the owned
`Byml::to_text` / `ParameterIO::to_text`, so YAML can be produced without
materializing an owned tree.

## Misc

- The AGLEnv alignment table used by `SarcWriter` is checked in as generated
  data (`src/sarc/aglenv.rs`) instead of being parsed from
  `data/aglenv_file_info.json` at runtime. `tests/sarc_aglenv.rs` verifies it
  against the JSON (`UPDATE_AGLENV=1` to regenerate).
- `Cargo.lock` is not tracked.
