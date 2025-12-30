# Parser DRY Refactor Progress Checklist


## 1. Inventory and Identify Duplication

### Parser Modules (document/)
- value.rs
- mapping.rs
- sequence.rs
- scalar.rs
- inline.rs
- explicit_key.rs
- helpers.rs
- contents.rs
- main_loop.rs
- parse.rs
- loop_guards.rs
- error_builder.rs
- context.rs
- bridge.rs

### Token-Driven Modules (document/tokens/)
- value.rs
- mapping.rs
- sequence.rs
- inline.rs
- mod.rs

### Key Functions (sample, not exhaustive)
- parse_value, parse_mapping, parse_sequence, parse_scalar, parse_inline, parse_explicit_mapping_entry
- parse_value_with_tokens, parse_mapping_with_tokens, parse_sequence_with_tokens
- token_dispatch, handle_multiple_explicit_keys, skip_whitespace_tokens, parse_comment_token

### Repeated Patterns to Identify
- Decorator handling (tag, anchor, alias extraction)
- Whitespace/comment skipping
- Explicit key parsing
- Error handling and reporting
- Token dispatch and node construction
- Manual char-by-char parsing vs. token-driven parsing

- [ ] List all parser modules and functions (**done above**)
- [ ] Identify repeated patterns (decorator handling, whitespace skipping, etc.) (**see above**)


## 2. Centralize Decorator and Whitespace Handling

### Decorator Handling (tag, anchor, alias)
- Found in: contents.rs, anchors.rs, tokens/value.rs, tokens/mapping.rs, tokens/sequence.rs
- Patterns: Extraction and application of tag/anchor/alias logic is repeated in both char-based and token-based modules.
- Recommendation: Move all decorator extraction to a single utility (e.g., token_stream.rs or a new decorators.rs helper). Refactor all parsing functions to use this shared utility.

### Whitespace/Comment Skipping
- Found in: inline_tokens.rs, parse.rs, sequence.rs, tokens/value.rs, tokens/sequence.rs, tokens/mapping.rs, main_loop.rs, helpers.rs (skip_whitespace_tokens)
- Patterns: Multiple skip_whitespace, skip_comments, skip_whitespace_and_comments functions and calls, both on ISource and TokenStream.
- Recommendation: Create a unified skip_whitespace_and_comments function for both char-based and token-based parsing. Remove or adapt all redundant skip functions to use this central utility.

- [ ] Move decorator extraction to a single utility (**see above for locations and plan**)
- [ ] Replace repeated whitespace/comment skipping with unified function (**see above for locations and plan**)


## 3. Abstract Token Dispatch and Node Construction

### Token Dispatch Logic
- Found in: contents.rs (token_dispatch), tokens/value.rs, tokens/mapping.rs, tokens/sequence.rs, mod.rs (various parse_* functions)
- Patterns: Matching on token types to route to the correct parse function is repeated in several modules, both for block and flow constructs.
- Recommendation: Create a generic token dispatch function (or trait) that can be reused for both block and flow parsing, reducing repeated match logic.

### Node Construction Logic
- Found in: tokens/value.rs, tokens/mapping.rs, tokens/sequence.rs, mod.rs, helpers.rs
- Patterns: Construction of Node variants (Node::Str, Node::Number, Node::Mapping, etc.) is repeated, often with similar match arms and conversion logic.
- Recommendation: Factor out repeated node construction into reusable helpers or builder functions. Consider a NodeBuilder or utility module for common node creation patterns.

- [ ] Create generic token dispatch function (**see above for locations and plan**)
- [ ] Factor out repeated node construction logic (**see above for locations and plan**)

## 4. Refactor Explicit Key and Mapping Entry Logic
- [ ] Consolidate explicit key parsing into a single function
- [ ] Use token-driven parsing for explicit keys and mapping entries

## 5. Generalize Error Handling
- [ ] Centralize error creation and reporting
- [ ] Replace ad-hoc error messages with standardized error builder

## 6. Replace Manual Char Parsing with TokenStream
- [ ] Refactor all functions using ISource directly to use TokenStream
- [ ] Remove/adapt utility functions that duplicate token stream logic

## 7. Consolidate Utility Functions
- [ ] Merge similar helpers into a single module
- [ ] Remove redundant helpers after migration

## 8. Modularize and Document
- [ ] Ensure each parsing concern is modularized
- [ ] Add documentation and usage examples for each shared utility

## 9. Test and Validate
- [ ] Run all tests after each refactor step
- [ ] Add/expand tests for edge cases
