use join_str::jstr;

use crate::{Error, Result};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum TagBasedType {
    Bool,
    Str,
    Int,
    Float,
    Null,
}

#[inline]
pub(crate) fn get_tag_based_type(tag: &str) -> Option<TagBasedType> {
    match tag {
        "tag:yaml.org,2002:str" | "Str" => Some(TagBasedType::Str),
        "tag:yaml.org,2002:float" | "Float" => Some(TagBasedType::Float),
        "tag:yaml.org,2002:int" | "Int" => Some(TagBasedType::Int),
        "tag:yaml.org,2002:bool" | "Bool" => Some(TagBasedType::Bool),
        "tag:yaml.org,2002:null" | "Null" => Some(TagBasedType::Null),
        _ => None,
    }
}

#[derive(Debug)]
pub(crate) enum Scalar {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    String(smartstring::alias::String),
}

#[inline]
fn is_infinity(input: &str) -> bool {
    matches!(
        input,
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF"
    )
}

#[inline]
fn is_negative_infinity(input: &str) -> bool {
    matches!(input, "-.inf" | "-.Inf" | "-.INF")
}

#[inline]
fn in_nan(input: &str) -> bool {
    matches!(input, ".nan" | ".NaN" | ".NAN")
}

#[inline]
// Integer conversions. Not YAML 1.2 compliant: base 8 is not supported as it's
// not useful.
fn parse_int(value: &str) -> Result<i128> {
    lexical::parse(value)
        .or_else(|_| {
            lexical::parse_with_options::<i128, _, { lexical::NumberFormatBuilder::hexadecimal() }>(
                value
                    .strip_prefix("0x")
                    .ok_or(lexical::Error::InvalidBasePrefix)?,
                &lexical::ParseIntegerOptions::default(),
            )
        })
        .map_err(|_| Error::InvalidDataD(jstr!("Invalid integer: {value}")))
}

#[inline]
// Floating-point conversions.
fn parse_float(value: &str) -> Result<f64> {
    if is_infinity(value) {
        Ok(f64::INFINITY)
    } else if is_negative_infinity(value) {
        Ok(f64::NEG_INFINITY)
    } else if in_nan(value) {
        Ok(f64::NAN)
    } else {
        lexical::parse(value.as_bytes())
            .map_err(|_| Error::InvalidDataD(jstr!("Invalid float: {value}")))
    }
}

/// Deliberately not compliant to the YAML 1.2 standard to get rid of unused
/// features that harm performance.
#[inline]
pub(crate) fn parse_scalar(
    tag_type: Option<TagBasedType>,
    value: &str,
    is_quoted: bool,
) -> Result<Scalar> {
    let is_possible_double = value.contains('.');
    if let Some(type_) = tag_type {
        match type_ {
            TagBasedType::Null => Ok(Scalar::Null),
            TagBasedType::Bool => Ok(Scalar::Bool(matches!(value, "true" | "True"))),
            TagBasedType::Int => Ok(Scalar::Int(parse_int(value)?)),
            TagBasedType::Float => Ok(Scalar::Float(parse_float(value)?)),
            TagBasedType::Str => Ok(Scalar::String(value.into())),
        }
    } else if matches!(value, "true" | "false") {
        Ok(Scalar::Bool(&value[..1] == "t"))
    } else if let Some(float) = is_possible_double
        .then(|| (!is_quoted).then(|| parse_float(value).ok()))
        .flatten()
        .flatten()
    {
        Ok(Scalar::Float(float))
    } else if let Some(int) = (!value.is_empty())
        .then(|| (!is_quoted).then(|| parse_int(value).ok()))
        .flatten()
        .flatten()
    {
        Ok(Scalar::Int(int))
    } else if matches!(value, "null" | "~" | "NULL") {
        Ok(Scalar::Null)
    } else {
        // Fall back to treating the value as a string.
        Ok(Scalar::String(value.into()))
    }
}

#[inline]
pub(crate) fn string_needs_quotes(value: &str) -> bool {
    matches!(value, "true" | "false")
        || value.starts_with('!')
        || (value.contains('.')
            && (is_infinity(value)
                || is_negative_infinity(value)
                || in_nan(value)
                || lexical::parse::<f64, &[u8]>(value.as_bytes()).is_ok()))
        || lexical::parse::<u64, &[u8]>(value.as_bytes()).is_ok()
        || value == "null"
        || value == "!"
        || value == "NULL"
}

macro_rules! format_hex {
    ($val:expr) => {
        [
            "0x",
            &lexical::to_string_with_options::<_, { lexical::NumberFormatBuilder::hexadecimal() }>(
                *$val,
                &Default::default(),
            ),
        ]
        .join("")
    };
}
pub(crate) use format_hex;
