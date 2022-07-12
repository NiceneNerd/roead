#include "roead/include/aamp.h"
#include "roead/src/lib.rs.h"
#include "rust/cxx.h"
#include <iostream>
#include <memory>
#include <nonstd/span.h>
#include <oead/aamp.h>
#include <stdexcept>
#include <string_view>

std::unique_ptr<ParameterIO> AampFromBinary(rust::Slice<const uint8_t> data)
{
  return std::make_unique<ParameterIO>(
      ParameterIO::FromBinary({data.data(), data.size()}));
}

std::unique_ptr<ParameterIO> AampFromText(rust::Str text)
{
  return std::make_unique<ParameterIO>(
      ParameterIO::FromText({ text.data(), text.size() }));
}

bool GetParamBool(const Parameter &param)
{
  return param.Get<ParamType::Bool>();
}

float GetParamF32(const Parameter &param)
{
  return param.Get<ParamType::F32>();
}

int GetParamInt(const Parameter &param)
{
  return param.Get<ParamType::Int>();
}

u32 GetParamU32(const Parameter &param)
{
  return param.Get<ParamType::U32>();
}

Vector2f GetParamVec2(const Parameter &param)
{
  const auto vec2 = param.Get<ParamType::Vec2>();
  return {vec2.x, vec2.y};
}

Vector3f GetParamVec3(const Parameter &param)
{
  const auto vec3 = param.Get<ParamType::Vec3>();
  return {vec3.x, vec3.y, vec3.z};
}

Vector4f GetParamVec4(const Parameter &param)
{
  const auto vec4 = param.Get<ParamType::Vec4>();
  return {vec4.x, vec4.y, vec4.z, vec4.t};
}

Color GetParamColor(const Parameter &param)
{
  const auto color = param.Get<ParamType::Color>();
  return {color.r, color.g, color.b, color.a};
}

Quat GetParamQuat(const Parameter &param)
{
  const auto quat = param.Get<ParamType::Quat>();
  return {quat.a, quat.b, quat.c, quat.d};
}

std::array<Curve, 1> GetParamCurve1(const Parameter &param)
{
  const auto curves = param.Get<ParamType::Curve1>();
  return std::array<Curve, 1>{ToRustCurve(curves[0])};
}

std::array<Curve, 2> GetParamCurve2(const Parameter &param)
{
  const auto curves = param.Get<ParamType::Curve2>();
  return std::array<Curve, 2>{ToRustCurve(curves[0]), ToRustCurve(curves[1])};
}

std::array<Curve, 3> GetParamCurve3(const Parameter &param)
{
  const auto curves = param.Get<ParamType::Curve3>();
  return std::array<Curve, 3>{ToRustCurve(curves[0]), ToRustCurve(curves[1]), ToRustCurve(curves[2])};
}

std::array<Curve, 4> GetParamCurve4(const Parameter &param)
{
  const auto curves = param.Get<ParamType::Curve4>();
  return std::array<Curve, 4>{ToRustCurve(curves[0]), ToRustCurve(curves[1]), ToRustCurve(curves[2]), ToRustCurve(curves[3])};
}

Curve ToRustCurve(const oead::Curve &curve)
{
  return {
      curve.a, curve.b, curve.floats};
}

rust::String GetParamString(const Parameter &param)
{
  const auto string = param.GetStringView();
  return rust::String(string.data(), string.size());
}

String32 GetParamString32(const Parameter &param)
{
  const auto string = param.Get<ParamType::String32>();
  return string;
}

String64 GetParamString64(const Parameter &param)
{
  const auto string = param.Get<ParamType::String64>();
  return string;
}

String256 GetParamString256(const Parameter &param)
{
  const auto string = param.Get<ParamType::String256>();
  return string;
}

rust::Vec<int> GetParamBufInt(const Parameter &param)
{
  const auto buf = param.Get<ParamType::BufferInt>();
  rust::Vec<int> vec;
  std::move(buf.begin(), buf.end(), std::back_inserter(vec));
  return vec;
}

rust::Vec<float> GetParamBufF32(const Parameter &param)
{
  const auto buf = param.Get<ParamType::BufferF32>();
  rust::Vec<float> vec;
  std::move(buf.begin(), buf.end(), std::back_inserter(vec));
  return vec;
}

rust::Vec<u32> GetParamBufU32(const Parameter &param)
{
  const auto buf = param.Get<ParamType::BufferU32>();
  rust::Vec<u32> vec;
  std::move(buf.begin(), buf.end(), std::back_inserter(vec));
  return vec;
}

std::unique_ptr<std::vector<uint8_t>> GetParamBufBin(const Parameter &param)
{
  const auto buf = param.Get<ParamType::BufferBinary>();
  return std::make_unique<std::vector<uint8_t>>(buf);
}

std::unique_ptr<ParameterMap> GetParams(const ParameterObject &pobj)
{
  return std::make_unique<ParameterMap>(pobj.params);
}

std::unique_ptr<ParameterListMap> GetParamLists(const ParameterList &plist)
{
  return std::make_unique<ParameterListMap>(plist.lists);
}

std::unique_ptr<ParameterObjectMap> GetParamObjs(const ParameterList &plist)
{
  return std::make_unique<ParameterObjectMap>(plist.objects);
}

std::unique_ptr<ParameterListMap> GetParamListsFromPio(const ParameterIO &pio)
{
  return std::make_unique<ParameterListMap>(pio.lists);
}

std::unique_ptr<ParameterObjectMap> GetParamObjsFromPio(const ParameterIO &plist)
{
  return std::make_unique<ParameterObjectMap>(plist.objects);
}

ParamPair GetParamAt(const ParameterMap &pmap, size_t idx)
{
  const auto [hash, val] = pmap.values_container().at(idx);
  return {hash.hash, std::make_unique<Parameter>(val)};
}

ParamObjPair GetParamObjAt(const ParameterObjectMap &pobjmap, size_t idx)
{
  const auto [hash, val] = pobjmap.values_container().at(idx);
  return {hash.hash, std::make_unique<ParameterObject>(val)};
}

ParamListPair GetParamListAt(const ParameterListMap &plmap, size_t idx)
{
  const auto [hash, val] = plmap.values_container().at(idx);
  return {hash.hash, std::make_unique<ParameterList>(val)};
}

u32 GetPioVersion(const ParameterIO &pio)
{
  return pio.version;
}

rust::String GetPioType(const ParameterIO &pio)
{
  return rust::String(pio.type);
}

oead::Curve CurveFromFfi(const Curve &curve)
{
  return {
      curve.a, curve.b, curve.floats};
}

Parameter ParamFromFfi(const RsParameter &param)
{
  switch (param.get_ffi_type())
  {
  case ParamType::Bool:
    return Parameter(param.get_bool());
  case ParamType::F32:
    return Parameter(oead::F32(param.get_f32()));
  case ParamType::Int:
    return Parameter(param.get_int());
  case ParamType::Vec2:
  {
    const auto vec = param.get_vec2();
    return Parameter(oead::Vector2f{vec.x, vec.y});
  }
  case ParamType::Vec3:
  {
    const auto vec = param.get_vec3();
    return Parameter(oead::Vector3f{vec.x, vec.y, vec.z});
  }
  case ParamType::Vec4:
  {
    const auto vec = param.get_vec4();
    return Parameter(oead::Vector4f{vec.x, vec.y, vec.z, vec.t});
  }
  case ParamType::Color:
  {
    const auto vec = param.get_color();
    return Parameter(oead::Color4f{vec.r, vec.g, vec.b, vec.a});
  }
  case ParamType::String32:
  {
    const auto str = param.get_string32();
    return Parameter(oead::FixedSafeString<32>(std::string_view{str.data(), str.size()}));
  }
  case ParamType::String64:
  {
    const auto str = param.get_string64();
    return Parameter(oead::FixedSafeString<64>(std::string_view{str.data(), str.size()}));
  }
  case ParamType::Curve1:
    return Parameter(std::array<oead::Curve, 1>{CurveFromFfi(param.get_curve1()[0])});
  case ParamType::Curve2:
  {
    const auto curves = param.get_curve2();
    return Parameter(std::array<oead::Curve, 2>{CurveFromFfi(curves[0]), CurveFromFfi(curves[1])});
  }
  case ParamType::Curve3:
  {
    const auto curves = param.get_curve3();
    return Parameter(std::array<oead::Curve, 3>{CurveFromFfi(curves[0]), CurveFromFfi(curves[1]), CurveFromFfi(curves[2])});
  }
  case ParamType::Curve4:
  {
    const auto curves = param.get_curve4();
    return Parameter(std::array<oead::Curve, 4>{CurveFromFfi(curves[0]), CurveFromFfi(curves[1]), CurveFromFfi(curves[2]), CurveFromFfi(curves[3])});
  }
  case ParamType::BufferInt:
  {
    const auto buf = param.get_buf_int();
    std::vector<int> vec(buf.size());
    for (auto v : buf)
    {
      vec.push_back(v);
    }
    return Parameter(vec);
  }
  case ParamType::BufferF32:
  {
    const auto buf = param.get_buf_f32();
    std::vector<float> vec(buf.size());
    for (auto v : buf)
    {
      vec.push_back(v);
    }
    return Parameter(vec);
  }
  case ParamType::String256:
  {
    const auto str = param.get_string_256();
    return Parameter(oead::FixedSafeString<256>(std::string_view{str.data(), str.size()}));
  }
  case ParamType::Quat:
  {
    const auto vec = param.get_quat();
    return Parameter(oead::Quatf{vec.a, vec.b, vec.c, vec.d});
  }
  case ParamType::U32:
    return Parameter(oead::U32(param.get_u32()));
  case ParamType::BufferU32:
  {
    const auto buf = param.get_buf_u32();
    std::vector<uint32_t> vec(buf.size());
    for (auto v : buf)
    {
      vec.push_back(v);
    }
    return Parameter(vec);
  }
  case ParamType::BufferBinary:
  {
    const auto buf = param.get_buf_bin();
    std::vector<uint8_t> vec(buf.size());
    for (auto v : buf)
    {
      vec.push_back(v);
    }
    return Parameter(vec);
  }
  case ParamType::StringRef:
  {
    const auto str = param.get_str_ref();
    return Parameter(std::string{str.data(), str.size()});
  }
  default:
    throw std::runtime_error("Uh oh");
  }
}

ParameterObject PobjFromFfi(const RsParameterObject &pobj)
{
  const auto size = pobj.len();
  auto map = ParameterMap(size);
  for (size_t i = 0; i < size; i++)
  {
    map[pobj.hash_at(i)] = ParamFromFfi(pobj.val_at(i));
  }
  return {map};
}

ParameterList PlistFromFfi(const RsParameterList &plist)
{
  const auto objSize = plist.object_count();
  auto objMap = ParameterObjectMap(objSize);
  for (size_t i = 0; i < objSize; i++)
  {
    objMap[plist.obj_hash_at(i)] = PobjFromFfi(plist.obj_at(i));
  }
  const auto listSize = plist.list_count();
  auto listMap = ParameterListMap(plist.list_count());
  for (size_t i = 0; i < listSize; i++)
  {
    listMap[plist.list_hash_at(i)] = PlistFromFfi(plist.list_at(i));
  }
  return {objMap, listMap};
}

ParameterIO PioFromFfi(const RsParameterIO &pio)
{
  const auto version = pio.version();
  const auto type = pio.pio_type();
  const auto objSize = pio.object_count();
  auto objMap = ParameterObjectMap(objSize);
  for (size_t i = 0; i < objSize; i++)
  {
    objMap[pio.obj_hash_at(i)] = PobjFromFfi(pio.obj_at(i));
  }
  const auto listSize = pio.list_count();
  auto listMap = ParameterListMap(pio.list_count());
  for (size_t i = 0; i < listSize; i++)
  {
    listMap[pio.list_hash_at(i)] = PlistFromFfi(pio.list_at(i));
  }
  return {objMap, listMap, version, std::string(type.data(), type.size())};
}

rust::String AampToText(const RsParameterIO &pio)
{
  const auto oead_pio = PioFromFfi(pio);
  const auto text = oead_pio.ToText();
  return rust::String(text.data(), text.size());
}

std::unique_ptr<std::vector<uint8_t>> AampToBinary(const RsParameterIO &pio)
{
  const auto oead_pio = PioFromFfi(pio);
  const auto data = oead_pio.ToBinary();
  return std::make_unique<std::vector<uint8_t>>(data);
}
