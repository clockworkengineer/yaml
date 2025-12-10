//! Helper functions for parsing block scalars (literal | and folded >)

use crate::nodes::node::{BlockStyle, Node, QuoteType};

/// Creates a block scalar node from parsed content
#[allow(dead_code)]
pub(crate) fn make_block_scalar_node(content: String, is_folded: bool) -> Node {
    let style = if is_folded {
        BlockStyle::Folded
    } else {
        BlockStyle::Literal
    };
    Node::Str(content, QuoteType::Unquoted, style)
}
