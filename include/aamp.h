#pragma once
#include "rust/cxx.h"
#include <memory>
#include <oead/aamp.h>

using oead::aamp::ParameterIO;
using oead::aamp::Parameter;
using ParamType = oead::aamp::Parameter::Type;
using oead::aamp::ParameterList;
using oead::aamp::ParameterObject;
using oead::aamp::ParameterListMap;
using oead::aamp::ParameterObjectMap;
using oead::aamp::ParameterMap;
struct Vector2f;
struct Vector3f;
struct Vector4f;
struct Color;
struct Quat;
struct Curve;

std::unique_ptr<ParameterIO> AampFromBinary(rust::Slice<const uint8_t> data);
std::unique_ptr<ParameterIO> AampFromText(rust::Str text);

bool GetParamBool(const Parameter &param);
float GetParamF32(const Parameter &param);
int GetParamInt(const Parameter &param);
u32 GetParamU32(const Parameter &param);
Vector2f GetParamVec2(const Parameter &param);
Vector3f GetParamVec3(const Parameter &param);
Vector4f GetParamVec4(const Parameter &param);
Color GetParamColor(const Parameter &param);
Quat GetParamQuat(const Parameter &param);
