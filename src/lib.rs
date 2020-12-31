pub mod byml;
pub mod sarc;
pub mod types;
pub mod yaz0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Big,
    Little,
}

#[cxx::bridge]
mod ffi {
    struct SarcWriteResult {
        alignment: usize,
        data: Vec<u8>,
    }

    #[repr(u32)]
    enum BymlType {
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
        fn v(self: &U8) -> u8;
        type U16;
        fn v(self: &U16) -> u16;
        type U32;
        fn v(self: &U32) -> u32;
        type U64;
        fn v(self: &U64) -> u64;
        type S8;
        fn v(self: &S8) -> i8;
        type S16;
        fn v(self: &S16) -> i16;
        type S32;
        fn v(self: &S32) -> i32;
        type S64;
        fn v(self: &S64) -> i64;
        type F32;
        fn v(self: &F32) -> f32;
        type F64;
        fn v(self: &F64) -> f64;

        include!("roead/include/byml.h");

        fn BymlFromBinary(data: &[u8]) -> Result<UniquePtr<Byml>>;
        fn BymlFromText(text: &str) -> Result<UniquePtr<Byml>>;
        fn BymlToBinary(node: &Byml, big_endian: bool, version: usize) -> Vec<u8>;
        fn BymlToText(node: &Byml) -> String;

        type Byml;
        type Hash;
        type BymlType;
        type HashNode;
        fn at<'a, 'b>(self: &'a Hash, key: &'b CxxString) -> &'a Byml;
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
    }
}
