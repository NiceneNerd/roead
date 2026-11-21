# Changelog

All notable changes to roead will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - Unreleased

### Added

- `TinyString`, a copyable fixed-capacity (16-byte) inline string used for
  alloc-free error messages.
- New typed, alloc-free `Error` variants: `IndexOutOfRange`, `TooManyFiles`,
  `CapacityExceeded`.

### Changed

- Bumped edition to 2024 and MSRV to 1.100; crate version is now 2.0.0.
- New feature split: `std` implies `alloc`, both are on by default, and
  `--no-default-features` builds a pure `no_std` library. The `std` feature
  gathers the std-only dependencies (yaml machinery, name table,
  `SarcWriter`'s serde_json, ...), while `alloc` gathers the allocator and
  everything that needs it.
- `Error` rework:
  - `Clone` for `Error` was removed (it was a panicking stub; wrapping errors
    are not generally cloneable).
  - `BadMagic`'s first field is now `TinyString` instead of `String`.
  - `TypeError`'s first field is now `&'static str` instead of an owned string.
  - `InvalidUtf8` is now `core::str::Utf8Error`.
  - `InvalidDataD` and `Any` require the `alloc` feature.
  - `Io` requires `std`.
  - `BinarySerde(binrw::Error)` requires `binrw`; under `std` it keeps
    `#[error(transparent)]` + `#[from]` so `?` still works on binrw errors,
    otherwise it is a plain variant with a descriptive display message.
- `Result` is now `core::result::Result<T, Error>`.
- `Endian`'s binrw derive is now gated on `binrw` instead of `sarc`.

### Breaking (transitional until steps 3–5)

- The `sarc` module requires `std`; `yaz0` requires `alloc`.
- BYML and AAMP binary APIs (`from_binary`, `to_binary`, `read`, `write`)
  require `alloc`; `sarc` remains `std`-gated until step 5.
- `sarc/parse.rs` now reports `IndexOutOfRange` and `TooManyFiles` instead of
  `InvalidDataD`.

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
