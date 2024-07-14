use binrw::{io::*, BinRead};
use parser::{ParseParam, Parser};

use super::*;
use crate::Result;

pub struct ParameterIOReader<'a> {
    parser: Parser<Cursor<&'a [u8]>>,
    root: ResParameterList,
    root_offset: u32,
}

impl<'a> ParameterIOReader<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let parser = Parser::new(Cursor::new(data))?;
        let root_offset = parser.header.pio_offset + 0x30;
        let root = parser.read_at(root_offset)?;
        Ok(Self {
            parser,
            root,
            root_offset,
        })
    }

    pub fn root(&'a self) -> ParameterListReader<'a> {
        ParameterListReader::new_with_header(self, self.root, self.root_offset)
    }

    pub fn doc_type(&self) -> Result<&str> {
        let buf = self.parser.buffer();
        let end = buf[0x30..]
            .iter()
            .position(|c| *c == 0)
            .ok_or(crate::Error::InvalidData(
                "Null terminator missing for data type",
            ))?;
        Ok(core::str::from_utf8(&buf[0x30..0x30 + end])?)
    }

    #[allow(clippy::misnamed_getters)]
    pub fn version(&self) -> u32 {
        self.parser.header.pio_version
    }
}

pub struct ParameterListsIterator<'a> {
    pio: &'a ParameterIOReader<'a>,
    list: ResParameterList,
    lists_offset: u32,
    idx: usize,
}

impl<'a> Iterator for ParameterListsIterator<'a> {
    type Item = (Name, ParameterListReader<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.list.list_count as usize {
            return None;
        }
        let next_offset = self.lists_offset + 0x8 * self.idx as u32;
        let list = ParameterListReader::new(self.pio, next_offset).ok()?;
        self.idx += 1;
        Some((list.header.name, list))
    }
}

pub struct ParameterObjectsIterator<'a> {
    pio: &'a ParameterIOReader<'a>,
    list: ResParameterList,
    objs_offset: u32,
    idx: usize,
}

impl<'a> Iterator for ParameterObjectsIterator<'a> {
    type Item = (Name, ParameterObjectReader<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.list.object_count as usize {
            return None;
        }
        let next_offset = self.objs_offset + 0x8 * self.idx as u32;
        let obj = ParameterObjectReader::new(self.pio, next_offset).ok()?;
        self.idx += 1;
        Some((obj.header.name, obj))
    }
}

pub struct ParameterListReader<'a> {
    pio: &'a ParameterIOReader<'a>,
    header: ResParameterList,
    lists_offset: u32,
    objs_offset: u32,
}

impl<'a> ParameterListReader<'a> {
    fn new(pio: &'a ParameterIOReader<'a>, offset: u32) -> Result<Self> {
        let header: ResParameterList = pio.parser.read_at(offset)?;
        Ok(Self {
            lists_offset: header.lists_rel_offset as u32 * 4 + offset,
            objs_offset: header.objects_rel_offset as u32 * 4 + offset,
            pio,
            header,
        })
    }

    fn new_with_header(
        pio: &'a ParameterIOReader<'a>,
        header: ResParameterList,
        offset: u32,
    ) -> Self {
        Self {
            lists_offset: header.lists_rel_offset as u32 * 4 + offset,
            objs_offset: header.objects_rel_offset as u32 * 4 + offset,
            pio,
            header,
        }
    }

    pub fn name(&self) -> Name {
        self.header.name
    }

    pub fn lists_len(&self) -> usize {
        self.header.list_count as usize
    }

    pub fn objs_len(&self) -> usize {
        self.header.object_count as usize
    }

    pub fn lists_empty(&self) -> bool {
        self.header.list_count == 0
    }

    pub fn objs_empty(&self) -> bool {
        self.header.object_count == 0
    }

    pub fn list(&'a self, name: impl Into<Name>) -> Option<ParameterListReader<'a>> {
        self.try_list(name).ok().flatten()
    }

    pub fn try_list(&'a self, name: impl Into<Name>) -> Result<Option<ParameterListReader<'a>>> {
        self._try_list(name.into())
    }

    fn _try_list(&'a self, name: Name) -> Result<Option<ParameterListReader<'a>>> {
        for i in 0..self.header.list_count {
            let offset = self.lists_offset + 0xC * i as u32;
            let list: ResParameterList = self.pio.parser.read_at(offset)?;
            if list.name != name {
                continue;
            }
            return Ok(Some(ParameterListReader::new_with_header(
                self.pio, list, offset,
            )));
        }
        Ok(None)
    }

    pub fn list_at(&'a self, index: usize) -> Option<ParameterListReader<'a>> {
        self.try_list_at(index).ok().flatten()
    }

    pub fn try_list_at(&'a self, index: usize) -> Result<Option<ParameterListReader<'a>>> {
        if index >= self.header.list_count as usize {
            Ok(None)
        } else {
            let offset = self.lists_offset + 0xC * index as u32;
            Ok(Some(ParameterListReader::new(self.pio, offset)?))
        }
    }

    pub fn object(&'a self, name: impl Into<Name>) -> Option<ParameterObjectReader<'a>> {
        self.try_object(name).ok().flatten()
    }

    pub fn try_object(
        &'a self,
        name: impl Into<Name>,
    ) -> Result<Option<ParameterObjectReader<'a>>> {
        self._try_object(name.into())
    }

    fn _try_object(&'a self, name: Name) -> Result<Option<ParameterObjectReader<'a>>> {
        for i in 0..self.header.object_count {
            let offset = self.objs_offset + 0x8 * i as u32;
            let list: ResParameterObj = self.pio.parser.read_at(offset)?;
            if list.name != name {
                continue;
            }
            return Ok(Some(ParameterObjectReader {
                header: list,
                offset,
                pio: self.pio,
            }));
        }
        Ok(None)
    }

    pub fn object_at(&'a self, index: usize) -> Option<ParameterObjectReader<'a>> {
        self.try_object_at(index).ok().flatten()
    }

    pub fn try_object_at(&'a self, index: usize) -> Result<Option<ParameterObjectReader<'a>>> {
        if index >= self.header.object_count as usize {
            Ok(None)
        } else {
            let offset = self.objs_offset + 0xC * index as u32;
            Ok(Some(ParameterObjectReader::new(self.pio, offset)?))
        }
    }

    pub fn iter_lists(&'a self) -> ParameterListsIterator<'a> {
        ParameterListsIterator {
            idx: 0,
            list: self.header,
            lists_offset: self.lists_offset,
            pio: self.pio,
        }
    }

    pub fn iter_objs(&'a self) -> ParameterObjectsIterator<'a> {
        ParameterObjectsIterator {
            pio: self.pio,
            list: self.header,
            objs_offset: self.objs_offset,
            idx: 0,
        }
    }
}

pub struct ParameterObjectReader<'a> {
    pio: &'a ParameterIOReader<'a>,
    header: ResParameterObj,
    offset: u32,
}

pub struct ParameterObjectIterator<'a> {
    reader: &'a ParameterObjectReader<'a>,
    idx: usize,
}

impl<'a> Iterator for ParameterObjectIterator<'a> {
    type Item = (Name, Type);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.reader.len() {
            None
        } else {
            let offset = self.reader.header.params_rel_offset as u32 * 4 + self.reader.offset;
            let param_offset = offset + 0x8 * self.idx as u32;
            let param: ResParameter = self.reader.pio.parser.read_at(param_offset).ok()?;
            self.idx += 1;
            Some((param.name, param.type_))
        }
    }
}

impl<'a> ParameterObjectReader<'a> {
    fn new(pio: &'a ParameterIOReader<'a>, offset: u32) -> Result<Self> {
        let header = pio.parser.read_at(offset)?;
        Ok(Self {
            pio,
            header,
            offset,
        })
    }

    pub fn len(&self) -> usize {
        self.header.param_count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<T: ParseParam<'a>>(&'a self, name: impl Into<Name>) -> Option<T> {
        self.try_get(name).ok().flatten()
    }

    pub fn get_at<T: ParseParam<'a>>(&'a self, index: usize) -> Option<T> {
        self.try_get_at(index).ok().flatten()
    }

    pub fn try_get<T: ParseParam<'a>>(&'a self, name: impl Into<Name>) -> Result<Option<T>> {
        let name = name.into();
        self._try_get(name)
    }

    fn _try_get<T: ParseParam<'a>>(&'a self, name: Name) -> Result<Option<T>> {
        let offset = self.header.params_rel_offset as u32 * 4 + self.offset;
        for i in 0..self.header.param_count {
            let param_offset = offset + 0x8 * i as u32;
            let param: ResParameter = self.pio.parser.read_at(param_offset)?;
            if param.name != name {
                continue;
            }
            if param.type_ != T::VARIANT {
                return Err(crate::Error::TypeError(
                    param.type_.name().into(),
                    T::VARIANT.name(),
                ));
            }
            let data_offset: u32 = param.data_rel_offset.as_u32() * 4 + param_offset;
            return T::parse(&self.pio.parser, data_offset).map(Some);
        }
        Ok(None)
    }

    pub fn try_get_at<T: ParseParam<'a>>(&'a self, index: usize) -> Result<Option<T>> {
        if index >= self.header.param_count as usize {
            return Ok(None);
        }
        let offset = self.header.params_rel_offset as u32 * 4 + self.offset;
        let param_offset = offset + 0x8 * index as u32;
        let param: ResParameter = self.pio.parser.read_at(param_offset)?;
        if param.type_ != T::VARIANT {
            return Err(crate::Error::TypeError(
                param.type_.name().into(),
                T::VARIANT.name(),
            ));
        }
        let data_offset = param.data_rel_offset.as_u32() * 4 + param_offset;
        T::parse(&self.pio.parser, data_offset).map(Some)
    }

    pub fn iter(&'a self) -> ParameterObjectIterator<'a> {
        ParameterObjectIterator {
            reader: self,
            idx: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parsing() {
        let data = std::fs::read("test/aamp/test.aamp").unwrap();
        let reader = super::ParameterIOReader::new(&data).unwrap();
        assert_eq!(reader.doc_type().unwrap(), "oead_test");
        let root = reader.root();
        assert_eq!(root.lists_len(), 0);
        assert_eq!(root.objs_len(), 1);
        let test_obj = root.object("TestContent").unwrap();
        assert_eq!(test_obj.get_at::<bool>(0), Some(true));
        assert_eq!(
            test_obj.get::<&[u8]>("BufferBinary"),
            Some([1, 2, 3, 4, 5, 6, 0xff, 1].as_slice())
        );
        assert_eq!(test_obj.get::<&str>("StringRef_2"), Some("fkisfj 2929 jdj"));
        assert!(matches!(
            test_obj
                .try_get_at::<&[f32]>(1)
                .expect_err("Wrong type detected"),
            crate::Error::TypeError(..)
        ));
    }
}
