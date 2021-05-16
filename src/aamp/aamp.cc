#include "roead/include/aamp.h"
#include "roead/src/lib.rs.h"
#include "rust/cxx.h"
#include <iostream>
#include <memory>
#include <nonstd/span.h>
#include <oead/aamp.h>
#include <stdexcept>
#include <string_view>

std::unique_ptr<ParameterIO> AampFromBinary(rust::Slice<const uint8_t> data) {
  return std::make_unique<ParameterIO>(
      ParameterIO::FromBinary({data.data(), data.size()}));
}

std::unique_ptr<ParameterIO> AampFromText(rust::Str text) {
  return std::make_unique<ParameterIO>(
      ParameterIO::FromText({text.data(), text.size()}));
}
