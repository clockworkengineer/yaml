# Lexer/Token Refactor Progress Summary (Dec 3, 2025)

## Current State

### mapping.rs / mapping_tokens.rs
- Complete: All mapping logic uses tokens and token streams.

### sequence.rs
- Mostly Complete: Main sequence parsing uses TokenStream and matches on tokens.
- Minor Work: Ensure all helpers and nested sequence logic are token-based; remove any legacy char/string checks.

### inline.rs
- Complete: Inline mapping parsing uses tokens for keys, values, and structure.

## Completed Work

### mapping.rs / mapping_tokens.rs
- All mapping logic uses tokens and token streams.

### sequence.rs
- Main sequence parsing uses TokenStream and matches on tokens.

### inline.rs
- Inline mapping parsing uses tokens for keys, values, and structure.

### block_scalar.rs
- Block scalar parsing uses tokens for content and indentation.

### explicit_key.rs
- Explicit key detection and entry parsing use tokens.

### helpers.rs
- Deprecated helpers removed.
- block_scalar.rs: Minor cleanup for edge cases and helpers.
