//! YAML emission directly from the zero-copy BYML readers ([`BymlRef`] and
//! [`BymlArena`]). Output is byte-identical to the owned emitter.
//! Requires the `yaml` feature.

use base64::Engine;
use core::cmp::Ordering;
use ryml::{NodeRef, Tree};

use super::arena::Store;
use super::*;
use crate::{Result, yaml::*};

#[cfg(feature = "alloc")]
impl<'a> BymlRef<'a> {
    /// Serialize the document to YAML, byte-identical to [`Byml::to_text`].
    pub fn to_text(&self) -> String {
        let mut tree = Tree::default();
        tree.reserve(20000);
        let root = match self {
            BymlRef::Map(_) | BymlRef::HashMap(_) | BymlRef::ValueHashMap(_) => {
                tree.to_map(0).expect("failed to make root map");
                tree.root_ref_mut().expect("root node missing")
            }
            BymlRef::Array(_) => {
                tree.to_seq(0).expect("failed to make root sequence");
                tree.root_ref_mut().expect("root node missing")
            }
            BymlRef::Null => return "null".to_string(),
            _ => panic!("BYML must be container or null to serialize"),
        };
        build_ref_node(root, self).expect("BymlRef should serialize to YAML without error");
        tree.emit()
            .expect("BymlRef should serialize to YAML without error")
    }
}

impl<'a, S: Store<Node<'a>>> BymlArena<S> {
    /// Serialize the document to YAML, byte-identical to [`Byml::to_text`].
    pub fn to_text(&self) -> String {
        let mut tree = Tree::default();
        tree.reserve(20000);
        let root = match self.root() {
            None => return "null".to_string(),
            Some(id) => id,
        };
        let root_node = self.node(root).expect("root node missing from arena");
        match root_node.payload {
            Payload::Map | Payload::HashMap | Payload::ValueHashMap => {
                tree.to_map(0).expect("failed to make root map")
            }
            Payload::Array => tree.to_seq(0).expect("failed to make root sequence"),
            Payload::Null => return "null".to_string(),
            _ => panic!("BYML must be container or null to serialize"),
        }
        let dest = tree.root_ref_mut().expect("root node missing");
        build_arena_node(self, dest, root)
            .expect("BymlArena should serialize to YAML without error");
        tree.emit()
            .expect("BymlArena should serialize to YAML without error")
    }
}

fn emit_scalar<'b, 'e>(
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
    payload: &Payload<'_>,
) -> Result<()> {
    match payload {
        Payload::String(s) => {
            node.set_val(s)?;
            if string_needs_quotes(s) {
                let flags = node.node_type()?;
                node.set_type_flags(flags | ryml::NodeType::WipValDquo)?;
            }
        }
        Payload::Bool(b) => node.set_val(if *b { "true" } else { "false" })?,
        Payload::Float(f) => node.set_val(&write_float(*f as f64)?)?,
        Payload::Double(d) => {
            node.set_val(&write_float(*d)?)?;
            node.set_val_tag("!f64")?;
        }
        Payload::I32(i) => node.set_val(&lexical::to_string(*i))?,
        Payload::I64(i) => {
            node.set_val(&lexical::to_string(*i))?;
            node.set_val_tag("!l")?;
        }
        Payload::U32(u) => {
            node.set_val(&format_hex!(u))?;
            node.set_val_tag("!u")?;
        }
        Payload::U64(u) => {
            node.set_val(&format_hex!(u))?;
            node.set_val_tag("!ul")?;
        }
        Payload::Null => node.set_val("null")?,
        Payload::BinaryData(data) => {
            let arena = node.tree().arena_capacity();
            node.tree_mut().reserve_arena(arena + data.len());
            node.set_val(&base64::engine::general_purpose::STANDARD.encode(data))?;
            node.set_val_tag("!!binary")?;
        }
        Payload::FileData(data) => {
            let arena = node.tree().arena_capacity();
            node.tree_mut().reserve_arena(arena + data.len());
            node.set_val(&base64::engine::general_purpose::STANDARD.encode(data))?;
            node.set_val_tag("!!file")?;
        }
        Payload::Array | Payload::Map | Payload::HashMap | Payload::ValueHashMap => {}
    }
    Ok(())
}

#[cfg(feature = "alloc")]
fn ref_should_use_inline(byml: &BymlRef<'_>) -> bool {
    let is_simple = |by: &BymlRef<'_>| !matches!(by, BymlRef::Array(_) | BymlRef::Map(_));
    match byml {
        BymlRef::Array(arr) => arr.len() < 10 && arr.iter().all(is_simple),
        BymlRef::Map(hash) => hash.len() < 10 && hash.iter().all(|(_, v)| is_simple(v)),
        _ => false,
    }
}

fn build_ref_node<'b, 'e>(
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
    byml: &BymlRef<'_>,
) -> Result<()> {
    match byml {
        BymlRef::Array(array) => {
            if ref_should_use_inline(byml) {
                node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
            } else {
                node.change_type(ryml::NodeType::Seq)?;
            }
            for item in array {
                let child = node.append_child()?;
                build_ref_node(child, item)?;
            }
        }
        BymlRef::Map(hash) => {
            if ref_should_use_inline(byml) {
                node.change_type(ryml::NodeType::Map | ryml::NodeType::WipStyleFlowSl)?;
            } else {
                node.change_type(ryml::NodeType::Map)?;
            }
            let mut map_items = hash.iter().collect::<Vec<_>>();
            map_items.sort_by(|a, b| a.0.cmp(b.0));
            for (key, value) in map_items {
                let mut child = node.append_child()?;
                child.set_key(key)?;
                if string_needs_quotes(key) {
                    let flags = child.node_type()?;
                    child.set_type_flags(flags | ryml::NodeType::WipKeySquo)?;
                }
                build_ref_node(child, value)?;
            }
        }
        BymlRef::HashMap(hash) => {
            if ref_should_use_inline(byml) {
                node.change_type(ryml::NodeType::Map | ryml::NodeType::WipStyleFlowSl)?;
            } else {
                node.change_type(ryml::NodeType::Map)?;
            }
            let mut map_items = hash.iter().collect::<Vec<_>>();
            map_items.sort_by(|a, b| a.0.cmp(b.0));
            for (key, value) in map_items {
                let mut child = node.append_child()?;
                child.set_key(&key.to_string())?;
                build_ref_node(child, value)?;
            }
            node.set_val_tag("!h")?;
        }
        BymlRef::ValueHashMap(hash) => {
            if ref_should_use_inline(byml) {
                node.change_type(ryml::NodeType::Map | ryml::NodeType::WipStyleFlowSl)?;
            } else {
                node.change_type(ryml::NodeType::Map)?;
            }
            let mut map_items = hash.iter().collect::<Vec<_>>();
            map_items.sort_by(|a, b| a.0.cmp(b.0));
            for (key, (value, _)) in map_items {
                let mut child = node.append_child()?;
                child.set_key(&key.to_string())?;
                build_ref_node(child, value)?;
            }
            node.set_val_tag("!vh")?;
        }
        BymlRef::String(s) => emit_scalar(node, &Payload::String(s))?,
        BymlRef::Bool(b) => emit_scalar(node, &Payload::Bool(*b))?,
        BymlRef::Float(f) => emit_scalar(node, &Payload::Float(*f))?,
        BymlRef::Double(d) => emit_scalar(node, &Payload::Double(*d))?,
        BymlRef::I32(i) => emit_scalar(node, &Payload::I32(*i))?,
        BymlRef::I64(i) => emit_scalar(node, &Payload::I64(*i))?,
        BymlRef::U32(u) => emit_scalar(node, &Payload::U32(*u))?,
        BymlRef::U64(u) => emit_scalar(node, &Payload::U64(*u))?,
        BymlRef::BinaryData(d) => emit_scalar(node, &Payload::BinaryData(d))?,
        BymlRef::FileData(d) => emit_scalar(node, &Payload::FileData(d))?,
        BymlRef::Null => emit_scalar(node, &Payload::Null)?,
    }
    Ok(())
}

/// Container children sorted by key, with duplicate keys collapsed to the
/// last occurrence (the owned maps overwrite on duplicate insert).
fn sorted_children<'a, S: Store<Node<'a>>>(
    arena: &BymlArena<S>,
    id: NodeId,
    ord: impl Fn(&Key<'a>, &Key<'a>) -> Ordering,
) -> Vec<(Key<'a>, NodeId)> {
    let mut children: Vec<(Key<'a>, NodeId)> = arena.children(id).collect();
    children.sort_by(|a, b| ord(&a.0, &b.0));
    let mut out: Vec<(Key<'a>, NodeId)> = Vec::with_capacity(children.len());
    for (key, child) in children {
        match out.last_mut() {
            Some(last) if last.0 == key => *last = (key, child),
            _ => out.push((key, child)),
        }
    }
    out
}

fn arena_should_use_inline<'a, S: Store<Node<'a>>>(arena: &BymlArena<S>, id: NodeId) -> bool {
    let node = arena.node(id).expect("node missing");
    let is_simple = |n: &Node<'_>| !matches!(n.payload, Payload::Array | Payload::Map);
    match node.payload {
        Payload::Array => {
            node.child_count() < 10
                && arena
                    .children(id)
                    .all(|(_, c)| is_simple(arena.node(c).expect("child missing")))
        }
        Payload::Map => {
            node.child_count() < 10
                && arena
                    .children(id)
                    .all(|(_, c)| is_simple(arena.node(c).expect("child missing")))
        }
        _ => false,
    }
}

fn build_arena_node<'a, 'b, 'e, S: Store<Node<'a>>>(
    arena: &BymlArena<S>,
    mut node: NodeRef<'b, 'e, '_, &'e mut Tree<'b>>,
    id: NodeId,
) -> Result<()> {
    let node_ref = arena.node(id).expect("node missing");
    match &node_ref.payload {
        Payload::Array => {
            if arena_should_use_inline(arena, id) {
                node.change_type(ryml::NodeType::Seq | ryml::NodeType::WipStyleFlowSl)?;
            } else {
                node.change_type(ryml::NodeType::Seq)?;
            }
            for (_, child_id) in arena.children(id) {
                let child = node.append_child()?;
                build_arena_node(arena, child, child_id)?;
            }
        }
        Payload::Map => {
            if arena_should_use_inline(arena, id) {
                node.change_type(ryml::NodeType::Map | ryml::NodeType::WipStyleFlowSl)?;
            } else {
                node.change_type(ryml::NodeType::Map)?;
            }
            let children = sorted_children(arena, id, |a, b| match (a, b) {
                (Key::Str(s1), Key::Str(s2)) => s1.as_bytes().cmp(s2.as_bytes()),
                _ => Ordering::Equal,
            });
            for (key, child_id) in children {
                let mut child = node.append_child()?;
                if let Key::Str(s) = key {
                    child.set_key(s)?;
                    if string_needs_quotes(s) {
                        let flags = child.node_type()?;
                        child.set_type_flags(flags | ryml::NodeType::WipKeySquo)?;
                    }
                }
                build_arena_node(arena, child, child_id)?;
            }
        }
        Payload::HashMap => {
            if arena_should_use_inline(arena, id) {
                node.change_type(ryml::NodeType::Map | ryml::NodeType::WipStyleFlowSl)?;
            } else {
                node.change_type(ryml::NodeType::Map)?;
            }
            let children = sorted_children(arena, id, |a, b| match (a, b) {
                (Key::Hash(h1), Key::Hash(h2)) => h1.cmp(h2),
                _ => Ordering::Equal,
            });
            for (key, child_id) in children {
                let mut child = node.append_child()?;
                if let Key::Hash(h) = key {
                    child.set_key(&h.to_string())?;
                }
                build_arena_node(arena, child, child_id)?;
            }
            node.set_val_tag("!h")?;
        }
        Payload::ValueHashMap => {
            if arena_should_use_inline(arena, id) {
                node.change_type(ryml::NodeType::Map | ryml::NodeType::WipStyleFlowSl)?;
            } else {
                node.change_type(ryml::NodeType::Map)?;
            }
            let children = sorted_children(arena, id, |a, b| match (a, b) {
                (Key::Hash(h1), Key::Hash(h2)) => h1.cmp(h2),
                _ => Ordering::Equal,
            });
            for (key, child_id) in children {
                let mut child = node.append_child()?;
                if let Key::Hash(h) = key {
                    child.set_key(&h.to_string())?;
                }
                build_arena_node(arena, child, child_id)?;
            }
            node.set_val_tag("!vh")?;
        }
        Payload::String(s) => emit_scalar(node, &Payload::String(s))?,
        Payload::Bool(b) => emit_scalar(node, &Payload::Bool(*b))?,
        Payload::Float(f) => emit_scalar(node, &Payload::Float(*f))?,
        Payload::Double(d) => emit_scalar(node, &Payload::Double(*d))?,
        Payload::I32(i) => emit_scalar(node, &Payload::I32(*i))?,
        Payload::I64(i) => emit_scalar(node, &Payload::I64(*i))?,
        Payload::U32(u) => emit_scalar(node, &Payload::U32(*u))?,
        Payload::U64(u) => emit_scalar(node, &Payload::U64(*u))?,
        Payload::BinaryData(d) => emit_scalar(node, &Payload::BinaryData(d))?,
        Payload::FileData(d) => emit_scalar(node, &Payload::FileData(d))?,
        Payload::Null => emit_scalar(node, &Payload::Null)?,
    }
    Ok(())
}
