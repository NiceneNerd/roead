#pragma once
#include "rust/cxx.h"
#include <memory>
#include <oead/aamp.h>

using oead::aamp::ParameterIO;

std::unique_ptr<ParameterIO> AampFromBinary(rust::Slice<const uint8_t> data);
std::unique_ptr<ParameterIO> AampFromText(rust::Str text);
