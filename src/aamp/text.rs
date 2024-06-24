use lexical::{FromLexical, FromLexicalWithOptions, ToLexical, ToLexicalWithOptions};
use ryml::*;

use super::*;
use crate::{types::*, yaml::*, Error, Result};

impl ParameterIO {
    /// Parse ParameterIO from YAML text.
    pub fn from_text(text: impl AsRef<str>) -> Result<Self> {
        let tree = Tree::parse(text.as_ref())?;
        let root_ref = tree.root_ref()?;
        read_parameter_io(&root_ref)
    }

    /// Serialize the parameter IO to YAML.
    pub fn to_text(&self) -> std::string::String {
        let mut tree = Tree::default();
        tree.reserve(10000);
        write_parameter_io(&mut tree, self)
            .expect("ParameterIO should serialize to YAML without error");
        tree.emit()
            .expect("ParameterIO should serialize to YAML without error")
    }
}

#[inline(always)]
fn recognize_tag(tag: &str) -> Option<TagBasedType> {
    match tag {
        "!str32" | "!str64" | "!str256" => Some(TagBasedType::Str),
        "!u" => Some(TagBasedType::Int),
        _ => None,
    }
}

fn scalar_to_value(tag: &str, scalar: Scalar) -> Result<Parameter> {
    Ok(match scalar {
        Scalar::String(s) => {
            match tag {
                "!str32" => Parameter::String32(s.into()),
                "!str64" => Parameter::String64(Box::new(s.into())),
                "!str256" => Parameter::String256(Box::new(s.into())),
                _ => Parameter::StringRef(s),
            }
        }
        Scalar::Int(i) => {
            if tag == "!u" {
                Parameter::U32(i as u32)
            } else {
                Parameter::I32(i as i32)
            }
        }
        Scalar::Float(f) => Parameter::F32(f as f32),
        Scalar::Bool(b) => Parameter::Bool(b),
        Scalar::Null => {
            match tag {
                "!str32" => Parameter::String32(Default::default()),
                "!str64" => Parameter::String64(Default::default()),
                "!str256" => Parameter::String256(Default::default()),
                _ => Parameter::StringRef(Default::default()),
            }
        }
    })
}

#[inline(always)]
fn parse_num<'a, 't, T>(node: &NodeRef<'a, 't, '_, &'t Tree<'a>>) -> Result<T>
where
    T: FromLexicalWithOptions + FromLexical,
{
    let val = node.val()?;
    match T::from_lexical(val.as_bytes()) {
        Ok(v) => Ok(v),
        Err(_) => {
            let opts = T::Options::default();
            Ok(T::from_lexical_with_options::<
                { lexical::NumberFormatBuilder::hexadecimal() },
            >(
                val.trim_start_matches("0x").as_bytes(), &opts
            )?)
        }
    }
}

macro_rules! impl_from_node_for_struct {
    ($type:tt, $($field:tt),+) => {
        impl<'a, 't, 'k, 'r> TryFrom<&'r NodeRef<'a, 't, 'k, &'t Tree<'a>>> for $type {
            type Error = Error;
            fn try_from(node: &'r NodeRef<'a, 't, 'k, &'t Tree<'a>>) -> Result<Self>
            {
                let mut iter = node.iter()?;
                let result = $type {
                    $(
                        $field: parse_num(
                            &iter.next()
                                .ok_or(Error::InvalidData(concat!(stringify!($type), " missing field", stringify!($field))))?
                        )?,
                    )+
                };
                Ok(result)
            }
        }
    };
}
impl_from_node_for_struct!(Vector2f, x, y);
impl_from_node_for_struct!(Vector3f, x, y, z);
impl_from_node_for_struct!(Vector4f, x, y, z, t);
impl_from_node_for_struct!(Quat, a, b, c, d);
impl_from_node_for_struct!(Color, r, g, b, a);

fn read_curves<'a, 't, const N: usize>(
    node: &NodeRef<'a, 't, '_, &'t Tree<'a>>,
) -> Result<[Curve; N]> {
    let mut iter = node.iter()?;
    let mut curves = [Curve::default(); N];
    for curve in &mut curves {
        curve.a = parse_num(
            &iter
                .next()
                .ok_or(Error::InvalidData("YAML curve missing a"))?,
        )?;
        curve.b = parse_num(
            &iter
                .next()
                .ok_or(Error::InvalidData("YAML curve missing a"))?,
        )?;
        for f in &mut curve.floats {
            *f = parse_num(
                &iter
                    .next()
                    .ok_or(Error::InvalidData("YAML curve missing a float"))?,
            )?;
        }
    }
    Ok(curves)
}

#[inline(always)]
fn read_buf<'a, 't, T: FromLexical + FromLexicalWithOptions>(
    node: &NodeRef<'a, 't, '_, &'t Tree<'a>>,
) -> Result<Vec<T>> {
    node.iter()?
        .map(|node| parse_num(&node))
        .collect::<Result<_>>()
}

fn parse_parameter<'a, 't>(node: &'_ NodeRef<'a, 't, '_, &'t Tree<'a>>) -> Result<Parameter> {
    if !node.is_valid() {
        return Err(Error::InvalidData("Invalid YAML node for parameter"));
    }
    let tag = node.val_tag().unwrap_or("");
    let param = if node.is_seq()? {
        match tag {
            "!vec2" => Vector2f::try_from(node)?.into(),
            "!vec3" => Vector3f::try_from(node)?.into(),
            "!vec4" => Vector4f::try_from(node)?.into(),
            "!quat" => Quat::try_from(node)?.into(),
            "!color" => Color::try_from(node)?.into(),
            "!curve" => {
                match node.num_children()? {
                    32 => read_curves::<1>(node)?.into(),
                    64 => read_curves::<2>(node)?.into(),
                    96 => read_curves::<3>(node)?.into(),
                    128 => read_curves::<4>(node)?.into(),
                    _ => return Err(Error::InvalidData("Invalid curve: wrong number of values")),
                }
            }
            "!buffer_int" => read_buf::<i32>(node)?.into(),
            "!buffer_f32" => read_buf::<f32>(node)?.into(),
            "!buffer_u32" => read_buf::<u32>(node)?.into(),
            "!buffer_binary" => read_buf::<u8>(node)?.into(),
            _ => {
                return Err(Error::InvalidData(
                    "Invalid parameter: sequence without known tag",
                ));
            }
        }
    } else {
        let tag_type = recognize_tag(tag).or_else(|| get_tag_based_type(tag));
        scalar_to_value(tag, parse_scalar(tag_type, node.val()?, node.is_quoted()?)?)?
    };
    Ok(param)
}

#[rustfmt::skip]
macro_rules! read_map {
    ($node:expr, $m:expr, $fn:expr) => {
        if !$node.is_map()? {
            return Err(Error::InvalidData("Expected map node"));
        }

        for child in $node.iter()? {
            let key = child.key()?;
            let value = $fn(&child)?;
            if !$node.is_key_quoted()? {
                if let Ok(hash) = lexical::parse::<u64, &str>(key) {
                    $m.insert(hash as u32, value);
                    continue;
                }
            }
            $m.insert(hash_name(key), value);
        }
    };
}

fn read_parameter_object<'a, 't>(
    node: &'_ NodeRef<'a, 't, '_, &'t Tree<'a>>,
) -> Result<ParameterObject> {
    if !node.is_valid() {
        return Err(Error::InvalidData("Invalid YAML node for parameter object"));
    }
    let mut param_object = ParameterObject::default();
    read_map!(node, param_object, parse_parameter);
    Ok(param_object)
}

fn read_parameter_list<'a, 't>(
    node: &'_ NodeRef<'a, 't, '_, &'t Tree<'a>>,
) -> Result<ParameterList> {
    if !node.is_valid() {
        return Err(Error::InvalidData("Invalid YAML node for parameter list"));
    }
    let mut param_list = ParameterList::default();
    let lists = node.get("lists")?;
    let objects = node.get("objects")?;
    read_map!(&objects, param_list.objects, read_parameter_object);
    read_map!(&lists, param_list.lists, read_parameter_list);
    Ok(param_list)
}

fn read_parameter_io<'a, 't>(node: &'_ NodeRef<'a, 't, '_, &'t Tree<'a>>) -> Result<ParameterIO> {
    if !node.is_valid() {
        return Err(Error::InvalidData("Invalid YAML node for parameter IO"));
    }
    let pio = ParameterIO {
        version: {
            let ver = node.get("version")?;
            parse_num(&ver)?
        },
        data_type: {
            let dt = node.get("type")?;
            dt.val()?.into()
        },
        param_root: {
            let pr = node.get("param_root")?;
            read_parameter_list(&pr)?
        },
    };
    Ok(pio)
}

macro_rules! fill_node_from_struct {
    ($node:expr, $tag:literal, $struct:expr, $($field:tt),+) => {{
        $node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
        $(
            let mut _child = $node.append_child()?;
            _child.set_val(&lexical::to_string($struct.$field))?;
        )+
        $node.set_val_tag($tag)?;
    }};
}

fn write_curves<'a, 't, const N: usize>(
    mut node: NodeRef<'a, 't, '_, &'t mut Tree<'a>>,
    curves: &[Curve; N],
) -> Result<()> {
    node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
    for curve in curves {
        let mut a = node.append_child()?;
        a.set_val(&lexical::to_string(curve.a))?;
        let mut b = node.append_child()?;
        b.set_val(&lexical::to_string(curve.b))?;
        for float in curve.floats {
            let mut f = node.append_child()?;
            f.set_val(&lexical::to_string(float))?;
        }
    }
    node.set_val_tag("!curve")?;
    Ok(())
}

#[inline]
fn write_buf<'a, 't, T: ToLexical + ToLexicalWithOptions>(
    mut node: NodeRef<'a, 't, '_, &'t mut Tree<'a>>,
    buf: &[T],
    use_hex: bool,
    tag: &str,
) -> Result<()> {
    node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
    for val in buf {
        let mut child = node.append_child()?;
        let val = if use_hex {
            format_hex!(val)
        } else {
            lexical::to_string(*val)
        };
        child.set_val(&val)?;
    }
    node.set_val_tag(tag)?;
    Ok(())
}

fn write_parameter<'a, 't>(
    param: &Parameter,
    mut node: NodeRef<'a, 't, '_, &'t mut Tree<'a>>,
) -> Result<()> {
    match param {
        Parameter::Bool(b) => node.set_val(if *b { "true" } else { "false" })?,
        Parameter::F32(f) => node.set_val(&lexical::to_string(*f))?,
        Parameter::I32(i) => node.set_val(&lexical::to_string(*i))?,
        Parameter::Vec2(v) => fill_node_from_struct!(node, "!vec2", v, x, y),
        Parameter::Vec3(v) => fill_node_from_struct!(node, "!vec3", v, x, y, z),
        Parameter::Vec4(v) => fill_node_from_struct!(node, "!vec4", v, x, y, z, t),
        Parameter::Color(c) => fill_node_from_struct!(node, "!color", c, r, g, b, a),
        Parameter::String32(s) => {
            node.set_val(s)?;
            node.set_val_tag("!str32")?;
        }
        Parameter::String64(s) => {
            node.set_val(s)?;
            node.set_val_tag("!str64")?;
        }
        Parameter::Curve1(c) => write_curves(node, c)?,
        Parameter::Curve2(c) => write_curves(node, c)?,
        Parameter::Curve3(c) => write_curves(node, c)?,
        Parameter::Curve4(c) => write_curves(node, c)?,
        Parameter::BufferInt(buf) => {
            write_buf(node, buf, false, "!buffer_int")?;
        }
        Parameter::BufferF32(buf) => {
            write_buf(node, buf, false, "!buffer_f32")?;
        }
        Parameter::String256(s) => {
            node.set_val(s)?;
            node.set_val_tag("!str256")?;
        }
        Parameter::Quat(q) => fill_node_from_struct!(node, "!quat", q, a, b, c, d),
        Parameter::U32(u) => {
            node.set_val(&format_hex!(u))?;
            node.set_val_tag("!u")?;
        }
        Parameter::BufferU32(buf) => {
            write_buf(node, buf, true, "!buffer_u32")?;
        }
        Parameter::BufferBinary(buf) => {
            write_buf(node, buf, true, "!buffer_binary")?;
        }
        Parameter::StringRef(s) => {
            if string_needs_quotes(s) {
                let ty = node.node_type()?;
                node.set_type_flags(ty | ryml::NodeType::WipValDquo)?;
            }
            node.set_val(s)?
        }
    }
    Ok(())
}

fn write_parameter_object<'a, 't>(
    pobj: &ParameterObject,
    parent_hash: u32,
    mut node: NodeRef<'a, 't, '_, &'t mut Tree<'a>>,
) -> Result<()> {
    node.change_type(ryml::NodeType::Map)?;
    for (i, (key, val)) in pobj.0.iter().enumerate() {
        let mut child = node.append_child()?;
        if let Some(name) = get_default_name_table().get_name(key.0, i, parent_hash) {
            if lexical::parse::<u64, _>(name.as_bytes()).is_ok() {
                let ty = child.node_type()?;
                child.set_type_flags(ty | ryml::NodeType::WipKeyDquo)?;
            }
            child.set_key(name)?;
        } else {
            child.set_key(&lexical::to_string(key.0))?;
        }
        write_parameter(val, child)?;
    }
    node.set_val_tag("!obj")?;
    Ok(())
}

fn write_parameter_list<'a, 't>(
    plist: &ParameterList,
    parent_hash: u32,
    mut node: NodeRef<'a, 't, '_, &'t mut Tree<'a>>,
) -> Result<()> {
    node.change_type(ryml::NodeType::Map)?;
    let mut objects = node.append_child()?;
    objects.set_key("objects")?;
    objects.change_type(ryml::NodeType::Map)?;
    for (i, (key, val)) in plist.objects.0.iter().enumerate() {
        let mut child = objects.append_child()?;
        if let Some(name) = get_default_name_table().get_name(key.0, i, parent_hash) {
            if lexical::parse::<u64, _>(name.as_bytes()).is_ok() {
                let ty = child.node_type()?;
                child.set_type_flags(ty | ryml::NodeType::WipKeyDquo)?;
            }
            child.set_key(name)?;
        } else {
            child.set_key(&lexical::to_string(key.0))?;
        }
        write_parameter_object(val, key.0, child)?;
    }
    let mut lists = node.append_child()?;
    lists.set_key("lists")?;
    lists.change_type(ryml::NodeType::Map)?;
    for (i, (key, val)) in plist.lists.0.iter().enumerate() {
        let mut child = lists.append_child()?;
        if let Some(name) = get_default_name_table().get_name(key.0, i, parent_hash) {
            if lexical::parse::<u64, _>(name.as_bytes()).is_ok() {
                let ty = child.node_type()?;
                child.set_type_flags(ty | ryml::NodeType::WipKeyDquo)?;
            }
            child.set_key(name)?;
        } else {
            child.set_key(&lexical::to_string(key.0))?;
        }
        write_parameter_list(val, key.0, child)?;
    }
    node.set_val_tag("!list")?;
    Ok(())
}

fn write_parameter_io(tree: &mut Tree<'_>, pio: &ParameterIO) -> Result<()> {
    let mut root = tree.root_ref_mut()?;
    root.change_type(ryml::NodeType::Map)?;
    root.set_val_tag("!io")?;
    root.get_mut("version")?
        .set_val(&lexical::to_string(pio.version))?;
    root.get_mut("type")?.set_val(&pio.data_type)?;
    let mut param_root = root.append_child()?;
    param_root.set_key("param_root")?;
    write_parameter_list(&pio.param_root, ROOT_KEY.0, param_root)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_names() {
        let table = get_default_name_table();
        let parent_hash: u32 = 2814088591;
        assert_eq!(table.get_name(parent_hash, 0, 0).unwrap(), "AI");
        let hash: u32 = 2157271501;
        let index: usize = 35;
        assert_eq!(table.get_name(hash, index, parent_hash).unwrap(), "AI_35");
    }

    #[test]
    fn parse() {
        let text = std::fs::read_to_string("test/aamp/test.yml").unwrap();
        let pio = ParameterIO::from_text(text).unwrap();
        dbg!(&pio);
        assert_eq!(
            pio.param_root
                .objects
                .0
                .get(&Name::from_str("TestContent"))
                .unwrap()
                .0
                .get(&Name::from_str("Bool_0")),
            Some(&Parameter::Bool(true))
        );
    }

    static TEST_NAMES: &[&str] = &[
        "Bool_0",
        "Bool_1",
        "F32_0",
        "F32_1",
        "F32_2",
        "Vec2",
        "Vec3",
        "Vec4",
        "Color",
        "Str32_0",
        "Str32_1",
        "Str32_2",
        "Str64",
        "Curve1",
        "BufferInt",
        "BufferF32",
        "Str256",
        "Str256_2",
        "Quat",
        "U32",
        "U32_1",
        "BufferU32",
        "BufferBinary",
        "StringRef_0",
        "StringRef_1",
        "StringRef_2",
        "StringRef_3",
    ];

    #[test]
    fn text_roundtrip() {
        {
            let table = get_default_name_table();
            for name in TEST_NAMES {
                table.add_name(*name);
            }
        }
        let text = std::fs::read_to_string("test/aamp/test.yml").unwrap();
        let pio = ParameterIO::from_text(text).unwrap();
        let text2 = pio.to_text();
        dbg!(&text2);
        let pio2 = ParameterIO::from_text(&text2).unwrap();
        assert_eq!(pio, pio2);
    }

    #[test]
    fn bin_to_text() {
        for file in jwalk::WalkDir::new("test/aamp")
            .into_iter()
            .filter_map(|f| {
                f.ok().and_then(|f| {
                    (f.file_type().is_file() && !f.file_name().to_str().unwrap().ends_with("yml"))
                        .then(|| f.path())
                })
            })
        {
            let data = std::fs::read(file).unwrap();
            let pio = ParameterIO::from_binary(data).unwrap();
            pio.to_text();
        }
    }
}
