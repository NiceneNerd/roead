#[inline(always)]
pub(crate) fn align(value: u32, size: u32) -> u32 {
    value + (size - value % size) % size
}

pub(crate) trait SeekShim {
    fn stream_len(&mut self) -> std::io::Result<u64>
    where
        Self: std::io::Read + std::io::Seek,
    {
        let old_pos = self.stream_position()?;
        let len = self.seek(std::io::SeekFrom::End(0))?;

        // Avoid seeking a third time when we were already at the end of the
        // stream. The branch is usually way cheaper than a seek operation.
        if old_pos != len {
            self.seek(std::io::SeekFrom::Start(old_pos))?;
        }

        Ok(len)
    }
}

impl<T> SeekShim for T where T: std::io::Read + std::io::Seek {}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct u24(pub u32);

impl u24 {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[cfg(feature = "binrw")]
const _: () = {
    impl binrw::BinRead for u24 {
        type Args<'b> = ();

        fn read_options<R: std::io::Read + std::io::Seek>(
            reader: &mut R,
            endian: binrw::Endian,
            _: (),
        ) -> binrw::BinResult<Self> {
            let buf: [u8; 3] = binrw::BinRead::read(reader)?;
            match endian {
                binrw::Endian::Little => {
                    Ok(u24(u32::from(buf[0])
                        | u32::from(buf[1]) << 8
                        | u32::from(buf[2]) << 16))
                }
                binrw::Endian::Big => {
                    Ok(u24(u32::from(buf[2])
                        | u32::from(buf[1]) << 8
                        | u32::from(buf[0]) << 16))
                }
            }
        }
    }

    impl binrw::BinWrite for u24 {
        type Args<'a> = ();

        fn write_options<W: std::io::Write + std::io::Seek>(
            &self,
            writer: &mut W,
            endian: binrw::Endian,
            args: Self::Args<'_>,
        ) -> binrw::BinResult<()> {
            let mut buf = [0; 3];
            match endian {
                binrw::Endian::Little => {
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
            buf.write_options(writer, endian, args)
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
        binrw::Endian::Little,
        (),
    )
    .unwrap();
    assert_eq!(buf, le_data);
    buf.clear();
    num.write_options(&mut std::io::Cursor::new(&mut buf), binrw::Endian::Big, ())
        .unwrap();
    assert_eq!(buf, be_data);
    buf.clear();
    let mut reader = std::io::Cursor::new(le_data);
    let num = u24::read_options(&mut reader, Endian::Little, ()).unwrap();
    assert_eq!(num.0, 8388608);
    reader = std::io::Cursor::new(be_data);
    let num = u24::read_options(&mut reader, Endian::Big, ()).unwrap();
    assert_eq!(num.0, 8388608);
}
