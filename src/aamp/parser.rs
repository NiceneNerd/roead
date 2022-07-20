use super::*;
use binrw::prelude::*;
use std::io::{Read, Seek};

impl ParameterIO {
    /// Read a parameter archive from a binary reader.
    pub fn read<R: Read + Seek>(reader: R) -> crate::Result<ParameterIO> {
        Ok(Parser::new(reader)?.parse()?)
    }

    /// Load a parameter archive from binary data.
    ///
    /// **Note**: If and only if the `yaz0` feature is enabled, this function
    /// automatically decompresses the data when necessary.
    pub fn from_binary(data: impl AsRef<[u8]>) -> crate::Result<ParameterIO> {
        #[cfg(feature = "yaz0")]
        {
            if data.as_ref().starts_with(b"Yaz0") {
                return Ok(Parser::new(std::io::Cursor::new(crate::yaz0::decompress(
                    data.as_ref(),
                )?))?
                .parse()?);
            }
        }
        Ok(Parser::new(std::io::Cursor::new(data.as_ref()))?.parse()?)
    }
}

struct Parser<R: Read + Seek> {
    reader: R,
    header: ResHeader,
    opts: binrw::ReadOptions,
}

impl<R: Read + Seek> Parser<R> {
    fn new(mut reader: R) -> Result<Self, AampError> {
        if reader.stream_len()? < 0x30 {
            return Err(AampError::InvalidData("Incomplete parameter archive"));
        }
        let header = ResHeader::read(&mut reader)?;
        if header.version != 2 {
            return Err(AampError::InvalidData(
                "Only version 2 parameter archives are supported",
            ));
        }
        if header.flags & 1 << 0 != 1 << 0 {
            return Err(AampError::InvalidData(
                "Only little endian parameter archives are supported",
            ));
        }
        if header.flags & 1 << 1 != 1 << 1 {
            return Err(AampError::InvalidData(
                "Only UTF-8 parameter archives are supported",
            ));
        }
        Ok(Self {
            reader,
            header,
            opts: binrw::ReadOptions::default().with_endian(binrw::Endian::Little),
        })
    }

    fn parse(&mut self) -> Result<ParameterIO, AampError> {
        let (root_name, param_root) = self.parse_list(self.header.pio_offset + 0x30)?;
        if root_name != ROOT_KEY {
            Err(AampError::InvalidData(
                "No param root found in parameter archive",
            ))
        } else {
            Ok(ParameterIO {
                version: self.header.version,
                data_type: {
                    self.seek(0x30)?;
                    self.read_null_string()?
                },
                param_root,
            })
        }
    }

    #[inline]
    fn seek(&mut self, offset: u32) -> Result<(), AampError> {
        self.reader.seek(std::io::SeekFrom::Start(offset as u64))?;
        Ok(())
    }

    #[inline]
    fn read<T: BinRead<Args = ()>>(&mut self) -> Result<T, AampError> {
        Ok(self.reader.read_le()?)
    }

    #[inline]
    fn read_null_string(&mut self) -> Result<String, AampError> {
        let mut string_ = String::new_const();
        let mut c: u8 = self.read()?;
        while c != 0 {
            string_.push(c as char);
            c = self.read()?;
        }
        Ok(string_)
    }

    #[inline]
    fn read_at<T: BinRead<Args = ()>>(&mut self, offset: u32) -> Result<T, AampError> {
        let old_pos = self.reader.stream_position()? as u32;
        self.seek(offset)?;
        let val = self.read()?;
        self.seek(old_pos)?;
        Ok(val)
    }

    #[inline]
    fn read_float(&mut self) -> Result<R32, AampError> {
        Ok(self.read::<f32>()?.into())
    }

    fn read_curve(&mut self) -> Result<Curve, AampError> {
        let mut curve = Curve {
            a: self.read()?,
            b: self.read()?,
            ..Default::default()
        };
        for i in 0..30 {
            curve.floats[i] = self.read_float()?;
        }
        Ok(curve)
    }

    fn read_buffer<T: BinRead<Args = ()> + Copy>(
        &mut self,
        offset: u32,
    ) -> Result<Vec<T>, AampError> {
        let size = self.read_at::<u32>(offset - 4)?;
        let buf = Vec::<T>::read_options(
            &mut self.reader,
            &self.opts,
            binrw::VecArgs::builder().count(size as usize).finalize(),
        )?;
        Ok(buf)
    }

    #[inline]
    fn read_float_buffer(&mut self, offset: u32) -> Result<Vec<R32>, AampError> {
        let size = self.read_at::<u32>(offset - 4)?;
        let mut buf = Vec::<R32>::with_capacity(size as usize);
        for _ in 0..size {
            buf.push(self.read_float()?);
        }
        Ok(buf)
    }

    fn parse_parameter(&mut self, offset: u32) -> Result<(Name, Parameter), AampError> {
        self.seek(offset)?;
        let info: ResParameter = self.read()?;
        let data_offset = info.data_rel_offset.as_u32() * 4 + offset;
        self.seek(data_offset)?;
        let value = match info.type_ {
            Type::Bool => Parameter::Bool(self.read::<u32>()? != 0),
            Type::F32 => Parameter::F32(self.read::<f32>()?.into()),
            Type::Int => Parameter::Int(self.read()?),
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

    fn parse_object(&mut self, offset: u32) -> Result<(Name, ParameterObject), AampError> {
        self.seek(offset)?;
        let info: ResParameterObj = self.read()?;
        let offset = info.params_rel_offset as u32 * 4 + offset;
        let params = (0..info.param_count)
            .into_iter()
            .map(|i| self.parse_parameter(offset + 0x8 * i as u32))
            .collect::<Result<_, AampError>>()?;
        Ok((info.name, params))
    }

    fn parse_list(&mut self, offset: u32) -> Result<(Name, ParameterList), AampError> {
        self.seek(offset)?;
        let info: ResParameterList = self.read()?;
        let lists_offset = info.lists_rel_offset as u32 * 4 + offset;
        let objects_offset = info.objects_rel_offset as u32 * 4 + offset;
        let plist = ParameterList {
            lists: (0..info.list_count)
                .into_iter()
                .map(|i| self.parse_list(lists_offset + 0xC * i as u32))
                .collect::<Result<_, AampError>>()?,
            objects: (0..info.object_count)
                .into_iter()
                .map(|i| self.parse_object(objects_offset + 0x8 * i as u32))
                .collect::<Result<_, AampError>>()?,
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
                f.ok()
                    .and_then(|f| f.file_type().is_file().then(|| f.path()))
            })
        {
            println!("{}", file.display());
            let data = std::fs::read(&file).unwrap();
            ParameterIO::from_binary(&data).unwrap();
        }
    }
}
