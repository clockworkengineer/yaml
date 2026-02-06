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

Integrations:

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
- Routed the quoted-scalar comment spacing check in [library/src/parser/lexer.rs](library/src/parser/lexer.rs) through `CommentErrors::comment_must_be_separated_from_quoted_scalar_by_whitespace(...)`.
- Behavior is unchanged; the exact message string is preserved while moving construction to a single place for future maintenance.
