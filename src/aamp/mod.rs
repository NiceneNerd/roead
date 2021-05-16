use crate::types::*;
use crate::ffi;
use crate::ffi::{Vector2f, Vector3f, Vector4f, Color, Quat, Curve};


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
    StringRef(String)
}

impl Parameter {
    fn from_ffi(fparam: &ffi::Parameter) -> Self {
        match fparam.GetType() {
            ffi::ParamType::Bool => Self::Bool(ffi::GetParamBool(fparam)),
            ffi::ParamType::F32 => Self::F32(ffi::GetParamF32(fparam)),
            ffi::ParamType::U32 => Self::U32(ffi::GetParamU32(fparam)),
            ffi::ParamType::Int => Self::Int(ffi::GetParamInt(fparam)),
            ffi::ParamType::Vec2 => Self::Vec2(ffi::GetParamVec2(fparam)),
            ffi::ParamType::Vec3 => Self::Vec3(ffi::GetParamVec3(fparam)),
            ffi::ParamType::Vec4 => Self::Vec4(ffi::GetParamVec4(fparam)),
            ffi::ParamType::Color => Self::Color(ffi::GetParamColor(fparam)),
            ffi::ParamType::Quat => Self::Quat(ffi::GetParamQuat(fparam)),
            _ => Self::Bool(true)
        }
    }
}
