# roead

[![crates.io](https://img.shields.io/crates/v/rstb)](https://crates.io/crates/roead)
[![api](https://img.shields.io/badge/api-rustdoc-558b2f)](https://nicenenerd.github.io/roead/roead/)
[![license](https://img.shields.io/badge/license-GPL-blue)](https://spdx.org/licenses/GPL-3.0-or-later.html)
[![build](https://img.shields.io/github/workflow/status/NiceneNerd/roead/Build%20and%20test)](https://github.com/NiceneNerd/roead/actions/workflows/rust.yml)

## Rust bindings for the oead C++ library
**oead** is a C++ library for common file formats that are used in modern
first-party Nintendo EAD (now EPD) titles.

Currently, oead only handles very common formats that are extensively used
in recent games such as *Breath of the Wild* and *Super Mario Odyssey*.

* [AAMP](https://zeldamods.org/wiki/AAMP) (binary parameter archive): Only version 2 is supported.
* [BYML](https://zeldamods.org/wiki/BYML) (binary YAML): Versions 2, 3, and 4 are supported.
* [SARC](https://zeldamods.org/wiki/SARC) (archive)
* [Yaz0](https://zeldamods.org/wiki/Yaz0) (compression algorithm)

The roead project attempts to provide safe and relatively idiomatic Rust
bindings to oead's core functionality. The Grezzo datasheets are not supported.
For more info on oead itself, visit [its GitHub repo](https://github.com/zeldamods/oead/).

For API documentation, see the docs for each module.

## Building from Source

Since roead is a wrapper for a C++ library, you will need to be able to compile both Rust and C++ to use it.

**Requirements**:
- Cargo (MSRV 1.48.0)
- CMake 3.12+
- A compiler that supports C++17
- Everything needed to build libyaml

First, clone the repository then enter the roead directory and run `git submodule update --init --recursive`. 

## Contributing

Issue tracker: https://github.com/NiceneNerd/roead/oead/issues
Source code: https://github.com/NiceneNerd/roead

This project is licensed under the GPLv3+ license. oead is licensed under the GPLv2+ license.
