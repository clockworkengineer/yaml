//! Streaming YAML Serialization
//!
//! Provides incremental serialization for large YAML documents, enabling output
//! without loading the entire document into memory. Supports efficient streaming to destinations.
//!
//! Copyright (c) 2026 YAML Library Developers

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::{Result, YamlError};
use crate::io::traits::IDestination;
use crate::nodes::node::Node;
use crate::stringify::format::{FormatContext, FormatOptions};

/// Streaming YAML serializer
pub struct StreamingSerializer<'a> {
    destination: &'a mut dyn IDestination,
    options: FormatOptions,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl<'a> StreamingSerializer<'a> {
    /// Create new streaming serializer
    pub fn new(destination: &'a mut dyn IDestination) -> Self {
        Self {
            destination,
            options: FormatOptions::default(),
            buffer: Vec::new(),
            buffer_size: 4096, // 4KB default buffer
        }
    }

    /// Create with custom options
    pub fn with_options(destination: &'a mut dyn IDestination, options: FormatOptions) -> Self {
        Self {
            destination,
            options,
            buffer: Vec::new(),
            buffer_size: 4096,
        }
    }

    /// Set buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Write string to stream
    fn write_str(&mut self, s: &str) -> Result<()> {
        self.buffer.extend_from_slice(s.as_bytes());

        // Flush if buffer is large enough
        if self.buffer.len() >= self.buffer_size {
            self.flush()?;
        }

        Ok(())
    }

    /// Flush buffer to destination
    pub fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            let s = String::from_utf8_lossy(&self.buffer);
            self.destination.add_bytes(&s);
            self.buffer.clear();
        }
        Ok(())
    }

    /// Serialize a node to the stream
    pub fn serialize_node(&mut self, node: &Node) -> Result<()> {
        let mut context = FormatContext::new();
        self.serialize_node_impl(node, &mut context)?;
        self.flush()
    }

    /// Internal serialization implementation
    fn serialize_node_impl(&mut self, node: &Node, context: &mut FormatContext) -> Result<()> {
        match node {
            Node::Str(s, _, _) => {
                self.write_str(&self.format_string(s))?;
            }
            Node::Number(n) => {
                self.write_str(&format!("{:?}", n))?;
            }
            Node::Boolean(b) => {
                self.write_str(if *b { "true" } else { "false" })?;
            }
            Node::None => {
                if self.options.emit_null {
                    self.write_str("null")?;
                }
            }
            Node::Array(items) => {
                self.serialize_array(items, context)?;
            }
            Node::Mapping(pairs) => {
                self.serialize_mapping(pairs, context)?;
            }
            Node::Set(items) => {
                self.serialize_set(items, context)?;
            }
            Node::Documents(docs) => {
                for (i, doc) in docs.iter().enumerate() {
                    if i > 0 {
                        for _ in 0..self.options.document_separator_lines {
                            self.write_str("\n")?;
                        }
                    }
                    if self.options.explicit_start {
                        self.write_str("---\n")?;
                    }
                    self.serialize_node_impl(doc, context)?;
                    if self.options.explicit_end {
                        self.write_str("\n...\n")?;
                    }
                }
            }
            Node::Document(items) => {
                for item in items.iter() {
                    self.serialize_node_impl(item, context)?;
                }
            }
            Node::Tagged(inner, tag) => {
                self.write_str(tag)?;
                self.write_str(" ")?;
                self.serialize_node_impl(inner, context)?;
            }
            _ => {
                // Handle other node types with default stringify
                let mut buffer = crate::io::destinations::buffer::Buffer::new();
                crate::stringify::default::stringify(node, &mut buffer)
                    .map_err(|e| YamlError::from(e.to_string()))?;
                self.write_str(&buffer.to_string())?;
            }
        }

        Ok(())
    }

    /// Serialize array
    fn serialize_array(&mut self, items: &[Node], context: &mut FormatContext) -> Result<()> {
        if items.is_empty() && self.options.flow_empty_collections {
            self.write_str("[]")?;
            return Ok(());
        }

        let use_flow = items.len() < self.options.block_threshold;

        if use_flow {
            self.write_str("[")?;
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    self.write_str(", ")?;
                }
                self.serialize_node_impl(item, context)?;
            }
            self.write_str("]")?;
        } else {
            context.indent();
            for item in items {
                self.write_str("\n")?;
                self.write_str(&context.indent_str(&self.options))?;
                self.write_str("- ")?;
                context.advance(2);
                self.serialize_node_impl(item, context)?;
                context.newline();
            }
            context.dedent();
        }

        Ok(())
    }

    /// Serialize mapping
    fn serialize_mapping(
        &mut self,
        pairs: &[(Node, Node)],
        context: &mut FormatContext,
    ) -> Result<()> {
        if pairs.is_empty() && self.options.flow_empty_collections {
            self.write_str("{}")?;
            return Ok(());
        }

        let use_flow = pairs.len() < self.options.block_threshold;

        // Sort keys if requested
        let mut sorted_pairs;
        let pairs_ref: &[(Node, Node)] = if self.options.sort_keys {
            sorted_pairs = pairs.to_vec();
            sorted_pairs.sort_by(|a, b| {
                let a_key = self.node_to_sort_key(&a.0);
                let b_key = self.node_to_sort_key(&b.0);
                a_key.cmp(&b_key)
            });
            &sorted_pairs
        } else {
            pairs
        };

        if use_flow {
            self.write_str("{")?;
            for (i, (key, value)) in pairs_ref.iter().enumerate() {
                if i > 0 {
                    self.write_str(", ")?;
                }
                self.serialize_node_impl(key, context)?;
                self.write_str(": ")?;
                self.serialize_node_impl(value, context)?;
            }
            self.write_str("}")?;
        } else {
            context.indent();
            for (key, value) in pairs_ref {
                self.write_str("\n")?;
                self.write_str(&context.indent_str(&self.options))?;
                self.serialize_node_impl(key, context)?;
                self.write_str(": ")?;
                self.serialize_node_impl(value, context)?;
                context.newline();
            }
            context.dedent();
        }

        Ok(())
    }

    /// Serialize set
    fn serialize_set(&mut self, items: &[Node], context: &mut FormatContext) -> Result<()> {
        // Sets are serialized as arrays in YAML
        self.serialize_array(items, context)
    }

    /// Format string with appropriate quoting
    fn format_string(&self, s: &str) -> String {
        use crate::stringify::format::QuoteStyle;

        // Check if string needs quoting
        let needs_quotes = s.is_empty()
            || s.contains(':')
            || s.contains('#')
            || s.contains('\n')
            || s.starts_with([
                ' ', '-', '?', '[', ']', '{', '}', ',', '&', '*', '!', '|', '>', '%', '@', '`',
            ])
            || s.ends_with(' ')
            || s == "true"
            || s == "false"
            || s == "null"
            || s.parse::<f64>().is_ok();

        match self.options.quote_style {
            QuoteStyle::None if !needs_quotes => s.to_string(),
            QuoteStyle::Auto if !needs_quotes => s.to_string(),
            QuoteStyle::Auto | QuoteStyle::Single | QuoteStyle::AlwaysSingle => {
                format!("'{}'", s.replace('\'', "''"))
            }
            _ => {
                format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
            }
        }
    }

    /// Convert node to sortable key
    fn node_to_sort_key(&self, node: &Node) -> String {
        match node {
            Node::Str(s, _, _) => s.clone(),
            Node::Number(n) => format!("{:?}", n),
            Node::Boolean(b) => b.to_string(),
            _ => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;
    use crate::nodes::node::Numeric;

    #[test]
    fn test_streaming_basic() {
        let mut buffer = Buffer::new();
        let mut serializer = StreamingSerializer::new(&mut buffer);

        let node = Node::from("hello");
        serializer.serialize_node(&node).unwrap();

        let result = buffer.to_string();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_streaming_array() {
        let mut buffer = Buffer::new();
        let mut serializer = StreamingSerializer::new(&mut buffer);

        let node = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        serializer.serialize_node(&node).unwrap();

        let result = buffer.to_string();
        assert!(result.contains("1") && result.contains("2") && result.contains("3"));
    }

    #[test]
    fn test_streaming_mapping() {
        let mut buffer = Buffer::new();
        let mut serializer = StreamingSerializer::new(&mut buffer);

        let node = Node::Mapping(vec![
            (Node::from("name"), Node::from("Alice")),
            (Node::from("age"), Node::Number(Numeric::Integer(30))),
        ]);

        serializer.serialize_node(&node).unwrap();

        let result = buffer.to_string();
        assert!(result.contains("name") && result.contains("Alice"));
        assert!(result.contains("age") && result.contains("30"));
    }

    #[test]
    fn test_flow_style() {
        let mut buffer = Buffer::new();
        let opts = FormatOptions::new()
            .with_collection_style(crate::stringify::format::CollectionStyle::Flow);
        let mut serializer = StreamingSerializer::with_options(&mut buffer, opts);

        let node = Node::Array(vec![Node::from(1), Node::from(2)]);
        serializer.serialize_node(&node).unwrap();

        let result = buffer.to_string();
        assert!(result.contains("[") && result.contains("]"));
    }

    #[test]
    fn test_sorted_keys() {
        let mut buffer = Buffer::new();
        let opts = FormatOptions::new().with_sorted_keys(true);
        let mut serializer = StreamingSerializer::with_options(&mut buffer, opts);

        let node = Node::Mapping(vec![
            (Node::from("zebra"), Node::from(1)),
            (Node::from("apple"), Node::from(2)),
            (Node::from("banana"), Node::from(3)),
        ]);

        serializer.serialize_node(&node).unwrap();

        let result = buffer.to_string();
        let apple_pos = result.find("apple").unwrap();
        let banana_pos = result.find("banana").unwrap();
        let zebra_pos = result.find("zebra").unwrap();

        assert!(apple_pos < banana_pos);
        assert!(banana_pos < zebra_pos);
    }

    #[test]
    fn test_buffer_flushing() {
        let mut buffer = Buffer::new();
        let mut serializer = StreamingSerializer::new(&mut buffer).with_buffer_size(10);

        // Write enough to trigger flush
        for _ in 0..20 {
            serializer.write_str("x").unwrap();
        }

        // Buffer should have been flushed
        assert!(buffer.to_string().len() > 0);
    }
}
