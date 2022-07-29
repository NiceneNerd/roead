use super::*;
use crate::{yaml::*, Error, Result};
use ryml::{NodeRef, Tree};

impl Byml {
    /// Parse BYML document from YAML text.
    pub fn from_text(text: impl AsRef<str>) -> Result<Byml> {
        Parser::new(text.as_ref()).unwrap().parse()
    }

    /// Serialize the document to YAML. This can only be done for Null, Array,
    /// or Hash nodes.
    pub fn to_text(&self) -> Result<std::string::String> {
        Emitter::new(self).emit()
    }
}

#[inline]
fn is_binary_tag(tag: &str) -> bool {
    tag == "tag:yaml.org,2002:binary" || tag == "!!binary"
}

#[inline]
fn recognize_tag(tag: &str) -> Option<TagBasedType> {
    match tag {
        "!f64" => Some(TagBasedType::Float),
        "!u" | "!l" | "!ul" => Some(TagBasedType::Int),
        "tag:yaml.org,2002:binary" | "!!binary" => Some(TagBasedType::Str),
        _ => None,
    }
}

struct Parser<'a>(Tree<'a>);

impl<'a> Parser<'a> {
    fn new(text: &str) -> Result<Self> {
        Ok(Self(Tree::parse(text).unwrap()))
    }

    fn parse_node(&self, node: NodeRef<'a, '_, '_, &Tree<'a>>) -> Result<Byml> {
        if node.is_map().unwrap() {
            Ok(Byml::Hash(
                node.iter()
                    .unwrap()
                    .map(|child| {
                        let key = child.key().unwrap();
                        let value = self.parse_node(child.clone()).unwrap();
                        Ok((key.into(), value))
                    })
                    .collect::<Result<_>>()
                    .unwrap(),
            ))
        } else if node.is_seq().unwrap() {
            Ok(Byml::Array(
                node.iter()
                    .unwrap()
                    .map(|child| self.parse_node(child.clone()))
                    .collect::<Result<_>>()
                    .unwrap(),
            ))
        } else {
            let tag = node.val_tag().unwrap_or("");
            let tag_type = get_tag_based_type(tag).or_else(|| recognize_tag(tag));
            let scalar =
                parse_scalar(tag_type, node.val().unwrap(), node.is_quoted().unwrap()).unwrap();
            match scalar {
                Scalar::Bool(b) => Ok(Byml::Bool(b)),
                Scalar::Float(f) => match tag {
                    "!f64" => Ok(Byml::Double(f)),
                    _ => Ok(Byml::Float(f as f32)),
                },
                Scalar::Int(i) => match tag {
                    "!u" => Ok(Byml::U32(i as u32)),
                    "!ul" => Ok(Byml::U64(i as u64)),
                    "!l" => Ok(Byml::I64(i)),
                    _ => Ok(Byml::I32(i as i32)),
                },
                Scalar::Null => Ok(Byml::Null),
                Scalar::String(s) => {
                    if is_binary_tag(tag) {
                        Ok(Byml::BinaryData(base64::decode(s).unwrap()))
                    } else {
                        Ok(Byml::String(s))
                    }
                }
            }
        }
    }

    fn parse(self) -> Result<Byml> {
        let root = self.0.root_ref().unwrap();
        self.parse_node(root)
    }
}

#[inline(always)]
fn should_use_inline(byml: &Byml) -> bool {
    let is_simple = |by: &Byml| !matches!(by, Byml::Array(_) | Byml::Hash(_));
    match byml {
        Byml::Array(arr) => arr.len() < 10 && arr.iter().all(is_simple),
        Byml::Hash(hash) => hash.len() < 10 && hash.iter().all(|(_, v)| is_simple(v)),
        _ => false,
    }
}

struct Emitter<'a, 'b>(&'a Byml, Tree<'b>);

impl<'a, 'b> Emitter<'a, 'b> {
    fn new(byml: &'a Byml) -> Self {
        let mut tree = Tree::default();
        tree.reserve(20000);
        Self(byml, tree)
    }

    fn build_node<'e>(
        byml: &Byml,
        mut dest_node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
    ) -> Result<()> {
        match byml {
            Byml::Array(array) => {
                if should_use_inline(byml) {
                    dest_node
                        .change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)
                        .unwrap();
                } else {
                    dest_node.change_type(ryml::NodeType::Seq).unwrap();
                }
                for item in array {
                    let node = dest_node.append_child().unwrap();
                    Self::build_node(item, node).unwrap();
                }
            }
            Byml::Hash(hash) => {
                if should_use_inline(byml) {
                    dest_node
                        .change_type(ryml::NodeType::Map | ryml::NodeType::WipStyleFlowSl)
                        .unwrap();
                } else {
                    dest_node.change_type(ryml::NodeType::Map).unwrap();
                }
                for (key, value) in hash {
                    let mut node = dest_node.append_child().unwrap();
                    node.set_key(key).unwrap();
                    if string_needs_quotes(key) {
                        let flags = node.node_type().unwrap();
                        node.set_type_flags(flags | ryml::NodeType::WipKeySquo)
                            .unwrap();
                    }
                    Self::build_node(value, node).unwrap();
                }
            }
            scalar => match scalar {
                Byml::String(s) => {
                    dest_node.set_val(s).unwrap();
                    if string_needs_quotes(s) {
                        let flags = dest_node.node_type().unwrap();
                        dest_node
                            .set_type_flags(flags | ryml::NodeType::WipValDquo)
                            .unwrap();
                    }
                }
                Byml::Bool(b) => dest_node
                    .set_val(if *b { "true" } else { "false" })
                    .unwrap(),
                Byml::Float(f) => dest_node.set_val(&lexical::to_string(*f)).unwrap(),
                Byml::Double(d) => {
                    dest_node.set_val(&lexical::to_string(*d)).unwrap();
                    dest_node.set_val_tag("!f64").unwrap();
                }
                Byml::I32(i) => dest_node.set_val(&lexical::to_string(*i)).unwrap(),
                Byml::I64(i) => {
                    dest_node.set_val(&lexical::to_string(*i)).unwrap();
                    dest_node.set_val_tag("!l").unwrap();
                }
                Byml::U32(u) => {
                    dest_node.set_val(&format_hex!(u)).unwrap();
                    dest_node.set_val_tag("!u").unwrap();
                }
                Byml::U64(u) => {
                    dest_node.set_val(&format_hex!(u)).unwrap();
                    dest_node.set_val_tag("!ul").unwrap();
                }
                Byml::Null => dest_node.set_val("null").unwrap(),
                Byml::BinaryData(data) => {
                    let arena = dest_node.tree().arena_capacity();
                    dest_node.tree_mut().reserve_arena(arena + data.len());
                    dest_node.set_val(&base64::encode(&data)).unwrap();
                    dest_node.set_val_tag("!!binary").unwrap();
                }
                _ => unsafe { std::hint::unreachable_unchecked() },
            },
        }
        Ok(())
    }

    fn emit(self) -> Result<std::string::String> {
        let Self(byml, mut tree) = self;
        match byml {
            Byml::Hash(_) => tree.to_map(0).unwrap(),
            Byml::Array(_) => tree.to_seq(0).unwrap(),
            Byml::Null => return Ok("null".to_string()),
            _ => {
                return Err(Error::Any(
                    "Can only serialize Hash, Array, or Null nodes to YAML".into(),
                ));
            }
        };
        Self::build_node(byml, tree.root_ref_mut().unwrap()).unwrap();
        Ok(tree.emit().unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_text() {
        for file in crate::byml::FILES {
            println!("{}", file);
            let text = std::fs::read_to_string(
                std::path::Path::new("test/byml").join([file, ".yml"].join("")),
            )
            .unwrap();
            let byml = Byml::from_text(text).unwrap();
            let bytes =
                std::fs::read(std::path::Path::new("test/byml").join([file, ".byml"].join("")))
                    .unwrap();
            let binary_byml = Byml::from_binary(bytes).unwrap();
            if byml != binary_byml {
                for (v1, v2) in byml["Actors"]
                    .array_ref()
                    .unwrap()
                    .iter()
                    .zip(binary_byml["Actors"].array_ref().unwrap().iter())
                {
                    assert_eq!(v1, v2);
                }
            }
        }
    }

    #[test]
    fn text_roundtrip() {
        for file in crate::byml::FILES {
            println!("{}", file);
            let text = std::fs::read_to_string(
                std::path::Path::new("test/byml").join([file, ".yml"].join("")),
            )
            .unwrap();
            let byml = Byml::from_text(text).unwrap();
            let text = byml.to_text().unwrap();
            let byml = Byml::from_text(text).unwrap();
            assert_eq!(byml, byml);
        }
    }
}
