//! # Rust bindings for the oead C++ library
//! **oead** is a C++ library for common file formats that are used in modern
//! first-party Nintendo EAD (now EPD) titles.
//! 
//! Currently, oead only handles very common formats that are extensively used
//! in recent games such as *Breath of the Wild* and *Super Mario Odyssey*.
//! 
//! * [AAMP](https://zeldamods.org/wiki/AAMP) (binary parameter archive): Only version 2 is supported.
//! * [BYML](https://zeldamods.org/wiki/BYML) (binary YAML): Versions 2, 3, and 4 are supported.
//! * [SARC](https://zeldamods.org/wiki/SARC) (archive)
//! * [Yaz0](https://zeldamods.org/wiki/Yaz0) (compression algorithm)
//! 
//! The roead project attempts to provide safe and relatively idiomatic Rust
//! bindings to oead's core functionality. The Grezzo datasheets are not supported.
//! For more info on oead itself, visit [its GitHub repo](https://github.com/zeldamods/oead/).
//! 
//! For API documentation, see the docs for each module.
pub mod aamp;
pub mod byml;
pub mod sarc;
pub mod types;
pub mod yaz0;

use crate::aamp::Parameter as RsParameter;
use crate::aamp::ParameterIO as RsParameterIO;
use crate::aamp::ParameterList as RsParameterList;
use crate::aamp::ParameterObject as RsParameterObject;
use crate::byml::Byml as RByml;

/// Represents endianness where applicable. Generally, big endian is used for 
/// Wii U and little endian is used for Switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Big,
    Little,
}

#[cxx::bridge]
pub(crate) mod ffi {
    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Vector2f {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Vector3f {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Vector4f {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub t: f32,
    }
    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Quat {
        pub a: f32,
        pub b: f32,
        pub c: f32,
        pub d: f32,
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Curve {
        pub a: u32,
        pub b: u32,
        pub floats: [f32; 30],
    }

    struct SarcWriteResult {
        alignment: usize,
        data: Vec<u8>,
    }

    struct ParamPair {
        hash: u32,
        param: UniquePtr<Parameter>,
    }

    struct ParamObjPair {
        hash: u32,
        param: UniquePtr<ParameterObject>,
    }

    struct ParamListPair {
        hash: u32,
        param: UniquePtr<ParameterList>,
    }

    #[repr(u32)]
    pub(crate) enum BymlType {
        Null = 0,
        String,
        Binary,
        Array,
        Hash,
        Bool,
        Int,
        Float,
        UInt,
        Int64,
        UInt64,
        Double,
    }

    #[repr(u8)]
    pub(crate) enum ParamType {
        Bool = 0,
        F32,
        Int,
        Vec2,
        Vec3,
        Vec4,
        Color,
        String32,
        String64,
        Curve1,
        Curve2,
        Curve3,
        Curve4,
        BufferInt,
        BufferF32,
        String256,
        Quat,
        U32,
        BufferU32,
        BufferBinary,
        StringRef,
    }

    struct U8 {
        value: u8,
    }
    struct U16 {
        value: u16,
    }
    struct U32 {
        value: u32,
    }
    struct U64 {
        value: u64,
    }
    struct S8 {
        value: i8,
    }
    struct S16 {
        value: i16,
    }
    struct S32 {
        value: i32,
    }
    struct S64 {
        value: i64,
    }

    extern "Rust" {
        type RByml;
        fn get_ffi_type(self: &RByml) -> BymlType;
        fn as_string(self: &RByml) -> Result<&str>;
        fn as_binary(self: &RByml) -> Result<&[u8]>;
        fn as_bool(self: &RByml) -> Result<bool>;
        fn as_int(self: &RByml) -> Result<i32>;
        fn as_int64(self: &RByml) -> Result<i64>;
        fn as_uint(self: &RByml) -> Result<u32>;
        fn as_uint64(self: &RByml) -> Result<u64>;
        fn as_float(self: &RByml) -> Result<f32>;
        fn as_double(self: &RByml) -> Result<f64>;
        fn len(self: &RByml) -> usize;
        fn get(self: &RByml, index: usize) -> &RByml;
        fn get_key_by_index(self: &RByml, index: usize) -> &String;

        type RsParameter;
        fn get_ffi_type(self: &RsParameter) -> ParamType;
        fn get_bool(self: &RsParameter) -> bool;
        fn get_f32(self: &RsParameter) -> f32;
        fn get_int(self: &RsParameter) -> i32;
        fn get_vec2(self: &RsParameter) -> &Vector2f;
        fn get_vec3(self: &RsParameter) -> &Vector3f;
        fn get_vec4(self: &RsParameter) -> &Vector4f;
        fn get_color(self: &RsParameter) -> &Color;
        fn get_string32(self: &RsParameter) -> &str;
        fn get_string64(self: &RsParameter) -> &str;
        fn get_curve1(self: &RsParameter) -> &[Curve; 1];
        fn get_curve2(self: &RsParameter) -> &[Curve; 2];
        fn get_curve3(self: &RsParameter) -> &[Curve; 3];
        fn get_curve4(self: &RsParameter) -> &[Curve; 4];
        fn get_buf_int(self: &RsParameter) -> &[i32];
        fn get_buf_f32(self: &RsParameter) -> &[f32];
        fn get_string_256(self: &RsParameter) -> &str;
        fn get_quat(self: &RsParameter) -> &Quat;
        fn get_u32(self: &RsParameter) -> u32;
        fn get_buf_u32(self: &RsParameter) -> &[u32];
        fn get_buf_bin(self: &RsParameter) -> &[u8];
        fn get_str_ref(self: &RsParameter) -> &str;
        type RsParameterIO;
        fn list_count(self: &RsParameterIO) -> usize;
        fn object_count(self: &RsParameterIO) -> usize;
        fn list_hash_at(self: &RsParameterIO, i: usize) -> u32;
        fn obj_hash_at(self: &RsParameterIO, i: usize) -> u32;
        fn list_at(self: &RsParameterIO, i: usize) -> &RsParameterList;
        fn obj_at(self: &RsParameterIO, i: usize) -> &RsParameterObject;
        fn version(self: &RsParameterIO) -> u32;
        fn pio_type(self: &RsParameterIO) -> &str;
        type RsParameterList;
        fn list_count(self: &RsParameterList) -> usize;
        fn object_count(self: &RsParameterList) -> usize;
        fn list_hash_at(self: &RsParameterList, i: usize) -> u32;
        fn obj_hash_at(self: &RsParameterList, i: usize) -> u32;
        fn list_at(self: &RsParameterList, i: usize) -> &RsParameterList;
        fn obj_at(self: &RsParameterList, i: usize) -> &RsParameterObject;
        type RsParameterObject;
        fn len(self: &RsParameterObject) -> usize;
        fn hash_at(self: &RsParameterObject, idx: usize) -> u32;
        fn val_at(self: &RsParameterObject, idx: usize) -> &RsParameter;
    }

    unsafe extern "C++" {
        include!("roead/include/sarc.h");

        type Sarc;
        fn num_files(self: &Sarc) -> u16;
        fn get_offset(self: &Sarc) -> u32;
        fn guess_align(self: &Sarc) -> usize;
        fn big_endian(self: &Sarc) -> bool;
        fn files_eq(self: &Sarc, other: &Sarc) -> bool;
        fn get_file_data(self: &Sarc, name: &str) -> Result<&[u8]>;
        fn idx_file_data(self: &Sarc, idx: u16) -> Result<&[u8]>;
        fn idx_file_name(self: &Sarc, idx: u16) -> Result<&str>;
        pub(crate) fn sarc_from_binary(data: &[u8]) -> Result<UniquePtr<Sarc>>;

        type SarcWriter;
        fn NewSarcWriter(big_endian: bool, legacy: bool) -> UniquePtr<SarcWriter>;
        fn SetMinAlignment(self: Pin<&mut SarcWriter>, alignment: usize);
        fn SetEndianness(self: Pin<&mut SarcWriter>, big_endian: bool);
        fn SetMode(self: Pin<&mut SarcWriter>, legacy: bool);
        fn SetFile(self: Pin<&mut SarcWriter>, name: &str, data: Vec<u8>);
        fn DelFile(self: Pin<&mut SarcWriter>, name: &str) -> bool;
        fn NumFiles(self: &SarcWriter) -> usize;
        fn FilesEqual(self: &SarcWriter, other: &SarcWriter) -> bool;
        fn Write(self: Pin<&mut SarcWriter>) -> SarcWriteResult;
        fn WriterFromSarc(archive: &Sarc) -> UniquePtr<SarcWriter>;

        include!("roead/include/yaz0.h");

        fn decompress(data: &[u8]) -> Result<Vec<u8>>;
        fn compress(data: &[u8], level: u8) -> Vec<u8>;

        include!("roead/include/types.h");

        type U8;
        type U16;
        type U32;
        type U64;
        type S8;
        type S16;
        type S32;
        type S64;
        type F32;
        type F64;

        include!("roead/include/byml.h");

        fn BymlFromBinary(data: &[u8]) -> Result<UniquePtr<Byml>>;
        fn BymlFromText(text: &str) -> Result<UniquePtr<Byml>>;
        fn BymlToBinary(node: &RByml, big_endian: bool, version: usize) -> Vec<u8>;
        fn BymlToText(node: &RByml) -> String;

        type Byml;
        type Hash;
        type BymlType;
        fn GetType(self: &Byml) -> BymlType;
        fn GetString(self: &Byml) -> &CxxString;
        fn GetBool(self: &Byml) -> bool;
        fn GetInt(self: &Byml) -> i32;
        fn GetUInt(self: &Byml) -> u32;
        fn GetInt64(self: &Byml) -> i64;
        fn GetUInt64(self: &Byml) -> u64;
        fn GetFloat(self: &Byml) -> f32;
        fn GetDouble(self: &Byml) -> f64;
        fn GetBinary(self: &Byml) -> &CxxVector<u8>;
        fn GetArray(self: &Byml) -> &CxxVector<Byml>;
        fn GetHash(self: &Byml) -> &Hash;

        fn GetHashKeys(hash: &Hash) -> UniquePtr<CxxVector<CxxString>>;
        fn at<'a, 'b>(self: &'a Hash, key: &'b CxxString) -> &'a Byml;

        include!("roead/include/aamp.h");

        pub(crate) type ParameterIO;
        type Parameter;
        type ParamType;
        pub(crate) type ParameterList;
        pub(crate) type ParameterObject;
        pub(crate) type ParameterListMap;
        pub(crate) type ParameterObjectMap;
        pub(crate) type ParameterMap;
        pub(crate) fn size(self: &ParameterMap) -> usize;
        pub(crate) fn size(self: &ParameterObjectMap) -> usize;
        pub(crate) fn size(self: &ParameterListMap) -> usize;

        pub(crate) fn AampFromBinary(data: &[u8]) -> Result<UniquePtr<ParameterIO>>;
        pub(crate) fn AampFromText(text: &str) -> Result<UniquePtr<ParameterIO>>;
        pub(crate) fn AampToText(pio: &RsParameterIO) -> String;
        pub(crate) fn AampToBinary(pio: &RsParameterIO) -> Vec<u8>;

        fn GetType(self: &Parameter) -> ParamType;
        pub(crate) fn GetParamBool(param: &Parameter) -> bool;
        pub(crate) fn GetParamInt(param: &Parameter) -> i32;
        pub(crate) fn GetParamU32(param: &Parameter) -> u32;
        pub(crate) fn GetParamF32(param: &Parameter) -> f32;
        pub(crate) fn GetParamVec2(param: &Parameter) -> Vector2f;
        pub(crate) fn GetParamVec3(param: &Parameter) -> Vector3f;
        pub(crate) fn GetParamVec4(param: &Parameter) -> Vector4f;
        pub(crate) fn GetParamColor(param: &Parameter) -> Color;
        pub(crate) fn GetParamQuat(param: &Parameter) -> Quat;
        pub(crate) fn GetParamCurve1(param: &Parameter) -> [Curve; 1];
        pub(crate) fn GetParamCurve2(param: &Parameter) -> [Curve; 2];
        pub(crate) fn GetParamCurve3(param: &Parameter) -> [Curve; 3];
        pub(crate) fn GetParamCurve4(param: &Parameter) -> [Curve; 4];
        pub(crate) fn GetParamString(param: &Parameter) -> String;
        pub(crate) fn GetParamBufInt(param: &Parameter) -> Vec<i32>;
        pub(crate) fn GetParamBufF32(param: &Parameter) -> Vec<f32>;
        pub(crate) fn GetParamBufU32(param: &Parameter) -> Vec<u32>;
        pub(crate) fn GetParamBufBin(param: &Parameter) -> Vec<u8>;
        pub(crate) fn GetParams(pobj: &ParameterObject) -> UniquePtr<ParameterMap>;
        pub(crate) fn GetParamObjs(plist: &ParameterList) -> UniquePtr<ParameterObjectMap>;
        pub(crate) fn GetParamLists(plist: &ParameterList) -> UniquePtr<ParameterListMap>;
        pub(crate) fn GetParamObjsFromPio(pio: &ParameterIO) -> UniquePtr<ParameterObjectMap>;
        pub(crate) fn GetParamListsFromPio(pio: &ParameterIO) -> UniquePtr<ParameterListMap>;
        pub(crate) fn GetParamAt(pmap: &ParameterMap, idx: usize) -> ParamPair;
        pub(crate) fn GetParamObjAt(pmap: &ParameterObjectMap, idx: usize) -> ParamObjPair;
        pub(crate) fn GetParamListAt(pmap: &ParameterListMap, idx: usize) -> ParamListPair;
        pub(crate) fn GetPioVersion(pio: &ParameterIO) -> u32;
        pub(crate) fn GetPioType(pio: &ParameterIO) -> String;
    }
}
