# Changelog

All notable changes to roead will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - Unreleased

### Added

- `TinyString`, a copyable fixed-capacity (16-byte) inline string used for
  alloc-free error messages.
- New typed, alloc-free `Error` variants: `IndexOutOfRange`, `TooManyFiles`,
  `CapacityExceeded`, `InvalidNodeType`.
- Zero-copy BYML readers, borrowing all strings and payloads from the
  input buffer:
  - `byml::BymlRef<'a>`, an enum-shaped borrowed tree mirroring `Byml`
    variant-for-variant (`to_owned()` materializes an owned `Byml`).
  - `byml::arena`, flat fixed-capacity readers that need no allocator:
    `BymlArena<S>` over an `OwnedStore` (alloc), an inline `FixedStore<N>`
    (`BymlStack<'a, N>` alias), or a caller-provided `BorrowedStore` (the
    same `storage` stores the AAMP arena uses).
  - `BymlArena::new()` + `parse_in_place(&mut self, data)` parse without
    moving the (potentially very large) inline store by value; re-parsing
    discards previous contents. `from_binary` remains available for
    `Default` stores, and `from_binary_in` for caller-owned memory.
- Zero-copy AAMP readers: `aamp::ParameterIORef<'a>` with
  `BufferRef<'a>`/`CurveRef<'a>` payload views (endian-safe iterators plus
  alignment-checked, little-endian-gated typed slices), and
  `aamp::arena::ParameterIOArena`/`ParameterIOStack` built on the same
  generic stores (`storage::{Store, OwnedStore, FixedStore, BorrowedStore}`).
- YAML emission directly from the zero-copy readers (`std` + `yaml`):
  `BymlRef::to_text()`, `BymlArena::to_text()`, `ParameterIORef::to_text()`
  and `ParameterIOArena::to_text()`. Output is byte-identical to the owned
  emitters, so no owned tree has to be materialized. `ParameterIOArena` also
  retains the header's `version` and `data_type` (`version()`, `data_type()`)
  so it can emit them without the owned type.
- Container nesting is capped at 128 levels (`BYML_MAX_DEPTH`,
  `AAMP_MAX_DEPTH`), which also bounds malicious self-referential input.

### Changed

- Bumped edition to 2024 and MSRV to 1.100; crate version is now 2.0.0.
- New feature split: `std` implies `alloc`, both are on by default, and
  `--no-default-features` builds a pure `no_std` library. The `std` feature
  gathers the std-only dependencies (yaml machinery, name table, ...), while
  `alloc` gathers the allocator and everything that needs it. The owned
  BYML/AAMP parsers and writers now require `alloc` rather than `std`.
- `SarcWriter` no longer parses `data/aglenv_file_info.json` at runtime: the
  AGLEnv alignment table is checked in as generated data
  (`src/sarc/aglenv.rs`) and verified against the source JSON by
  `tests/sarc_aglenv.rs` (`UPDATE_AGLENV=1` to regenerate, `cargo fmt` to
  format). `serde` and `serde_json` are no longer dependencies of the `std`
  feature (`serde` remains available via `with-serde`).
- `Error` rework:
  - `Clone` for `Error` was removed.
  - `BadMagic`'s first field is now `TinyString` instead of `String`.
  - `TypeError`'s first field is now `&'static str` instead of an owned string.
  - `InvalidUtf8` is now `core::str::Utf8Error`.
  - `InvalidDataD` and `Any` require the `alloc` feature.
  - `Io` requires `std`.
  - `BinarySerde(binrw::Error)` requires `binrw`; under `std` it keeps
    `#[error(transparent)]` + `#[from]` so `?` still works on binrw errors,
    otherwise it is a plain variant with a descriptive display message.
- `Result` is now `core::result::Result<T, Error>`.
- `Endian`'s binrw derive was removed entirely; use `Endian::from_bom` to
  interpret a SARC BOM.
- The SARC reader no longer requires `std` and works without an allocator:
  - Without `alloc`, `Sarc::new(data: &'a [u8]) -> Result<Sarc<'a>>` borrows
    the data and never allocates or owns it. Decompress Yaz0-compressed
    archives with `yaz0::decompress_if` first, then
    `Sarc::new(decompressed.as_ref())`.
  - With `alloc` (in the default feature set), `Sarc::new` keeps the 1.x
    signature `new<T: Into<Cow<'a, [u8]>>>` and, with `yaz0` enabled,
    decompresses Yaz0 input automatically.
  - The `sarc` module is gated on `feature = "sarc"` only; the writer
    (`SarcWriter`) remains `std`-only.
- Yaz0 has zero dependencies (`yaz0 = []`); `decompress_into`,
  `decompress_unchecked`, and header parsing (`get_header`/`checked_header`)
  are core-only and available without `alloc`. Compression, `decompress`, and
  `decompress_if` require `alloc`; `compress_if` requires `std`.
- `File` equality now compares `name`, `data`, and `index` only (O(file))
  instead of the whole backing archive (O(files × size)).

### Fixed

- AAMP writer data-section deduplication keyed the entry on the parameter
  value hash alone, so a buffer parameter (which carries an element-count
  prefix at `data_offset - 4`) could be deduplicated onto inline struct data,
  or an inline parameter onto buffer data (read 4 bytes off), producing
  corrupt archives. Dedup now also keys on the parameter's data shape, so
  buffer/non-buffer pairs are always split while byte-identical wide4 structs
  (`Vec4`/`Color`/`Quat`) still deduplicate safely.
- AAMP parsing of `BufferF32` parameters read the element count from the
  parameter entry offset (`param_offset - 4`) instead of `data_offset - 4`
  like every other buffer type, so any archive containing a float buffer
  failed to parse (`BinarySerde`). The count is now read from
  `data_offset - 4`.
- AAMP `read_null_string` used `str::from_utf8_unchecked` (undefined
  behaviour on invalid UTF-8); it now validates with
  `core::str::from_utf8` and returns `Error::InvalidUtf8`.
- `Curve` is now `#[repr(C)]`. Rust had been free to reorder its fields, so
  reinterpreting `&[Curve; N]` as bytes for curve parameters produced
  corrupted values.

## [1.0.0] - 2024-08-17

** Changed

- Drastically improved multithreaded AAMP->YAML performance
- Updated a number of deps
- Bumped MSRV to 1.80 for stable `LazyLock`

** Fixed

- Fixed issues parsing `Null` strings and quoted numeric keys in YAML for AAMP files
- Fixed missing negative sign when emitting `-0.0` in YAML

## [0.25.3]

## Changed

- Now treating of all null values in AAMP YAML as empty strings

## [0.25.2]

## Fixed

- Fixed handling null/empty tagged strings in AAMP YAML

## [0.25.1]

## Added

- Added docs for new macros

### Fixed

- Fixed broken BYML `array!` macro

## [0.25.0]

### Added

- Added `map` and `array` BYML macros, plus `params`, `objs`, `lists` AAMP
  macros.

## [0.24.0]

### Changed

- Update dependencies, including breaking `indexmap` changes

## [0.23.1]

### Changed

- Updated ryml to 0.3.2, which *may* fix YAML issues with some TOTK BYML files.

## [0.23.0]

### Added

- Added support for BYML versions 5-7. This introduces 3 new nodes types, which
  means a *breaking change* to the BYML enum. Besides the difference it makes
  for exhaustive pattern matching, the introduction of two new kinds of hash
  nodes has triggered a renaming, so all references to `Byml::Hash` or the
  `byml::Hash` type alias are now `Map`, and the new hash types are `HashMap`
  and `ValueHashMap`.

### Changed 

- `Byml::to_text()` no longer returns a `Result`. Instead it panics if called on
  an invalid node type. This is more convenient in 90% of use cases, and the
  user can still explicitly check the node type first if they want to.

## [0.22.1]

### Fixed

- Fixed parsing long values (u64, i64, f64)

## [0.22.0]

### Added

- Added support for stable Rust toolchain (MSRV 1.69)
