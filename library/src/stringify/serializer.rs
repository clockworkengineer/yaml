//! Custom serializer trait and implementations
//!
//! Allows users to define custom serialization logic for specific node types.

use alloc::string::String;
use alloc::boxed::Box;

use crate::nodes::node::Node;
use crate::stringify::format::{FormatOptions, FormatContext};

/// Result type for serialization operations
pub type SerializeResult = Result<String, String>;

/// Trait for custom node serializers
pub trait Serializer {
    /// Check if this serializer can handle the given node
    fn can_serialize(&self, node: &Node) -> bool;
    
    /// Serialize the node to a string
    fn serialize(&self, node: &Node, options: &FormatOptions, context: &FormatContext) -> SerializeResult;
    
    /// Get serializer priority (higher = checked first)
    fn priority(&self) -> i32 {
        0
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

    fn serialize(&self, node: &Node, options: &FormatOptions, context: &FormatContext) -> SerializeResult {
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

    fn serialize(&self, node: &Node, options: &FormatOptions, context: &FormatContext) -> SerializeResult {
        (self.serialize_fn)(node, options, context)
    }
}

/// Registry for custom serializers
pub struct SerializerRegistry {
    serializers: alloc::vec::Vec<Box<dyn Serializer>>,
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
        self.serializers.sort_by(|a, b| b.priority().cmp(&a.priority()));
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
        let serializer = TaggedSerializer::new("!custom", |node, _opts, _ctx| {
            match node {
                Node::Tagged(inner, _) => {
                    if let Node::Str(s, _, _) = &**inner {
                        Ok(format!("CUSTOM: {}", s))
                    } else {
                        Err("Expected string".to_string())
                    }
                }
                _ => Err("Not a tagged node".to_string()),
            }
        });

        let node = Node::Tagged(
            Box::new(Node::from("test")),
            "!custom".to_string(),
        );

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
        let result = registry.serialize(&large, &opts, &ctx, |_, _, _| {
            Ok("DEFAULT".to_string())
        }).unwrap();
        assert_eq!(result, "5K");

        // Should use default
        let result = registry.serialize(&small, &opts, &ctx, |_, _, _| {
            Ok("DEFAULT".to_string())
        }).unwrap();
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
        let result = registry.serialize(&node, &opts, &ctx, |_, _, _| {
            Ok("DEFAULT".to_string())
        }).unwrap();
        assert_eq!(result, "HIGH");
    }
}
