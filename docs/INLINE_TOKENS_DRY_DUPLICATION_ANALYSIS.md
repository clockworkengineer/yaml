# Step 1: Identify Duplicated Logic in inline_tokens.rs

## Overview
This document lists all duplicated or similar logic found in `inline_tokens.rs` that can be refactored using DRY principles.

## 1. Error Handling
- Multiple calls to `syntax_error(stream.source_mut(), ...)` for error construction in both sequence and mapping parsers.
- Similar error messages for unexpected tokens, end of input, and malformed input.

## 2. Token Trivia Skipping
- Repeated calls to `stream.skip_trivia()?` before parsing values, keys, or after consuming tokens in both sequence and mapping functions.

## 3. Progress Checks
- The `ensure_progress()` function is used after parsing keys and values in mapping, but similar logic could be needed elsewhere.

## 4. Parsing Values
- Both sequence and mapping parsers use `parse_value_with_tokens(stream, directives, depth + 1)?` for value parsing.
- Special handling for double-colon scalars in sequences and for empty keys/values in mappings.

## 5. Handling Trailing Commas and End Tokens
- Both sequence and mapping parsers allow trailing commas and check for closing brackets/braces.
- Logic for handling `Some(Token::Comma)` and `Some(Token::FlowSequenceEnd)`/`Some(Token::FlowMappingEnd)` is similar.

## 6. Node Construction
- Construction of `Node::Array(items)` and `Node::Mapping(pairs)` is repeated.
- Special-case handling for sets (`is_set` flag) in mapping parser.

## 7. Test Patterns
- Test cases for empty, simple, and nested collections repeat similar setup and assertions.

## 8. Iteration and Loop Structure
- Both parsers use a loop to process tokens, with similar structure for handling items/entries and error conditions.

## Next Steps
- Centralize error handling with a macro or helper function.
- Abstract trivia skipping into a single helper.
- Generalize value/key parsing logic.
- Consolidate trailing comma and end token handling.
- Create helpers for node construction.
- Refactor test setup if possible.

---
This analysis will guide the next steps in refactoring for DRY compliance.
