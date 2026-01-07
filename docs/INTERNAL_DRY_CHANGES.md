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
