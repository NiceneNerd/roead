use binrw::io::{Cursor, Read, Seek, SeekFrom};
use binrw::prelude::*;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;

use super::*;
use crate::{
    Error, Result,
    util::{SeekShim, u24},
};

#[derive(Debug)]
#[cfg_attr(feature = "binrw", binrw::binrw, brw(little, magic = b"AAMP"))]
pub(crate) struct ResHeader {
    pub version: u32,     // 0x4
    pub flags: u32,       // 0x8
    pub file_size: u32,   // 0xC
    pub pio_version: u32, // 0x10
    /// Offset to parameter IO (relative to 0x30)
    pub pio_offset: u32, // 0x14
    /// Number of lists (including root)
    pub list_count: u32, // 0x18
    pub object_count: u32, // 0x1C
    pub param_count: u32, // 0x20
    pub data_section_size: u32, // 0x24
    pub string_section_size: u32, // 0x28
    pub unknown_section_size: u32, // 0x2C
}

#[derive(Debug)]
#[cfg_attr(feature = "binrw", binrw::binrw, brw(little))]
pub(crate) struct ResParameter {
    pub name: Name,
    pub data_rel_offset: u24,
    pub type_: Type,
}

#[derive(Debug)]
#[cfg_attr(feature = "binrw", binrw::binrw, brw(little))]
pub(crate) struct ResParameterObj {
    pub name: Name,
    pub params_rel_offset: u16,
    pub param_count: u16,
}

#[derive(Debug)]
#[cfg_attr(feature = "binrw", binrw::binrw, brw(little))]
pub(crate) struct ResParameterList {
    pub name: Name,
    pub lists_rel_offset: u16,
    pub list_count: u16,
    pub objects_rel_offset: u16,
    pub object_count: u16,
}

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
                return Parser::new(Cursor::new(crate::yaz0::decompress(data.as_ref())?))?.parse();
            }
        }
        Parser::new(Cursor::new(data.as_ref()))?.parse()
    }
}

struct Parser<R: Read + Seek> {
    reader: R,
    header: ResHeader,
    endian: binrw::Endian,
}

impl<R: Read + Seek> Parser<R> {
    fn new(mut reader: R) -> Result<Self> {
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
                    self.read_null_string()?.into()
                },
                param_root,
            })
        }
    }

    #[inline]
    fn seek(&mut self, offset: u32) -> binrw::BinResult<()> {
        self.reader.seek(SeekFrom::Start(offset as u64))?;
        Ok(())
    }

    #[inline]
    fn read<'a, T: BinRead<Args<'a> = ()>>(&mut self) -> Result<T> {
        Ok(binrw::BinReaderExt::read_le(&mut self.reader)?)
    }

    /// Reads a NUL-terminated string with checked UTF-8 validation.
    ///
    /// Corpus files are valid UTF-8, so this only changes behavior for
    /// invalid input, which previously constructed a `&str` from unchecked
    /// bytes (undefined behavior).
    #[inline]
    fn read_null_string(&mut self) -> Result<String> {
        let mut string_ = [0u8; 0x256];
        let mut c: u8 = self.read()?;
        let mut len = 0;
        while c != 0 {
            if len >= string_.len() {
                return Err(Error::InvalidData("String exceeds the 0x256 byte limit"));
            }
            string_[len] = c;
            len += 1;
            c = self.read()?;
        }
        Ok(core::str::from_utf8(&string_[..len])?.into())
    }

    #[inline]
    fn read_at<'a, T: BinRead<Args<'a> = ()>>(&mut self, offset: u32) -> Result<T> {
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
            Type::StringRef => Parameter::StringRef(self.read_null_string()?.into()),
            // The element count is stored at `data_offset - 4`, with the
            // floats at `data_offset` — pass `data_offset`, not `offset`.
            Type::BufferInt => Parameter::BufferInt(self.read_buffer::<i32>(data_offset)?),
            Type::BufferU32 => Parameter::BufferU32(self.read_buffer::<u32>(data_offset)?),
            Type::BufferF32 => Parameter::BufferF32(self.read_float_buffer(data_offset)?),
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
            lists: (0..info.list_count)
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
