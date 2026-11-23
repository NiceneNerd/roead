# Migrating from roead 1.x to 2.0

The owned BYML, AAMP, and SARC APIs keep their names, signatures, and output,
so most users need no code changes. This file lists only what requires an
action. Everything else, including the new zero-copy readers and the
dependency changes, is in [CHANGELOG.md](CHANGELOG.md).

## Toolchain

- The MSRV is now 1.88 (was 1.80), and the crate uses edition 2024 (was
  2021). Upgrade the toolchain before upgrading roead.

## Features

- The crate is split into `std` and `alloc` features now. Both are in the
  default set, so default builds are unaffected.
- If you build with `default-features = false`, the owned BYML and AAMP
  types and parsers now require `alloc`, and `SarcWriter` requires `std`. In
  1.x, everything required `std`.
- `yaml` implies `std` now.

## Errors

- `Error` no longer implements `Clone`. The 1.x implementation panicked.
- `Error::BadMagic` holds a `TinyString` (a copyable 16-byte inline string)
  instead of a `String`.
- `Error::TypeError` holds a `&'static str` instead of an owned string.
- `Error::Yaz0Error` was removed. The Yaz0 module is pure Rust now and
  reports failures through the other variants.
- `byml::BymlError` was removed. BYML parse errors are `roead::Error` now.

## SARC

- `Endian` no longer derives the `binrw` traits. Use `Endian::from_bom` to
  interpret a SARC big-endian marker.
- `File` equality compares `name`, `data`, and `index` now, instead of the
  whole backing archive.

## Yaz0

- `yaz0::get_header` takes a `&[u8]` instead of `impl AsRef<[u8]>`.
