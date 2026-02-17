//! Custom serializer trait and implementations
//!
//! Allows users to define custom serialization logic for specific node types.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::io::traits::IDestination;
use crate::nodes::node::Node;
use crate::stringify::format::{FormatContext, FormatOptions};

/// Result type for serialization operations
pub type SerializeResult = Result<String, String>;

/// Result type for streaming serialization operations
pub type StreamResult = Result<(), String>;

/// Trait for custom node serializers
pub trait Serializer {
    /// Check if this serializer can handle the given node
    fn can_serialize(&self, node: &Node) -> bool;

    /// Serialize the node to a string
    fn serialize(
        &self,
        node: &Node,
        options: &FormatOptions,
        context: &FormatContext,
    ) -> SerializeResult;

    /// Get serializer priority (higher = checked first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Trait implemented by concrete format writers (JSON, XML, TOML, bencode).
///
/// This trait focuses on **how** individual node types are emitted, while the
/// traversal over the `Node` tree is handled by `walk_node` below. Backends
/// that previously performed their own recursive descent can now delegate that
/// logic to the shared walker while keeping their output identical.
pub trait FormatWriter {
    /// Underlying output destination
    fn dest(&mut self) -> &mut dyn IDestination;

    /// Emit a YAML `null` value
    fn write_null(&mut self) -> StreamResult;

    /// Emit a boolean value
    fn write_bool(&mut self, value: bool) -> StreamResult;

    /// Emit a numeric node
    fn write_number(&mut self, num: &crate::nodes::node::Numeric) -> StreamResult;

    /// Emit a string scalar
    fn write_string(&mut self, value: &str) -> StreamResult;

    /// Emit an array-like collection
    fn start_array(&mut self, len: usize) -> StreamResult;
    fn array_value_separator(&mut self, index: usize) -> StreamResult;
    fn end_array(&mut self) -> StreamResult;

    /// Emit a set-like collection; many formats treat sets like arrays but
    /// some (e.g. XML) may want a different representation.
    fn start_set(&mut self, len: usize) -> StreamResult;
    fn set_value_separator(&mut self, index: usize) -> StreamResult;
    fn end_set(&mut self) -> StreamResult;

    /// Emit a mapping/dictionary-like collection
    fn start_mapping(&mut self, len: usize) -> StreamResult;
    fn write_mapping_key(&mut self, key: &Node) -> StreamResult;
    fn mapping_key_value_separator(&mut self) -> StreamResult;
    fn mapping_entry_separator(&mut self, index: usize) -> StreamResult;
    fn end_mapping(&mut self) -> StreamResult;

    /// Emit a standalone comment node if the format supports it. The default
    /// implementation is a no-op so formats that ignore comments can skip it.
    fn write_comment(&mut self, _comment: &str) -> StreamResult {
        Ok(())
    }

    /// Begin a logical document wrapper when serializing `Node::Document` or
    /// `Node::Documents` containers. Most formats either inline or treat these
    /// as arrays and can leave the default no-op implementation.
    fn start_document(&mut self, _index: usize, _total: usize) -> StreamResult {
        Ok(())
    }

    /// End a logical document wrapper.
    fn end_document(&mut self, _index: usize, _total: usize) -> StreamResult {
        Ok(())
    }
}

/// Shared recursive walker over `Node` trees.
///
/// This function centralizes the traversal logic used by all stringify
/// backends. Format-specific behavior is expressed through the `FormatWriter`
/// trait; the order in which nodes are visited matches the legacy per-backend
/// implementations so existing byte-level expectations remain valid.
pub fn walk_node<W: FormatWriter>(writer: &mut W, node: &Node) -> StreamResult {
    use crate::nodes::node::Node as N;

    match node {
        N::None => writer.write_null(),
        N::Boolean(b) => writer.write_bool(*b),
        N::Str(s, _qt, _style) => writer.write_string(s),
        N::Number(num) => writer.write_number(num),
        N::Array(items) => {
            writer.start_array(items.len())?;
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    writer.array_value_separator(i)?;
                }
                walk_node(writer, it)?;
            }
            writer.end_array()
        }
        N::Set(items) => {
            writer.start_set(items.len())?;
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    writer.set_value_separator(i)?;
                }
                walk_node(writer, it)?;
            }
            writer.end_set()
        }
        N::Mapping(pairs) => {
            writer.start_mapping(pairs.len())?;
            for (idx, (k, v)) in pairs.iter().enumerate() {
                if idx > 0 {
                    writer.mapping_entry_separator(idx)?;
                }
                writer.write_mapping_key(k)?;
                writer.mapping_key_value_separator()?;
                walk_node(writer, v)?;
            }
            writer.end_mapping()
        }
        N::Document(nodes) => {
            if nodes.len() == 1 {
                writer.start_document(0, 1)?;
                walk_node(writer, &nodes[0])?;
                writer.end_document(0, 1)
            } else {
                for (i, n) in nodes.iter().enumerate() {
                    writer.start_document(i, nodes.len())?;
                    walk_node(writer, n)?;
                    writer.end_document(i, nodes.len())?;
                }
                Ok(())
            }
        }
        N::Tagged(inner, _tag) => walk_node(writer, inner),
        N::Anchored(inner, _name) => walk_node(writer, inner),
        N::Alias(_name) => writer.write_null(),
        N::Documents(docs) => {
            for (i, d) in docs.iter().enumerate() {
                writer.start_document(i, docs.len())?;
                walk_node(writer, d)?;
                writer.end_document(i, docs.len())?;
            }
            Ok(())
        }
        N::Comment(c) => writer.write_comment(c),
    }
}

/// Serializer for custom tagged nodes
pub struct TaggedSerializer {
    tag: String,
    serialize_fn: Box<dyn Fn(&Node, &FormatOptions, &FormatContext) -> SerializeResult>,
}

impl TaggedSerializer {
    pub fn new<F>(tag: impl Into<String>, serialize_fn: F) -> Self
    where
        F: Fn(&Node, &FormatOptions, &FormatContext) -> SerializeResult + 'static,
    {
        Self {
            tag: tag.into(),
            serialize_fn: Box::new(serialize_fn),
        }
    }
}

impl Serializer for TaggedSerializer {
    fn can_serialize(&self, node: &Node) -> bool {
        match node {
            Node::Tagged(_, tag) => tag == &self.tag,
            _ => false,
        }
    }

    fn serialize(
        &self,
        node: &Node,
        options: &FormatOptions,
        context: &FormatContext,
    ) -> SerializeResult {
        (self.serialize_fn)(node, options, context)
    }

    fn priority(&self) -> i32 {
        10 // Tagged serializers get higher priority
    }
}

/// Serializer for specific node types
pub struct TypeSerializer {
    serialize_fn: Box<dyn Fn(&Node, &FormatOptions, &FormatContext) -> SerializeResult>,
    can_serialize_fn: Box<dyn Fn(&Node) -> bool>,
}

impl TypeSerializer {
    pub fn new<F, C>(can_serialize: C, serialize_fn: F) -> Self
    where
        F: Fn(&Node, &FormatOptions, &FormatContext) -> SerializeResult + 'static,
        C: Fn(&Node) -> bool + 'static,
    {
        Self {
            serialize_fn: Box::new(serialize_fn),
            can_serialize_fn: Box::new(can_serialize),
        }
    }
}

impl Serializer for TypeSerializer {
    fn can_serialize(&self, node: &Node) -> bool {
        (self.can_serialize_fn)(node)
    }

    fn serialize(
        &self,
        node: &Node,
        options: &FormatOptions,
        context: &FormatContext,
    ) -> SerializeResult {
        (self.serialize_fn)(node, options, context)
    }
}

/// Registry for custom serializers
pub struct SerializerRegistry {
    serializers: Vec<Box<dyn Serializer>>,
}

impl SerializerRegistry {
    pub fn new() -> Self {
        Self {
            serializers: alloc::vec::Vec::new(),
        }
    }

    /// Register a custom serializer
    pub fn register(&mut self, serializer: Box<dyn Serializer>) {
        self.serializers.push(serializer);
        // Sort by priority (descending)
        self.serializers
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Find a serializer for the given node
    pub fn find_serializer(&self, node: &Node) -> Option<&dyn Serializer> {
        self.serializers
            .iter()
            .find(|s| s.can_serialize(node))
            .map(|s| s.as_ref())
    }

    /// Serialize using registered serializers, fallback to default
    pub fn serialize<F>(
        &self,
        node: &Node,
        options: &FormatOptions,
        context: &FormatContext,
        default: F,
    ) -> SerializeResult
    where
        F: FnOnce(&Node, &FormatOptions, &FormatContext) -> SerializeResult,
    {
        if let Some(serializer) = self.find_serializer(node) {
            serializer.serialize(node, options, context)
        } else {
            default(node, options, context)
        }
    }
}

impl Default for SerializerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::Numeric;

    #[test]
    fn test_tagged_serializer() {
        let serializer = TaggedSerializer::new("!custom", |node, _opts, _ctx| match node {
            Node::Tagged(inner, _) => {
                if let Node::Str(s, _, _) = &**inner {
                    Ok(format!("CUSTOM: {}", s))
                } else {
                    Err("Expected string".to_string())
                }
            }
            _ => Err("Not a tagged node".to_string()),
        });

        let node = Node::Tagged(Box::new(Node::from("test")), "!custom".to_string());

        assert!(serializer.can_serialize(&node));

        let opts = FormatOptions::default();
        let ctx = FormatContext::new();
        let result = serializer.serialize(&node, &opts, &ctx).unwrap();
        assert_eq!(result, "CUSTOM: test");
    }

    #[test]
    fn test_type_serializer() {
        let serializer = TypeSerializer::new(
            |node| matches!(node, Node::Number(Numeric::Integer(n)) if *n > 1000),
            |node, _opts, _ctx| {
                if let Node::Number(Numeric::Integer(n)) = node {
                    Ok(format!("{}K", n / 1000))
                } else {
                    Err("Not a large integer".to_string())
                }
            },
        );

        let small = Node::Number(Numeric::Integer(100));
        let large = Node::Number(Numeric::Integer(5000));

        assert!(!serializer.can_serialize(&small));
        assert!(serializer.can_serialize(&large));

        let opts = FormatOptions::default();
        let ctx = FormatContext::new();
        let result = serializer.serialize(&large, &opts, &ctx).unwrap();
        assert_eq!(result, "5K");
    }

    #[test]
    fn test_serializer_registry() {
        let mut registry = SerializerRegistry::new();

        // Register a custom serializer for large integers
        registry.register(Box::new(TypeSerializer::new(
            |node| matches!(node, Node::Number(Numeric::Integer(n)) if *n > 1000),
            |node, _opts, _ctx| {
                if let Node::Number(Numeric::Integer(n)) = node {
                    Ok(format!("{}K", n / 1000))
                } else {
                    Err("Not a large integer".to_string())
                }
            },
        )));

        let large = Node::Number(Numeric::Integer(5000));
        let small = Node::Number(Numeric::Integer(100));

        let opts = FormatOptions::default();
        let ctx = FormatContext::new();

        // Should use custom serializer
        let result = registry
            .serialize(&large, &opts, &ctx, |_, _, _| Ok("DEFAULT".to_string()))
            .unwrap();
        assert_eq!(result, "5K");

        // Should use default
        let result = registry
            .serialize(&small, &opts, &ctx, |_, _, _| Ok("DEFAULT".to_string()))
            .unwrap();
        assert_eq!(result, "DEFAULT");
    }

    #[test]
    fn test_priority_ordering() {
        let mut registry = SerializerRegistry::new();

        // Register low priority serializer
        registry.register(Box::new(TypeSerializer::new(
            |_| true,
            |_, _, _| Ok("LOW".to_string()),
        )));

        // Register high priority tagged serializer
        registry.register(Box::new(TaggedSerializer::new("!test", |_, _, _| {
            Ok("HIGH".to_string())
        })));

        let node = Node::Tagged(Box::new(Node::from("value")), "!test".to_string());

        let opts = FormatOptions::default();
        let ctx = FormatContext::new();

        // Should use high priority serializer
        let result = registry
            .serialize(&node, &opts, &ctx, |_, _, _| Ok("DEFAULT".to_string()))
            .unwrap();
        assert_eq!(result, "HIGH");
    }
}
