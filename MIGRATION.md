# Migrating from roead 1.x to 2.0

The owned BYML, AAMP, and SARC APIs keep their names, signatures, and output,
so most users need no code changes. The sections below list what requires an
action, and the last one is optional guidance for read-only workloads.
Everything else, including the new zero-copy readers and the dependency
changes, is in [CHANGELOG.md](CHANGELOG.md).

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

## Read-only workloads

This part is optional. The owned parsers copy every string and payload into
the heap, and 2.0 can read a document in place instead. That is worth doing
if you only read documents.

```rust
use roead::byml::{Byml, BymlRef};

// 1.x
let doc = Byml::from_binary(&data)?;
let name = doc.as_map()?.get("Name").unwrap().as_string()?;

// 2.0
let doc = BymlRef::from_binary(&data)?;
let name = doc.as_map()?.get("Name").unwrap().as_str()?;
```

The same applies to AAMP: `ParameterIO::from_binary` becomes
`ParameterIORef::from_binary`. With the `yaml` feature, `BymlRef::to_text`
and `ParameterIORef::to_text` emit the same YAML as the owned emitters, byte
for byte, so a tree does not have to be built just to write it out.

### Accessor differences

- Strings and binary payloads are borrowed. On `BymlRef`, `as_string`
  becomes `as_str`, and `as_binary_data` and `as_file_data` return
  `&'a [u8]` straight from the input buffer. On `ParameterRef`, the string
  accessors (`as_string32`, `as_string64`, `as_string256`, `as_string_ref`)
  return `&'a str` instead of `&FixedSafeString`.
- On `ParameterRef`, the `Copy` payload accessors `as_vec2`, `as_vec3`,
  `as_vec4`, `as_color`, and `as_quat` keep the same signatures as on
  `Parameter`.
- Buffers and curves need one extra step. `as_buffer_f32` returns
  `&BufferRef` instead of `&[f32]`; get the slice with
  `BufferRef::as_f32()`, iterate with `iter_f32()`, or get the length with
  `len_f32()` (likewise for the int and u32 buffers). `as_curve1` returns
  `&CurveRef` instead of `&[Curve; 1]`; iterate with `CurveRef::iter()` or
  read one entry with `curve(i)`.
- The generic `as_int` and `as_num` casting accessors are available on the
  borrowed types too (`BymlRef`, `ParameterRef`, and the arena `Node`), so
  code that tolerated an int where a float was expected, or the reverse,
  carries over unchanged.
- Mutation is owned-only. The `as_mut_*` accessors, the `into_*` consuming
  accessors, and the `with_*` builders do not exist on the borrowed types;
  call `to_owned()` first when a document has to be modified. `From<&Byml>`
  and `From<&ParameterIO>` convert an owned tree into a borrowed one.

### Query and iteration differences

- Indexing works the same on both sides: `doc["key"]` and `doc[0]` work on
  `Byml` and on `BymlRef`, and `obj["key"]` works on `ParameterObject` and
  on `ParameterObjectRef`. All of them panic on a wrong index or node type,
  exactly like the owned types. `IndexMut` exists only on the owned types.
- `ParameterIO` and `ParameterIORef` both expose `version`, `data_type`, and
  `param_root` as public fields; only `data_type` differs in type (`String`
  vs `&'a str`). The borrowed `lists()`, `objects()`, `list(name)`, and
  `object(name)` return borrowed maps.
- Iteration order matches on the ref types: AAMP objects and lists are
  `IndexMap`s on both owned and borrowed, so iteration stays in document
  order, and BYML maps are `FxHashMap`s on both, so map iteration is in
  hash order either way. The arena readers are the exception for BYML:
  their `children(id)` iterates in document order rather than hash order
  (see below).
- `BymlRef` and `ParameterRef` are enums mirroring `Byml` and `Parameter`
  variant for variant, so existing `match` code carries over; only the
  payload types inside the variants change.

### Parsing notes

- The zero-copy `from_binary` functions do not decompress Yaz0 input, while
  the owned `Byml::from_binary` and `ParameterIO::from_binary` do when the
  `yaz0` feature is enabled. Decompress first with `yaz0::decompress_if` and
  pass the resulting slice to the borrowed parser.
- `BymlRef` and `ParameterIORef` still need the `alloc` feature: they build
  the container `Vec`s and maps, but borrow every string and payload. The
  arena readers do not need an allocator when they use a fixed-capacity
  store.

### Arena readers

For large documents, `byml::arena::BymlArena` and
`aamp::arena::ParameterIOArena` walk a document flat, without recursion:

- Nodes are addressed by `NodeId`, not by reference or index. BYML offers
  `root()`, `node(id)`, `children(id)`, `get(id, key)`, `get_hash(id, hash)`,
  and `child(id, index)`; AAMP offers `root()`, `node(id)`, `children(id)`,
  `get(id, name)`, plus `version()` and `data_type()`. `children(id)` yields
  entries in document order.
- A node's leaf accessors are the borrowed ones (`as_str`, `as_i32`, and so
  on); containers are walked through `children`. `AampNode` exposes its
  `payload` as an `AampPayload` enum instead of going through `as_*`
  methods.
- `from_binary` needs a store implementing `Default` (an `OwnedStore`, or a
  `FixedStore` for `BymlStack`/`ParameterIOStack`). A full fixed store fails
  with `Error::CapacityExceeded`. Use `new()` + `parse_in_place(data)` to
  avoid moving a large inline store by value, or `from_binary_in(data,
  slots)` with caller-owned memory (`BorrowedStore`).
