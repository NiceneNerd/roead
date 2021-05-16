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

bool GetParamBool(const Parameter &param) {
    return param.Get<ParamType::Bool>();
}

float GetParamF32(const Parameter &param) {
    return param.Get<ParamType::F32>();
}

int GetParamInt(const Parameter &param) {
    return param.Get<ParamType::Int>();
}

u32 GetParamU32(const Parameter &param) {
    return param.Get<ParamType::U32>();
}

Vector2f GetParamVec2(const Parameter &param) {
  const auto vec2 = param.Get<ParamType::Vec2>();
  return { vec2.x, vec2.y };
}


Vector3f GetParamVec3(const Parameter &param) {
  const auto vec3 = param.Get<ParamType::Vec3>();
  return { vec3.x, vec3.y, vec3.z };
}

Vector4f GetParamVec4(const Parameter &param) {
  const auto vec4 = param.Get<ParamType::Vec4>();
  return { vec4.x, vec4.y, vec4.z, vec4.t };
}

Color GetParamColor(const Parameter &param) {
  const auto color = param.Get<ParamType::Color>();
  return { color.r, color.g, color.b, color.a };
}

Quat GetParamQuat(const Parameter &param) {
  const auto quat = param.Get<ParamType::Quat>();
  return { quat.a, quat.b, quat.c, quat.d };
}
