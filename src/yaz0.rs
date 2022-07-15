#[cxx::bridge(namespace = "oead::yaz0")]
mod ffi {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    struct Header {
        magic: [u8; 4],
        uncompressed_size: u32,
        data_alignment: u32,
        reserved: [u8; 4],
    }

    unsafe extern "C++" {
        include!("roead/src/include/oead/yaz0.h");
        fn GetHeader(data: &[u8]) -> Result<Header>;
        fn Decompress(data: &[u8]) -> Result<Vec<u8>>;
        #[rust_name = "DecompressIntoBuffer"]
        fn Decompress(data: &[u8], dest: &mut [u8]) -> Result<()>;
        unsafe fn DecompressUnsafe(data: &[u8], dest: &mut [u8]) -> Result<()>;
        fn Compress(data: &[u8], data_alignment: u32, level: i32) -> Vec<u8>;
    }
}
