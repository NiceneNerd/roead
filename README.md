# roead

[![api](https://img.shields.io/badge/api-rustdoc-558b2f)](https://nicenenerd.github.io/roead/roead)
[![license](https://img.shields.io/crates/l/roead)](https://spdx.org/licenses/GPL-3.0-or-later.html)

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
