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

#### 2. ✅ Split `lexer.rs` (1729 lines, 61KB) — COMPLETED

Used the `include!` macro strategy so private struct fields remain accessible without any visibility changes.

**Implemented structure:**
```
parser/lexer/
  mod.rs           - Token enum, Lexer struct, new/accessors, scan_token,
                     peek_ahead, scan_until_newline, snapshot/restore, tests
  block.rs         - peek_next_non_whitespace, skip_horizontal_whitespace,
                     emit_indentation_token_if_any, scan_indentation, handle_newline
  flow.rs          - validate_post_flow_closer
  decorators.rs    - scan_tag, scan_anchor, scan_alias, scan_directive
  strings.rs       - scan_single_quoted, scan_double_quoted
  scalars.rs       - scan_plain_scalar
```

All callers unchanged — the `include!` macro injects each file into the `lexer` module at compile time, preserving all private-field access patterns.
Verified: `cargo check` clean, 402/402 YAML test suite tests passing.

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
  ├── lexer/ ✅ Split into 6 focused files
  │   ├── mod.rs       - struct, Token, scan_token, utilities, tests
  │   ├── block.rs     - indentation & newline handling
  │   ├── flow.rs      - flow-closer validation
  │   ├── decorators.rs - tag/anchor/alias/directive scanning
  │   ├── strings.rs   - quoted scalar scanning
  │   └── scalars.rs   - plain scalar scanning
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

The parser structure is significantly improved. Steps 1 and 2 of the recommended improvements are now complete:
1. ✅ **helpers.rs** — Split into 5 focused sub-modules (`core`, `document_markers`, `validation`, `peek_ahead`, `comments`)
2. ✅ **lexer.rs** — Split into 6 focused sub-files using the `include!` macro strategy

The remaining large files (`tokens/mapping.rs`, `tokens/value.rs`) are focused on specific responsibilities and are maintainable as-is.
