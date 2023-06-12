# Changelog

All notable changes to roead will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
