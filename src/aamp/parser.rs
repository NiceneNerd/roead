use super::*;
use crate::util::u24;
use binrw::{binrw, prelude::*, NullString};
use enumflags2::{bitflags, BitFlags};
use std::io::{Read, Seek};

#[binrw]
#[brw(repr(u32), little)]
#[bitflags]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum HeaderFlag {
    LittleEndian = 1 << 0,
    Utf8 = 1 << 1,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResHeader {
    magic: [u8; 4],
    version: u32,
    flags: HeaderFlag,
    file_size: u32,
    pio_version: u32,
    /// Offset to parameter IO (relative to 0x30)
    pio_offset: u32,
    /// Number of lists (including root)
    list_count: u32,
    object_count: u32,
    param_count: u32,
    data_section_size: u32,
    string_section_size: u32,
    unknown_section_size: u32,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameter {
    name: Name,
    data_rel_offset: u24,
    type_: Type,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameterObj {
    name: Name,
    params_rel_offset: u16,
    param_count: u16,
}

#[derive(Debug)]
#[binrw]
#[brw(little)]
struct ResParameterList {
    name: Name,
    lists_rel_offset: u16,
    list_count: u16,
    objects_rel_offset: u16,
    object_count: u16,
}

struct Parser<R: Read + Seek> {
    reader: R,
    opts: binrw::ReadOptions,
}

impl<R: Read + Seek> Parser<R> {
    fn new(mut reader: R) -> Result<Self, AampError> {
        if reader.stream_len()? < 0x30 {
            return Err(AampError::InvalidData("Incomplete parameter archive"));
        }
        let header = ResHeader::read(&mut reader)?;
        if &header.magic != b"AAMP" {
            return Err(AampError::InvalidData("Invalid magic"));
        }
        if header.version != 2 {
            return Err(AampError::InvalidData(
                "Only version 2 parameter archives are supported",
            ));
        }
        let flags = BitFlags::from_flag(header.flags);
        if !flags.contains(HeaderFlag::LittleEndian) {
            return Err(AampError::InvalidData(
                "Only little endian parameter archives are supported",
            ));
        }
        if !flags.contains(HeaderFlag::Utf8) {
            return Err(AampError::InvalidData(
                "Only UTF-8 parameter archives are supported",
            ));
        }
        Ok(Self {
            reader,
            opts: binrw::ReadOptions::default().with_endian(binrw::Endian::Little),
        })
    }

    #[inline]
    fn seek(&mut self, offset: u32) -> Result<(), AampError> {
        self.reader.seek(std::io::SeekFrom::Start(offset as u64))?;
        Ok(())
    }

    #[inline]
    fn read<T: BinRead<Args = ()>>(&mut self) -> Result<T, AampError> {
        Ok(T::read_options(&mut self.reader, &self.opts, ())?)
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
        let info = ResParameter::read(&mut self.reader)?;
        let data_offset = info.data_rel_offset.as_u32() + offset;
        self.seek(data_offset)?;
        let value = match info.type_ {
            Type::Bool => Parameter::Bool(self.read::<u32>()? != 0),
            Type::F32 => Parameter::F32(self.read_float()?),
            Type::Int => Parameter::Int(self.read()?),
            Type::Vec2 => Parameter::Vec2(Vector2f {
                x: self.read_float()?,
                y: self.read_float()?,
            }),
            Type::Vec3 => Parameter::Vec3(Vector3f {
                x: self.read_float()?,
                y: self.read_float()?,
                z: self.read_float()?,
            }),
            Type::Vec4 => Parameter::Vec4(Vector4f {
                x: self.read_float()?,
                y: self.read_float()?,
                z: self.read_float()?,
                t: self.read_float()?,
            }),
            Type::Quat => Parameter::Quat(Quat {
                a: self.read_float()?,
                b: self.read_float()?,
                c: self.read_float()?,
                d: self.read_float()?,
            }),
            Type::Color => Parameter::Color(Color {
                r: self.read_float()?,
                g: self.read_float()?,
                b: self.read_float()?,
                a: self.read_float()?,
            }),
            Type::U32 => Parameter::U32(self.read()?),
            Type::Curve1 => Parameter::Curve1([self.read_curve()?]),
            Type::Curve2 => Parameter::Curve2([self.read_curve()?, self.read_curve()?]),
            Type::Curve3 => {
                Parameter::Curve3([self.read_curve()?, self.read_curve()?, self.read_curve()?])
            }
            Type::Curve4 => Parameter::Curve4([
                self.read_curve()?,
                self.read_curve()?,
                self.read_curve()?,
                self.read_curve()?,
            ]),
            Type::String32 => Parameter::String32(self.read()?),
            Type::String64 => Parameter::String64(self.read()?),
            Type::String256 => Parameter::String256(self.read()?),
            Type::StringRef => Parameter::StringRef(
                std::str::from_utf8(self.read::<NullString>()?.as_slice())?.into(),
            ),
            Type::BufferInt => Parameter::BufferInt(self.read_buffer::<i32>(data_offset)?),
            Type::BufferU32 => Parameter::BufferU32(self.read_buffer::<u32>(data_offset)?),
            Type::BufferF32 => Parameter::BufferF32(self.read_float_buffer(offset)?),
            Type::BufferBinary => Parameter::BufferBinary(self.read_buffer::<u8>(data_offset)?),
        };
        Ok((info.name, value))
    }

    fn parse_object(&mut self, offset: u32) -> Result<(Name, ParameterObject), AampError> {
        self.seek(offset)?;
        let info = ResParameterObj::read(&mut self.reader)?;
        let offset = info.params_rel_offset as u32 + offset;
        let mut params = ParameterObject(
            (0..=info.param_count)
                .into_iter()
                .map(|i| {
                    Ok((
                        info.name,
                        self.parse_parameter(offset as u32 + 0x8 * i as u32)?,
                    ))
                })
                .collect::<Result<_, AampError>>()?,
        );
        Ok((info.name, params))
    }
}
