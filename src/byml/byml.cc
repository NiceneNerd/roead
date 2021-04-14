#include "roead/include/byml.h"
#include "roead/src/lib.rs.h"
#include "rust/cxx.h"
#include <iostream>
#include <memory>
#include <nonstd/span.h>
#include <oead/byml.h>
#include <oead/sarc.h>
#include <stdexcept>
#include <string_view>

std::unique_ptr<Byml> BymlFromBinary(rust::Slice<const uint8_t> data) {
  return std::make_unique<Byml>(
      oead::Byml::FromBinary({data.data(), data.size()}));
}

std::unique_ptr<Byml> BymlFromText(rust::Str text) {
  return std::make_unique<Byml>(
      oead::Byml::FromText({text.data(), text.size()}));
}

rust::Vec<uint8_t> BymlToBinary(const RByml &ffiNode, bool big_endian,
                                size_t version) {
  const auto node = FromFfi(ffiNode);
  std::vector<uint8_t> data = node.ToBinary(big_endian, version);
  rust::Vec<uint8_t> vec;
  std::move(data.begin(), data.end(), std::back_inserter(vec));
  return vec;
}

rust::String BymlToText(const RByml &ffiNode) {
  const auto node = FromFfi(ffiNode);
  std::string text = node.ToText();
  return rust::String(text);
}

rust::String GetBymlString(Byml &byml) {
  std::string str = byml.GetString();
  return rust::String(str.data(), str.size());
}

std::unique_ptr<std::vector<std::string>> GetHashKeys(const Hash &hash) {
  std::vector<std::string> keys;
  keys.reserve(hash.size());
  for (auto &[key, _] : hash) {
    keys.push_back(key);
  }
  return std::make_unique<std::vector<std::string>>(keys);
}

Byml FromFfi(const RByml &node) {
  switch (node.get_ffi_type()) {
  case BymlType::Array: {
    std::vector<Byml> vec;
    vec.reserve(node.len());
    for (int i = 0; i < node.len(); i++) {
      vec.push_back(FromFfi(node.get(i)));
    }
    return Byml(vec);
  }
  case BymlType::Hash: {
    absl::btree_map<std::string, Byml> hash;
    for (int i = 0; i < node.len(); i++) {
      auto key = node.get_key_by_index(i);
      hash.insert(std::make_pair<std::string, Byml>(std::string(key),
                                                    FromFfi(node.get(i))));
    }
    return Byml(hash);
  }
  case BymlType::Bool:
    return Byml(node.as_bool());
  case BymlType::String:
    return Byml(std::string(node.as_string()));
  case BymlType::Int:
    return Byml(oead::S32(node.as_int()));
  case BymlType::Int64:
    return Byml(oead::S64(node.as_int64()));
  case BymlType::UInt:
    return Byml(oead::U32(node.as_uint()));
  case BymlType::UInt64:
    return Byml(oead::U64(node.as_uint64()));
  case BymlType::Float:
    return Byml(oead::F32(node.as_float()));
  case BymlType::Double:
    return Byml(oead::F64(node.as_double()));
  }
}
