pub mod sarc;
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
    }
}
