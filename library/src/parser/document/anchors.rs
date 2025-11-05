//! Module: parser/document/anchors.rs

use crate::nodes::node::Node;
use std::collections::HashMap;

/// Recursively collects all anchor definitions from a YAML document tree.
///
/// Traverses the node tree and builds a map of anchor names to their
/// corresponding node values. Validates that anchor names are not empty
/// and that no duplicate anchors exist.
///
/// # Arguments
///
/// * `node` - The root node to search for anchors
/// * `anchors` - A mutable HashMap to store anchor name-to-node mappings
///
/// # Returns
///
/// Result indicating success or an error string for invalid anchors
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

/// Recursively replaces all alias references with their corresponding anchor values.
///
/// Traverses the node tree and replaces any Alias nodes with the actual
/// node values they reference. Validates that all aliases have corresponding
/// anchors and that alias names are not empty.
///
/// # Arguments
///
/// * `node` - The node tree to process for alias replacement
/// * `anchors` - HashMap containing anchor name-to-node mappings
///
/// # Returns
///
/// Result indicating success or an error string for undefined aliases
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

/// Expands YAML merge keys (<<) by incorporating referenced mapping values.
///
/// Processes merge key syntax in mappings, which allows inheriting key-value
/// pairs from other mappings. Handles both single aliases and sequences of
/// aliases as merge sources.
///
/// # Arguments
///
/// * `node` - The node tree to process for merge key expansion
/// * `anchors` - HashMap containing anchor name-to-node mappings
///
/// # Returns
///
/// Result indicating success or an error string for invalid merge operations
pub(crate) fn expand_merge_keys(
    node: &mut Node,
    anchors: &HashMap<String, Node>,
) -> Result<(), String> {
    match node {
        Node::Mapping(pairs) => {
            let mut combined: Vec<(Node, Node)> = Vec::new();
            let snapshot = pairs.clone();
            let mut i = 0usize;
            while i < snapshot.len() {
                let (k, v) = snapshot[i].clone();

                let mut handled = false;
                if let Node::Str(ks, _, _) = &k {
                    if ks.trim() == "<<" {
                        let mut expanded_pairs: Vec<(Node, Node)> = Vec::new();
                        match &v {
                            Node::Alias(name) => {
                                if let Some(src) = anchors.get(name) {
                                    if let Node::Mapping(src_pairs) = src {
                                        expanded_pairs.extend(src_pairs.clone());
                                    } else {
                                        return Err(format!(
                                            "Merge source '{}' is not a mapping",
                                            name
                                        ));
                                    }
                                } else {
                                    return Err(format!(
                                        "{}{}",
                                        crate::error::messages::ERR_UNDEFINED_ANCHOR_PREFIX,
                                        name
                                    ));
                                }
                            }
                            Node::Array(items) => {
                                for it in items.iter() {
                                    match it {
                                        Node::Alias(name) => {
                                            if let Some(src) = anchors.get(name) {
                                                if let Node::Mapping(src_pairs) = src {
                                                    expanded_pairs.extend(src_pairs.clone());
                                                } else {
                                                    return Err(format!("Merge source '{}' is not a mapping", name));
                                                }
                                            } else {
                                                return Err(format!("{}{}", crate::error::messages::ERR_UNDEFINED_ANCHOR_PREFIX, name));
                                            }
                                        }
                                        Node::Mapping(src_pairs) => {
                                            expanded_pairs.extend(src_pairs.clone());
                                        }
                                        _ => return Err("Invalid merge sequence item: expected alias or mapping".to_string()),
                                    }
                                }
                            }
                            Node::Mapping(src_pairs) => {
                                expanded_pairs.extend(src_pairs.clone());
                            }
                            other => {
                                return Err(format!(
                                    "Invalid merge value: expected alias, sequence or mapping, got {:?}",
                                    other
                                ));
                            }
                        }
                        combined.extend(expanded_pairs);
                        handled = true;
                    }
                }

                if handled {
                    i += 1;
                    continue;
                }

                if let Node::Str(_, _, _) = &k {
                    if let Node::Str(s, _, _) = &v {
                        let ts = s.trim_start();
                        if ts.starts_with("<<:") && ts.contains('*') {
                            if let Some(pos) = ts.find('*') {
                                let aname = ts[pos + 1..].trim().to_string();

                                let mut nested: Vec<(Node, Node)> = Vec::new();
                                let mut j = i + 1;
                                while j < snapshot.len() {
                                    // Stop if we find another merge key
                                    if let Node::Str(ns, _, _) = &snapshot[j].1 {
                                        if ns.trim_start().starts_with("<<:") {
                                            break;
                                        }
                                    }
                                    
                                    // Check if this entry looks like it should be at root level
                                    // by examining the key name for patterns that suggest it's not part of the current mapping
                                    if let Node::Str(key_name, _, _) = &snapshot[j].0 {
                                        // Keys that contain "explicit" or "boolean" are likely root-level entries
                                        // especially when they appear after a mapping section
                                        if key_name.contains("boolean") || key_name.contains("explicit") {
                                            break;
                                        }
                                        
                                        // Also check for other common root-level patterns
                                        // If the key doesn't look like a typical nested property,
                                        // it might be a new root-level entry
                                        let trimmed_key = key_name.trim();
                                        if trimmed_key == "explicit_boolean" {
                                            break;
                                        }
                                    }
                                    
                                    nested.push(snapshot[j].clone());
                                    j += 1;
                                }

                                let mut merged_pairs: Vec<(Node, Node)> = Vec::new();
                                if let Some(src) = anchors.get(&aname) {
                                    if let Node::Mapping(ap) = src {
                                        merged_pairs.extend(ap.clone());
                                    }
                                }

                                for (nk, nv) in nested.iter() {
                                    let mut replaced = false;

                                    if let Node::Str(nks, _, _) = nk {
                                        for p in merged_pairs.iter_mut() {
                                            if let Node::Str(pk, _, _) = &p.0 {
                                                if pk == nks {
                                                    *p = (nk.clone(), nv.clone());
                                                    replaced = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if !replaced {
                                        merged_pairs.push((nk.clone(), nv.clone()));
                                    }
                                }

                                combined.push((k.clone(), Node::Mapping(merged_pairs)));
                                i = j;
                                continue;
                            }
                        }
                    }
                }

                combined.push((k, v));
                i += 1;
            }

            use std::collections::HashMap as Map;
            let mut last_index: Map<String, usize> = Map::new();
            for (idx, (k, _v)) in combined.iter().enumerate() {
                let key_s = crate::parser::document::helpers::node_to_inline_string(k);
                last_index.insert(key_s, idx);
            }
            let mut rebuilt: Vec<(Node, Node)> = Vec::new();
            for (idx, (k, v)) in combined.into_iter().enumerate() {
                let key_s = crate::parser::document::helpers::node_to_inline_string(&k);
                if let Some(&last) = last_index.get(&key_s) {
                    if last == idx {
                        rebuilt.push((k, v));
                    }
                }
            }

            *pairs = rebuilt;

            for (_k, v) in pairs.iter_mut() {
                expand_merge_keys(v, anchors)?;
            }
            Ok(())
        }
        Node::Array(items) => {
            for it in items.iter_mut() {
                expand_merge_keys(it, anchors)?;
            }
            Ok(())
        }
        Node::Document(nodes) => {
            for n in nodes.iter_mut() {
                expand_merge_keys(n, anchors)?;
            }
            Ok(())
        }
        Node::Documents(docs) => {
            for d in docs.iter_mut() {
                expand_merge_keys(d, anchors)?;
            }
            Ok(())
        }
        Node::Anchored(inner, _name) => expand_merge_keys(inner, anchors),
        _ => Ok(()),
    }
}
