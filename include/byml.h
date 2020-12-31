#pragma once
#include "rust/cxx.h"
#include <memory>
#include <oead/byml.h>

using oead::Byml;
using Hash = absl::btree_map<std::string, oead::Byml>;
using Array = oead::Byml::Array;
using BymlType = oead::Byml::Type;

std::unique_ptr<oead::Byml> BymlFromBinary(rust::Slice<const uint8_t> data);
std::unique_ptr<oead::Byml> BymlFromText(rust::Str text);
rust::Vec<uint8_t> BymlToBinary(const Byml &node, bool big_endian,
                                size_t version);
rust::String BymlToText(const Byml &node);

rust::String GetBymlString(Byml &byml);

class HashNode {
public:
  rust::String key() const;
  const Byml &value() const;
  std::string m_key;
  const Byml &m_value;
  HashNode(std::string key, const Byml &value) : m_key(key), m_value(value) {}
};

const std::vector<HashNode> &GetHashNodes(const Hash &hash);
std::unique_ptr<std::vector<std::string>> GetHashKeys(const Hash &hash);
std::vector<Byml> &GetHashVals(Hash &hash,
                               const std::vector<std::string> &keys);