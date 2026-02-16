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

Status: Implemented for primary internal traversals.

Implementation summary:
- Confirmed and extended the shared visitor utilities in `parser::utils::visit`:
  - Existing `visit` (read-only pre-order traversal) and `visit_mut` (mutable traversal) remain the core APIs.
  - Added `visit_with_depth(node, depth, f)` for depth-aware read-only traversals used by statistics code.
- Metrics/stats in `utils/performance.rs`:
  - `DocumentStats::from_node` now uses `visit_with_depth` instead of a bespoke recursive `analyze_node` helper.
  - All counters (`total_nodes`, `max_depth`, per-type counts, largest array/mapping, string bytes) are updated inside the visitor closure, keeping behavior identical.
- Tag validation in `parser/document/main_loop.rs`:
  - Replaced the manual `validate_tags_rec` recursion with a call to `visit`.
  - During traversal, when encountering `Node::Tagged(_, tag_raw)`, the closure calls `DirectiveContext::validate_tag_handle_usage(tag_raw)` and records the first error, preserving QLJ7 semantics and messages.
- Other existing traversals (anchor collection/replacement, merge expansion, etc.) already use `visit` / `visit_mut` per `INTERNAL_DRY_CHANGES.md` and were left unchanged.

Behavior: visiting order and covered node variants remain the same; `cargo check` and the YAML test suite both pass, confirming no behavior change.

## 7. Validation Error Message DRY

Goal: keep wording and formatting of validation error messages consistent and centralized.

Status: Implemented for validator descriptions.

Implementation summary:
- Added `validation/messages.rs` with helpers that preserve existing wording:
  - `type_must_be`, `value_must_be_between`, `value_must_be_at_least`, `value_must_be_at_most`, `no_range_restriction`.
  - `length_must_be_between`, `length_must_be_at_least`, `length_must_be_at_most`, `no_length_restriction`.
  - `must_be_one_of` for enum descriptions.
- Updated `validation/validators.rs` to use these helpers in `description()` implementations for:
  - `TypeValidator`, `RangeValidator`, `LengthValidator`, and `EnumValidator`.
- Left `validation/error.rs` unchanged; its `Display` implementation for `ValidationError` was already centralized and not duplicated elsewhere.

Behavior: description strings remain text-identical to previous formats, and both `cargo check` and the YAML test suite pass without changes in validation outcomes.

## 8. Official Suite Fix Test Harness Helpers

Goal: reduce boilerplate in `integration_tests/official_suite_fixes.rs` so new suite-based regression tests are cheaper to add.

Status: Implemented.

Implementation summary:
- Added shared helpers inside the `tests` module of `integration_tests/official_suite_fixes.rs`:
  - `fn parse_expect_ok(yaml: &[u8], label: &str) -> Node` wraps `BufferSource::new`, `parse`, optional debug printing, and panics with the provided label if parsing fails; used by tests that need to inspect the resulting node tree.
  - `fn assert_parses(yaml: &[u8], label: &str)` calls `parse_expect_ok` and discards the result for simple "should parse" checks.
  - `fn assert_fails(yaml: &[u8], label: &str)` wraps `parse` and asserts `is_err()`, including the label in the failure message.
- Refactored existing official-suite fix tests to use these helpers where appropriate:
  - Negative cases like `test_5trb_unterminated_quoted_scalar`, `test_h7j7_invalid_anchored_mapping_value`, and `test_8xdj_invalid_plain_then_mapping_at_same_indent` now call `assert_fails`.
  - Positive parse-only cases like `test_26dv_whitespace_around_colon`, `test_2cms_plain_multiline`, `test_4cqq_multiline_flow_scalars`, `test_4hvu_sequence_indentation`, `test_4zym_line_prefixes`, and `test_bu8l_valid_anchored_mapping_value` now call `assert_parses`.
  - Structure-inspecting cases like `test_229q_sequence_of_mappings`, `test_3rln_tabs_in_double_quoted`, and `test_basic_sequence_of_mappings` now use `parse_expect_ok` and then assert on the returned `Node`.
- Left exploratory/logging-only tests (e.g., `test_36f6_multiline_with_empty_line`, `test_4fj6_nested_implicit_keys`) unchanged to preserve their current diagnostic behavior.

Behavior: no change to which inputs are accepted or rejected; only test ergonomics and consistency of debug output improve. Full `cargo test` and the YAML test suite both pass with these changes.

---

All of the above are intended as **behavior-neutral** refactors. Each item should be implemented incrementally with `cargo test` runs (and, where appropriate, YAML test-suite runs) after each cluster of changes to guard against regressions.
