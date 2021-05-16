pub mod aamp;
pub mod byml;
pub mod sarc;
pub mod types;
pub mod yaz0;

use crate::byml::Byml as RByml;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Big,
    Little,
}

#[cxx::bridge]
pub(crate) mod ffi {

    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Vector2f {
        x: f32,
        y: f32
    }
    
    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Vector3f {
        x: f32,
        y: f32,
        z: f32
    }
    
    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Vector4f {
        x: f32,
        y: f32,
        z: f32,
        t: f32
    }
    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Quat {
        a: f32,
        b: f32,
        c: f32,
        d: f32
    }
    
    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Color {
        r: f32,
        g: f32,
        b: f32,
        a: f32
    }
    
    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Curve {
        a: u32,
        b: u32,
        floats: [f32; 30]
    }
    
    struct SarcWriteResult {
        alignment: usize,
        data: Vec<u8>,
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

        type ParameterIO;
        type Parameter;
        type ParamType;
        type ParameterList;
        type ParameterObject;
        type ParameterListMap;
        type ParameterObjectMap;
        type ParameterMap;

        fn AampFromBinary(data: &[u8]) -> Result<UniquePtr<ParameterIO>>;
        fn AampFromText(text: &str) -> Result<UniquePtr<ParameterIO>>;

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
    }
}
