## Goals
## Phase 0 – Inventory & Guardrails
## Phase 1 – Error Handling & Result Types
### 1.1 Unify parser error representation
### 1.2 Centralize structure/indentation/limit error text
### 2.1 Centralize token-based indentation validation
### 2.2 Unify mapping/sequence head classification
### 2.3 DRY comment handling and trivia skipping
### 3.1 Consolidate `Node` → key/string helpers
### 3.2 DRY string escaping logic
### 3.3 Centralize traversal & path logic
### 4.1 Enforce consistent loop/collection guards
### 4.2 Align `CapacityHints` and `NodeBuilder` usage
### 5.1 Reconcile `StringPool` and `StringInterner`
### 5.2 DRY stats & reporting
### 6.1 Validation engine & validators
### 6.2 Devtools traversal & formatting
### 6.3 Embedded types & conversions
### 6.4 IO abstractions
### 7.1 Granular refactor steps
### 7.2 Test coverage checkpoints
### 7.3 Backward compatibility
### 8.1 Prioritization
### 8.2 High impact / low risk (start here)
### 8.3 Medium impact
### 8.4 Higher impact / optional
This plan should be treated as a living document; as refactors land, update sections with concrete status, links to PRs, and any additional DRY opportunities discovered during implementation.
# YAML Library DRY Refactor Plan

_Last updated: 2026-01-13_

This document outlines a crate-wide DRY refactor plan for the `library` crate. It focuses on consolidating duplicated patterns, centralizing cross-cutting concerns (error handling, traversal, limits), and improving maintainability without changing the public API surface of `yaml_lib` in the first phase.

This plan complements, but does not replace, the existing internal notes in [docs/INTERNAL_DRY_CHANGES.md](INTERNAL_DRY_CHANGES.md) and the parser-specific notes in [docs/PARSER_TOKENSTREAM_REFACTOR_PLAN.md](PARSER_TOKENSTREAM_REFACTOR_PLAN.md).

## Goals

- [ ] Reduce copy-paste logic across parser, stringify, validation, and devtools.
- [ ] Centralize library-wide idioms (errors, traversal, limits) into reusable utilities.
- [ ] Make it easier to evolve behavior (e.g., indentation rules, string escaping) in one place.
- [ ] Keep external API stable in Phase 1; allow only additive/internal changes.

## Phase 0 – Inventory & Guardrails

1. **Baseline inventory**
  - [ ] Confirm module map in [library/src](../library/src) is complete and up to date.
  - [ ] Tag major subsystems for DRY work:
     - Core model: [library/src/nodes](../library/src/nodes)
     - Parser: [library/src/parser](../library/src/parser)
     - Stringify: [library/src/stringify](../library/src/stringify)
     - Validation: [library/src/validation](../library/src/validation)
     - Error handling: [library/src/error](../library/src/error)
     - Utils (perf, streaming, interning): [library/src/utils](../library/src/utils)
     - IO: [library/src/io](../library/src/io)
     - Embedded: [library/src/embedded](../library/src/embedded)
     - Devtools: [library/src/devtools](../library/src/devtools)

2. **Guardrails**
  - [ ] No public signature changes in `lib.rs` exports in Phase 1; refactors are internal.
  - [ ] Maintain behavior for all existing tests in [library/tests](../library/tests) and [library/src/integration_tests](../library/src/integration_tests).
  - [ ] Prefer internal helpers/traits over new feature flags.

---

## Phase 1 – Error Handling & Result Types

### 1.1 Unify parser error representation

**Problem:**
- Parser/document code (e.g. [library/src/parser/document/main_loop.rs](../library/src/parser/document/main_loop.rs), [library/src/parser/document/helpers.rs](../library/src/parser/document/helpers.rs)) predominantly uses `Result<T, String>` with hand-built error strings.
- Library-wide error type is `YamlError` / `ErrorKind` in [library/src/error/mod.rs](../library/src/error/mod.rs) and `error/enhanced.rs`.

**Plan:**
- [x] Introduce an internal parser error alias:
  - [x] `type ParseResult<T> = core::result::Result<T, crate::error::YamlError>;`
  - [x] Place in [library/src/parser/mod.rs](../library/src/parser/mod.rs) or a new [library/src/parser/error.rs](../library/src/parser/error.rs).
- [x] Update `error_builder` functions in [parser/document/error_builder.rs](../library/src/parser/document/error_builder.rs) to build `YamlError` as well as `String`:
  - [x] Map `ErrorCategory` to `ErrorKind` via a new `build_yaml()` method.
  - [ ] Optionally enrich `YamlError` with line/column when available.
- [x] Add thin conversion functions for legacy callers that still expect `String`:
  - [x] `fn to_string_error(err: YamlError) -> String` for transitional use.
  - [ ] Gradually migrate document parsing functions from `Result<T, String>` to `ParseResult<T>` starting at the leaves (block/inline value parsing, anchor/tag helpers) and moving up to `parse_document`.
  - [x] Convert core document helpers to `ParseResult<T>`: `parse_document_main_loop`, `parse_document_contents`, `parse_value`, `parse_mapping`, `parse_sequence`, `is_document_marker`, `is_doc_end`, and anchor/merge helpers in `document/anchors.rs`.

### 1.2 Centralize structure/indentation/limit error text

**Problem:**
- Similar phrases and patterns for structure/indentation/limit errors are scattered (e.g. `structure_error`, `forbidden_error`, plain `format!()` in helpers).

**Plan:**
- [x] Keep `ErrorBuilder` in [parser/document/error_builder.rs](../library/src/parser/document/error_builder.rs) as the single entry for parser error message shape.
- [x] Add dedicated helpers:
  - [x] `fn tab_indentation_error(source: &mut dyn ISource) -> YamlError`
  - [x] `fn invalid_comment_spacing_error(...) -> YamlError`
  - [x] `fn mapping_key_error(...) -> YamlError`
- [x] Replace ad-hoc `return Err(structure_error(...))`/`forbidden_error` calls across document parsing with these new helpers to ensure consistent wording and category mapping.
  - [x] Use `mapping_key_error_yaml` + `to_string_error` for mapping-key and anchored-mapping structural errors in token-based mapping/value/inline parsers and H7J7 document post-check.

---

## Phase 2 – TokenStream, Indentation, and Comment Handling

### 2.1 Centralize token-based indentation validation

**Problem:**
- `validate_indentation_and_whitespace` and `validate_indentation_tokens` in [parser/document/helpers.rs](../library/src/parser/document/helpers.rs) are the intended central hook but are only partially used.
- Individual parsing functions risk re-implementing similar checks.

**Plan:**
- [x] Promote `validate_indentation_tokens` to a public (within parser) utility in [parser/utils](../library/src/parser/utils):
  - [x] Move it into a new `indentation.rs` module under `parser/utils` and expose it via `parser::utils::indentation::validate_indentation_tokens`.
  - [x] Keep `ParsingContext` in [parser/document/context.rs](../library/src/parser/document/context.rs) as the shared context struct, and use it from the shared indentation utility.
- [x] Standardize a small set of entry-points:
  - [x] `validate_indentation_at_line_start(stream: &TokenStream, ctx: &ParsingContext)`
  - [x] `validate_trailing_content_after_document_end(stream: &mut TokenStream)` (wrapping `validate_no_inline_content_after_document_end`).
- [x] Replace any direct ad-hoc newline/indentation scanning in document parsing with calls to these utilities (or to `ErrorBuilder`-backed helpers in `error_builder.rs`).

### 2.2 Unify mapping/sequence head classification

**Problem:**
- There are overlapping concepts between `peek_ahead_for_mapping_key` and `classify_block_head` in [helpers.rs](../library/src/parser/document/helpers.rs), plus legacy character-based checks.

**Plan:**
- [x] Finalize `BlockHeadKind`/`classify_block_head` as the single classifier API.
  - [x] Use `classify_block_head` in `parse_document_contents` to centralize head decisions for mappings/sequences/plain scalars/quoted keys.
- [x] Replace remaining character-level lookahead branches in [parser/document/value.rs](../library/src/parser/document/value.rs), [parser/document/mapping.rs](../library/src/parser/document/mapping.rs), and [parser/document/sequence.rs](../library/src/parser/document/sequence.rs) with token-stream based parsing driven by `classify_block_head` at the document level.
- [x] Keep `peek_ahead_for_mapping_key` as an implementation detail of `classify_block_head` (not called directly from higher-level parse functions).

### 2.3 DRY comment handling and trivia skipping

**Problem:**
- `parse_document_main_loop` reconstructs `TokenStream` just to skip comments and detect patterns like `8XDJ`.
- Other parsing sites perform similar comment-skipping or inline-comment handling.

**Plan:**
- [x] In `TokenStream` (see [parser/token_stream.rs](../library/src/parser/token_stream.rs)), harden/extend helpers:
  - [x] `skip_trivia()` (whitespace + comments + newlines as appropriate).
  - [x] `skip_newlines_and_comments()` as already used.
- [x] Introduce a dedicated helper for top-level comment + indentation validation (used by `8XDJ`, etc.) in `parser/utils/comments.rs` and use it from `parse_document_main_loop`.
- [x] Replace the ad-hoc 8XDJ comment loop in `parse_document_main_loop` with this shared helper.

---

## Phase 3 – Node Traversal, Key/String Conversion, and Formatting

### 3.1 Consolidate `Node` → key/string helpers

**Problem:**
- Very similar `node_to_key_string` implementations exist in:
  - [stringify/json.rs](../library/src/stringify/json.rs)
  - [stringify/xml.rs](../library/src/stringify/xml.rs)
  - (Likely similar logic in TOML and Bencode stringifiers.)
- Numeric → string conversion is duplicated for all `Numeric` variants.

**Plan:**
- [x] Add a pair of core helpers in [stringify/format.rs](../library/src/stringify/format.rs):
  - [x] `fn node_to_string_lossy(node: &Node) -> String` (general-purpose, using YAML stringify as fallback).
  - [x] `fn node_to_key_like_string(node: &Node) -> String` (rules tuned for keys: `None` → `""`, `Boolean` → `"true"/"false"`, etc.).
- [x] Refactor JSON/XML/TOML/Bencode stringifiers to use these helpers instead of local `node_to_key_string` copies.
- [x] For numeric formatting, add methods to `Numeric` in [nodes/node.rs](../library/src/nodes/node.rs):
  - [x] `fn to_string_lossy(&self) -> String` used wherever we currently manually `match` on each variant.

### 3.2 DRY string escaping logic

**Problem:**
- `escape_json_string` (JSON) and `escape_xml_string` (XML) are specialized but share patterns.
- YAML default string escaping (in [stringify/default.rs](../library/src/stringify/default.rs)) likely reimplements a subset.

**Plan:**
- [x] Add `utils::escape` module under [library/src/utils](../library/src/utils):
  - [x] `fn escape_for_json(s: &str) -> String`
  - [x] `fn escape_for_xml(s: &str) -> String`
  - [ ] (Optionally) `fn escape_for_yaml_plain(s: &str) -> String` and `escape_for_yaml_double_quoted`.
- [x] Replace inline implementations in `stringify/json.rs` and `stringify/xml.rs` with calls into `utils::escape`.
- [ ] Consider wiring YAML helpers in `stringify/default.rs` to shared escape utilities if/when we want cross-format reuse.
- [x] Ensure tests in corresponding modules validate escape behavior before and after refactor (covered by full library test suite).

### 3.3 Centralize traversal & path logic

**Problem:**
- `NodeIterator`, `NodeIteratorExt`, `NodePath`, and `NodeStream` in [utils/streaming.rs](../library/src/utils/streaming.rs) encapsulate generic traversal.
- Devtools (debug/diff/inspect) and validation schema engine may still implement ad-hoc walks over `Node` trees.

**Plan:**
- [x] Make `NodeIteratorExt` the canonical way to walk `Node` trees:
  - [x] Replace manual recursion in devtools inspection ([library/src/devtools/inspect.rs](../library/src/devtools/inspect.rs)) for `find_by_type` with `iter_depth_first`.
  - [ ] In other devtools modules ([library/src/devtools](../library/src/devtools)), incrementally migrate ad-hoc traversals to `iter_depth_first` / `NodeStream` where this meaningfully reduces duplication.
  - [ ] In validation ([library/src/validation/engine.rs](../library/src/validation/engine.rs)), prefer `NodePath` and `NodeIteratorExt` for generic walking where appropriate; keep schema-specific object/array handling where it adds clarity.
- [ ] If performance hotspots exist, introduce specialized iterators under `utils/streaming.rs` rather than duplicating traversal logic.

---

## Phase 4 – Limits, Guards, and Capacity Hints

### 4.1 Enforce consistent loop/collection guards

**Status:**
- [x] Top-level stream/document and legacy sequence/explicit-key loops now use loop_guard macros.
- [x] Macro usage errors fixed; all tests pass.
- [ ] Audit remaining parser loops for guard macros (in progress).

### 4.2 Align `CapacityHints` and `NodeBuilder` usage

**Status (2026-01-14):**
- [x] Defined parser-facing capacity profile using `CapacityHints::small()` in mapping/sequence and inline token parsers.
- [x] Refactored mapping, sequence, and inline token parsers to use `NodeBuilder` for pre-allocated Vec allocation.
- [x] All tests pass after refactor (767/767).
- [x] Audited and updated additional allocation sites in inline token parsers.
- [ ] Continue auditing for further allocation consistency as needed.

---

## Phase 5 – String Pooling vs String Interning

### 5.1 Reconcile `StringPool` and `StringInterner`

**Problem:**
- `StringPool` in [utils/optimization.rs](../library/src/utils/optimization.rs) and `StringInterner` / `SimpleInterner` in [utils/string_interner.rs](../library/src/utils/string_interner.rs) offer overlapping capabilities and separate stats.

**Plan:**
- [x] Choose one primary abstraction for general-purpose deduplication:
  - [x] Use `StringInterner` (thread-safe) for `std`, `SimpleInterner` for `no_std + alloc`.
- [x] Deprecate `StringPool` and migrate callers to `StringInterner` directly.
- [x] Ensure `PerformanceOptimizer` exposes a single, coherent configuration for string deduplication.

### 5.2 DRY stats & reporting

**Problem:**
- `InternerStats` (hits/misses/unique) and `memory_savings` logic exist only in `StringInterner`.

**Plan:**
- [x] Expose a small, shared stats struct used by both interner variants (thread-safe and simple).
- [x] Add a utility function for human-readable summaries that devtools can reuse (e.g., `format_interner_stats(stats: &InternerStats) -> String`).

---

## Phase 6 – Validation, Devtools, Embedded, IO (in progress)

### 6.1 Validation engine & validators

**Problem (likely):**
- Built-in validators in [validation/validators.rs](../library/src/validation/validators.rs) may duplicate:
  - Error construction patterns (messages, path prefixes).
  - Bounds checking and type dispatch.

**Plan:**
- [ ] Introduce a small internal `ValidationContextCore` under [validation/engine.rs](../library/src/validation/engine.rs):
  - [ ] Provides helpers like `fail_type_mismatch(expected, found)`, `fail_range(...)`, `fail_required(...)`.
- [ ] Refactor individual validators to use these common helpers instead of hand-building messages.
- [ ] Ensure all validators attach consistent path / schema location info.

### 6.2 Devtools traversal & formatting

**Problem (likely):**
- `devtools::debug`, `devtools::diff`, and `devtools::inspect` each walk `Node` structures and build string representations.

**Plan:**
- [ ] Rebase devtools traversal on `NodeIteratorExt` and `NodePath` from [utils/streaming.rs](../library/src/utils/streaming.rs).
- [ ] Create a shared internal `NodePrinter` utility for:
  - [ ] Indented tree printing (used by `print_tree`, debug, and tracing).
  - [ ] Compact inline representation (can delegate to `node_to_inline_string` in [parser/document/helpers.rs](../library/src/parser/document/helpers.rs) or move that function to a shared `utils` location).

### 6.3 Embedded types & conversions

**Problem (likely):**
- Embedded modules ([library/src/embedded](../library/src/embedded)) may hand-roll conversions between full `Node`/`Numeric` and lightweight representations.

**Plan:**
- [ ] Define `From`/`TryFrom` implementations between full and lightweight node types where possible.
- [ ] Centralize numeric down-casting logic using the helpers on `Numeric` (e.g., `to_i32`, `to_f32`).
- [ ] Reuse any size-accounting logic from embedded-specific `Numeric` extensions (see [nodes/node.rs](../library/src/nodes/node.rs) under `#[cfg(feature = "embedded")]`).

### 6.4 IO abstractions

**Problem (likely):**
- IO modules ([library/src/io](../library/src/io)) provide both source and destination wrappers with overlapping buffer/file logic.

**Plan:**
- [ ] Identify duplicated patterns between `io::sources::buffer`, `io::sources::file`, `io::destinations::buffer`, and `io::destinations::file`.
- [ ] Introduce internal shared helpers (e.g., `fn read_all<R: Read>(...)`, `fn write_all<W: Write>(...)`) where appropriate, keeping public types unchanged.

---

## Phase 7 – Testing & Migration Strategy

1. **Granular refactor steps**
   - [ ] For each area above, refactor in small, well-scoped PRs:
     - [ ] One for parser error unification.
     - [ ] One for indentation/comment centralization.
     - [ ] One for `Node` formatting helpers and stringify deduplication.
     - [ ] One for limits/guards.
     - [ ] One for interning/pooling consolidation.

2. **Test coverage checkpoints**
   - [ ] After each refactor:
     - [ ] Run `cargo test` for the library crate (and dedicated parser/stringify tests where present).
     - [ ] Re-run the YAML test suite harness under `yaml-test-suite` if part of the normal CI.

3. **Backward compatibility**
  - [ ] Keep `Result<T, String>` interfaces as thin wrappers around internal `Result<T, YamlError>` until all callsites are migrated.
  - [ ] Only once internal migration is done, consider exposing richer error types in the public API as a separate, clearly labeled breaking change.

---

## Prioritization

1. **High impact / low risk (start here):**
  - [ ] Parser error unification via `YamlError`.
  - [ ] Centralized indentation/comment validation using `ParsingContext` and `TokenStream`.
  - [ ] `Node` → key/string helpers + numeric formatting consolidation.
2. **Medium impact:**
  - [ ] Consistent loop/collection guards.
  - [ ] Devtools & validation refactors to reuse traversal/path utilities.
3. **Higher impact / optional:**
  - [ ] Full consolidation of string pooling vs string interning.
  - [ ] Embedded conversion refinements that might affect size/perf trade-offs.

This plan should be treated as a living document; as refactors land, update sections with concrete status, links to PRs, and any additional DRY opportunities discovered during implementation.