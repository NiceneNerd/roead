use super::Byml;
use crate::yaml::*;
use yaml_peg::{parser::*, NodeRc};

type Result<T> = std::result::Result<T, YamlError>;

fn is_binary_tag(tag: &str) -> bool {
    tag == "tag:yaml.org,2002:binary" || tag == "!!binary"
}

fn recognize_tag(tag: &str) -> Option<TagBasedType> {
    match tag {
        "!f64" => Some(TagBasedType::Float),
        "!u" | "!l" | "!ul" => Some(TagBasedType::Int),
        "tag:yaml.org,2002:binary" | "!!binary" => Some(TagBasedType::Str),
        _ => None,
    }
}

fn node_to_value(node: &NodeRc) -> Result<Byml> {
    match node.yaml() {
        yaml_peg::Yaml::Null => Ok(Byml::Null),
        yaml_peg::Yaml::Bool(b) => Ok(Byml::Bool(*b)),
        yaml_peg::Yaml::Int(_) => todo!(),
        yaml_peg::Yaml::Float(_) => todo!(),
        yaml_peg::Yaml::Str(_) => todo!(),
        yaml_peg::Yaml::Seq(_) => todo!(),
        yaml_peg::Yaml::Map(_) => todo!(),
        yaml_peg::Yaml::Alias(_) => todo!(),
    }
}
