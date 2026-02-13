# Internal DRY Refactors Summary

This document summarizes recent internal, behavior-neutral DRY refactors applied to token-based parsing paths in `yaml_lib`.

Highlights:
- Token helpers leveraged consistently where safe:
  - `TokenStream::skip_newlines_and_comments()` used inside per-iteration loops to avoid manual `Newline`/`Comment` handling.
  - `TokenStream::skip_comments()` replaces local comment-skipping loops.
- Legacy wrapper `parse_sequence_inner()` now calls `skip_newlines_and_comments()` once per loop iteration.
- `value` token parser uses `skip_comments()` before value dispatch.
- `mapping` token parser removes redundant `Newline`/`Comment` match arms; relies on the initial trivia skip.

New shared utilities:
- Added `parser::utils::visit::{visit, visit_mut}` for DRY recursive traversal over `Node` trees.
- Refactored `parser/document/anchors.rs` to use `visit` and `visit_mut` in `collect_anchors`, `replace_aliases`, and `expand_merge_keys` respectively.
- Introduced `parser::document::node_utils::dedupe_mapping_pairs_by_last_occurrence()` and replaced local dedupe logic in merge expansion.
- Removed unused imports in refactored modules as part of cleanup.
 - In `parser/document/tokens/mapping.rs`, removed redundant array→string key normalization in `parse_mapping_pair`; rely on `force_key_to_string()` at insertion for consistent key handling.

Behavior guarantees:
- No semantic changes intended or observed; all changes are strictly internal DRY cleanups.
- Baseline test results remain stable (all library tests pass).

Verification:
- Quiet runs confirm stability; see files under `target/` such as `yaml_suite_full_after_dry.txt` for recent results.
- Library tests run clean after visitor refactor; see `cargo test --package yaml_lib --lib -- --nocapture` output.
- Static error scans are used for incremental regression checks during DRY passes (no console test logs emitted).

Planned DRY extensions:
- Adopt `visit_mut` to centralize recursion in additional modules that traverse `Node` trees (e.g., merge handling, validation passes, stringify helpers).
- Incremental application with tests after each file-level refactor to ensure no functional regressions.
- Replace ad-hoc mapping dedupe patterns with the shared `dedupe_mapping_pairs_by_last_occurrence()` helper where applicable.

## Centralized Indentation Policy

To reduce duplication and make future indentation rule changes safer, indentation validation was centralized in [library/src/parser/document/indentation.rs](library/src/parser/document/indentation.rs).

- Added `ensure_indent_at_least(...)` for source-context errors when validating indent constraints.
- Added `ensure_indent_at_least_no_source(...)` to validate without borrowing the source (prevents borrow conflicts alongside `TokenStream`).
- Scaffolded `ensure_valid_child_indent(...)` using `ParsingContext` for future child-indent policies.

Integrations:
- Sequence head handling in [library/src/parser/document/contents.rs](library/src/parser/document/contents.rs) now uses the no-source variant before constructing a `TokenStream`.
- Mapping head handling uses the source-context variant to retain detailed error messages.

Behavior note: This is a behavior-neutral refactor; it does not alter acceptance/rejection of YAML inputs. It only consolidates policy and error construction to ease future fixes.

## Centralized Flow Punctuation

To remove duplication around common flow collection errors, flow punctuation error construction was centralized in [library/src/parser/document/flow_punctuation.rs](library/src/parser/document/flow_punctuation.rs).

  - Sequence: "Expected comma or ] in flow sequence"
  - Mapping: "Expected comma or } in flow mapping"
  - Expected ':' in flow mappings via `flow_punctuation::expected_colon_in_flow_mapping`, with callers (e.g., `TokenStream::consume_single_colon`) routed to the helper to preserve message text while reducing duplication

Integrations:
  - Inline parsers now leverage `ensure_separator_or_end(...)` to validate the token following a flow item/entry, while keeping consumption logic unchanged.
  - Centralized additional flow-context errors:
    - `unexpected_extra_closing_bracket_in_flow_sequence(...)`
    - `leading_or_double_comma_in_flow_sequence(...)`
    - `unexpected_eof_in_flow_mapping_unclosed(...)`
    - `invalid_bare_dash_entries_in_flow_sequence(...)` (keeps "Invalid use of '-' indicators inside flow sequence")
    All messages remain identical to previous strings.

Behavior note: This refactor is strictly behavior-neutral. Messages and decision points remain identical; the centralization aims to make future punctuation policy adjustments safer and localized.

## Centralized Anchor/Alias Errors

To reduce duplication and keep anchor/alias error messages consistent, error construction was centralized in [library/src/parser/document/anchor_errors.rs](library/src/parser/document/anchor_errors.rs).


Integrations:

## Centralized directive errors

- Added `library/src/parser/document/directive_errors.rs` to unify construction of directive-related errors and messages while preserving exact current strings.
- Routed the following through `DirectiveErrors` (behavior-neutral):
  - Duplicate `%YAML` directive
  - Missing YAML version after `%YAML`
  - Invalid YAML major/minor version (generic and numeric variants)
  - Malformed `%TAG` directive format
  - Undefined tag handle usage without `%TAG` declaration
  - "Directive must be followed by a document"
  - Mid-stream directive disallowance message when previous document didn't end with `...`
- Updated call sites in `parser/directives.rs` and `parser/document/parse.rs` accordingly.

## Centralized block scalar errors

- Added `library/src/parser/document/block_scalar_errors.rs` to consolidate error construction for block scalar parsing:
  - Invalid block scalar header: unexpected text after `|` or `>`
  - Invalid block scalar indentation indicator: must be a single digit 1–9
  - Literal block scalar indentation validation: blank lines before content more indented than the first content line
- Updated call sites in [library/src/parser/document/scalar.rs](library/src/parser/document/scalar.rs) to use these helpers. Messages and behavior remain identical.

## Centralized comment spacing errors

- Added [library/src/parser/document/comment_errors.rs](library/src/parser/document/comment_errors.rs) to centralize construction of comment-related parsing errors.
- Routed the quoted-scalar comment spacing checks in [library/src/parser/lexer.rs](library/src/parser/lexer.rs) through `CommentErrors::comment_must_be_separated_from_quoted_scalar_by_whitespace(...)` (both single-quoted and double-quoted closures).
- Behavior is unchanged; the exact message string is preserved while moving construction to a single place for future maintenance.

## Centralized tab/indentation errors

- Added [library/src/parser/document/indentation_errors.rs](library/src/parser/document/indentation_errors.rs) to centralize tab/indentation error construction.
- Provided helpers:
  - `tabs_not_allowed_yaml_syntax(...)` → "Tabs are not allowed as indentation in YAML" (Syntax category), used by lexer checks.
  - `tabs_not_allowed_flow_collections(...)` → "Tabs are not allowed as indentation in YAML flow collections" (Syntax category), used for flow indentation validations.
  - `tabs_not_allowed_yaml_block(...)` → wraps `tab_indentation_error_yaml(...)` (Indentation category) for block-context validations.
- Routed call sites in [library/src/parser/lexer.rs](library/src/parser/lexer.rs) and [library/src/parser/document/contents.rs](library/src/parser/document/contents.rs) to use these helpers.
- Behavior and exact messages remain identical to previous strings; only construction is centralized.

## Centralized mapping errors

- Added [library/src/parser/document/mapping_errors.rs](library/src/parser/document/mapping_errors.rs) to centralize construction of common mapping-specific errors while preserving exact strings and enhanced formatting:
  - `mapping_key_without_value_expected_value_after_colon(...)` → "YAML compliance error: Mapping key without value (expected value after colon)" with code E001
  - `invalid_indentation_after_comment_in_mapping_value(...)` → "Invalid indentation after comment: indented content cannot extend a completed scalar mapping value" with code E007 and note
  - `inconsistent_dedent_within_mapping_value_for_keys(...)` → "Invalid indentation for nested mapping key: inconsistent dedent within mapping value" with code E009 and note
- Updated call sites in [library/src/parser/document/tokens/mapping.rs](library/src/parser/document/tokens/mapping.rs) to route through these helpers. Behavior and message formatting (including error codes and notes) remain identical.

- New helpers routed in this sweep:
  - `invalid_trailing_plain_text_after_quoted_scalar(...)` → mapping value: trailing plain text after quoted scalar on the same line (syntax error)
  - `expected_explicit_key_token(...)` → explicit key parsing expects `?` and reports the current token when missing

### Mapping-specific anchor errors

- Added mapping-specific anchor error helpers while preserving exact messages and enhanced formatting:
  - `invalid_anchored_alias_key_on_alias_nodes(...)` → "Invalid anchored alias key: anchors cannot be applied to alias nodes" with code E004 and note "Anchors are not allowed on alias nodes."
  - `multiple_anchors_on_mapping_key(...)` → "A mapping key cannot have multiple anchors" with code E005 and note "A key can only have one anchor."
- Refactored `apply_decorators_to_key()` in [library/src/parser/document/tokens/mapping.rs](library/src/parser/document/tokens/mapping.rs) to call these helpers instead of constructing errors inline.

## Centralized token/value errors

- Extended [library/src/parser/document/token_errors.rs](library/src/parser/document/token_errors.rs) with additional helpers and routed call sites:
  - Duplicate decorators: `duplicate_tag_found(...)`, `duplicate_anchor_found(...)`
  - Tag handle usage: `invalid_tag_handle_usage(source, message)`; used in `TokenStream` and `value.rs`
  - Quoted-string EOF: `unterminated_single_quoted_eof(...)`, `unterminated_double_quoted_eof(...)`, `unterminated_double_quoted_eof_after_escape(...)`
  - Escape/Unicode: `invalid_escape_x_expected_2_hex(...)`, `invalid_escape_u_expected_4_hex(...)`, `invalid_escape_U_expected_8_hex(...)`, `invalid_unicode_codepoint_u4(...)`, `invalid_unicode_codepoint_u8(...)`, `invalid_escape_generic(...)`
  - Value/scalar: `unexpected_token_in_value(...)`, `expected_scalar_token(...)`
  - Document structure: `document_unexpected_plain_after_top_level_sequence(...)`

- Routed call sites:
  - [library/src/parser/token_stream.rs](library/src/parser/token_stream.rs): duplicate tag/anchor and tag handle validation
  - [library/src/parser/document/tokens/value.rs](library/src/parser/document/tokens/value.rs): tag handle validation; unexpected token in value
  - [library/src/parser/document/scalar.rs](library/src/parser/document/scalar.rs): expected scalar token
  - [library/src/parser/document/explicit_key.rs](library/src/parser/document/explicit_key.rs): expected `?` token
  - [library/src/parser/document/main_loop.rs](library/src/parser/document/main_loop.rs): missing `---` structure error
  - [library/src/parser/lexer.rs](library/src/parser/lexer.rs): quoted-string EOF and escape/unicode errors; comment spacing after closing quotes

Behavior note: All changes maintain identical error strings, categories, codes, and notes. The YAML suite baseline remains unchanged (353 passed, 49 failed).

## Flow context: post-closer validations

- Centralized flow-closer adjacent content validations:
  - `CommentErrors::comment_must_be_preceded_by_whitespace_after_flow_closer(...)` for comments immediately after `}` or `]` without whitespace.
  - `flow_punctuation::invalid_content_immediately_after_flow_closer(...)` for invalid non-whitespace characters directly following a flow closer.
- Routed `lexer.validate_post_flow_closer()` to these helpers. Messages remain identical.
