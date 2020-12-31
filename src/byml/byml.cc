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

rust::Vec<uint8_t> BymlToBinary(const Byml &node, bool big_endian,
                                size_t version) {
  std::vector<uint8_t> data = node.ToBinary(big_endian, version);
  rust::Vec<uint8_t> vec;
  std::move(data.begin(), data.end(), std::back_inserter(vec));
  return vec;
}

rust::String BymlToText(const Byml &node) {
  std::string text = node.ToText();
  return rust::String(text);
}

rust::String GetBymlString(Byml &byml) {
  std::string str = byml.GetString();
  return rust::String(str.data(), str.size());
}

rust::String HashNode::key() const { return rust::String(m_key); }

const Byml &HashNode::value() const { return m_value; }

const std::vector<HashNode> &GetHashNodes(const Hash &hash) {
  std::vector<HashNode> nodes;
  for (const auto &[key, val] : hash) {
    nodes.push_back(HashNode(key, val));
  }
  return nodes;
}

std::unique_ptr<std::vector<std::string>> GetHashKeys(const Hash &hash) {
  std::vector<std::string> keys;
  keys.reserve(hash.size());
  for (auto &[key, _] : hash) {
    keys.push_back(key);
  }
  return std::make_unique<std::vector<std::string>>(keys);
}

std::vector<Byml> &GetHashVals(Hash &hash,
                               const std::vector<std::string> &keys) {
  std::vector<Byml> vals;
  for (const auto &key : keys) {
    vals.push_back(hash.at(key));
  }
  return vals;
}