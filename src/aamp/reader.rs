use std::cell::RefCell;

use binrw::{io::*, BinRead};
use parser::Parser;

use super::*;
use crate::Result;

pub struct ParameterIOReader<R: Read + Seek> {
    inner: RefCell<Parser<R>>,
}

impl<R: Read + Seek> ParameterIOReader<R> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            inner: RefCell::new(Parser::new(reader)?),
        })
    }

    fn parser(&self) -> std::cell::Ref<'_, Parser<R>> {
        self.inner.borrow()
    }

    fn parser_mut(&self) -> std::cell::RefMut<'_, Parser<R>> {
        self.inner.borrow_mut()
    }
}

struct ParameterListsReader<R: Read + Seek> {
    io_reader: ParameterIOReader<R>,
}

struct ParameterObjectsReader<R: Read + Seek> {
    io_reader: ParameterIOReader<R>,
}

struct ParameterListReader<R: Read + Seek> {
    io_reader: ParameterIOReader<R>,
}

struct ParameterObjectReader<R: Read + Seek> {
    io_reader: ParameterIOReader<R>,
    obj_header: ResParameterObj,
    offset: u32,
}

impl<R: Read + Seek> ParameterObjectReader<R> {
    pub fn len(&self) -> usize {
        self.obj_header.param_count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<T: Into<Parameter> + BinRead<Args<'static> = ()>>(
        &mut self,
        name: impl Into<Name>,
    ) -> Option<T> {
        self.try_get(name).ok().flatten()
    }

    pub fn get_at<T: Into<Parameter> + BinRead<Args<'static> = ()>>(
        &mut self,
        index: usize,
    ) -> Option<T> {
        self.try_get_at(index).ok().flatten()
    }

    pub fn try_get<T: Into<Parameter> + BinRead<Args<'static> = ()>>(
        &mut self,
        name: impl Into<Name>,
    ) -> Result<Option<T>> {
        let name = name.into();
        let offset = self.obj_header.params_rel_offset as u32 * 4 + self.offset;
        for i in 0..self.obj_header.param_count {
            let param_offset = offset + 0x8 * i as u32;
            let param: ResParameter = self.io_reader.parser_mut().read_at(param_offset)?;
            if param.name != name {
                continue;
            }
            let data_offset = param.data_rel_offset.as_u32() * 4 + offset;
            return self.io_reader.parser_mut().read_at(data_offset).map(Some);
        }
        Ok(None)
    }

    pub fn try_get_at<T: Into<Parameter> + BinRead<Args<'static> = ()>>(
        &mut self,
        index: usize,
    ) -> Result<Option<T>> {
        if index >= self.obj_header.param_count as usize {
            return Ok(None);
        }
        let offset = self.obj_header.params_rel_offset as u32 * 4 + self.offset;
        let param_offset = offset + 0x8 * index as u32;
        let param: ResParameter = self.io_reader.parser_mut().read_at(param_offset)?;
        let data_offset = param.data_rel_offset.as_u32() * 4 + offset;
        self.io_reader.parser_mut().read_at(data_offset).map(Some)
    }
}
