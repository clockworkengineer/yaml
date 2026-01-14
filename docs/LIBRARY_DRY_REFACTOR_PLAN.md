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
- [ ] Add a pair of core helpers in [nodes/node.rs](../library/src/nodes/node.rs) or a new `nodes/format.rs`:
  - [ ] `fn node_to_string_lossy(node: &Node) -> String` (general-purpose, using YAML stringify as fallback).
  - [ ] `fn node_to_key_like_string(node: &Node) -> String` (rules tuned for keys: `None` → `""`, `Boolean` → `"true"/"false"`, etc.).
- [ ] Refactor JSON/XML/TOML/Bencode stringifiers to use these helpers instead of local `node_to_key_string` copies.
- [ ] For numeric formatting, add methods to `Numeric`:
  - [ ] `fn to_string_lossy(&self) -> String` used wherever we currently manually `match` on each variant.

### 3.2 DRY string escaping logic

**Problem:**
- `escape_json_string` (JSON) and `escape_xml_string` (XML) are specialized but share patterns.
- YAML default string escaping (in [stringify/default.rs](../library/src/stringify/default.rs)) likely reimplements a subset.

**Plan:**
- [ ] Add `utils::escape` module under [library/src/utils](../library/src/utils):
  - [ ] `fn escape_for_json(s: &str) -> String`
  - [ ] `fn escape_for_xml(s: &str) -> String`
  - [ ] (Optionally) `fn escape_for_yaml_plain(s: &str) -> String` and `escape_for_yaml_double_quoted`.
- [ ] Replace inline implementations in `stringify/json.rs`, `stringify/xml.rs`, and `stringify/default.rs` with calls into `utils::escape`.
- [ ] Ensure tests in corresponding modules validate escape behavior before and after refactor.

### 3.3 Centralize traversal & path logic

**Problem:**
- `NodeIterator`, `NodeIteratorExt`, `NodePath`, and `NodeStream` in [utils/streaming.rs](../library/src/utils/streaming.rs) encapsulate generic traversal.
- Devtools (debug/diff/inspect) and validation schema engine may still implement ad-hoc walks over `Node` trees.

**Plan:**
- [ ] Make `NodeIteratorExt` the canonical way to walk `Node` trees:
  - [ ] Replace manual recursion in devtools modules ([library/src/devtools](../library/src/devtools)) with `iter_depth_first` / `NodeStream` where appropriate.
  - [ ] In validation ([library/src/validation/engine.rs](../library/src/validation/engine.rs)), prefer `NodePath` and `NodeIteratorExt` for walking nested structures instead of reimplementing index/key traversal.
- [ ] If performance hotspots exist, introduce specialized iterators under `utils/streaming.rs` rather than duplicating traversal logic.

---

## Phase 4 – Limits, Guards, and Capacity Hints

### 4.1 Enforce consistent loop/collection guards

**Problem:**
- `loop_guard_init!`, `loop_guard_check!`, `collection_size_check!`, and `combined_loop_guard!` live in [parser/document/loop_guards.rs](../library/src/parser/document/loop_guards.rs) but might not be used uniformly across all loops constructing sequences/mappings.

**Plan:**
- [ ] Audit all parser loops in [parser/document](../library/src/parser/document) and [parser/lexer.rs](../library/src/parser/lexer.rs) for:
  - [ ] `while let Some(...)` loops that consume tokens/characters.
  - [ ] Loops that push into `Vec<Node>` or `(Node, Node)`.
- [ ] For each such loop:
  - [ ] Introduce `loop_guard_init!(counter);` at loop setup.
  - [ ] Use `loop_guard_check!` or `combined_loop_guard!` at the top of the loop body.
- [ ] Tie `MAX_LOOP_ITERATIONS`, `MAX_SEQUENCE_ITEMS`, `MAX_MAPPING_PAIRS` to embedded/config limits where appropriate to avoid divergent hard-coded values.

### 4.2 Align `CapacityHints` and `NodeBuilder` usage

**Problem:**
- `CapacityHints`, `PerformanceOptimizer`, and `NodeBuilder` in [utils/optimization.rs](../library/src/utils/optimization.rs) encode the same idea (size hints and pre-allocation), but usage may be inconsistent.

**Plan:**
- [ ] Define a simple `CapacityProfile` for the parser in [parser/config.rs](../library/src/parser/config.rs) that maps to `CapacityHints`.
- [ ] Standardize construction of arrays/mappings in parser code via `NodeBuilder`:
  - [ ] For example, in sequence/mapping parsing modules, replace `Vec::new()` / `Vec::with_capacity` with `node_builder.build_array_with_capacity(...)` or `build_mapping_with_capacity(...)`.
- [ ] Update `NodeBuilder::update_hints` to be called from hot parsing paths once actual sizes are known.

---

## Phase 5 – String Pooling vs String Interning

### 5.1 Reconcile `StringPool` and `StringInterner`

**Problem:**
- `StringPool` in [utils/optimization.rs](../library/src/utils/optimization.rs) and `StringInterner` / `SimpleInterner` in [utils/string_interner.rs](../library/src/utils/string_interner.rs) offer overlapping capabilities and separate stats.

**Plan:**
- [ ] Choose one primary abstraction for general-purpose deduplication:
  - [ ] Likely `StringInterner` (thread-safe) for `std`, `SimpleInterner` for `no_std + alloc`.
- [ ] Re-implement `StringPool` as a thin adapter over `StringInterner`:
  - [ ] Or deprecate `StringPool` and migrate callers to `StringInterner` directly.
- [ ] Ensure `PerformanceOptimizer` exposes a single, coherent configuration for string deduplication.

### 5.2 DRY stats & reporting

**Problem:**
- `InternerStats` (hits/misses/unique) and `memory_savings` logic exist only in `StringInterner`.

**Plan:**
- [ ] Expose a small, shared stats struct used by both interner variants (thread-safe and simple).
- [ ] Add a utility function for human-readable summaries that devtools can reuse (e.g., `format_interner_stats(stats: &InternerStats) -> String`).

---

## Phase 6 – Validation, Devtools, Embedded, IO

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