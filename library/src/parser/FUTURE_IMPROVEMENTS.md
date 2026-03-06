# Parser Module - Future Improvements

This document tracks potential improvements to the parser module structure.

## Completed Improvements

### Steps 1-6: Core Restructure (Completed)
- ✅ Moved error modules from `parser/document/errors/` to `parser/errors/`
- ✅ Moved token modules from `parser/document/tokens/` to `parser/tokens/`  
- ✅ Centralized utilities from `parser/document/` to `parser/utils/`
- ✅ Reviewed and kept macros in `parser/utils/macros/`
- ✅ Verified document submodules organization
- ✅ Updated all mod.rs files

### Additional Improvements
- ✅ Removed redundant `tokens/inline.rs` re-export file

## Recommended Future Improvements

### High Priority

#### ~~1. Split `utils/helpers.rs` (1008 lines, 41KB)~~ ✅ Completed

**Implemented structure:**
```
parser/utils/
  helpers/
    mod.rs              - Re-exports all helper functions (no logic)
    core.rs             - handle_directives, to_yaml_error, is_token,
                          parse_error_token, node_to_inline_string
    document_markers.rs - DocMarkerKind, classify_doc_marker,
                          peek_tag_after_doc_start, parse_document_markers,
                          parse_document_end_marker
    validation.rs       - validate_indentation_and_whitespace,
                          validate_no_tab_indentation_tokens,
                          validate_trailing_content_after_document_end,
                          validate_comment_spacing_token (+ all validation tests)
    peek_ahead.rs       - BlockHeadKind, peek_ahead_for_mapping_key,
                          classify_block_head
    comments.rs         - parse_comment_token
```

All callers unchanged — `mod.rs` re-exports every public symbol.
Verified: `cargo check` clean, 402/402 YAML test suite tests passing.

### Medium Priority

#### 2. Split `lexer.rs` (1478 lines, 61KB)
The lexer is very large and could be modularized:

**Proposed structure:**
```
parser/lexer/
  mod.rs           - Main lexer struct and core logic
  scalars.rs       - Scalar tokenization (plain, folded, literal)
  strings.rs       - Quoted string handling (single, double)
  flow.rs          - Flow collection tokenization
  block.rs         - Block structure tokenization
  directives.rs    - Directive tokenization (%YAML, %TAG)
```

**Benefits:**
- Easier to understand lexer components
- Better testability of individual tokenization features
- Reduced cognitive load per file

**Effort:** High (lexer is complex with shared state)

### Lower Priority

#### 3. Consider splitting large token files
- `tokens/mapping.rs` (929 lines, 38KB)
- `tokens/value.rs` (879 lines, 37KB)

These are large but focused on specific responsibilities. Splitting might not provide significant benefits unless clear sub-responsibilities emerge.

## Current Structure Summary

```
parser/
  ├── config.rs (15KB) - Parser configuration
  ├── directives.rs (9.5KB) - Directive handling
  ├── lexer.rs (61KB) ⚠️ Very large
  ├── token_stream.rs (29KB) - Token stream wrapper
  ├── mod.rs
  ├── document/ (17 files)
  │   ├── anchors.rs (12KB)
  │   ├── contents.rs (22KB)
  │   ├── inline_tokens.rs (28KB)
  │   ├── parse.rs (27KB)
  │   ├── scalar.rs (21KB)
  │   └── ... (smaller focused files)
  ├── tokens/ (3 files)
  │   ├── mapping.rs (38KB) - Large but focused
  │   ├── value.rs (37KB) - Large but focused
  │   └── sequence.rs (18KB)
  ├── utils/ (10 files)
  │   ├── helpers.rs (41KB) ⚠️ Very large, should split
  │   ├── error_builder.rs (15KB)
  │   ├── context.rs (9KB)
  │   └── ... (smaller utilities)
  └── errors/ (8 small files, 1-8KB each) ✅ Well organized
```

## Notes

The parser structure is significantly improved after steps 1-6. The main remaining opportunities are:
1. **helpers.rs** - Would benefit most from splitting
2. **lexer.rs** - Could be split but requires more careful design

Both improvements are optional and the current structure is maintainable.
