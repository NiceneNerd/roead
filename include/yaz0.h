#pragma once
#include "rust/cxx.h"
#include <oead/yaz0.h>
#include <memory>

std::unique_ptr<std::vector<uint8_t>> decompress(const rust::Slice<const uint8_t> data);
rust::Vec<uint8_t> compress(rust::Slice<const uint8_t> data, uint8_t level);
