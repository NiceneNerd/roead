use crate::util::align;

use super::*;
use binrw::prelude::*;
use rustc_hash::FxHashMap;
use std::io::{Cursor, Seek, SeekFrom, Write};

fn write_buffer<W: Write + Seek, T: BinWrite<Args = ()>>(
    writer: &mut W,
    buffer: &[T],
) -> BinResult<()> {
    writer.write_le(&(buffer.len() as u32))?;
    writer.write_le(&buffer)?;
    Ok(())
}

struct WriteContext<'pio, W: Write + Seek> {
    writer: W,
    list_count: u32,
    object_count: u32,
    param_count: u32,
    param_queue: Vec<&'pio ResParameter>,
    string_param_queue: Vec<&'pio ResParameter>,
    offsets: FxHashMap<usize, u32>,
    string_offsets: FxHashMap<&'pio str, u32>,
}

impl<'pio, W: Write + Seek> WriteContext<'pio, W> {
    #[inline(always)]
    fn get_offset<T>(&mut self, data: &T) -> u32 {
        self.offsets[&(data as *const T as usize)]
    }

    #[inline(always)]
    fn align(&mut self) -> BinResult<()> {
        let pos = self.writer.stream_position()? as u32;
        let aligned = align(pos, 4);
        self.writer.seek(SeekFrom::Start(aligned as u64))?;
        Ok(())
    }

    #[inline]
    fn write_at<T: BinWrite<Args = ()>>(&mut self, offset: u32, data: T) -> BinResult<()> {
        let old_pos = self.writer.stream_position()?;
        self.writer.seek(SeekFrom::Start(offset as u64))?;
        self.writer.write_le(&data)?;
        self.writer.seek(SeekFrom::Start(old_pos))?;
        Ok(())
    }

    #[inline]
    fn write_current_pos_at(&mut self, offset: u32) -> BinResult<()> {
        let pos = self.writer.stream_position()? as u32;
        self.write_at(offset, pos)?;
        Ok(())
    }

    fn write_parameter_data(
        &mut self,
        param: &Parameter,
        lookup_start_offset: u32,
    ) -> BinResult<()> {
        assert!(
            !matches!(
                param,
                Parameter::String32(_)
                    | Parameter::String64(_)
                    | Parameter::String256(_)
                    | Parameter::StringRef(_)
            ),
            "`write_parameter_data` called with string parameter"
        );

        let mut tmp_writer = Cursor::new([0u8; 0x200]);
        match param {
            Parameter::Bool(b) => tmp_writer.write_le(&if *b { 1u32 } else { 0u32 })?,
            Parameter::F32(v) => tmp_writer.write_le(&(*v.as_ref() as u32))?,
            Parameter::Int(v) => tmp_writer.write_le(&v)?,
            Parameter::Vec2(v) => tmp_writer.write_le(&v)?,
            Parameter::Vec3(v) => tmp_writer.write_le(&v)?,
            Parameter::Vec4(v) => tmp_writer.write_le(&v)?,
            Parameter::Color(v) => tmp_writer.write_le(&v)?,
            Parameter::Curve1(v) => tmp_writer.write_le(&v)?,
            Parameter::Curve2(v) => tmp_writer.write_le(&v)?,
            Parameter::Curve3(v) => tmp_writer.write_le(&v)?,
            Parameter::Curve4(v) => tmp_writer.write_le(&v)?,
            Parameter::Quat(v) => tmp_writer.write_le(&v)?,
            Parameter::U32(v) => tmp_writer.write_le(&v)?,
            Parameter::BufferInt(v) => write_buffer(&mut tmp_writer, v)?,
            Parameter::BufferU32(v) => write_buffer(&mut tmp_writer, v)?,
            Parameter::BufferF32(v) => {
                tmp_writer.write_le(&(v.len() as u32))?;
                for f in v {
                    tmp_writer.write_le(&(*f.as_ref() as u32))?;
                }
            }
            Parameter::BufferBinary(v) => write_buffer(&mut tmp_writer, v)?,
            _ => unreachable!("unhandled parameter type"),
        }

        let parent_offset = self.get_offset(param);
        // let data_offset = self.writer.stream_position()? as u32 + ;
        Ok(())
    }

    fn write_string(&mut self, param: &'pio Parameter) -> BinResult<()> {
        let parent_offset = self.get_offset(param);
        let string_ = param.as_str().unwrap();
        let pos = self.writer.stream_position()? as u32;
        let mut existed = true;
        let offset = *self.string_offsets.entry(string_).or_insert_with(|| {
            existed = false;
            pos
        });
        self.write_at(parent_offset + 0x4, u24((offset - parent_offset) / 4))?;
        if !existed {
            self.writer.write_le(&string_.as_bytes())?;
            self.writer.write_le(&0u8)?;
            self.align()?;
        }
        Ok(())
    }

    fn write_list(&mut self, name: Name, list: &ParameterList) -> BinResult<()> {
        let offset = self.writer.stream_position()? as u32;
        self.offsets.insert(list as *const _ as usize, offset);
        self.list_count += 1;
        self.writer.write_le(&ResParameterList {
            name,
            list_count: list.lists.len() as u16,
            lists_rel_offset: 0,
            object_count: list.objects.len() as u16,
            objects_rel_offset: 0,
        })?;
        Ok(())
    }

    fn write_object(&mut self, name: Name, object: &ParameterObject) -> BinResult<()> {
        let offset = self.writer.stream_position()? as u32;
        self.offsets.insert(object as *const _ as usize, offset);
        self.object_count += 1;
        self.writer.write_le(&ResParameterObj {
            name,
            param_count: object.len() as u16,
            params_rel_offset: 0,
        })?;
        Ok(())
    }

    fn write_parameter(&mut self, name: Name, param: &Parameter) -> BinResult<()> {
        let offset = self.writer.stream_position()? as u32;
        self.offsets.insert(param as *const _ as usize, offset);
        self.param_count += 1;
        self.writer.write_le(&ResParameter {
            name,
            type_: param.get_type(),
            data_rel_offset: u24(0),
        })?;
        Ok(())
    }

    fn write_offset_for_parent<T>(&mut self, parent: &T, offset_in_parent: u32) -> BinResult<()> {
        let parent_offset = self.get_offset(parent);
        let current_rel_offset = (self.writer.stream_position()? as u32 - parent_offset) / 4;
        self.write_at(parent_offset + offset_in_parent, current_rel_offset as u16)?;
        Ok(())
    }
}
