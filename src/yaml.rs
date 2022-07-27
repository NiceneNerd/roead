use crate::{Error, Result};
use join_str::jstr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum TagBasedType {
    Bool,
    Str,
    Int,
    Float,
    Null,
}

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

pub(crate) enum ScalarValue {
    Null,
    Bool(bool),
    Int(u64),
    Float(f64),
    String(smartstring::alias::String),
}

fn is_infinity(input: &str) -> bool {
    matches!(
        input,
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF"
    )
}

fn is_negative_infinity(input: &str) -> bool {
    matches!(input, "-.inf" | "-.Inf" | "-.INF")
}

fn in_nan(input: &str) -> bool {
    matches!(input, ".nan" | ".NaN" | ".NAN")
}

/// Deliberately not compliant to the YAML 1.2 standard to get rid of unused features
/// that harm performance.
pub(crate) fn parse_scalar(
    tag_type: Option<TagBasedType>,
    value: &str,
    is_quoted: bool,
) -> Result<ScalarValue> {
    if tag_type == Some(TagBasedType::Bool) || matches!(value, "true" | "false") {
        Ok(ScalarValue::Bool(&value[..1] == "t"))
    } else {
        // Floating-point conversions.
        let is_possible_double = value.contains('.');
        if tag_type == Some(TagBasedType::Float)
            || (tag_type.is_none() && is_possible_double && !is_quoted)
        {
            if is_infinity(value) {
                return Ok(ScalarValue::Float(f64::INFINITY));
            } else if is_negative_infinity(value) {
                return Ok(ScalarValue::Float(f64::NEG_INFINITY));
            } else if in_nan(value) {
                return Ok(ScalarValue::Float(f64::NAN));
            } else {
                match lexical::parse(value.as_bytes()) {
                    Ok(v) => return Ok(ScalarValue::Float(v)),
                    Err(_) => {
                        if tag_type == Some(TagBasedType::Float) {
                            return Err(Error::InvalidDataD(jstr!("Invalid float: {value}")));
                        }
                    }
                }
            }
        }
        // Integer conversions. Not YAML 1.2 compliant: base 8 is not supported as it's not useful.
        if tag_type == Some(TagBasedType::Int)
            || (tag_type.is_none() && !value.is_empty() && !is_quoted)
        {
            match lexical::parse(value).or_else(|_| {
                lexical::parse_with_options::<
                    u64,
                    _,
                    { lexical::NumberFormatBuilder::hexadecimal() },
                >(value.trim_start_matches("0x"), &lexical::ParseIntegerOptions::default())
            }) {
                Ok(v) => return Ok(ScalarValue::Int(v)),
                Err(_) => {
                    if tag_type == Some(TagBasedType::Int) {
                        return Err(Error::InvalidDataD(jstr!("Invalid int: {value}")));
                    }
                }
            }
        }
        if tag_type == Some(TagBasedType::Null) || matches!(value, "null" | "~" | "NULL" | "Null") {
            Ok(ScalarValue::Null)
        } else {
            // Fall back to treating the value as a string.
            Ok(ScalarValue::String(value.into()))
        }
    }
}

fn string_needs_quotes(value: &str) -> bool {
    matches!(value, "true" | "false")
        || (value.contains('.')
            && (is_infinity(value)
                || is_negative_infinity(value)
                || in_nan(value)
                || lexical::parse::<f64, &[u8]>(value.as_bytes()).is_ok()))
        || lexical::parse::<u64, &[u8]>(value.as_bytes()).is_ok()
        || value == "null"
}
