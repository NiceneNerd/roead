#include "roead/include/yaz0.h"
#include "roead/src/lib.rs.h"
#include "rust/cxx.h"
#include <memory>
#include <iostream>
#include <nonstd/span.h>
#include <oead/sarc.h>

rust::Vec<uint8_t> decompress(const rust::Slice<const uint8_t> data) {
    auto dec = oead::yaz0::Decompress(tcb::span(data.data(), data.size()));
    rust::Vec<uint8_t> vec;
    std::move(dec.begin(), dec.end(), std::back_inserter(vec));
    return vec;
}

rust::Vec<uint8_t> compress(const rust::Slice<const uint8_t> data, uint8_t level) {
    auto com = oead::yaz0::Compress(tcb::span(data.data(), data.size()), 0, level);
    rust::Vec<uint8_t> vec;
    std::move(com.begin(), com.end(), std::back_inserter(vec));
    return vec;
}