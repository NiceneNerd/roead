#pragma once
#include "rust/cxx.h"
#include <memory>
#include <oead/aamp.h>
#include <oead/types.h>

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
struct ParamPair;
struct ParamObjPair;
struct ParamListPair;
struct RsParameter;
struct RsParameterIO;
struct RsParameterList;
struct RsParameterObject;

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
std::array<Curve, 1> GetParamCurve1(const Parameter &param);
std::array<Curve, 2> GetParamCurve2(const Parameter &param);
std::array<Curve, 3> GetParamCurve3(const Parameter &param);
std::array<Curve, 4> GetParamCurve4(const Parameter &param);
Curve ToRustCurve(const oead::Curve &curve);
rust::String GetParamString(const Parameter &param);
rust::Vec<int> GetParamBufInt(const Parameter &param);
rust::Vec<float> GetParamBufF32(const Parameter &param);
rust::Vec<u32> GetParamBufU32(const Parameter &param);
rust::Vec<u8> GetParamBufBin(const Parameter &param);
const ParameterMap& GetParams(const ParameterObject &pobj);
const ParameterObjectMap& GetParamObjs(const ParameterList &plist);
const ParameterListMap& GetParamLists(const ParameterList &plist);
const ParameterObjectMap& GetParamObjsFromPio(const ParameterIO &pio);
const ParameterListMap& GetParamListsFromPio(const ParameterIO &pio);
ParamPair GetParamAt(const ParameterMap &pmap, size_t idx);
ParamObjPair GetParamObjAt(const ParameterObjectMap &pobjmap, size_t idx);
ParamListPair GetParamListAt(const ParameterListMap &plmap, size_t idx);
u32 GetPioVersion(const ParameterIO &pio);
rust::String GetPioType(const ParameterIO &pio);

Parameter ParamFromFfi(const RsParameter &param);
ParameterObject PobjFromFfi(const RsParameterObject &pobj);
ParameterList PlistFromFfi(const RsParameterList &plist);
ParameterIO PioFromFfi(const RsParameterIO &pio);

rust::String AampToText(const RsParameterIO &pio);
rust::Vec<uint8_t> AampToBinary(const RsParameterIO &pio);
