use super::*;
use crate::{util::align, Result};
use binrw::prelude::*;
use rustc_hash::FxHashMap;
use std::{
    cell::RefCell,
    collections::hash_map::Entry,
    hash::Hasher,
    io::{Cursor, Seek, SeekFrom, Write},
    rc::Rc,
    sync::Mutex,
};

impl ParameterIO {
    /// Serialize the parameter IO to binary using the given writer.
    pub fn write<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut ctx = WriteContext {
            writer,
            list_count: Default::default(),
            object_count: Default::default(),
            param_count: Default::default(),
            param_queue: Default::default(),
            string_param_queue: Default::default(),
            offsets: Default::default(),
            string_offsets: Default::default(),
            buffer_offsets: Default::default(),
        };
        ctx.writer.seek(SeekFrom::Start(0x30))?;
        ctx.writer.write_le(&self.data_type.as_bytes())?;
        ctx.writer.write_le(&0u8)?;
        ctx.align()?;
        let pio_offset = ctx.writer.stream_position()?;
        let root = &self.param_root;

        ctx.write_lists(self)?;
        ctx.write_objects(root)?;
        ctx.collect_parameters(self);
        ctx.write_parameters(root)?;

        let data_section_begin = ctx.writer.stream_position()?;
        ctx.write_data_section()?;

        let string_section_begin = ctx.writer.stream_position()?;
        ctx.write_string_section()?;

        let unknown_section_begin = ctx.writer.stream_position()?;
        ctx.align()?;

        let header = ResHeader {
            version: 2,
            flags: 3,
            file_size: ctx.writer.stream_position()? as u32,
            pio_version: self.version,
            pio_offset: (pio_offset - 0x30) as u32,
            list_count: ctx.list_count,
            object_count: ctx.object_count,
            param_count: ctx.param_count,
            data_section_size: (string_section_begin - data_section_begin) as u32,
            string_section_size: (unknown_section_begin - string_section_begin) as u32,
            unknown_section_size: 0,
        };
        ctx.writer.seek(SeekFrom::Start(0))?;
        ctx.writer.write_le(&header)?;
        ctx.writer.flush()?;
        Ok(())
    }

    /// Serialize the parameter IO to in-memory bytes.
    pub fn to_binary(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.write(Cursor::new(&mut buf)).unwrap();
        buf
    }
}

#[inline]
fn write_buffer<W: Write + Seek, T: BinWrite<Args = ()>>(
    writer: &mut W,
    buffer: &[T],
) -> BinResult<()> {
    writer.write_le(&(buffer.len() as u32))?;
    writer.write_le(&buffer)?;
    Ok(())
}

#[inline]
fn hash_param_data(param: &Parameter) -> u64 {
    let mut hasher = rustc_hash::FxHasher::default();
    std::hash::Hash::hash(param, &mut hasher);
    hasher.finish()
}

struct WriteContext<'pio, W: Write + Seek> {
    writer: W,
    list_count: u32,
    object_count: u32,
    param_count: u32,
    param_queue: Vec<&'pio Parameter>,
    string_param_queue: Vec<&'pio Parameter>,
    offsets: FxHashMap<usize, u32>,
    string_offsets: FxHashMap<&'pio str, u32>,
    buffer_offsets: FxHashMap<u64, u32>,
}

impl<'pio, W: Write + Seek> WriteContext<'pio, W> {
    #[inline(always)]
    fn get_offset<T: std::fmt::Debug>(&mut self, data: &T) -> u32 {
        self.offsets[&(data as *const _ as usize)]
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

    fn write_lists(&mut self, pio: &'pio ParameterIO) -> BinResult<()> {
        fn write<W: Write + Seek>(
            ctx: &mut WriteContext<W>,
            list: &ParameterList,
        ) -> BinResult<()> {
            ctx.write_offset_for_parent(list, 0x4)?;
            for (name, list) in &list.lists.0 {
                ctx.write_list(*name, list)?;
            }
            for list in list.lists.0.values() {
                write(ctx, list)?;
            }
            Ok(())
        }
        self.write_list(ROOT_KEY, &pio.param_root)?;
        write(self, &pio.param_root)?;
        Ok(())
    }

    fn write_objects(&mut self, list: &ParameterList) -> BinResult<()> {
        self.write_offset_for_parent(list, 0x8)?;
        for (name, object) in &list.objects.0 {
            self.write_object(*name, object)?;
        }

        for list in list.lists.0.values() {
            self.write_objects(list)?;
        }
        Ok(())
    }

    fn write_parameters(&mut self, list: &ParameterList) -> BinResult<()> {
        for list in list.lists.0.values() {
            self.write_parameters(list)?;
        }

        for object in list.objects.0.values() {
            self.write_offset_for_parent(object, 0x4)?;
            for (name, param) in &object.0 {
                self.write_parameter(*name, param)?;
            }
        }
        Ok(())
    }

    fn collect_parameters(&mut self, pio: &'pio ParameterIO) {
        // For some reason, the order in which parameter data is serialized is
        // not the order of parameter objects or even parameters... Rather, for
        // the majority of binary parameter archives the order is determined
        // with a rather convoluted algorithm:
        //
        // * First, process all of the parameter IO's objects (i.e. add all their
        //   parameters to the parameter queue).
        // * Recursively collect all objects for child lists. For lists, object
        //   processing happens after recursively processing child lists; however every
        //   2 lists one object from the parent list is processed.
        fn do_collect<'ctx, 'pio, W: Write + Seek>(
            ctx: Rc<Mutex<&mut WriteContext<'pio, W>>>,
            list: &'pio ParameterList,
            process_top_objects_first: bool,
        ) where
            'pio: 'ctx,
        {
            let mut obj_iter = list.objects.0.values();
            let object = RefCell::new(obj_iter.next());

            let mut process_one_object = || {
                if let Some(obj) = object.borrow().as_ref() {
                    for param in obj.0.values() {
                        let mut ctx = ctx.lock().unwrap();
                        if param.is_string_type() {
                            ctx.string_param_queue.push(param);
                        } else {
                            ctx.param_queue.push(param);
                        }
                    }
                }
                object.replace(obj_iter.next());
            };

            // If the parameter IO is a Breath of the Wild AIProgram, then it appears that
            // even the parameter IO's objects are processed after child lists.
            // This is likely a hack, but it does match observations...
            let is_botw_aiprog = !list.objects.is_empty()
                && list.objects.0.keys().next() == Some(&Name::from_str("DemoAIActionIdx"));

            if process_top_objects_first && !is_botw_aiprog {
                // Again this is probably a hack but it is required for matching BoneControl
                // documents...
                let mut i = 0;
                while object.borrow().is_some() && i < 7 {
                    process_one_object();
                    i += 1;
                }
            }

            for (i, child_list) in list.lists.0.values().enumerate() {
                if !is_botw_aiprog && i % 2 == 0 && object.borrow().is_some() {
                    process_one_object();
                }
                do_collect(ctx.clone(), child_list, false);
            }

            while object.borrow().is_some() {
                process_one_object();
            }
        }
        do_collect(Rc::new(Mutex::new(self)), &pio.param_root, true)
    }

    fn write_data_section(&mut self) -> BinResult<()> {
        let queue = std::mem::take(&mut self.param_queue);
        for param in queue {
            self.write_parameter_data(param)?;
        }
        self.align()?;
        Ok(())
    }

    fn write_string_section(&mut self) -> BinResult<()> {
        let queue = std::mem::take(&mut self.string_param_queue);
        for param in queue {
            self.write_string(param)?;
        }
        self.align()?;
        Ok(())
    }

    fn write_parameter_data(&mut self, param: &Parameter) -> BinResult<()> {
        debug_assert!(
            !param.is_string_type(),
            "`write_parameter_data` called with string parameter"
        );

        let parent_offset = self.get_offset(param);
        let mut data_offset =
            self.writer.stream_position()? as u32 + if param.is_buffer_type() { 4 } else { 0 };
        let mut existed = true;

        // We're going to do this very differently from the oead method
        // because we want to support any writer, even one without an
        // accessible underlying buffer. Moreover, by hasing the parameter
        // first we get the chance to skip writing the data even to a temp
        // buffer if it's already been written.
        let hash = hash_param_data(param);
        data_offset = match self.buffer_offsets.entry(hash) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let mut tmp_writer = Cursor::new(Vec::<u8>::with_capacity(0x200));
                match param {
                    Parameter::Bool(b) => tmp_writer.write_le(&if *b { 1u32 } else { 0u32 })?,
                    Parameter::F32(v) => tmp_writer.write_le(&v.to_bits())?,
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
                            tmp_writer.write_le(f)?;
                        }
                    }
                    Parameter::BufferBinary(v) => write_buffer(&mut tmp_writer, v)?,
                    _ => unreachable!("unhandled parameter type"),
                }
                self.writer.write_all(tmp_writer.into_inner().as_slice())?;
                existed = false;
                *entry.insert(data_offset)
            }
        };

        self.write_at(parent_offset + 0x4, u24((data_offset - parent_offset) / 4))?;
        if !existed {
            self.align()?;
        }
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

    fn write_offset_for_parent<T: std::fmt::Debug>(
        &mut self,
        parent: &T,
        offset_in_parent: u32,
    ) -> BinResult<()> {
        let parent_offset = self.get_offset(parent);
        let current_rel_offset = (self.writer.stream_position()? as u32 - parent_offset) / 4;
        self.write_at(parent_offset + offset_in_parent, current_rel_offset as u16)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_roundtrip() {
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
            let pio = ParameterIO::from_binary(&data).unwrap();
            let new_bytes = pio.to_binary();
            let new_pio = ParameterIO::from_binary(&new_bytes).unwrap();
            assert_eq!(pio, new_pio);
        }
    }
}
