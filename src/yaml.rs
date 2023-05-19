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

/// Deliberately not compliant to the YAML 1.2 standard to get rid of unused
/// features that harm performance.
#[inline]
pub(crate) fn parse_scalar(
    tag_type: Option<TagBasedType>,
    value: &str,
    is_quoted: bool,
) -> Result<Scalar> {
    if tag_type == Some(TagBasedType::Bool) || matches!(value, "true" | "false") {
        Ok(Scalar::Bool(&value[..1] == "t"))
    } else {
        // Floating-point conversions.
        let is_possible_double = value.contains('.');
        if tag_type == Some(TagBasedType::Float)
            || (tag_type.is_none() && is_possible_double && !is_quoted)
        {
            if is_infinity(value) {
                return Ok(Scalar::Float(f64::INFINITY));
            } else if is_negative_infinity(value) {
                return Ok(Scalar::Float(f64::NEG_INFINITY));
            } else if in_nan(value) {
                return Ok(Scalar::Float(f64::NAN));
            } else {
                match lexical::parse(value.as_bytes()) {
                    Ok(v) => return Ok(Scalar::Float(v)),
                    Err(_) => {
                        if tag_type == Some(TagBasedType::Float) {
                            return Err(Error::InvalidDataD(jstr!("Invalid float: {value}")));
                        }
                    }
                }
            }
        }
        // Integer conversions. Not YAML 1.2 compliant: base 8 is not supported as it's
        // not useful.
        if tag_type == Some(TagBasedType::Int)
            || (tag_type.is_none() && !value.is_empty() && !is_quoted)
        {
            match lexical::parse(value) {
                Ok(v) => return Ok(Scalar::Int(v)),
                Err(e) => {
                    if tag_type == Some(TagBasedType::Int) {
                        if value.starts_with("0x") {
                            match lexical::parse_with_options::<
                                i128,
                                _,
                                { lexical::NumberFormatBuilder::hexadecimal() },
                            >(
                                value.trim_start_matches("0x"),
                                &lexical::ParseIntegerOptions::default(),
                            ) {
                                Ok(v) => return Ok(Scalar::Int(v)),
                                Err(_) => {
                                    return Err(Error::InvalidDataD(jstr!(
                                        "Invalid integer: {value}"
                                    )));
                                }
                            }
                        }
                    } else if tag_type == Some(TagBasedType::Int) {
                        return Err(Error::InvalidDataD(jstr!("Invalid integer: {value}")));
                    }
                }
            }
        }
        if tag_type == Some(TagBasedType::Null) || matches!(value, "null" | "~" | "NULL" | "Null") {
            Ok(Scalar::Null)
        } else {
            // Fall back to treating the value as a string.
            Ok(Scalar::String(value.into()))
        }
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
