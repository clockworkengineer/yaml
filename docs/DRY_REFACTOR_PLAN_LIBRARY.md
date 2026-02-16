# DRY Refactor Plan for `yaml_lib`

This document collects medium-term DRY refactors that are behavior-neutral but should reduce duplication and make future changes safer.

## 1. Whitespace / Tab / Comment Handling

Goal: eliminate hand-written character tests like `c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == '#'` in favor of small, shared helpers that build on `ISource::is_whitespace` and the `CHAR_*` constants.

Suggested steps:
- Add helper functions in `library/src/utils/mod.rs` (or a small submodule) such as:
  - `is_line_terminator(c: char) -> bool` (uses `CHAR_NEWLINE` / `CHAR_CARRIAGE_RETURN`).
  - `is_horizontal_space(c: char) -> bool` (uses `CHAR_SPACE` / `CHAR_TAB`).
  - `is_comment_start(c: char) -> bool` (uses `CHAR_HASH`).
- Use these helpers from:
  - `parser/document/parse.rs` (directive scans, `%` lookahead, `---`+tag guard).
  - `parser/document/helpers.rs` (post-`---` tag scan in `parse_document_markers`).
  - `parser/directives.rs` (line parsing and directive loop).
  - `utils/mod.rs` (`skip_whitespace_and_comments`, `skip_whitespace_and_comments_validate_tabs`, `unescape_double_quoted`).

Behavior: should be strictly behavior-neutral; the goal is only to centralize the character sets and rely on `constants::CHAR_*` everywhere.

## 2. Document Marker Handling ("---" / "...")

Goal: centralize detection and classification of document start/end markers so no call site needs to special-case the raw `"---"` or `"..."` strings.

Suggested steps:
- Extend `parser/document/helpers.rs` or introduce a small `document_markers` helper module providing:
  - `enum DocMarker { Start, End }`.
  - `fn classify_marker(ts: &mut TokenStream) -> Option<DocMarker>` that returns `Some(Start)` for `Token::DocumentStart`, `Some(End)` for `Token::DocumentEnd`, and `None` otherwise.
- Use `classify_marker` in:
  - `parser/document/parse.rs` (stream-level parsing loop).
  - `parser/document/contents.rs` (`is_doc_end` helper and any marker-aware branches).
  - `parser/document/main_loop.rs` (`is_document_marker`).
  - `parse_plain_multiline_scalar` in `contents.rs` where it currently compares `line == "---" || line == "..."` to decide whether to stop.

Behavior: keep using `Token::DocumentStart` / `Token::DocumentEnd` under the hood; only centralize the branching logic.

## 3. Tag-Handle Validation After `---`

Goal: unify the logic that looks for explicit tag handles on the same line as `---` and validates them against `%TAG` directives (QLJ7).

Current duplication:
- Character-level and token-level checks appear both in:
  - `parser/document/helpers.rs::parse_document_markers`.
  - `parser/document/parse.rs` (extra QLJ7 guard right after computing `has_document_start`).

Suggested steps:
- Introduce a helper in `parser/document/helpers.rs`, e.g.:
  - `fn validate_tag_handle_after_doc_start(source: &mut dyn ISource, directives: &DirectiveContext) -> ParseResult<()>`.
- Move the logic that:
  - Consumes `---` at the current position.
  - Skips horizontal whitespace.
  - Scans the raw tag text up to whitespace or `#`.
  - Calls `DirectiveContext::validate_tag_handle_usage` and returns a formatted `YamlError` on failure.
- Replace the duplicated code in both `parse_document_markers` and `parse.rs` with calls to this helper.

Behavior: messages and error conditions should remain identical; we only want a single implementation.

## 4. Block Scalar Header Parsing

Goal: prevent divergence between the quick header validation in `parse_scalar_dispatch` and the block-scalar body logic in `parse_block_scalar`.

Suggested steps:
- In `parser/document/scalar.rs`, introduce a small struct and helper:
  - `struct BlockHeader { indicator: char, chomping: Option<char>, explicit_indent: Option<u8>, raw_meta: String }`.
  - `fn parse_block_header(s: &str) -> Result<BlockHeader, BlockScalarErrors>` that:
    - Validates the first character is `CHAR_VERTICAL_BAR` or `CHAR_GREATER_THAN`.
    - Parses the chomping indicator (`+`/`-`, if present) and optional single-digit indent 1–9.
    - Applies the same constraints as the current inline code (no `0`, at most one digit, etc.).
- Update:
  - `parse_scalar_dispatch` to call `parse_block_header(s)` instead of reimplementing the checks; it can use the result merely to choose between block vs plain.
  - `parse_block_scalar` to consume and use the same `BlockHeader` instance for `indicator`, indentation, and chomping rather than re-deriving this from the raw header string.

Behavior: keep the same error variants and texts via `block_scalar_errors.rs`; this refactor is internal.

## 5. Flow Punctuation Enforcement

Goal: ensure all flow-collection punctuation checks (comma / closer / ':' expectations) go through `flow_punctuation.rs` rather than per-site conditionals.

Status: Implemented.

Implementation summary:
- `parser/document/flow_punctuation.rs` centralizes flow punctuation behavior via:
  - `FlowContext` and `ensure_separator_or_end` for "comma or closing brace/bracket" checks in inline sequences/mappings.
  - `consume_trailing_separators_and_closers_in_block_sequence`, which mirrors the legacy post-item loop in the block sequence parser and consumes trailing `,`, `]`, and `}` tokens without changing semantics.
- `parser/document/inline_tokens.rs` uses `ensure_separator_or_end` for:
  - Inline flow sequences when a value appears without a preceding comma.
  - Inline flow mappings after each key-value pair (requiring comma or `}`).
- `parser/document/tokens/sequence.rs` now calls `consume_trailing_separators_and_closers_in_block_sequence` after parsing an inline flow collection as a sequence item, instead of open-coded loops.

Behavior: no intended changes; this guarantees one place to evolve punctuation rules while matching existing error messages and control flow.

## 6. Shared Node Traversal Helpers

Goal: avoid repeating ad-hoc recursive matches over `Node` in different subsystems (tag validation, stats, validation engine, etc.).

Current call sites that walk `Node` trees:
- Tag validation in `parser/document/main_loop.rs` (`validate_tags_rec`).
- Validation engine in `validation/engine.rs` (collecting paths, applying schema).
- Metrics/stats in `utils/performance.rs` (`DocumentStats::from_node`).

Suggested steps:
- Confirm and reuse the existing visitor utilities noted in `INTERNAL_DRY_CHANGES.md` (`parser::utils::visit::{visit, visit_mut}` and `node_utils` helpers).
- For modules that still implement their own recursion over `Node`, refactor to:
  - Use `visit` for read-only traversals.
  - Use `visit_mut` where mutation is needed (e.g., anchor resolution, merge expansion).
- Keep domain logic (e.g., tag validation, stats accumulation) in small closures passed into the visitor rather than spreading recursion across modules.

Behavior: visiting order and covered node variants must remain the same; tests should confirm no behavior change.

## 7. Validation Error Message DRY

Goal: keep wording and formatting of validation error messages consistent and centralized.

Current pattern:
- `validation/error.rs` and `validation/validators.rs` contain repeated string templates like:
  - "Value must be between {} and {}" / "Length must be between {} and {}".
  - "Must be one of: {}".

Suggested steps:
- Add a small `validation/messages.rs` (similar in spirit to `error/messages.rs`) with string constants or helper constructors for these recurring phrases.
- Update validators to use these helpers while preserving exact current texts to avoid surprising users/tests.

Behavior: should be behavior-neutral in terms of both messages and validation outcomes.

## 8. Official Suite Fix Test Harness Helpers

Goal: reduce boilerplate in `integration_tests/official_suite_fixes.rs` so new suite-based regression tests are cheaper to add.

Suggested steps:
- Introduce small helpers or macros in the same module, e.g.:
  - `fn assert_parses(yaml: &[u8])` that wraps `BufferSource::new`, `parse`, and `assert!(result.is_ok(), ...)`.
  - `fn assert_fails(yaml: &[u8], label: &str)` that wraps the `is_err()` assertion and includes the label in the message.
- Refactor existing tests like `test_5trb_unterminated_quoted_scalar`, `test_229q_sequence_of_mappings`, `test_26dv_whitespace_around_colon`, `test_2cms_plain_multiline`, etc. to use these helpers.

Behavior: no change to which inputs are accepted or rejected; only test ergonomics improve.

---

All of the above are intended as **behavior-neutral** refactors. Each item should be implemented incrementally with `cargo test` runs (and, where appropriate, YAML test-suite runs) after each cluster of changes to guard against regressions.
