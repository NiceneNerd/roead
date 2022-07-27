use super::*;
use crate::{yaml::*, Error, Result};
use ryml::{NodeRef, Tree};
use smartstring::alias::String;

impl Byml {
    #[allow(missing_docs)]
    pub fn from_text(text: impl AsRef<str>) -> Result<Byml> {
        Parser::new(text.as_ref())?.parse()
    }
}

fn is_binary_tag(tag: &str) -> bool {
    tag == "tag:yaml.org,2002:binary" || tag == "!!binary"
}

fn recognize_tag(tag: &str) -> Option<TagBasedType> {
    match tag {
        "f64" => Some(TagBasedType::Float),
        "u" | "l" | "ul" => Some(TagBasedType::Int),
        "tag:yaml.org,2002:binary" | "binary" => Some(TagBasedType::Str),
        _ => None,
    }
}

struct Parser<'a>(Tree<'a>);

impl<'a> Parser<'a> {
    fn new(text: &str) -> Result<Self> {
        Ok(Self(Tree::parse(text)?))
    }

    fn parse_node(&self, node: NodeRef<'a, '_, '_, &Tree<'a>>) -> Result<Byml> {
        if node.is_map()? {
            Ok(Byml::Hash(
                node.iter()?
                    .map(|child| {
                        let key = child.key()?;
                        let value = self.parse_node(child.clone())?;
                        Ok((key.into(), value))
                    })
                    .collect::<Result<_>>()?,
            ))
        } else if node.is_seq()? {
            Ok(Byml::Array(
                node.iter()?
                    .map(|child| self.parse_node(child.clone()))
                    .collect::<Result<_>>()?,
            ))
        } else {
            let tag = node.val_tag().unwrap_or("");
            let tag_type = get_tag_based_type(tag).or_else(|| recognize_tag(tag));
            let scalar = parse_scalar(tag_type, node.val()?, node.is_quoted()?)?;
            match scalar {
                ScalarValue::Bool(b) => Ok(Byml::Bool(b)),
                ScalarValue::Float(f) => match tag {
                    "!f64" => Ok(Byml::Double(f)),
                    _ => Ok(Byml::Float(f as f32)),
                },
                ScalarValue::Int(i) => match tag {
                    "u" => Ok(Byml::U32(i as u32)),
                    "ul" => Ok(Byml::U64(i)),
                    "l" => Ok(Byml::I64(i as i64)),
                    _ => Ok(Byml::I32(i as i32)),
                },
                ScalarValue::Null => Ok(Byml::Null),
                ScalarValue::String(s) => Ok(Byml::String(s)),
            }
        }
    }

    fn parse(self) -> Result<Byml> {
        let root = self.0.root_ref()?;
        self.parse_node(root)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_text() {
        for file in crate::byml::FILES {
            println!("{}", file);
            let bytes = std::fs::read_to_string(
                std::path::Path::new("test/byml").join([file, ".yml"].join("")),
            )
            .unwrap();
            let byml = Byml::from_text(bytes).unwrap();
            match byml {
                Byml::Array(arr) => println!("  Array with {} elements", arr.len()),
                Byml::Hash(hash) => println!("  Hash with {} entries", hash.len()),
                _ => println!("{:?}", byml),
            }
        }
    }
}
