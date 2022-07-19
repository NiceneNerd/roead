use crate::types::*;
use decorum::R32;
use enum_as_inner::EnumAsInner;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use smartstring::alias::String;

#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumAsInner)]
pub enum Parameter {
    Bool(bool),
    F32(R32),
    Int(i32),
    Vec2(Vector2f),
    Vec3(Vector3f),
    Vec4(Vector4f),
    Color(Color),
    String32(FixedSafeString<32>),
    String64(FixedSafeString<64>),
    Curve1([Curve; 1]),
    Curve2([Curve; 2]),
    Curve3([Curve; 3]),
    Curve4([Curve; 4]),
    BufferInt(Vec<i32>),
    BufferF32(Vec<R32>),
    String256(FixedSafeString<256>),
    Quat(Quat),
    U32(u32),
    BufferU32(Vec<u32>),
    BufferBinary(Vec<u8>),
    StringRef(String),
}
