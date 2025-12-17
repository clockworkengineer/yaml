//! Bridge module for gradual migration from character-based to token-based parsing
//!
//! This module provides adapters that allow mixing character-based and token-based
//! parsing during the migration phase.

use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::parser::directives::DirectiveContext;
use crate::parser::document::tokens::value::parse_value_with_tokens;
use crate::parser::token_stream::TokenStream;
