use crate::nodes::node::Node;
use std::collections::HashMap;

pub(crate) fn collect_anchors(
    node: &Node,
    anchors: &mut HashMap<String, Node>,
) -> Result<(), String> {
    match node {
        Node::Anchored(inner, name) => {
            if name.trim().is_empty() {
                return Err(crate::error::messages::ERR_EMPTY_ANCHOR_NAME.to_string());
            }
            if anchors.contains_key(name) {
                return Err(format!(
                    "{}{}",
                    crate::error::messages::ERR_DUPLICATE_ANCHOR_PREFIX,
                    name
                ));
            }
            anchors.insert(name.clone(), (**inner).clone());
            collect_anchors(inner, anchors)?;
            Ok(())
        }
        Node::Mapping(pairs) => {
            for (k, v) in pairs {
                collect_anchors(k, anchors)?;
                collect_anchors(v, anchors)?;
            }
            Ok(())
        }
        Node::Array(items) => {
            for it in items {
                collect_anchors(it, anchors)?;
            }
            Ok(())
        }
        Node::Document(nodes) => {
            for n in nodes {
                collect_anchors(n, anchors)?;
            }
            Ok(())
        }
        Node::Documents(docs) => {
            for d in docs {
                collect_anchors(d, anchors)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub(crate) fn replace_aliases(
    node: &mut Node,
    anchors: &HashMap<String, Node>,
) -> Result<(), String> {
    match node {
        Node::Alias(name) => {
            if let Some(found) = anchors.get(name) {
                *node = found.clone();
                Ok(())
            } else {
                Err(format!(
                    "{}{}",
                    crate::error::messages::ERR_UNDEFINED_ANCHOR_PREFIX,
                    name
                ))
            }
        }
        Node::Anchored(inner, _name) => {
            let replacement = (**inner).clone();
            *node = replacement;
            replace_aliases(node, anchors)
        }
        Node::Mapping(pairs) => {
            for (k, v) in pairs.iter_mut() {
                replace_aliases(k, anchors)?;
                replace_aliases(v, anchors)?;
            }
            Ok(())
        }
        Node::Array(items) => {
            for it in items.iter_mut() {
                replace_aliases(it, anchors)?;
            }
            Ok(())
        }
        Node::Document(nodes) => {
            for n in nodes.iter_mut() {
                replace_aliases(n, anchors)?;
            }
            Ok(())
        }
        Node::Documents(docs) => {
            for d in docs.iter_mut() {
                replace_aliases(d, anchors)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
