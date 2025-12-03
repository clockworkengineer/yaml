# Parsing Functions Using Raw Character Access: Refactor Plan

## mapping.rs
- parse_mapping
- is_plain_safe_value
- is_plain_safe_key
**Plan:** Refactor to use tokens from the lexer, remove manual char/string checks.

## sequence.rs
- (Sequence parsing functions)
**Plan:** Refactor sequence parsing to use tokens, update loop/conditional logic.

## inline.rs
- (Inline value parsing functions)
**Plan:** Refactor inline parsing to use tokens, replace manual char checks.

## scalar.rs
- (Scalar parsing functions)
**Plan:** Refactor scalar parsing to consume tokens, remove legacy helpers.

## helpers.rs
- parse_error
- validate_indentation
- skip_whitespace
- skip_whitespace_no_tabs
- validate_no_tab_indentation
- parse_quoted_scalar
- peek_ahead_for_document_start_end
- peek_ahead_for_mapping_key
- parse_mapping_key
- parse_comment
- validate_comment_spacing
**Plan:** Refactor/remove helpers using raw character streams, update error helpers, remove deprecated whitespace/indentation functions.

## block_scalar.rs
- parse_block_scalar
- make_block_scalar_node
**Plan:** Refactor block scalar parsing to use tokens.

## explicit_key.rs
- is_explicit_key_start
- parse_explicit_mapping_entry
**Plan:** Refactor explicit key detection and entry parsing to use tokens.

---
For each file: update all parsing functions to consume tokens from the lexer, remove/refactor helpers using raw character access, and update error handling/tests for token-based parsing.