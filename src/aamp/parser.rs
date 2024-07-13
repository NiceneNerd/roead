use binrw::{
    io::{Read, Seek},
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

trait ParseParam<'a>: Sized {
    const VARIANT: Type;

    fn parse_value<R: Read + Seek>(parser: &'a mut Parser<R>, data_offset: u32) -> Result<Self>;

    fn parse<R: Read + Seek>(parser: &'a mut Parser<R>, offset: u32) -> Result<Self> {
        parser.seek(offset)?;
        let info: ResParameter = parser.read()?;
        if info.type_ != Self::VARIANT {
            Err(Error::TypeError(
                info.type_.name().into(),
                Self::VARIANT.name(),
            ))
        } else {
            let data_offset = info.data_rel_offset.as_u32() * 4 + offset;
            parser.seek(data_offset)?;
            Self::parse_value(parser, data_offset)
        }
    }
}

impl ParseParam<'_> for bool {
    const VARIANT: Type = Type::Bool;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read::<u32>().map(|v| v != 0)
    }
}

impl ParseParam<'_> for f32 {
    const VARIANT: Type = Type::F32;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read::<f32>()
    }
}

impl ParseParam<'_> for i32 {
    const VARIANT: Type = Type::Int;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl ParseParam<'_> for Vector2f {
    const VARIANT: Type = Type::Vec2;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for Vector3f {
    const VARIANT: Type = Type::Vec3;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for Vector4f {
    const VARIANT: Type = Type::Vec4;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for Quat {
    const VARIANT: Type = Type::Quat;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for Color {
    const VARIANT: Type = Type::Color;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl ParseParam<'_> for u32 {
    const VARIANT: Type = Type::U32;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl ParseParam<'_> for [Curve; 1] {
    const VARIANT: Type = Type::Curve1;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for [Curve; 2] {
    const VARIANT: Type = Type::Curve2;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for [Curve; 3] {
    const VARIANT: Type = Type::Curve3;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for [Curve; 4] {
    const VARIANT: Type = Type::Curve4;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl ParseParam<'_> for FixedSafeString<32> {
    const VARIANT: Type = Type::String32;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for FixedSafeString<64> {
    const VARIANT: Type = Type::String64;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}
impl ParseParam<'_> for FixedSafeString<256> {
    const VARIANT: Type = Type::String256;

    fn parse_value<R: Read + Seek>(parser: &mut Parser<R>, _data_offset: u32) -> Result<Self> {
        parser.read()
    }
}

impl<'a> ParseParam<'a> for &'a str {
    const VARIANT: Type = Type::StringRef;

    fn parse_value<R: Read + Seek>(parser: &'a mut Parser<R>, _data_offset: u32) -> Result<Self> {
        let mut c: u8 = parser.read()?;
        let mut len = 0;
        while c != 0 {
            parser.string_buf[len] = c;
            len += 1;
            c = parser.read()?;
        }
        Ok(unsafe { std::str::from_utf8_unchecked(&parser.string_buf[..len]) })
    }
}

pub(super) struct Parser<R: Read + Seek> {
    reader: R,
    header: ResHeader,
    endian: binrw::Endian,
    string_buf: [u8; 4096],
}

impl<R> Copy for Parser<R> where R: Read + Seek + Copy {}
impl<R> Clone for Parser<R>
where
    R: Read + Seek + Clone,
{
    fn clone(&self) -> Self {
        Self {
            reader: self.reader.clone(),
            header: self.header,
            endian: self.endian,
            string_buf: self.string_buf,
        }
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
            reader,
            header,
            endian: binrw::Endian::Little,
            string_buf: [0; 4096],
        })
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
    fn seek(&mut self, offset: u32) -> Result<()> {
        self.reader
            .seek(binrw::io::SeekFrom::Start(offset as u64))?;
        Ok(())
    }

    #[inline]
    fn read<'a, T: BinRead<Args<'a> = ()>>(&mut self) -> Result<T> {
        Ok(self.reader.read_le()?)
    }

    #[inline]
    fn read_null_string(&mut self) -> Result<String> {
        let mut c: u8 = self.read()?;
        let mut len = 0;
        while c != 0 {
            self.string_buf[len] = c;
            len += 1;
            c = self.read()?;
        }
        Ok(unsafe { std::str::from_utf8_unchecked(&self.string_buf[..len]) }.into())
    }

    pub(super) fn read_at<'a, T: BinRead<Args<'a> = ()>>(&mut self, offset: u32) -> Result<T> {
        let old_pos = self.reader.stream_position()? as u32;
        self.seek(offset)?;
        let val = self.read()?;
        self.seek(old_pos)?;
        Ok(val)
    }

    fn read_buffer<T>(&mut self, offset: u32) -> Result<Vec<T>>
    where
        T: for<'a> BinRead<Args<'a> = ()> + Clone + 'static,
    {
        let size = self.read_at::<u32>(offset - 4)?;
        let buf = binrw::BinRead::read_options(
            &mut self.reader,
            self.endian,
            binrw::VecArgs::builder().count(size as usize).finalize(),
        )?;
        Ok(buf)
    }

    #[inline]
    fn read_float_buffer(&mut self, offset: u32) -> Result<Vec<f32>> {
        let size = self.read_at::<u32>(offset - 4)?;
        let mut buf = Vec::<f32>::with_capacity(size as usize);
        for _ in 0..size {
            buf.push(self.read()?);
        }
        Ok(buf)
    }

    pub(super) fn read_parameter<'a, T: Into<Parameter> + BinRead<Args<'a> = ()>>(
        &mut self,
        offset: u32,
    ) -> Result<(Name, T)> {
        self.seek(offset)?;
        let info: ResParameter = self.read()?;
        let data_offset = info.data_rel_offset.as_u32() * 4 + offset;
        self.seek(data_offset)?;
        let value = self.read()?;
        Ok((info.name, value))
    }

    fn parse_parameter(&mut self, offset: u32) -> Result<(Name, Parameter)> {
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

    fn parse_object(&mut self, offset: u32) -> Result<(Name, ParameterObject)> {
        self.seek(offset)?;
        let info: ResParameterObj = self.read()?;
        let offset = info.params_rel_offset as u32 * 4 + offset;
        let params = (0..info.param_count)
            .map(|i| self.parse_parameter(offset + 0x8 * i as u32))
            .collect::<Result<_>>()?;
        Ok((info.name, params))
    }

    fn parse_list(&mut self, offset: u32) -> Result<(Name, ParameterList)> {
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
