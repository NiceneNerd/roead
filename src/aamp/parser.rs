use core::cell::UnsafeCell;

use binrw::{
    io::{Cursor, Read, Seek},
    prelude::*,
};

use super::*;
use crate::{util::SeekShim, Error, Result};

impl ParameterIO {
    /// Read a parameter archive from a binary reader.
    pub fn read<R: Read + Seek>(reader: R) -> Result<ParameterIO> {
        Parser::new(reader)?.parse()
    }

    /// Load a parameter archive from binary data.
    ///
    /// **Note**: If and only if the `yaz0` feature is enabled, this function
    /// automatically decompresses the data when necessary.
    pub fn from_binary(data: impl AsRef<[u8]>) -> Result<ParameterIO> {
        #[cfg(feature = "yaz0")]
        {
            if data.as_ref().starts_with(b"Yaz0") {
                return Parser::new(binrw::io::Cursor::new(crate::yaz0::decompress(
                    data.as_ref(),
                )?))?
                .parse();
            }
        }
        Parser::new(binrw::io::Cursor::new(data.as_ref()))?.parse()
    }
}

pub(crate) trait ParseParam<'a>: Sized {
    const VARIANT: Type;

    fn parse(parser: &'a Parser<Cursor<&'a [u8]>>, data_offset: u32) -> Result<Self>;
}

impl ParseParam<'_> for bool {
    const VARIANT: Type = Type::Bool;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read::<u32>().map(|v| v != 0)
    }
}

impl ParseParam<'_> for f32 {
    const VARIANT: Type = Type::F32;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read::<f32>()
    }
}

impl ParseParam<'_> for i32 {
    const VARIANT: Type = Type::Int;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl ParseParam<'_> for Vector2f {
    const VARIANT: Type = Type::Vec2;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for Vector3f {
    const VARIANT: Type = Type::Vec3;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for Vector4f {
    const VARIANT: Type = Type::Vec4;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for Quat {
    const VARIANT: Type = Type::Quat;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for Color {
    const VARIANT: Type = Type::Color;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl ParseParam<'_> for u32 {
    const VARIANT: Type = Type::U32;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl ParseParam<'_> for [Curve; 1] {
    const VARIANT: Type = Type::Curve1;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for [Curve; 2] {
    const VARIANT: Type = Type::Curve2;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for [Curve; 3] {
    const VARIANT: Type = Type::Curve3;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for [Curve; 4] {
    const VARIANT: Type = Type::Curve4;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl ParseParam<'_> for FixedSafeString<32> {
    const VARIANT: Type = Type::String32;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for FixedSafeString<64> {
    const VARIANT: Type = Type::String64;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for FixedSafeString<256> {
    const VARIANT: Type = Type::String256;

    fn parse(parser: &Parser<Cursor<&'_ [u8]>>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl<'a> ParseParam<'a> for &'a str {
    const VARIANT: Type = Type::StringRef;

    fn parse(parser: &'a Parser<Cursor<&'a [u8]>>, data_offset: u32) -> Result<Self> {
        let data_offset = data_offset as usize;
        let buf = *parser.reader().get_ref();
        let len = buf[data_offset..].iter().position(|b| *b == 0);
        len.ok_or(Error::InvalidData("Null string missing terminator"))
            .and_then(move |len| Ok(std::str::from_utf8(&buf[data_offset..data_offset + len])?))
    }
}

impl<'a> ParseParam<'a> for &'a [u8] {
    const VARIANT: Type = Type::BufferBinary;

    fn parse(parser: &'a Parser<Cursor<&'a [u8]>>, data_offset: u32) -> Result<Self> {
        let buf = *parser.reader().get_ref();
        let size = parser.read_at::<u32>(data_offset - 4)? as usize;
        dbg!(data_offset, size);
        Ok(&buf[data_offset as usize..data_offset as usize + size])
    }
}

impl<'a> ParseParam<'a> for &'a [f32] {
    const VARIANT: Type = Type::BufferF32;

    fn parse(parser: &'a Parser<Cursor<&'a [u8]>>, data_offset: u32) -> Result<Self> {
        let buf = *parser.reader().get_ref();
        let size = parser.read_at::<u32>(data_offset - 4)? as usize;
        Ok(unsafe {
            core::mem::transmute::<&[u8], &[f32]>(
                &buf[data_offset as usize..data_offset as usize + size * size_of::<f32>()],
            )
        })
    }
}

impl<'a> ParseParam<'a> for &'a [u32] {
    const VARIANT: Type = Type::BufferU32;

    fn parse(parser: &'a Parser<Cursor<&'a [u8]>>, data_offset: u32) -> Result<Self> {
        let buf = *parser.reader().get_ref();
        let size = parser.read_at::<u32>(data_offset - 4)? as usize;
        Ok(unsafe {
            core::mem::transmute::<&[u8], &[u32]>(
                &buf[data_offset as usize..data_offset as usize + size * size_of::<u32>()],
            )
        })
    }
}

impl<'a> ParseParam<'a> for &'a [i32] {
    const VARIANT: Type = Type::BufferInt;

    fn parse(parser: &'a Parser<Cursor<&'a [u8]>>, data_offset: u32) -> Result<Self> {
        let buf = *parser.reader().get_ref();
        let size = parser.read_at::<u32>(data_offset - 4)? as usize;
        Ok(unsafe {
            core::mem::transmute::<&[u8], &[i32]>(
                &buf[data_offset as usize..data_offset as usize + size * size_of::<i32>()],
            )
        })
    }
}

pub(super) struct Parser<R: Read + Seek> {
    reader: UnsafeCell<R>,
    pub(super) header: ResHeader,
    endian: binrw::Endian,
}

impl<R> Clone for Parser<R>
where
    R: Read + Seek + Clone,
{
    fn clone(&self) -> Self {
        Self {
            reader: UnsafeCell::new(
                unsafe { self.reader.get().as_ref().unwrap_unchecked() }.clone(),
            ),
            header: self.header,
            endian: self.endian,
        }
    }
}

impl Parser<Cursor<&'_ [u8]>> {
    pub(super) fn buffer(&self) -> &[u8] {
        self.reader().get_ref()
    }
}

impl<R: Read + Seek> Parser<R> {
    pub(super) fn new(mut reader: R) -> Result<Self> {
        if SeekShim::stream_len(&mut reader)? < 0x30 {
            return Err(Error::InvalidData("Incomplete parameter archive"));
        }
        let header = ResHeader::read(&mut reader)?;
        if header.version != 2 {
            return Err(Error::InvalidData(
                "Only version 2 parameter archives are supported",
            ));
        }
        if header.flags & 1 << 0 != 1 << 0 {
            return Err(Error::InvalidData(
                "Only little endian parameter archives are supported",
            ));
        }
        if header.flags & 1 << 1 != 1 << 1 {
            return Err(Error::InvalidData(
                "Only UTF-8 parameter archives are supported",
            ));
        }
        Ok(Self {
            reader: UnsafeCell::new(reader),
            header,
            endian: binrw::Endian::Little,
        })
    }

    #[allow(clippy::mut_from_ref)]
    fn reader(&self) -> &mut R {
        unsafe { self.reader.get().as_mut().unwrap_unchecked() }
    }

    fn parse(&mut self) -> Result<ParameterIO> {
        let (root_name, param_root) = self.parse_list(self.header.pio_offset + 0x30)?;
        if root_name != ROOT_KEY {
            Err(Error::InvalidData(
                "No param root found in parameter archive",
            ))
        } else {
            Ok(ParameterIO {
                version: self.header.pio_version,
                data_type: {
                    self.seek(0x30)?;
                    self.read_null_string()?
                },
                param_root,
            })
        }
    }

    #[inline]
    fn seek(&self, offset: u32) -> Result<()> {
        self.reader()
            .seek(binrw::io::SeekFrom::Start(offset as u64))?;
        Ok(())
    }

    #[inline]
    fn read<'a, T: BinRead<Args<'a> = ()>>(&self) -> Result<T> {
        Ok(self.reader().read_le()?)
    }

    #[inline]
    fn read_null_string(&self) -> Result<String> {
        let mut string_ = [0u8; 0x256];
        let mut c: u8 = self.read()?;
        let mut len = 0;
        while c != 0 {
            string_[len] = c;
            len += 1;
            c = self.read()?;
        }
        Ok(std::str::from_utf8(&string_[..len])
            .map(|s| s.into())
            .unwrap_or_else(|_| std::string::String::from_utf8_lossy(&string_[..len]).into()))
    }

    pub(super) fn read_at<'a, T: BinRead<Args<'a> = ()>>(&self, offset: u32) -> Result<T> {
        let old_pos = self.reader().stream_position()? as u32;
        self.seek(offset)?;
        let val = self.read()?;
        self.seek(old_pos)?;
        Ok(val)
    }

    fn read_buffer<T>(&self, offset: u32) -> Result<Vec<T>>
    where
        T: for<'a> BinRead<Args<'a> = ()> + Clone + 'static,
    {
        let size = self.read_at::<u32>(offset - 4)?;
        let buf = binrw::BinRead::read_options(
            self.reader(),
            self.endian,
            binrw::VecArgs::builder().count(size as usize).finalize(),
        )?;
        Ok(buf)
    }

    #[inline]
    fn read_float_buffer(&self, offset: u32) -> Result<Vec<f32>> {
        let size = self.read_at::<u32>(offset - 4)?;
        let mut buf = Vec::<f32>::with_capacity(size as usize);
        for _ in 0..size {
            buf.push(self.read()?);
        }
        Ok(buf)
    }

    fn parse_parameter(&self, offset: u32) -> Result<(Name, Parameter)> {
        self.seek(offset)?;
        let info: ResParameter = self.read()?;
        let data_offset = info.data_rel_offset.as_u32() * 4 + offset;
        self.seek(data_offset)?;
        let value = match info.type_ {
            Type::Bool => Parameter::Bool(self.read::<u32>()? != 0),
            Type::F32 => Parameter::F32(self.read::<f32>()?),
            Type::Int => Parameter::I32(self.read()?),
            Type::Vec2 => Parameter::Vec2(self.read()?),
            Type::Vec3 => Parameter::Vec3(self.read()?),
            Type::Vec4 => Parameter::Vec4(self.read()?),
            Type::Quat => Parameter::Quat(self.read()?),
            Type::Color => Parameter::Color(self.read()?),
            Type::U32 => Parameter::U32(self.read()?),
            Type::Curve1 => Parameter::Curve1(self.read()?),
            Type::Curve2 => Parameter::Curve2(self.read()?),
            Type::Curve3 => Parameter::Curve3(self.read()?),
            Type::Curve4 => Parameter::Curve4(self.read()?),
            Type::String32 => Parameter::String32(self.read()?),
            Type::String64 => Parameter::String64(self.read()?),
            Type::String256 => Parameter::String256(self.read()?),
            Type::StringRef => Parameter::StringRef(self.read_null_string()?),
            Type::BufferInt => Parameter::BufferInt(self.read_buffer::<i32>(data_offset)?),
            Type::BufferU32 => Parameter::BufferU32(self.read_buffer::<u32>(data_offset)?),
            Type::BufferF32 => Parameter::BufferF32(self.read_float_buffer(offset)?),
            Type::BufferBinary => Parameter::BufferBinary(self.read_buffer::<u8>(data_offset)?),
        };
        Ok((info.name, value))
    }

    fn parse_object(&self, offset: u32) -> Result<(Name, ParameterObject)> {
        self.seek(offset)?;
        let info: ResParameterObj = self.read()?;
        let offset = info.params_rel_offset as u32 * 4 + offset;
        let params = (0..info.param_count)
            .map(|i| self.parse_parameter(offset + 0x8 * i as u32))
            .collect::<Result<_>>()?;
        Ok((info.name, params))
    }

    fn parse_list(&self, offset: u32) -> Result<(Name, ParameterList)> {
        self.seek(offset)?;
        let info: ResParameterList = self.read()?;
        let lists_offset = info.lists_rel_offset as u32 * 4 + offset;
        let objects_offset = info.objects_rel_offset as u32 * 4 + offset;
        let plist = ParameterList {
            lists:   (0..info.list_count)
                .map(|i| self.parse_list(lists_offset + 0xC * i as u32))
                .collect::<Result<_>>()?,
            objects: (0..info.object_count)
                .map(|i| self.parse_object(objects_offset + 0x8 * i as u32))
                .collect::<Result<_>>()?,
        };
        Ok((info.name, plist))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse() {
        for file in jwalk::WalkDir::new("test/aamp")
            .into_iter()
            .filter_map(|f| {
                f.ok().and_then(|f| {
                    (f.file_type().is_file() && !f.file_name().to_str().unwrap().ends_with("yml"))
                        .then(|| f.path())
                })
            })
        {
            println!("{}", file.display());
            let data = std::fs::read(&file).unwrap();
            ParameterIO::from_binary(data).unwrap();
        }
    }
}
