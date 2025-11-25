//! Module: parser/document/context.rs
//!
//! Provides parsing context management to enable context-aware validation
//! and reduce regression risk when adding new validation rules.

/// Represents the parsing context at a specific point in the YAML document.
///
/// This structure tracks important state information that affects how validation
/// rules should be applied. By maintaining this context, we can implement
/// context-aware validation that avoids false positives.
///
/// # Purpose
///
/// The parsing context helps solve several problems:
/// - Tab validation: tabs are forbidden in block indentation but allowed in flow context
/// - Indentation validation: rules differ between block and flow contexts
/// - Whitespace handling: after newlines vs inline
/// - Nested structure tracking: knowing parent context helps with edge cases
///
/// # Example
///
/// ```ignore
/// let ctx = ParsingContext::new(0);
/// let child_ctx = ctx.child_block_context(2);
/// let flow_ctx = ctx.child_flow_context();
/// ```
#[derive(Debug, Clone)]
pub struct ParsingContext {
    /// The current indentation level (number of spaces from line start)
    pub indent_level: usize,
    
    /// Whether we're currently in a flow context (inside [], {}, or after indicators in flow)
    /// Flow contexts have different whitespace and indentation rules
    pub in_flow: bool,
    
    /// Whether the parser just consumed a newline character
    /// This affects tab validation (tabs after newlines are indentation, which is forbidden)
    pub after_newline: bool,
    
    /// The type of collection context we're in (if any)
    pub collection_type: CollectionType,
    
    /// Reference to parent context for nested structure validation
    /// Using Option<Box<>> to keep struct size manageable while allowing arbitrary nesting
    pub parent: Option<Box<ParsingContext>>,
}

/// Types of collection contexts that affect parsing behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CollectionType {
    /// Not in a collection (top-level document)
    None,
    /// Inside a block sequence (lines starting with -)
    BlockSequence,
    /// Inside a block mapping (key: value pairs)
    BlockMapping,
    /// Inside a flow sequence ([...])
    FlowSequence,
    /// Inside a flow mapping ({...})
    FlowMapping,
}

#[allow(dead_code)]
impl ParsingContext {
    /// Creates a new root parsing context
    ///
    /// # Arguments
    ///
    /// * `indent_level` - The initial indentation level (usually 0 for document root)
    ///
    /// # Returns
    ///
    /// A new ParsingContext with default settings for block context
    pub fn new(indent_level: usize) -> Self {
        Self {
            indent_level,
            in_flow: false,
            after_newline: false,
            collection_type: CollectionType::None,
            parent: None,
        }
    }
    
    /// Creates a child context for a nested block collection
    ///
    /// # Arguments
    ///
    /// * `indent_level` - The indentation level of the child context
    /// * `collection_type` - The type of block collection being entered
    ///
    /// # Returns
    ///
    /// A new ParsingContext with this context as parent
    pub fn child_block_context(&self, indent_level: usize, collection_type: CollectionType) -> Self {
        Self {
            indent_level,
            in_flow: false,
            after_newline: self.after_newline,
            collection_type,
            parent: Some(Box::new(self.clone())),
        }
    }
    
    /// Creates a child context for entering a flow collection
    ///
    /// Flow contexts inherit indentation but have different validation rules
    ///
    /// # Returns
    ///
    /// A new ParsingContext in flow mode
    pub fn child_flow_context(&self, collection_type: CollectionType) -> Self {
        Self {
            indent_level: self.indent_level,
            in_flow: true,
            after_newline: false, // Flow content can span lines
            collection_type,
            parent: Some(Box::new(self.clone())),
        }
    }
    
    /// Updates the context to indicate a newline was just consumed
    ///
    /// This is important for tab validation - tabs immediately after newlines
    /// are considered indentation and are forbidden per YAML 1.2 spec.
    pub fn mark_newline_consumed(&mut self) {
        self.after_newline = true;
    }
    
    /// Updates the context to indicate non-whitespace content was found
    ///
    /// After content is found, tabs are no longer indentation (they're part of content)
    pub fn mark_content_found(&mut self) {
        self.after_newline = false;
    }
    
    /// Checks if tabs should be validated as indentation at current position
    ///
    /// Tabs are only forbidden when:
    /// - We're in block context (not flow)
    /// - We're after a newline (indentation position)
    ///
    /// # Returns
    ///
    /// true if tab validation should be enforced, false otherwise
    pub fn should_validate_tab_indentation(&self) -> bool {
        !self.in_flow && self.after_newline
    }
    
    /// Gets the parent context if it exists
    ///
    /// # Returns
    ///
    /// Reference to parent context, or None if this is root
    pub fn parent_context(&self) -> Option<&ParsingContext> {
        self.parent.as_ref().map(|b| b.as_ref())
    }
    
    /// Checks if current indent is valid relative to parent
    ///
    /// # Arguments
    ///
    /// * `current_indent` - The indentation level to validate
    ///
    /// # Returns
    ///
    /// true if indent is valid (more than parent), false otherwise
    pub fn is_valid_child_indent(&self, current_indent: usize) -> bool {
        if let Some(parent) = self.parent_context() {
            current_indent > parent.indent_level
        } else {
            true // Root context - any indent is valid
        }
    }
}

impl Default for ParsingContext {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_context() {
        let ctx = ParsingContext::new(0);
        assert_eq!(ctx.indent_level, 0);
        assert!(!ctx.in_flow);
        assert!(!ctx.after_newline);
        assert_eq!(ctx.collection_type, CollectionType::None);
        assert!(ctx.parent.is_none());
    }
    
    #[test]
    fn test_child_block_context() {
        let root = ParsingContext::new(0);
        let child = root.child_block_context(2, CollectionType::BlockSequence);
        
        assert_eq!(child.indent_level, 2);
        assert!(!child.in_flow);
        assert_eq!(child.collection_type, CollectionType::BlockSequence);
        assert!(child.parent.is_some());
    }
    
    #[test]
    fn test_child_flow_context() {
        let root = ParsingContext::new(0);
        let flow = root.child_flow_context(CollectionType::FlowSequence);
        
        assert!(flow.in_flow);
        assert_eq!(flow.collection_type, CollectionType::FlowSequence);
        assert!(flow.parent.is_some());
    }
    
    #[test]
    fn test_newline_marking() {
        let mut ctx = ParsingContext::new(0);
        assert!(!ctx.after_newline);
        
        ctx.mark_newline_consumed();
        assert!(ctx.after_newline);
        assert!(ctx.should_validate_tab_indentation());
        
        ctx.mark_content_found();
        assert!(!ctx.after_newline);
        assert!(!ctx.should_validate_tab_indentation());
    }
    
    #[test]
    fn test_tab_validation_in_flow() {
        let root = ParsingContext::new(0);
        let mut flow = root.child_flow_context(CollectionType::FlowSequence);
        
        flow.mark_newline_consumed();
        // Even after newline, tabs are OK in flow context
        assert!(!flow.should_validate_tab_indentation());
    }
    
    #[test]
    fn test_valid_child_indent() {
        let parent = ParsingContext::new(2);
        let child = parent.child_block_context(4, CollectionType::BlockMapping);
        
        assert!(child.is_valid_child_indent(6));
        assert!(child.is_valid_child_indent(4));
        assert!(!child.is_valid_child_indent(2));
        assert!(!child.is_valid_child_indent(0));
    }
    
    #[test]
    fn test_parent_access() {
        let root = ParsingContext::new(0);
        let child = root.child_block_context(2, CollectionType::BlockSequence);
        
        assert!(root.parent_context().is_none());
        assert!(child.parent_context().is_some());
        assert_eq!(child.parent_context().unwrap().indent_level, 0);
    }
}
