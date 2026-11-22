//! YAML emission directly from the zero-copy AAMP readers
//! ([`ParameterIORef`] and [`ParameterIOArena`]). Output is byte-identical to
//! the owned emitter in [`super::text`].
//!
//! The walk mirrors the owned writer exactly: objects and lists are emitted
//! in document order under `objects` and `lists`, names are resolved through
//! the default name table (falling back to the raw hash), and all scalar
//! formatting (floats, hex integers, quoting) is shared with the owned
//! emitter. Requires the `yaml` feature.

use alloc::string::String;
use alloc::vec::Vec;
use ryml::{NodeRef, Tree};

use super::*;
use crate::Result;
use crate::storage::Store;
use crate::yaml::*;

/// Emits the key of `$child` from the name table, quoting it when the
/// resolved name is all digits.
macro_rules! set_key_name {
    ($child:expr, $key:expr, $index:expr, $parent:expr) => {{
        let key: Name = $key;
        if let Some(name) = get_default_name_table().get_name(key.0, $index, $parent) {
            if lexical::parse::<u64, _>(name.as_bytes()).is_ok() {
                let ty = $child.node_type()?;
                $child.set_type_flags(ty | ryml::NodeType::WipKeyDquo)?;
            }
            $child.set_key(name)?;
        } else {
            $child.set_key(&lexical::to_string(key.0))?;
        }
    }};
}

/// Writes a flow-style sequence of floats tagged `$tag` from a struct.
macro_rules! write_flow_struct {
    ($node:expr, $tag:literal, $value:expr, $($field:tt),+) => {{
        $node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
        $(
            let mut child = $node.append_child()?;
            child.set_val(&write_float($value.$field as f64)?)?;
        )+
        $node.set_val_tag($tag)?;
    }};
}

impl<'a> ParameterIORef<'a> {
    /// Serialize the parameter IO to YAML, byte-identical to
    /// [`ParameterIO::to_text`](super::ParameterIO::to_text).
    pub fn to_text(&self) -> String {
        let mut tree = Tree::default();
        tree.reserve(10000);
        write_io_ref(&mut tree, self)
            .expect("ParameterIORef should serialize to YAML without error");
        tree.emit()
            .expect("ParameterIORef should serialize to YAML without error")
    }
}

impl<'a, S: Store<AampNode<'a>>> ParameterIOArena<'a, S> {
    /// Serialize the parameter IO to YAML, byte-identical to
    /// [`ParameterIO::to_text`](super::ParameterIO::to_text).
    pub fn to_text(&self) -> String {
        let mut tree = Tree::default();
        tree.reserve(10000);
        write_io_arena(&mut tree, self)
            .expect("ParameterIOArena should serialize to YAML without error");
        tree.emit()
            .expect("ParameterIOArena should serialize to YAML without error")
    }
}

fn write_io_ref(tree: &mut Tree<'_>, pio: &ParameterIORef<'_>) -> Result<()> {
    let mut root = tree.root_ref_mut()?;
    root.change_type(ryml::NodeType::Map)?;
    root.set_val_tag("!io")?;
    root.get_mut("version")?
        .set_val(&lexical::to_string(pio.version))?;
    root.get_mut("type")?.set_val(pio.data_type)?;
    let mut param_root = root.append_child()?;
    param_root.set_key("param_root")?;
    write_list_ref(&pio.param_root, ROOT_KEY.0, param_root)?;
    Ok(())
}

fn write_list_ref<'b, 'e>(
    plist: &ParameterListRef<'_>,
    parent_hash: u32,
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
) -> Result<()> {
    node.change_type(ryml::NodeType::Map)?;
    let mut objects = node.append_child()?;
    objects.set_key("objects")?;
    objects.change_type(ryml::NodeType::Map)?;
    for (i, (key, val)) in plist.objects.iter().enumerate() {
        let mut child = objects.append_child()?;
        set_key_name!(child, *key, i, parent_hash);
        write_object_ref(val, key.0, child)?;
    }
    let mut lists = node.append_child()?;
    lists.set_key("lists")?;
    lists.change_type(ryml::NodeType::Map)?;
    for (i, (key, val)) in plist.lists.iter().enumerate() {
        let mut child = lists.append_child()?;
        set_key_name!(child, *key, i, parent_hash);
        write_list_ref(val, key.0, child)?;
    }
    node.set_val_tag("!list")?;
    Ok(())
}

fn write_object_ref<'b, 'e>(
    pobj: &ParameterObjectRef<'_>,
    parent_hash: u32,
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
) -> Result<()> {
    node.change_type(ryml::NodeType::Map)?;
    for (i, (key, val)) in pobj.0.iter().enumerate() {
        let mut child = node.append_child()?;
        set_key_name!(child, *key, i, parent_hash);
        write_param_ref(val, child)?;
    }
    node.set_val_tag("!obj")?;
    Ok(())
}

fn write_param_ref<'b, 'e>(
    param: &ParameterRef<'_>,
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
) -> Result<()> {
    match param {
        ParameterRef::Bool(b) => node.set_val(if *b { "true" } else { "false" })?,
        ParameterRef::F32(f) => node.set_val(&write_float(*f as f64)?)?,
        ParameterRef::I32(i) => node.set_val(&lexical::to_string(*i))?,
        ParameterRef::Vec2(v) => write_flow_struct!(node, "!vec2", v, x, y),
        ParameterRef::Vec3(v) => write_flow_struct!(node, "!vec3", v, x, y, z),
        ParameterRef::Vec4(v) => write_flow_struct!(node, "!vec4", v, x, y, z, t),
        ParameterRef::Color(c) => write_flow_struct!(node, "!color", c, r, g, b, a),
        ParameterRef::Quat(q) => write_flow_struct!(node, "!quat", q, a, b, c, d),
        ParameterRef::String32(s) => {
            node.set_val(s)?;
            node.set_val_tag("!str32")?;
        }
        ParameterRef::String64(s) => {
            node.set_val(s)?;
            node.set_val_tag("!str64")?;
        }
        ParameterRef::String256(s) => {
            node.set_val(s)?;
            node.set_val_tag("!str256")?;
        }
        ParameterRef::Curve1(c)
        | ParameterRef::Curve2(c)
        | ParameterRef::Curve3(c)
        | ParameterRef::Curve4(c) => write_curves_ref(node, c)?,
        ParameterRef::BufferInt(b) => write_buf_num(node, b.iter_i32(), "!buffer_int")?,
        ParameterRef::BufferF32(b) => write_buf_num(node, b.iter_f32(), "!buffer_f32")?,
        ParameterRef::U32(u) => {
            node.set_val(&format_hex!(u))?;
            node.set_val_tag("!u")?;
        }
        ParameterRef::BufferU32(b) => write_buf_hex(node, b.iter_u32(), "!buffer_u32")?,
        ParameterRef::BufferBinary(b) => {
            write_buf_hex(node, b.iter().copied(), "!buffer_binary")?;
        }
        ParameterRef::StringRef(s) => {
            if string_needs_quotes(s) {
                let ty = node.node_type()?;
                node.set_type_flags(ty | ryml::NodeType::WipValDquo)?;
            }
            node.set_val(s)?;
        }
    }
    Ok(())
}

fn write_curves_ref<'b, 'e>(
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
    curves: &CurveRef<'_>,
) -> Result<()> {
    node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
    for curve in curves.iter() {
        let mut a = node.append_child()?;
        a.set_val(&lexical::to_string(curve.a))?;
        let mut b = node.append_child()?;
        b.set_val(&lexical::to_string(curve.b))?;
        for float in curve.floats {
            let mut f = node.append_child()?;
            f.set_val(&write_float(float as f64)?)?;
        }
    }
    node.set_val_tag("!curve")?;
    Ok(())
}

fn write_buf_num<'b, 'e, T: lexical::ToLexical>(
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
    vals: impl Iterator<Item = T>,
    tag: &str,
) -> Result<()> {
    node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
    for val in vals {
        let mut child = node.append_child()?;
        child.set_val(&lexical::to_string(val))?;
    }
    node.set_val_tag(tag)?;
    Ok(())
}

fn write_buf_hex<'b, 'e, T: lexical::ToLexical + lexical::ToLexicalWithOptions>(
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
    vals: impl Iterator<Item = T>,
    tag: &str,
) -> Result<()> {
    node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
    for val in vals {
        let mut child = node.append_child()?;
        child.set_val(&format_hex!(&val))?;
    }
    node.set_val_tag(tag)?;
    Ok(())
}

fn write_io_arena<'a, S: Store<AampNode<'a>>>(
    tree: &mut Tree<'_>,
    arena: &ParameterIOArena<'a, S>,
) -> Result<()> {
    let mut root = tree.root_ref_mut()?;
    root.change_type(ryml::NodeType::Map)?;
    root.set_val_tag("!io")?;
    root.get_mut("version")?
        .set_val(&lexical::to_string(arena.version()))?;
    root.get_mut("type")?.set_val(arena.data_type())?;
    let mut param_root = root.append_child()?;
    param_root.set_key("param_root")?;
    let root_id = arena.root().expect("arena root node missing");
    write_list_arena(arena, root_id, ROOT_KEY.0, param_root)?;
    Ok(())
}

/// The children of `id` whose payload satisfies `keep`, with duplicate names
/// collapsed the way the owned maps collapse them: the first occurrence
/// fixes the position and the last occurrence supplies the value. Objects
/// and lists are separate namespaces in the owned model, so dedup runs
/// within each namespace.
fn dedup_children<'a, S: Store<AampNode<'a>>>(
    arena: &ParameterIOArena<'a, S>,
    id: NodeId,
    keep: impl Fn(&AampPayload) -> bool,
) -> Vec<(Name, NodeId)> {
    // IndexMap insert keeps the first position and overwrites the value:
    // exactly the owned maps' duplicate semantics.
    let mut out: indexmap::IndexMap<Name, NodeId> = indexmap::IndexMap::new();
    for (name, child) in arena.children(id) {
        if keep(&arena.node(child).expect("child node missing").payload) {
            out.insert(name, child);
        }
    }
    out.into_iter().collect()
}

fn write_list_arena<'a, 'b, 'e, S: Store<AampNode<'a>>>(
    arena: &ParameterIOArena<'a, S>,
    id: NodeId,
    parent_hash: u32,
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
) -> Result<()> {
    node.change_type(ryml::NodeType::Map)?;

    let mut objects = node.append_child()?;
    objects.set_key("objects")?;
    objects.change_type(ryml::NodeType::Map)?;
    let object_children = dedup_children(arena, id, |p| matches!(p, AampPayload::Object));
    for (index, (key, child_id)) in object_children.into_iter().enumerate() {
        let mut child = objects.append_child()?;
        set_key_name!(child, key, index, parent_hash);
        write_object_arena(arena, child_id, key.0, child)?;
    }

    let mut lists = node.append_child()?;
    lists.set_key("lists")?;
    lists.change_type(ryml::NodeType::Map)?;
    let list_children = dedup_children(arena, id, |p| matches!(p, AampPayload::List));
    for (index, (key, child_id)) in list_children.into_iter().enumerate() {
        let mut child = lists.append_child()?;
        set_key_name!(child, key, index, parent_hash);
        write_list_arena(arena, child_id, key.0, child)?;
    }
    node.set_val_tag("!list")?;
    Ok(())
}

fn write_object_arena<'a, 'b, 'e, S: Store<AampNode<'a>>>(
    arena: &ParameterIOArena<'a, S>,
    id: NodeId,
    parent_hash: u32,
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
) -> Result<()> {
    node.change_type(ryml::NodeType::Map)?;
    let params = dedup_children(arena, id, |p| !matches!(p, AampPayload::List | AampPayload::Object));
    for (index, (key, child_id)) in params.into_iter().enumerate() {
        let mut child = node.append_child()?;
        set_key_name!(child, key, index, parent_hash);
        write_param_arena(arena, child_id, child)?;
    }
    node.set_val_tag("!obj")?;
    Ok(())
}

fn write_param_arena<'a, 'b, 'e, S: Store<AampNode<'a>>>(
    arena: &ParameterIOArena<'a, S>,
    id: NodeId,
    node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
) -> Result<()> {
    let payload = arena
        .node(id)
        .expect("parameter node missing from arena")
        .payload;
    // Only parameter children reach this point (`write_object_arena` filters
    // containers out), so the payload always has a parameter view.
    match payload.parameter() {
        Some(param) => write_param_ref(&param, node),
        None => unreachable!("container payload passed to write_param_arena"),
    }
}
