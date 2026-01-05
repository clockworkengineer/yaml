# Internal DRY Refactors Summary

This document summarizes recent internal, behavior-neutral DRY refactors applied to token-based parsing paths in `yaml_lib`.

Highlights:
- Token helpers leveraged consistently where safe:
  - `TokenStream::skip_newlines_and_comments()` used inside per-iteration loops to avoid manual `Newline`/`Comment` handling.
  - `TokenStream::skip_comments()` replaces local comment-skipping loops.
- Legacy wrapper `parse_sequence_inner()` now calls `skip_newlines_and_comments()` once per loop iteration.
- `value` token parser uses `skip_comments()` before value dispatch.
- `mapping` token parser removes redundant `Newline`/`Comment` match arms; relies on the initial trivia skip.

Behavior guarantees:
- No semantic changes intended or observed; all changes are strictly internal DRY cleanups.
- Baseline test results remain stable (all library tests pass).

Verification:
- Quiet runs confirm stability; see files under `target/` such as `yaml_suite_full_after_dry.txt` for recent results.
