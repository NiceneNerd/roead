#[inline(always)]
pub(crate) fn align(value: u32, size: u32) -> u32 {
    (value + (size - value % size) % size) as u32
}

#[allow(non_camel_case_types)]
pub(crate) struct u24(u32);

impl u24 {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[cfg(feature = "binrw")]
const _: () = {
    impl binrw::BinRead for u24 {
        type Args = ();
        fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            opts: &binrw::ReadOptions,
            _: (),
        ) -> binrw::BinResult<Self> {
            let buf: [u8; 3] = binrw::BinRead::read(reader)?;
            match opts.endian() {
                binrw::Endian::Little | binrw::Endian::Native => Ok(u24(u32::from(buf[0])
                    | u32::from(buf[1]) << 8
                    | u32::from(buf[2]) << 16)),
                binrw::Endian::Big => Ok(u24(u32::from(buf[2])
                    | u32::from(buf[1]) << 8
                    | u32::from(buf[0]) << 16)),
            }
        }
    }

    impl binrw::BinWrite for u24 {
        type Args = ();
        fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            options: &binrw::WriteOptions,
            args: Self::Args,
        ) -> binrw::BinResult<()> {
            let mut buf = [0; 3];
            match options.endian() {
                binrw::Endian::Little | binrw::Endian::Native => {
                    buf[0] = self.0 as u8;
                    buf[1] = (self.0 >> 8) as u8;
                    buf[2] = (self.0 >> 16) as u8;
                }
                binrw::Endian::Big => {
                    buf[0] = (self.0 >> 16) as u8;
                    buf[1] = (self.0 >> 8) as u8;
                    buf[2] = self.0 as u8;
                }
            }
            buf.write_options(writer, options, args)
        }
    }
};

#[cfg(test)]
#[cfg(feature = "binrw")]
#[test]
fn test_u24_rw() {
    use binrw::*;
    let num = u24(8388608);
    let le_data = b"\x00\x00\x80";
    let be_data = b"\x80\x00\x00";
    let mut buf = Vec::new();
    num.write_options(
        &mut std::io::Cursor::new(&mut buf),
        &binrw::WriteOptions::default().with_endian(binrw::Endian::Little),
        (),
    )
    .unwrap();
    assert_eq!(buf, le_data);
    buf.clear();
    num.write_options(
        &mut std::io::Cursor::new(&mut buf),
        &binrw::WriteOptions::default().with_endian(binrw::Endian::Big),
        (),
    )
    .unwrap();
    assert_eq!(buf, be_data);
    buf.clear();
    let mut reader = std::io::Cursor::new(le_data);
    let num = u24::read_options(
        &mut reader,
        &binrw::ReadOptions::default().with_endian(Endian::Little),
        (),
    )
    .unwrap();
    assert_eq!(num.0, 8388608);
    reader = std::io::Cursor::new(be_data);
    let num = u24::read_options(
        &mut reader,
        &binrw::ReadOptions::default().with_endian(Endian::Big),
        (),
    )
    .unwrap();
    assert_eq!(num.0, 8388608);
}
