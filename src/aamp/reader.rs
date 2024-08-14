//! This module provides a simple reader for an AAMP Parameter IO document. Lists, objects, and
//! parameters can be iterated, queried, etc. without *any* heap allocations, except (temporarily)
//! in the case of type errors. In a sense this API is more low-level than the fully parsed and
//! allocated [`ParameterIO`] type. Since nearly all of the parsing is lazy/on-demand, many of the
//! tools of this API are fallible and use [`Result`]s, but for simplicity some of the basic query
//! methods have simpler `get` methods that return only [`Option<T>`] values but also `try_get`
//! methods that return [`Result<Option<T>>`].
//!
//! For example:
//! ```
//! # use roead::{Error, aamp::*};
//! let data = std::fs::read("test/aamp/test.aamp").unwrap();
//! let reader = ParameterIOReader::new(&data).unwrap();
//! assert_eq!(reader.doc_type().unwrap(), "oead_test");
//! let root = reader.root();
//! assert_eq!(root.lists_len(), 0);
//! assert_eq!(root.objs_len(), 1);
//! let test_obj = root.object("TestContent").unwrap();
//! assert_eq!(test_obj.get_at::<bool>(0), Some(true));
//! assert_eq!(
//!     test_obj.get::<&[u8]>("BufferBinary"),
//!     Some([1, 2, 3, 4, 5, 6, 0xff, 1].as_slice())
//! );
//! assert_eq!(test_obj.get::<&str>("StringRef_2"), Some("fkisfj 2929 jdj"));
//! assert!(matches!(
//!     test_obj
//!         .try_get_at::<&[f32]>(1)
//!         .expect_err("Wrong type detected"),
//!     Error::TypeError(..)
//! ));
//! ```
use binrw::{io::*, BinRead};
use parser::{ParseParam, Parser};

use super::*;
use crate::Result;

/// A readonly view of an AAMP Parameter IO document.
pub struct ParameterIOReader<'a> {
    parser: Parser<Cursor<&'a [u8]>>,
    root: ResParameterList,
    root_offset: u32,
}

impl<'a> ParameterIOReader<'a> {
    /// Construct a [`ParameterIOReader`] from binary data. Unlike parsing into a [`ParameterIO`],
    /// which can read progresively from anything which implements [`std::io::Read`], this requires
    /// access to the whole archive as a byte slice.
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

    /// Returns a [`ParameterListReader`] for the root parameter list.
    pub fn root(&'a self) -> ParameterListReader<'a> {
        ParameterListReader::new_with_header(self, self.root, self.root_offset)
    }

    /// Returns the data type identifier. Typically `xml`.
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

    /// Returns the data version (not the AAMP format version). Typically 0.
    #[allow(clippy::misnamed_getters)]
    pub fn version(&self) -> u32 {
        self.parser.header.pio_version
    }
}

/// Iterator over parameter lists
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

/// Iterator over parameter objects
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

/// Parameter list reader. Used to query the name, lists, and objects of a parameter list. It also
/// exposes an iterator over the lists and objects.
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

    /// Returns the hashed name of the parameter list.
    pub fn name(&self) -> Name {
        self.header.name
    }

    /// Returns the number of child lists in the parameter list.
    pub fn lists_len(&self) -> usize {
        self.header.list_count as usize
    }

    /// Returns the number of parameter objects in the parameter list.
    pub fn objs_len(&self) -> usize {
        self.header.object_count as usize
    }

    /// Returns `true` if the parameter list contains no child lists.
    pub fn lists_empty(&self) -> bool {
        self.header.list_count == 0
    }

    /// Returns `true` if the parameter list contains no parameter objects.
    pub fn objs_empty(&self) -> bool {
        self.header.object_count == 0
    }

    /// Attempts to get a [`ParameterListReader`] for reading the child list under the given key.
    ///
    /// Return [`None`] if the child list does not exist or if any parsing errors occur. If you need
    /// to catch the error information, use [`ParameterListReader::try_list`].
    pub fn list(&'a self, name: impl Into<Name>) -> Option<ParameterListReader<'a>> {
        self.try_list(name).ok().flatten()
    }

    /// Attempts to get a [`ParameterListReader`] for reading the child list under the given key.
    ///
    /// Returns [`Ok`]`(`[`None`]`)` if the child list does not exist. Otherwise, it will return
    /// [`Ok`]`(`[`Some`]`::<`[`ParameterListReader`]`>)` if there was no error and the child
    /// exists, or an error if there was a parsing error.
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

    /// Attempts to get a [`ParameterListReader`] for reading the child list at the given index.
    ///
    /// Return [`None`] if the index is out of range or if any parsing errors occur. If you need
    /// to catch the error information, use [`ParameterListReader::try_list_at`].
    pub fn list_at(&'a self, index: usize) -> Option<ParameterListReader<'a>> {
        self.try_list_at(index).ok().flatten()
    }

    /// Attempts to get a [`ParameterListReader`] for reading the child list at the given index.
    ///
    /// Returns [`Ok`]`(`[`None`]`)` if the index is out of range. Otherwise, it will return
    /// [`Ok`]`(`[`Some`]`::<`[`ParameterListReader`]`>)` if there was no error and the child
    /// exists, or an error if there was a parsing error.
    pub fn try_list_at(&'a self, index: usize) -> Result<Option<ParameterListReader<'a>>> {
        if index >= self.header.list_count as usize {
            Ok(None)
        } else {
            let offset = self.lists_offset + 0xC * index as u32;
            Ok(Some(ParameterListReader::new(self.pio, offset)?))
        }
    }

    /// Attempts to get a [`ParameterObjectReader`] for reading the parameter object under the given
    /// key.
    ///
    /// Return [`None`] if the object does not exist or if any parsing errors occur. If you need
    /// to catch the error information, use [`ParameterListReader::try_object`].
    pub fn object(&'a self, name: impl Into<Name>) -> Option<ParameterObjectReader<'a>> {
        self.try_object(name).ok().flatten()
    }

    /// Attempts to get a [`ParameterObjectReader`] for reading the parameter object under the given
    /// key.
    ///
    /// Returns [`Ok`]`(`[`None`]`)` if the object does not exist. Otherwise, it will return
    /// [`Ok`]`(`[`Some`]`::<`[`ParameterListReader`]`>)` if there was no error and the object
    /// exists, or an error if there was a parsing error.
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

    /// Attempts to get a [`ParameterObjectReader`] for reading the parameter object at the given
    /// index.
    ///
    /// Return [`None`] if the object does not exist or if any parsing errors occur. If you need
    /// to catch the error information, use [`ParameterListReader::try_object`].
    pub fn object_at(&'a self, index: usize) -> Option<ParameterObjectReader<'a>> {
        self.try_object_at(index).ok().flatten()
    }

    /// Attempts to get a [`ParameterObjectReader`] for reading the parameter object at the given
    /// index.
    ///
    /// Returns [`Ok`]`(`[`None`]`)` if the index is out of range. Otherwise, it will return
    /// [`Ok`]`(`[`Some`]`::<`[`ParameterListReader`]`>)` if there was no error and the object
    /// exists, or an error if there was a parsing error.
    pub fn try_object_at(&'a self, index: usize) -> Result<Option<ParameterObjectReader<'a>>> {
        if index >= self.header.object_count as usize {
            Ok(None)
        } else {
            let offset = self.objs_offset + 0xC * index as u32;
            Ok(Some(ParameterObjectReader::new(self.pio, offset)?))
        }
    }

    /// Returns an iterator over the child parameter lists in the form `(`[`Name`]`,
    /// `[`ParameterListReader`]`)`.
    pub fn iter_lists(&'a self) -> ParameterListsIterator<'a> {
        ParameterListsIterator {
            idx: 0,
            list: self.header,
            lists_offset: self.lists_offset,
            pio: self.pio,
        }
    }

    /// Returns an iterator over the parameter objects in the form `(`[`Name`]`,
    /// `[`ParameterObjectReader`]`)`.
    pub fn iter_objs(&'a self) -> ParameterObjectsIterator<'a> {
        ParameterObjectsIterator {
            pio: self.pio,
            list: self.header,
            objs_offset: self.objs_offset,
            idx: 0,
        }
    }
}

/// Parameter object reader. Used to query the name and parameters in a parameter object. It also
/// exposes an iterator over the parameters.
pub struct ParameterObjectReader<'a> {
    pio: &'a ParameterIOReader<'a>,
    header: ResParameterObj,
    offset: u32,
}

/// Iterator over the parameters in a parameter object.
///
/// Note that since the possible value of a parameter comes in many types, but we do not want any
/// heap allocations (such as from using [`Box`]), each parameter is returned as a
/// [`ParameterValue`] enum, which is quite large (256 bytes) in order to accomodate *all* of the
/// parameter types. Generally speaking, for this kind of readonly parsing, you are probably best
/// off sticking to known parameters which you can access directly with
/// [`ParameterObjectReader::get`] or [`ParameterObjectReader::get_at`].
pub struct ParameterObjectIterator<'a> {
    reader: &'a ParameterObjectReader<'a>,
    idx: usize,
}

impl<'a> Iterator for ParameterObjectIterator<'a> {
    type Item = (Name, ParameterValue<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.reader.len() {
            None
        } else {
            let offset = self.reader.header.params_rel_offset as u32 * 4 + self.reader.offset;
            let param_offset = offset + 0x8 * self.idx as u32;
            let param: ResParameter = self.reader.pio.parser.read_at(param_offset).ok()?;
            self.idx += 1;
            ParameterValue::new(self.reader.pio, param_offset, param)
                .ok()
                .map(|val| (param.name, val))
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

    /// Returns the number of parameters in this parameter object.
    pub fn len(&self) -> usize {
        self.header.param_count as usize
    }

    /// Returns true if the parameter object has no parameters.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Attempts to get the value of the parameter of type `T` with a given key in the parameter
    /// object. `T` must be a valid kind of parameter value.
    ///
    /// Return [`None`] if the index is out of range or if any parsing errors occur. If you need
    /// to catch the error information, use [`ParameterObjectReader::try_get`].
    pub fn get<T: ParseParam<'a>>(&'a self, name: impl Into<Name>) -> Option<T> {
        self.try_get(name).ok().flatten()
    }

    /// Attempts to get the value of the parameter of type `T` at the given index in the parameter
    /// object. `T` must be a valid kind of parameter value.
    ///
    /// Return [`None`] if the index is out of range or if any parsing errors occur. If you need
    /// to catch the error information, use [`ParameterObjectReader::try_get_at`].
    pub fn get_at<T: ParseParam<'a>>(&'a self, index: usize) -> Option<T> {
        self.try_get_at(index).ok().flatten()
    }

    /// Attempts to get the value of the parameter of type `T` with a given key in the parameter
    /// object. `T` must be a valid kind of parameter value.
    ///
    /// Returns [`Ok`]`(`[`None`]`)` if the parameter with the key does not exist. Otherwise, it
    /// will return [`Ok`]`(`[`Some`]`::<T>)` if there was no error and the parameter exists, or
    /// an error if there was a parsing error.
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

    /// Attempts to get the value of the parameter of type `T` at the given index in the parameter
    /// object. `T` must be a valid kind of parameter value.
    ///
    /// Returns [`Ok`]`(`[`None`]`)` if the parameter with the key does not exist. Otherwise, it
    /// will return [`Ok`]`(`[`Some`]`::<T>)` if there was no error and the parameter exists, or
    /// an error if there was a parsing error.
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

    fn parse_any_str(&'a self, type_: Type, data_offset: u32) -> Result<&'a str> {
        let len = match type_ {
            Type::String32 => Some(32),
            Type::String64 => Some(64),
            Type::String256 => Some(256),
            Type::StringRef => None,
            _ => {
                return Err(crate::Error::TypeError(
                    type_.name().into(),
                    "any string type",
                ));
            }
        };
        match len {
            None => <&str>::parse(&self.pio.parser, data_offset),
            Some(len) => {
                let data = &self.pio.parser.buffer()[data_offset as usize..];
                let null_idx = data
                    .iter()
                    .take(len)
                    .position(|c| *c != 0)
                    .ok_or(crate::Error::InvalidData("Unterminated string"))?;
                Ok(core::str::from_utf8(
                    &data[data_offset as usize..data_offset as usize + null_idx],
                )?)
            }
        }
    }

    pub fn get_str(&'a self, name: impl Into<Name>) -> Option<&'a str> {
        self.try_get_str(name).ok().flatten()
    }

    pub fn try_get_str(&'a self, name: impl Into<Name>) -> Result<Option<&'a str>> {
        self._try_get_str(name.into())
    }

    fn _try_get_str(&'a self, name: Name) -> Result<Option<&'a str>> {
        let offset = self.header.params_rel_offset as u32 * 4 + self.offset;
        for i in 0..self.header.param_count {
            let param_offset = offset + 0x8 * i as u32;
            let param: ResParameter = self.pio.parser.read_at(param_offset)?;
            if param.name != name {
                continue;
            }
            let data_offset: u32 = param.data_rel_offset.as_u32() * 4 + param_offset;
            return self.parse_any_str(param.type_, data_offset).map(Some);
        }
        Ok(None)
    }

    pub fn get_str_at(&'a self, index: usize) -> Option<&'a str> {
        self.try_get_str_at(index).ok().flatten()
    }

    pub fn try_get_str_at(&'a self, index: usize) -> Result<Option<&'a str>> {
        if index >= self.header.param_count as usize {
            return Ok(None);
        }
        let offset = self.header.params_rel_offset as u32 * 4 + self.offset;
        let param_offset = offset + 0x8 * index as u32;
        let param: ResParameter = self.pio.parser.read_at(param_offset)?;
        let data_offset = param.data_rel_offset.as_u32() * 4 + param_offset;
        self.parse_any_str(param.type_, data_offset).map(Some)
    }

    /// Returns an iterator over the parameter objects in the form `(`[`Name`]`,
    /// `[`ParameterValue`]`)`.
    ///
    /// Note that since the possible value of a parameter comes in many types, but we do not want
    /// any heap allocations (such as from using [`Box`]), each parameter is returned as a
    /// [`ParameterValue`] enum, which is quite large (256 bytes) in order to accomodate *all* of
    /// the parameter types. Generally speaking, for this kind of readonly parsing, you are
    /// probably best off sticking to known parameters which you can access directly with
    /// [`ParameterObjectReader::get`] or [`ParameterObjectReader::get_at`].
    pub fn iter(&'a self) -> ParameterObjectIterator<'a> {
        ParameterObjectIterator {
            reader: self,
            idx: 0,
        }
    }
}

pub enum ParameterValue<'a> {
    /// Boolean.
    Bool(bool),
    /// Float.
    F32(f32),
    /// Int.
    I32(i32),
    /// 2D vector.
    Vec2(Vector2f),
    /// 3D vector.
    Vec3(Vector3f),
    /// 4D vector.
    Vec4(Vector4f),
    /// Color.
    Color(Color),
    /// String (max length 32 bytes).
    String32(FixedSafeString<32>),
    /// String (max length 64 bytes).
    String64(FixedSafeString<64>),
    /// A single curve.
    Curve1([Curve; 1]),
    /// Two curves.
    Curve2([Curve; 2]),
    /// Three curves.
    Curve3([Curve; 3]),
    /// Four curves.
    Curve4([Curve; 4]),
    /// Buffer of signed ints.
    BufferInt(&'a [i32]),
    /// Buffer of floats.
    BufferF32(&'a [f32]),
    /// String (max length 256 bytes).
    String256(FixedSafeString<256>),
    /// Quaternion.
    Quat(Quat),
    /// Unsigned int.
    U32(u32),
    /// Buffer of unsigned ints.
    BufferU32(&'a [u32]),
    /// Buffer of binary data.
    BufferBinary(&'a [u8]),
    /// String (no length limit).
    StringRef(&'a str),
}

impl<'a> ParameterValue<'a> {
    fn new(pio: &'a ParameterIOReader<'a>, offset: u32, header: ResParameter) -> Result<Self> {
        let data_offset = header.data_rel_offset.as_u32() * 4 + offset;
        match header.type_ {
            Type::Bool => bool::parse(&pio.parser, data_offset).map(Self::Bool),
            Type::F32 => f32::parse(&pio.parser, data_offset).map(Self::F32),
            Type::Int => i32::parse(&pio.parser, data_offset).map(Self::I32),
            Type::Vec2 => Vector2f::parse(&pio.parser, data_offset).map(Self::Vec2),
            Type::Vec3 => Vector3f::parse(&pio.parser, data_offset).map(Self::Vec3),
            Type::Vec4 => Vector4f::parse(&pio.parser, data_offset).map(Self::Vec4),
            Type::Color => Color::parse(&pio.parser, data_offset).map(Self::Color),
            Type::String32 => {
                FixedSafeString::<32>::parse(&pio.parser, data_offset).map(Self::String32)
            }
            Type::String64 => {
                FixedSafeString::<64>::parse(&pio.parser, data_offset).map(Self::String64)
            }
            Type::Curve1 => <[Curve; 1]>::parse(&pio.parser, data_offset).map(Self::Curve1),
            Type::Curve2 => <[Curve; 2]>::parse(&pio.parser, data_offset).map(Self::Curve2),
            Type::Curve3 => <[Curve; 3]>::parse(&pio.parser, data_offset).map(Self::Curve3),
            Type::Curve4 => <[Curve; 4]>::parse(&pio.parser, data_offset).map(Self::Curve4),
            Type::BufferInt => <&[i32]>::parse(&pio.parser, data_offset).map(Self::BufferInt),
            Type::BufferF32 => <&[f32]>::parse(&pio.parser, data_offset).map(Self::BufferF32),
            Type::String256 => {
                FixedSafeString::<256>::parse(&pio.parser, data_offset).map(Self::String256)
            }
            Type::Quat => Quat::parse(&pio.parser, data_offset).map(Self::Quat),
            Type::U32 => u32::parse(&pio.parser, data_offset).map(Self::U32),
            Type::BufferU32 => <&[u32]>::parse(&pio.parser, data_offset).map(Self::BufferU32),
            Type::BufferBinary => <&[u8]>::parse(&pio.parser, data_offset).map(Self::BufferBinary),
            Type::StringRef => <&str>::parse(&pio.parser, data_offset).map(Self::StringRef),
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
