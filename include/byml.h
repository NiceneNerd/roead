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

struct RByml;

Byml FromFfi(const RByml &node);
rust::Vec<uint8_t> BymlToBinary(const RByml &node, bool big_endian,
                                size_t version);
rust::String BymlToText(const RByml &node);


std::unique_ptr<std::vector<std::string>> GetHashKeys(const Hash &hash);