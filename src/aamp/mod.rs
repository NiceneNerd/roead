#[allow(dead_code)]
use crate::ffi;
use crate::ffi::{Color, Curve, ParamType, Quat, Vector2f, Vector3f, Vector4f};
use crc::crc32::checksum_ieee;
use indexmap::IndexMap;
use thiserror::Error;

type Result<T> = std::result::Result<T, AampError>;

/// An error when serializing/deserializing AAMP documents
#[derive(Error, Debug)]
pub enum AampError {
    #[error("Invalid AAMP magic, expected \"AAMP\", found {0}")]
    MagicError(String),
    /// Wraps any other error returned by `oead` in C++
    #[error("Failed to parse AAMP: {0}")]
    OeadError(#[from] cxx::Exception),
}

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

impl From<&ffi::Parameter> for Parameter {
    fn from(fparam: &ffi::Parameter) -> Self {
        match fparam.GetType() {
            ParamType::Bool => Self::Bool(ffi::GetParamBool(fparam)),
            ParamType::F32 => Self::F32(ffi::GetParamF32(fparam)),
            ParamType::U32 => Self::U32(ffi::GetParamU32(fparam)),
            ParamType::Int => Self::Int(ffi::GetParamInt(fparam)),
            ParamType::Vec2 => Self::Vec2(ffi::GetParamVec2(fparam)),
            ParamType::Vec3 => Self::Vec3(ffi::GetParamVec3(fparam)),
            ParamType::Vec4 => Self::Vec4(ffi::GetParamVec4(fparam)),
            ParamType::Color => Self::Color(ffi::GetParamColor(fparam)),
            ParamType::Quat => Self::Quat(ffi::GetParamQuat(fparam)),
            ParamType::Curve1 => Self::Curve1(ffi::GetParamCurve1(fparam)),
            ParamType::Curve2 => Self::Curve2(ffi::GetParamCurve2(fparam)),
            ParamType::Curve3 => Self::Curve3(ffi::GetParamCurve3(fparam)),
            ParamType::Curve4 => Self::Curve4(ffi::GetParamCurve4(fparam)),
            ParamType::String32 => Self::String32(ffi::GetParamString(fparam)),
            ParamType::String64 => Self::String64(ffi::GetParamString(fparam)),
            ParamType::String256 => Self::String256(ffi::GetParamString(fparam)),
            ParamType::StringRef => Self::StringRef(ffi::GetParamString(fparam)),
            ParamType::BufferInt => Self::BufferInt(ffi::GetParamBufInt(fparam)),
            ParamType::BufferF32 => Self::BufferF32(ffi::GetParamBufF32(fparam)),
            ParamType::BufferU32 => Self::BufferU32(ffi::GetParamBufU32(fparam)),
            ParamType::BufferBinary => Self::BufferBinary(ffi::GetParamBufBin(fparam)),
            _ => unreachable!(),
        }
    }
}

impl Parameter {
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

    pub(crate) fn as_bool(&self) -> bool {
        if let Self::Bool(v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_f32(&self) -> f32 {
        if let Self::F32(v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_int(&self) -> i32 {
        if let Self::Int(v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_vec2(&self) -> &Vector2f {
        if let Self::Vec2(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_vec3(&self) -> &Vector3f {
        if let Self::Vec3(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_vec4(&self) -> &Vector4f {
        if let Self::Vec4(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_color(&self) -> &Color {
        if let Self::Color(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_string32(&self) -> &str {
        if let Self::String32(v) = self {
            v.as_str()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_string64(&self) -> &str {
        if let Self::String64(v) = self {
            v.as_str()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_curve1(&self) -> &[Curve; 1] {
        if let Self::Curve1(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_curve2(&self) -> &[Curve; 2] {
        if let Self::Curve2(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_curve3(&self) -> &[Curve; 3] {
        if let Self::Curve3(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_curve4(&self) -> &[Curve; 4] {
        if let Self::Curve4(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_buf_int(&self) -> &[i32] {
        if let Self::BufferInt(v) = self {
            v.as_slice()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_buf_f32(&self) -> &[f32] {
        if let Self::BufferF32(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_string_256(&self) -> &str {
        if let Self::String256(v) = self {
            v.as_str()
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_quat(&self) -> &Quat {
        if let Self::Quat(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_u32(&self) -> u32 {
        if let Self::U32(v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_buf_u32(&self) -> &[u32] {
        if let Self::BufferU32(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_buf_bin(&self) -> &[u8] {
        if let Self::BufferBinary(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub(crate) fn as_str_ref(&self) -> &str {
        if let Self::StringRef(v) = self {
            v.as_str()
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ParameterObject(IndexMap<u32, Parameter>);

impl PartialEq for ParameterObject {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len() && self.0.iter().zip(other.0.iter()).all(|(e, e2)| e == e2)
    }
}

impl From<cxx::UniquePtr<ffi::ParameterObject>> for ParameterObject {
    fn from(pobj: cxx::UniquePtr<ffi::ParameterObject>) -> Self {
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
    /// Attempt to get a `Parameter` by name, returns None if not found
    pub fn param(&self, name: &str) -> Option<&Parameter> {
        self.0.get(&checksum_ieee(name.as_bytes()))
    }

    /// Sets a parameter value
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

    pub(crate) fn size(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn hash_at(&self, i: usize) -> u32 {
        *self.0.keys().nth(i).unwrap()
    }

    pub(crate) fn val_at(&self, i: usize) -> &Parameter {
        self.0.values().nth(i).unwrap()
    }
}

trait ParamList {
    fn lists(&self) -> &IndexMap<u32, ParameterList>;
    fn objects(&self) -> &IndexMap<u32, ParameterObject>;
    fn lists_mut(&mut self) -> &mut IndexMap<u32, ParameterList>;
    fn objects_mut(&mut self) -> &mut IndexMap<u32, ParameterObject>;
    fn list(&self, name: &str) -> Option<&ParameterList> {
        self.lists().get(&checksum_ieee(name.as_bytes()))
    }
    fn object(&self, name: &str) -> Option<&ParameterObject> {
        self.objects().get(&checksum_ieee(name.as_bytes()))
    }
    fn set_list(&mut self, name: &str, plist: ParameterList) {
        self.lists_mut()
            .insert(checksum_ieee(name.as_bytes()), plist);
    }
    fn set_object(&mut self, name: &str, pobj: ParameterObject) {
        self.objects_mut()
            .insert(checksum_ieee(name.as_bytes()), pobj);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterList {
    lists: IndexMap<u32, ParameterList>,
    objects: IndexMap<u32, ParameterObject>,
}

impl From<&ffi::ParameterList> for ParameterList {
    fn from(plist: &ffi::ParameterList) -> Self {
        let list_map = ffi::GetParamLists(plist);
        let lists = (0usize..list_map.size())
            .map(|i| {
                let pair = ffi::GetParamListAt(list_map, i);
                (pair.hash, pair.param.into())
            })
            .collect::<IndexMap<u32, ParameterList>>();
        let obj_map = ffi::GetParamObjs(plist);
        let objects = (0usize..obj_map.size())
            .map(|i| {
                let pair = ffi::GetParamObjAt(obj_map, i);
                (pair.hash, pair.param.into())
            })
            .collect::<IndexMap<u32, ParameterObject>>();
        Self { lists, objects }
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

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterIO {
    pub version: u32,
    pub r#type: String,
    lists: IndexMap<u32, ParameterList>,
    objects: IndexMap<u32, ParameterObject>,
}

impl From<cxx::UniquePtr<ffi::ParameterIO>> for ParameterIO {
    fn from(pio: cxx::UniquePtr<ffi::ParameterIO>) -> Self {
        let version = ffi::GetPioVersion(&pio);
        let r#type = ffi::GetPioType(&pio);
        let list_map = ffi::GetParamListsFromPio(&pio);
        let lists = (0usize..list_map.size())
            .map(|i| {
                let pair = ffi::GetParamListAt(list_map, i);
                (pair.hash, pair.param.into())
            })
            .collect::<IndexMap<u32, ParameterList>>();
        let obj_map = ffi::GetParamObjsFromPio(&pio);
        let objects = (0usize..obj_map.size())
            .map(|i| {
                let pair = ffi::GetParamObjAt(obj_map, i);
                (pair.hash, pair.param.into())
            })
            .collect::<IndexMap<u32, ParameterObject>>();
        Self {
            version,
            r#type,
            lists,
            objects,
        }
    }
}

impl ParameterIO {
    pub fn from_binary<B: AsRef<[u8]>>(data: B) -> Result<ParameterIO> {
        let data = data.as_ref();
        if &data[0..4] != b"AAMP" {
            return Err(AampError::MagicError(
                String::from_utf8_lossy(&data[0..4]).to_string(),
            ));
        }
        Ok(ffi::AampFromBinary(data.as_ref())?.into())
    }

    pub fn from_text<S: AsRef<str>>(text: S) -> Result<ParameterIO> {
        Ok(ffi::AampFromText(text.as_ref())?.into())
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
    pub fn to_text(&self) -> String {
        ffi::AampToText(&self)
    }

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
        self.r#type.as_str()
    }

    pub(crate) fn version(&self) -> u32 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use crc::crc32::checksum_ieee;

    use crate::aamp::ParamList;

    use super::{Parameter, ParameterIO};

    #[test]
    fn parse_aamps_binary() {
        for file in glob::glob("include/oead/test/aamp/files/**/*.b*")
            .unwrap()
            .filter_map(|f| f.ok())
        {
            let data = std::fs::read(&file).unwrap();
            ParameterIO::from_binary(&data).unwrap();
        }
    }

    #[test]
    fn parse_aamp_text() {
        let text = std::fs::read_to_string("include/oead/test/aamp/test.yml").unwrap();
        let pio = ParameterIO::from_text(&text).unwrap();
        assert_eq!(&pio.r#type, "oead_test");
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
        for file in glob::glob("include/oead/test/aamp/files/**/*.b*")
            .unwrap()
            .filter_map(|f| f.ok())
            .take(50)
        {
            let data = std::fs::read(&file).unwrap();
            let pio = ParameterIO::from_binary(&data).unwrap();
            let text = pio.to_text();
            let pio2 = ParameterIO::from_text(&text).unwrap();
            assert_eq!(pio, pio2);
        }
    }

    #[test]
    fn aamp_binary_roundtrip() {
        for file in glob::glob("include/oead/test/aamp/files/**/*.b*")
            .unwrap()
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
