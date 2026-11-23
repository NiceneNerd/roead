# roead

[![crates.io](https://img.shields.io/crates/v/roead)](https://crates.io/crates/roead)
[![api](https://img.shields.io/badge/api-rustdoc-558b2f)](https://nicenenerd.github.io/roead/roead/)
[![license](https://img.shields.io/badge/license-GPL-blue)](https://spdx.org/licenses/GPL-3.0-or-later.html)
[![build](https://img.shields.io/github/actions/workflow/status/NiceneNerd/roead/test.yml)](https://github.com/NiceneNerd/roead/actions/workflows/test.yml)

## A Rusty child of the oead C++ library
**oead** is a C++ library for common file formats that are used in modern
first-party Nintendo EAD (now EPD) titles.

Currently, oead only handles very common formats that are extensively used
in recent games such as *Breath of the Wild* and *Super Mario Odyssey*.

* [AAMP](https://zeldamods.org/wiki/AAMP) (binary parameter archive): Only version 2 is supported.
* [BYML](https://zeldamods.org/wiki/BYML) (binary YAML): Versions 2, 3, and 4
  are supported (up to v7 in roead).
* [SARC](https://zeldamods.org/wiki/SARC) (archive)
* [Yaz0](https://zeldamods.org/wiki/Yaz0) (compression algorithm)

The roead project brings oead's core functionality by directly porting it.
(The Grezzo datasheets are not supported.) For more info on oead itself, visit
[its GitHub repo](https://github.com/zeldamods/oead/).

Each of roead's major modules is configurable as a feature. The default feature
set includes `std`, `alloc`, `byml`, `aamp`, `sarc`, and `yaz0`. For
compatibility with many existing tools for these formats, there is also a
`yaml` feature which enables serializing/deserializing AAMP and BYML files as
YAML documents. Finally, serde support is available using the `with-serde`
feature.

## `no_std` and zero-copy readers

roead supports `no_std`: `--no-default-features` builds a pure `core`
library, `alloc` adds the allocating types, and `std` (which implies `alloc`)
enables the standard library. The owned `Byml` and `ParameterIO` types need
`alloc`, but the format walks do not:

* BYML: `BymlRef` borrows a tree in place; `byml::arena::BymlArena` (over an
  `OwnedStore`, an inline `BymlStack`, or a caller-owned `BorrowedStore`) is
  flat, fixed-capacity, and needs no allocator.
* AAMP: `ParameterIORef` mirrors `ParameterIO` over the input buffer, and
  `aamp::arena::ParameterIOArena`/`ParameterIOStack` provide the flat
  allocator-free form.
* SARC: without `alloc`, `Sarc::new` borrows the archive and never
  allocates; with `alloc` (the default), it accepts `Into<Cow<'a, [u8]>>`
  and decompresses Yaz0 input automatically when `yaz0` is on.
* Yaz0: decompression into a caller-provided buffer is core-only, and the
  crate has no dependencies at all.

With the `yaml` feature, all four borrowed readers can emit YAML directly
(`BymlRef::to_text`, `BymlArena::to_text`, `ParameterIORef::to_text`,
`ParameterIOArena::to_text`).

For API documentation, see the docs for each module.

## Building from Source

roead is pure Rust (except the optional `yaml` feature, which builds the `ryml`
C++ library) and builds with Rust 1.100 or later using edition 2024. See
[MIGRATION.md](MIGRATION.md) for the changes between 1.x and 2.0.

## Contributing

Issue tracker: https://github.com/NiceneNerd/roead/issues  
Source code: https://github.com/NiceneNerd/roead

This project is licensed under the GPLv3+ license. oead is licensed under the GPLv2+ license.
