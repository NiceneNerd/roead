//! Bindings for the `oead::aamp` module.
//!
//! Only version 2, little endian and UTF-8 binary parameter archives are supported.  
//! All parameter types including buffers are supported.  
//! The YAML output is compatible with the pure Python aamp library.
//!
//! The main type is the `ParameterIO`, which will usually be constructed
//! from binary data of a YAML document. Some sample usage:
//! ```
//! # use roead::aamp::*;
//! # fn doctest() -> std::result::Result<(), Box<dyn std::error::Error>> {
//! let data = std::fs::read("test/Chuchu_Middle.baiprog")?;
//! let pio = ParameterIO::from_binary(&data)?; // Parse AAMP from binary data
//! for (hash, list) in pio.lists().iter() {
//!     // Do stuff with lists
//! }
//! if let Some(demo_obj) = pio.object("DemoAIActionIdx") { // Access a parameter object
//!     for (hash, parameter) in demo_obj.params() {
//!         // Do stuff with parameters
//!     }
//! }
//! // Dumps YAML representation to a String
//! let yaml_dump: String = pio.to_text();
//! # Ok(())
//! # }
//! ```
use crate::ffi;
use crate::ffi::{Color, Curve, ParamType, Quat, Vector2f, Vector3f, Vector4f};
use crc::crc32::checksum_ieee;
use cxx::UniquePtr;
use indexmap::IndexMap;
use thiserror::Error;

pub mod names;
pub type Result<T> = std::result::Result<T, AampError>;

/// An error when serializing/deserializing AAMP documents
#[derive(Error, Debug)]
pub enum AampError {
    #[error("Invalid AAMP magic, expected \"AAMP\", found {0}")]
    MagicError(String),
    #[error("Parameter value is not of expected type")]
    TypeError,
    /// Wraps any other error returned by `oead` in C++
    #[error("Failed to parse AAMP: {0}")]
    OeadError(#[from] cxx::Exception),
}

/// Represents a single AAMP parameter, with many possible types.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Parameter {
    Bool(bool),
    F32(f32),
    Int(i32),
    Vec2(Vector2f),
    Vec3(Vector3f),
    Vec4(Vector4f),
    Color(Color),
    String32(String),
    String64(String),
    Curve1([Curve; 1]),
    Curve2([Curve; 2]),
    Curve3([Curve; 3]),
    Curve4([Curve; 4]),
    BufferInt(Vec<i32>),
    BufferF32(Vec<f32>),
    String256(String),
    Quat(Quat),
    U32(u32),
    BufferU32(Vec<u32>),
    BufferBinary(Vec<u8>),
    StringRef(String),
}

impl From<UniquePtr<ffi::Parameter>> for Parameter {
    fn from(fparam: UniquePtr<ffi::Parameter>) -> Self {
        match fparam.GetType() {
            ParamType::Bool => Self::Bool(ffi::GetParamBool(&fparam)),
            ParamType::F32 => Self::F32(ffi::GetParamF32(&fparam)),
            ParamType::U32 => Self::U32(ffi::GetParamU32(&fparam)),
            ParamType::Int => Self::Int(ffi::GetParamInt(&fparam)),
            ParamType::Vec2 => Self::Vec2(ffi::GetParamVec2(&fparam)),
            ParamType::Vec3 => Self::Vec3(ffi::GetParamVec3(&fparam)),
            ParamType::Vec4 => Self::Vec4(ffi::GetParamVec4(&fparam)),
            ParamType::Color => Self::Color(ffi::GetParamColor(&fparam)),
            ParamType::Quat => Self::Quat(ffi::GetParamQuat(&fparam)),
            ParamType::Curve1 => Self::Curve1(ffi::GetParamCurve1(&fparam)),
            ParamType::Curve2 => Self::Curve2(ffi::GetParamCurve2(&fparam)),
            ParamType::Curve3 => Self::Curve3(ffi::GetParamCurve3(&fparam)),
            ParamType::Curve4 => Self::Curve4(ffi::GetParamCurve4(&fparam)),
            ParamType::String32 => Self::String32(ffi::GetParamString(&fparam)),
            ParamType::String64 => Self::String64(ffi::GetParamString(&fparam)),
            ParamType::String256 => Self::String256(ffi::GetParamString(&fparam)),
            ParamType::StringRef => Self::StringRef(ffi::GetParamString(&fparam)),
            ParamType::BufferInt => Self::BufferInt(ffi::GetParamBufInt(&fparam)),
            ParamType::BufferF32 => Self::BufferF32(ffi::GetParamBufF32(&fparam)),
            ParamType::BufferU32 => Self::BufferU32(ffi::GetParamBufU32(&fparam)),
            ParamType::BufferBinary => Self::BufferBinary(ffi::GetParamBufBin(&fparam)),
            _ => unreachable!(),
        }
    }
}

impl Parameter {
    /// Check if the parameter is any string type
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(
            self,
            Parameter::String32(_)
                | Parameter::String64(_)
                | Parameter::String256(_)
                | Parameter::StringRef(_)
        )
    }

    /// Check if the parameter is any buffer type
    #[inline]
    pub fn is_buffer(&self) -> bool {
        matches!(
            self,
            Parameter::BufferBinary(_)
                | Parameter::BufferF32(_)
                | Parameter::BufferInt(_)
                | Parameter::BufferU32(_)
        )
    }

    pub(crate) fn get_ffi_type(&self) -> ParamType {
        match self {
            Self::Bool(_) => ParamType::Bool,
            Self::F32(_) => ParamType::F32,
            Self::Int(_) => ParamType::Int,
            Self::Vec2(_) => ParamType::Vec2,
            Self::Vec3(_) => ParamType::Vec3,
            Self::Vec4(_) => ParamType::Vec4,
            Self::Color(_) => ParamType::Color,
            Self::String32(_) => ParamType::String32,
            Self::String64(_) => ParamType::String64,
            Self::Curve1(_) => ParamType::Curve1,
            Self::Curve2(_) => ParamType::Curve2,
            Self::Curve3(_) => ParamType::Curve3,
            Self::Curve4(_) => ParamType::Curve4,
            Self::BufferInt(_) => ParamType::BufferInt,
            Self::BufferF32(_) => ParamType::BufferF32,
            Self::String256(_) => ParamType::String256,
            Self::Quat(_) => ParamType::Quat,
            Self::U32(_) => ParamType::U32,
            Self::BufferU32(_) => ParamType::BufferU32,
            Self::BufferBinary(_) => ParamType::BufferBinary,
            Self::StringRef(_) => ParamType::StringRef,
        }
    }

    pub(crate) fn get_bool(&self) -> bool {
        if let Self::Bool(v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_f32(&self) -> f32 {
        if let Self::F32(v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_int(&self) -> i32 {
        if let Self::Int(v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_vec2(&self) -> &Vector2f {
        if let Self::Vec2(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_vec3(&self) -> &Vector3f {
        if let Self::Vec3(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_vec4(&self) -> &Vector4f {
        if let Self::Vec4(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_color(&self) -> &Color {
        if let Self::Color(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_string32(&self) -> &str {
        if let Self::String32(v) = self {
            v.as_str()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_string64(&self) -> &str {
        if let Self::String64(v) = self {
            v.as_str()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_curve1(&self) -> &[Curve; 1] {
        if let Self::Curve1(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_curve2(&self) -> &[Curve; 2] {
        if let Self::Curve2(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_curve3(&self) -> &[Curve; 3] {
        if let Self::Curve3(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_curve4(&self) -> &[Curve; 4] {
        if let Self::Curve4(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_buf_int(&self) -> &[i32] {
        if let Self::BufferInt(v) = self {
            v.as_slice()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_buf_f32(&self) -> &[f32] {
        if let Self::BufferF32(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_string_256(&self) -> &str {
        if let Self::String256(v) = self {
            v.as_str()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_quat(&self) -> &Quat {
        if let Self::Quat(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_u32(&self) -> u32 {
        if let Self::U32(v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_buf_u32(&self) -> &[u32] {
        if let Self::BufferU32(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_buf_bin(&self) -> &[u8] {
        if let Self::BufferBinary(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_str_ref(&self) -> &str {
        if let Self::StringRef(v) = self {
            v.as_str()
        } else {
            unreachable!()
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        if let Self::Bool(v) = self {
            Ok(*v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_f32(&self) -> Result<f32> {
        if let Self::F32(v) = self {
            Ok(*v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_int(&self) -> Result<i32> {
        if let Self::Int(v) = self {
            Ok(*v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_vec2(&self) -> Result<&Vector2f> {
        if let Self::Vec2(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_vec3(&self) -> Result<&Vector3f> {
        if let Self::Vec3(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_vec4(&self) -> Result<&Vector4f> {
        if let Self::Vec4(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_color(&self) -> Result<&Color> {
        if let Self::Color(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_string32(&self) -> Result<&str> {
        if let Self::String32(v) = self {
            Ok(v.as_str())
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_string64(&self) -> Result<&str> {
        if let Self::String64(v) = self {
            Ok(v.as_str())
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_curve1(&self) -> Result<&[Curve; 1]> {
        if let Self::Curve1(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_curve2(&self) -> Result<&[Curve; 2]> {
        if let Self::Curve2(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_curve3(&self) -> Result<&[Curve; 3]> {
        if let Self::Curve3(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_curve4(&self) -> Result<&[Curve; 4]> {
        if let Self::Curve4(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_buf_int(&self) -> Result<&[i32]> {
        if let Self::BufferInt(v) = self {
            Ok(v.as_slice())
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_buf_f32(&self) -> Result<&[f32]> {
        if let Self::BufferF32(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_string_256(&self) -> Result<&str> {
        if let Self::String256(v) = self {
            Ok(v.as_str())
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_quat(&self) -> Result<&Quat> {
        if let Self::Quat(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_u32(&self) -> Result<u32> {
        if let Self::U32(v) = self {
            Ok(*v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_buf_u32(&self) -> Result<&[u32]> {
        if let Self::BufferU32(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_buf_bin(&self) -> Result<&[u8]> {
        if let Self::BufferBinary(v) = self {
            Ok(v)
        } else {
            Err(AampError::TypeError)
        }
    }

    pub fn as_str_ref(&self) -> Result<&str> {
        if let Self::StringRef(v) = self {
            Ok(v.as_str())
        } else {
            Err(AampError::TypeError)
        }
    }
}

/// Wraps a map of parameters and their name hashes
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ParameterObject(IndexMap<u32, Parameter>);

impl PartialEq for ParameterObject {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len() && self.0.iter().zip(other.0.iter()).all(|(e, e2)| e == e2)
    }
}

impl From<UniquePtr<ffi::ParameterObject>> for ParameterObject {
    fn from(pobj: UniquePtr<ffi::ParameterObject>) -> Self {
        let map = ffi::GetParams(&pobj);
        Self(
            (0usize..map.size())
                .map(|i| {
                    let pair = ffi::GetParamAt(&map, i);
                    (pair.hash, pair.param.into())
                })
                .collect::<IndexMap<u32, Parameter>>(),
        )
    }
}

impl ParameterObject {
    /// Create an empty ParameterObject
    pub fn new() -> Self {
        Self(IndexMap::new())
    }

    /// Attempt to get a `Parameter` by name, returns None if not found
    pub fn param(&self, name: &str) -> Option<&Parameter> {
        self.0.get(&checksum_ieee(name.as_bytes()))
    }

    /// Set a parameter value
    pub fn set_param(&mut self, name: &str, value: Parameter) {
        self.0.insert(checksum_ieee(name.as_bytes()), value);
    }
    /// Expose reference to underlying IndexMap
    pub fn params(&self) -> &IndexMap<u32, Parameter> {
        &self.0
    }

    /// Expose mutable reference to underlying IndexMap
    pub fn params_mut(&mut self) -> &mut IndexMap<u32, Parameter> {
        &mut self.0
    }

    /// Count the number of parameters
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn hash_at(&self, i: usize) -> u32 {
        *self.0.keys().nth(i).unwrap()
    }

    pub(crate) fn val_at(&self, i: usize) -> &Parameter {
        self.0.values().nth(i).unwrap()
    }
}

/// A trait representing any kind of parameter list, which can be used
/// for both a proper ParameterList and a ParameterIO
pub trait ParamList {
    /// Get a map of child parameter lists and their name hashes
    fn lists(&self) -> &IndexMap<u32, ParameterList>;
    /// Get a map of child parameter objects and their name hashes
    fn objects(&self) -> &IndexMap<u32, ParameterObject>;
    /// Get a mutable map of child parameter lists and their name hashes
    fn lists_mut(&mut self) -> &mut IndexMap<u32, ParameterList>;
    /// Get a mutable map of child parameter objects and their name hashes
    fn objects_mut(&mut self) -> &mut IndexMap<u32, ParameterObject>;
    /// Get a child parameter list by name
    fn list(&self, name: &str) -> Option<&ParameterList> {
        self.lists().get(&checksum_ieee(name.as_bytes()))
    }
    /// Get a child parameter object by name
    fn object(&self, name: &str) -> Option<&ParameterObject> {
        self.objects().get(&checksum_ieee(name.as_bytes()))
    }
    /// Set a child parameter list by name
    fn set_list(&mut self, name: &str, plist: ParameterList) {
        self.lists_mut()
            .insert(checksum_ieee(name.as_bytes()), plist);
    }
    /// Set a child parameter object by name
    fn set_object(&mut self, name: &str, pobj: ParameterObject) {
        self.objects_mut()
            .insert(checksum_ieee(name.as_bytes()), pobj);
    }
}

/// Represents a parameter list consisting of child parameter lists
/// and parameter objects
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterList {
    lists: IndexMap<u32, ParameterList>,
    objects: IndexMap<u32, ParameterObject>,
}

impl From<UniquePtr<ffi::ParameterList>> for ParameterList {
    fn from(plist: UniquePtr<ffi::ParameterList>) -> Self {
        let list_map = ffi::GetParamLists(&plist);
        let lists = (0usize..list_map.size())
            .map(|i| {
                let pair = ffi::GetParamListAt(&list_map, i);
                (pair.hash, pair.param.into())
            })
            .collect::<IndexMap<u32, ParameterList>>();
        let obj_map = ffi::GetParamObjs(&plist);
        let objects = (0usize..obj_map.size())
            .map(|i| {
                let pair = ffi::GetParamObjAt(&obj_map, i);
                (pair.hash, pair.param.into())
            })
            .collect::<IndexMap<u32, ParameterObject>>();
        Self { lists, objects }
    }
}

impl From<ParameterIO> for ParameterList {
    fn from(pio: ParameterIO) -> Self {
        Self {
            lists: pio.lists,
            objects: pio.objects,
        }
    }
}

impl ParamList for ParameterList {
    fn lists(&self) -> &IndexMap<u32, ParameterList> {
        &self.lists
    }

    fn objects(&self) -> &IndexMap<u32, ParameterObject> {
        &self.objects
    }

    fn lists_mut(&mut self) -> &mut IndexMap<u32, ParameterList> {
        &mut self.lists
    }

    fn objects_mut(&mut self) -> &mut IndexMap<u32, ParameterObject> {
        &mut self.objects
    }
}

impl ParameterList {
    /// Create an empty ParameterIO
    pub fn new() -> Self {
        ParameterList {
            lists: IndexMap::new(),
            objects: IndexMap::new(),
        }
    }

    pub(crate) fn list_count(&self) -> usize {
        self.lists.len()
    }

    pub(crate) fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub(crate) fn list_hash_at(&self, i: usize) -> u32 {
        *self.lists.keys().nth(i).unwrap()
    }

    pub(crate) fn obj_hash_at(&self, i: usize) -> u32 {
        *self.objects.keys().nth(i).unwrap()
    }

    pub(crate) fn list_at(&self, i: usize) -> &ParameterList {
        self.lists.values().nth(i).unwrap()
    }

    pub(crate) fn obj_at(&self, i: usize) -> &ParameterObject {
        self.objects.values().nth(i).unwrap()
    }
}

/// Represents a parameter IO document. This is the root parameter list and
/// the only structure that can be serialized to or deserialized from a binary
/// parameter archive.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterIO {
    /// Data version (not the AAMP format version). Typically 0.
    pub version: u32,
    /// Data type identifier. Typically “xml”.
    pub doc_type: String,
    lists: IndexMap<u32, ParameterList>,
    objects: IndexMap<u32, ParameterObject>,
}

impl From<UniquePtr<ffi::ParameterIO>> for ParameterIO {
    fn from(pio: UniquePtr<ffi::ParameterIO>) -> Self {
        let version = ffi::GetPioVersion(&pio);
        let r#type = ffi::GetPioType(&pio);
        let list_map = ffi::GetParamListsFromPio(&pio);
        let lists = (0usize..list_map.size())
            .map(|i| {
                let pair = ffi::GetParamListAt(&list_map, i);
                (pair.hash, pair.param.into())
            })
            .collect::<IndexMap<u32, ParameterList>>();
        let obj_map = ffi::GetParamObjsFromPio(&pio);
        let objects = (0usize..obj_map.size())
            .map(|i| {
                let pair = ffi::GetParamObjAt(&obj_map, i);
                (pair.hash, pair.param.into())
            })
            .collect::<IndexMap<u32, ParameterObject>>();
        Self {
            version,
            doc_type: r#type,
            lists,
            objects,
        }
    }
}

impl From<ParameterList> for ParameterIO {
    fn from(plist: ParameterList) -> Self {
        Self {
            doc_type: "xml".to_owned(),
            version: 0,
            lists: plist.lists,
            objects: plist.objects,
        }
    }
}

impl ParamList for ParameterIO {
    fn lists(&self) -> &IndexMap<u32, ParameterList> {
        &self.lists
    }

    fn objects(&self) -> &IndexMap<u32, ParameterObject> {
        &self.objects
    }

    fn lists_mut(&mut self) -> &mut IndexMap<u32, ParameterList> {
        &mut self.lists
    }

    fn objects_mut(&mut self) -> &mut IndexMap<u32, ParameterObject> {
        &mut self.objects
    }
}

impl ParameterIO {
    /// Create an empty ParameterIO
    pub fn new() -> Self {
        ParameterIO {
            doc_type: "xml".to_owned(),
            version: 0,
            lists: IndexMap::new(),
            objects: IndexMap::new(),
        }
    }

    /// Load a ParameterIO from a binary parameter archive.
    pub fn from_binary<B: AsRef<[u8]>>(data: B) -> Result<ParameterIO> {
        let data = data.as_ref();
        if &data[0..4] != b"AAMP" {
            return Err(AampError::MagicError(
                String::from_utf8_lossy(&data[0..4]).to_string(),
            ));
        }
        Ok(ffi::AampFromBinary(data.as_ref())?.into())
    }

    /// Load a ParameterIO from a YAML representation.
    pub fn from_text<S: AsRef<str>>(text: S) -> Result<ParameterIO> {
        Ok(ffi::AampFromText(text.as_ref())?.into())
    }

    /// Serialize the ParameterIO to a YAML representation.
    pub fn to_text(&self) -> String {
        ffi::AampToText(&self)
    }

    /// Serialize the ParameterIO to a binary parameter archive.
    pub fn to_binary(&self) -> Vec<u8> {
        ffi::AampToBinary(&self)
    }

    pub(crate) fn list_count(&self) -> usize {
        self.lists.len()
    }

    pub(crate) fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub(crate) fn list_hash_at(&self, i: usize) -> u32 {
        *self.lists.keys().nth(i).unwrap()
    }

    pub(crate) fn obj_hash_at(&self, i: usize) -> u32 {
        *self.objects.keys().nth(i).unwrap()
    }

    pub(crate) fn list_at(&self, i: usize) -> &ParameterList {
        self.lists.values().nth(i).unwrap()
    }

    pub(crate) fn obj_at(&self, i: usize) -> &ParameterObject {
        self.objects.values().nth(i).unwrap()
    }

    pub(crate) fn pio_type(&self) -> &str {
        self.doc_type.as_str()
    }

    pub(crate) fn version(&self) -> u32 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::{Parameter, ParameterIO};
    use crate::aamp::ParamList;
    use crc::crc32::checksum_ieee;
    use rayon::prelude::*;
    use std::path::PathBuf;

    #[test]
    fn parse_aamps_binary() {
        for file in glob::glob("include/oead/test/aamp/files/**/*.b*")
            .unwrap()
            .filter_map(|f| f.ok())
            .take(300)
        {
            let data = std::fs::read(&file).unwrap();
            ParameterIO::from_binary(&data).unwrap();
        }
    }

    #[test]
    fn multithread_aamps() {
        let files: Vec<PathBuf> = glob::glob("include/oead/test/aamp/files/**/*.b*")
            .unwrap()
            .filter_map(|f| f.ok())
            .take(300)
            .collect();
        files.into_par_iter().for_each(|file| {
            let data = std::fs::read(&file).unwrap();
            ParameterIO::from_binary(&data).unwrap();
        });
    }

    #[test]
    fn parse_aamp_text() {
        let text = std::fs::read_to_string("include/oead/test/aamp/test.yml").unwrap();
        let pio = ParameterIO::from_text(&text).unwrap();
        assert_eq!(&pio.doc_type, "oead_test");
        let obj = pio.object("TestContent").unwrap();
        let (name, val) = obj.0.get_index(3).unwrap();
        assert_eq!(name, &checksum_ieee(b"F32_1"));
        match val {
            Parameter::F32(v) => assert_eq!(v, &500.12),
            _ => panic!("Wrong variant"),
        };
    }

    #[test]
    fn aamp_text_roundtrip() {
        let files: Vec<PathBuf> = glob::glob("test/aamp/*.yml")
            .unwrap()
            .filter_map(|f| f.ok())
            .collect();
        files.into_par_iter().for_each(|file| {
            let data = std::fs::read_to_string(&file).unwrap();
            let pio = ParameterIO::from_text(&data).unwrap();
            let text = pio.to_text();
            let pio2 = ParameterIO::from_text(&text).unwrap();
            assert_eq!(pio, pio2);
        });
    }

    #[test]
    fn aamp_binary_roundtrip() {
        for file in glob::glob("include/oead/test/aamp/files/**/*.b*")
            .unwrap()
            .take(300)
            .filter_map(|f| f.ok())
        {
            let data = std::fs::read(&file).unwrap();
            let pio = ParameterIO::from_binary(&data).unwrap();
            let text = pio.to_binary();
            let pio2 = ParameterIO::from_binary(&text).unwrap();
            assert_eq!(pio, pio2);
        }
    }
}
