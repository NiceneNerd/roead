use super::*;
use crate::{types::*, yaml::*, Error, Result};
use lexical::{FromLexical, FromLexicalWithOptions};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use ryml::*;
use smartstring::alias::String;
use std::{
    borrow::{Borrow, Cow},
    cell::RefCell,
    collections::hash_map::{Entry, VacantEntry},
    sync::Arc,
};

impl ParameterIO {
    /// Parse ParameterIO from YAML text.
    pub fn from_text(text: impl AsRef<str>) -> Result<Self> {
        let tree = Tree::parse(text.as_ref())?;
        let root_ref = tree.root_ref()?;
        read_parameter_io(&root_ref)
    }
}

static NAMES: &str = include_str!("../../data/botw_hashed_names.txt");
static NUMBERED_NAMES: &str = include_str!("../../data/botw_numbered_names.txt");

/// Since there are basically no good runtime string formatting options in Rust,
/// we'll just do this instead.
struct ChildFormatIterator<'a> {
    pub string: &'a str,
    pub pos: usize,
    pub index: usize,
}

impl Iterator for ChildFormatIterator<'_> {
    type Item = std::string::String;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.index;
        self.index += 1;
        match idx {
            0 => Some(format!("{}{}", self.string, self.pos)),
            1 => Some(format!("{}{:02}", self.string, self.pos)),
            2 => Some(format!("{}{:03}", self.string, self.pos)),
            3 => Some(format!("{}_{}", self.string, self.pos)),
            4 => Some(format!("{}_{:02}", self.string, self.pos)),
            5 => Some(format!("{}_{:03}", self.string, self.pos)),
            _ => None,
        }
    }
}

impl ExactSizeIterator for ChildFormatIterator<'_> {
    fn len(&self) -> usize {
        6
    }
}

#[inline(always)]
fn format_number(format: &str, pos: usize) -> std::string::String {
    match format {
        "%d" | "%u" => format!("{}", pos),
        "%02d" | "%02u" => format!("{:02}", pos),
        "%03d" => format!("{:03}", pos),
        "%04d" => format!("{:04}", pos),
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
}

fn format_numbered_name(name: &str, pos: usize) -> std::string::String {
    for fmt in ["%d", "%02d", "%03d", "%04d", "%u", "%02u"].iter() {
        if name.contains(fmt) {
            let mut split = name.split(fmt);
            return [
                split.next().unwrap(),
                &format_number(fmt, pos),
                split.next().unwrap_or(""),
            ]
            .join("");
        }
    }
    unsafe { std::hint::unreachable_unchecked() }
}

macro_rules! free_cow {
    ($cow:expr, $life:tt) => {{
        let cow = $cow as *const _;
        unsafe { &*(cow as *const Cow<$life, str>) }
    }};
}

/// A table of names that is used to recover original names in binary parameter
/// archives which store only name hashes.
///
/// Because binary parameter archives only store CRC32 hashes of structure
/// names, recovering the original names – which is useful for converting
/// archives to a human-readable format – requires the use of a name table.
///
/// When serializing to YAML, by default roead will use a table that contains
/// strings from Breath of the Wild’s executable.
#[derive(Debug, Default)]
pub struct NameTable<'a> {
    names: RwLock<FxHashMap<u32, Cow<'a, str>>>,
    numbered_names: Vec<&'a str>,
}

impl<'a> NameTable<'a> {
    /// Create a new name table, optionally including default BOTW strings.
    pub fn new(botw_strings: bool) -> NameTable<'a> {
        if botw_strings {
            Self {
                names: RwLock::new(NAMES.lines().map(|n| (hash_name(n), n.into())).collect()),
                numbered_names: NUMBERED_NAMES.lines().collect(),
            }
        } else {
            Default::default()
        }
    }

    /// Add a known string to the name table.
    pub fn add_name(&self, name: impl Into<Cow<'a, str>>) {
        let name = name.into();
        let hash = hash_name(&name);
        self.names.write().entry(hash).or_insert(name);
    }

    /// Add a known string to the name table if you already know the hash (to
    /// avoid computing it).
    pub fn add_name_with_hash(&self, name: impl Into<Cow<'a, str>>, hash: u32) {
        self.names
            .write()
            .entry(hash)
            .or_insert_with(|| name.into());
    }

    /// Add a known string to the name table.
    pub fn add_name_str<'s: 'a>(&'a self, name: &'s str) {
        let hash = hash_name(name);
        self.names
            .write()
            .entry(hash)
            .or_insert_with(|| name.into());
    }

    /// Tries to guess the name that is associated with the given hash and index
    /// (of the parameter / object / list in its parent).
    ///
    /// The table is automatically updated with any newly found names if an
    /// indice-based guess was necessary.
    pub fn get_name(&self, hash: u32, index: usize, parent_hash: u32) -> Option<&Cow<'_, str>> {
        fn test_names<'a: 'b, 'b>(
            entry: VacantEntry<'b, u32, Cow<'a, str>>,
            hash: u32,
            index: usize,
            prefix: &str,
        ) -> std::result::Result<&'b Cow<'a, str>, VacantEntry<'b, u32, Cow<'a, str>>> {
            for i in index..(index + 1) {
                for fmt in (ChildFormatIterator {
                    string: prefix,
                    pos: i,
                    index: 0,
                }) {
                    #[allow(irrefutable_let_patterns)]
                    if let candidate = hash_name(&fmt) && candidate == hash
                    {
                        let name = entry.insert(fmt.into());
                        return Ok(free_cow!(name, 'a));
                    }
                }
            }
            Err(entry)
        }

        let mut names = self.names.write();
        let parent_name = names.get(&parent_hash).map(|c| free_cow!(c, 'a));
        match names.entry(hash) {
            Entry::Occupied(entry) => Some(free_cow!(entry.get(), 'a)),
            Entry::Vacant(entry) => {
                let mut entry = entry;
                if let Some(parent_name) = parent_name
                // Try to guess the name from the parent structure if possible.
                {
                    match test_names(entry, hash, index, parent_name)
                        .or_else(|entry| test_names(entry, hash, index, "Children"))
                        .or_else(|entry| test_names(entry, hash, index, "Child"))
                        .or_else(|entry| {
                            // Sometimes the parent name is plural and the object names are
                            // singular.
                            let mut entry = entry;
                            for suffix in ["s", "es", "List"] {
                                if let Some(singular) = parent_name.strip_suffix(suffix) {
                                    match test_names(entry, hash, index, singular) {
                                        Ok(found) => return Ok(found),
                                        Err(ret_entry) => entry = ret_entry,
                                    }
                                }
                            }
                            Err(entry)
                        }) {
                        Ok(found) => return Some(free_cow!(found, 'a)),
                        Err(ret_entry) => {
                            entry = ret_entry;
                        }
                    }
                }
                // Last resort: test all numbered names.
                for format in &self.numbered_names {
                    for i in 0..(index + 2) {
                        let name = format_numbered_name(format, i);
                        #[allow(irrefutable_let_patterns)]
                        if let candidate = hash_name(&name) && candidate == hash {
                            let name = entry.insert(name.into());
                            return Some(free_cow!(name, 'a));
                        }
                    }
                }
                None
            }
        }
    }
}

static DEFAULT_NAME_TABLE: Lazy<Arc<NameTable<'static>>> =
    Lazy::new(|| Arc::new(NameTable::new(true)));

/// Returns the default instance of the name table, which is automatically
/// populated with Breath of the Wild strings. It is initialised on first use
/// and has interior mutability.
pub fn get_default_name_table() -> &'static Lazy<Arc<NameTable<'static>>> {
    &DEFAULT_NAME_TABLE
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
        Scalar::String(s) => match tag {
            "!str32" => Parameter::String32(s.into()),
            "!str64" => Parameter::String64(s.into()),
            "!str256" => Parameter::String256(s.into()),
            _ => Parameter::StringRef(s),
        },
        Scalar::Int(i) => {
            if tag == "!u" {
                Parameter::U32(i as u32)
            } else {
                Parameter::Int(i as i32)
            }
        }
        Scalar::Float(f) => Parameter::F32(f as f32),
        Scalar::Bool(b) => Parameter::Bool(b),
        Scalar::Null => return Err(Error::InvalidData("AAMP does not support null values")),
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

fn read_curves<'a, 't, 'k, 'r, const N: usize>(
    node: &'r NodeRef<'a, 't, 'k, &'t Tree<'a>>,
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

fn parse_parameter<'a, 't, 'k, 'r>(
    node: &'r NodeRef<'a, 't, 'k, &'t Tree<'a>>,
) -> Result<Parameter> {
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
            "!curve" => match node.num_children()? {
                32 => read_curves::<1>(node)?.into(),
                64 => read_curves::<2>(node)?.into(),
                96 => read_curves::<3>(node)?.into(),
                128 => read_curves::<4>(node)?.into(),
                _ => return Err(Error::InvalidData("Invalid curve: wrong number of values")),
            },
            "!buffer_int" => read_buf::<i32>(node)?.into(),
            "!buffer_f32" => read_buf::<f32>(node)?.into(),
            "!buffer_u32" => read_buf::<u32>(node)?.into(),
            "!buffer_binary" => read_buf::<u8>(node)?.into(),
            _ => {
                return Err(Error::InvalidData(
                    "Invalid parameter: sequence without known tag",
                ))
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
            if !$node.is_key_quoted()?
                && let Ok(hash) = lexical::parse::<u64, &str>(key)
            {
                $m.insert(hash as u32, value);
            } else {
                $m.insert(hash_name(key), value);
            }
        }
    };
}

fn read_parameter_object<'a, 't, 'k, 'r>(
    node: &'r NodeRef<'a, 't, 'k, &'t Tree<'a>>,
) -> Result<ParameterObject> {
    if !node.is_valid() {
        return Err(Error::InvalidData("Invalid YAML node for parameter object"));
    }
    let mut param_object = ParameterObject::default();
    read_map!(node, param_object, parse_parameter);
    Ok(param_object)
}

fn read_parameter_list<'a, 't, 'k, 'r>(
    node: &'r NodeRef<'a, 't, 'k, &'t Tree<'a>>,
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

fn read_parameter_io<'a, 't, 'k, 'r>(
    node: &'r NodeRef<'a, 't, 'k, &'t Tree<'a>>,
) -> Result<ParameterIO> {
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

#[cfg(test)]
mod tests {
    use crate::h;

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
        let pio = ParameterIO::from_text(&text).unwrap();
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
}
