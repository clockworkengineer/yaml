# YAML Test Suite False-Positive Reduction Plan (2026-01-09)

## Goal

Reduce and eventually eliminate cases where the official YAML 1.2 test suite expects an error but the parser currently succeeds (false positives), while keeping existing library tests green.

This plan builds on the existing TokenStream refactor work and focuses specifically on the 69 error-marked cases that still parse successfully.

---

## 1. Classify Current Failures by Category

1. Extract failing IDs and tags from the yaml-test-suite data release:
   - IDs: 236B, 2CMS, 4H7K, 4HVU, 4JVG, 5LLU, 5TRB, 5U3A, 6S55, 7LBH, 7MNF, 8XDJ, 9C9N, 9CWY, 9HCY, BD7L, BF9H, BS4K, C2SP, CXX2,
     D49Q, DK4H, DMG6, EB22, EW3V, G5U8, G7JE, G9HC, GDY7, GT5M, H7J7, H7TQ, HU3P, JKF3, JY7Z, KS4U, N4JP, P2EQ, Q4CL, QB6E, QLJ7,
     RHX7, RXY3, S4GJ, S98Z, SR86, SU5Z, SU74, SY6V, TD5N, U44R, U99R, W9L4, X4QW, YJV2, ZCZ6, ZL4Z, ZVH3, plus numbered variants
     like 00/01/06/000/004–008.

2. Group by yaml-test-suite tags and directory structure:
   - **indent / whitespace / folding**: tags `indent`, `whitespace`, `literal`, `folded`, `comment`.
   - **block mappings / sequences**: tags `mapping`, `sequence`, `flow`.
   - **scalars / comments**: tags `scalar`, `literal`, `folded`, `comment`.
   - **tags / directives / headers**: tags `tag`, `header`, `footer`, `unknown-tag`.

3. For each category, document representative examples (in this file) as they are understood, linking back to the official tests.

_Status: initial classification complete; representative IDs identified per category._

---

## 2. Strengthen Indentation and Whitespace Validation

**Objective:** Ensure invalid indentation and whitespace patterns are rejected early and consistently, matching tests tagged `indent`, `whitespace`, `literal`, `folded`, and `comment`.

Planned steps:

1. Introduce a central indentation/whitespace validator in the parser layer that operates over `TokenStream + ParsingContext`:
   - Validates allowed indent changes between lines for each `CollectionType` (None, BlockSequence, BlockMapping, Flow*).
   - Rejects lines where indentation is not consistent with the active collection or where tabs appear in forbidden positions.
   - Integrates comment positioning rules (e.g., comment-only lines vs inline comments after content).

2. Wire this validator into key entry points:
   - `parse_document_main_loop` (document-level scanning).
   - Block scalar parsing (literal and folded) so that mis-indented or over-indented scalar lines become errors instead of being folded.
   - Mapping and sequence token parsers where indentation boundaries are currently only loosely enforced.

3. Re-run the official suite, then iterate on error messages to ensure they carry at least one of the canonical substrings used by existing error-handling tests (e.g., "Unexpected", "Invalid", "indentation").

Success criteria:
   - A noticeable reduction in failures for IDs tagged `indent`, `whitespace`, `literal`, `folded`, and `comment` (e.g., W9L4, X4QW, Y79Y/000,004–008, N4JP, DMG6, QB6E, ZVH3).

---

## 3. Token-First Classification of Mapping / Sequence / Scalar

**Objective:** Eliminate structural false positives where invalid combinations of `-`, `key:`, and indentation are currently accepted as mappings or sequences.

Planned steps:

1. Implement a small classifier function (or enum) in the document parser, e.g. `classify_block_head(TokenStream, &ParsingContext) -> BlockHeadKind`, which decides:
   - `BlockSequenceStart`, `BlockMappingStart`, `PlainScalarStart`, `DocumentMarker`, `DirectiveStart`, or `Invalid`.

2. Replace remaining character-level decisions in `parse_document_contents` with this classifier:
   - For lines starting with alphanumeric/plain tokens: check if tokens on that line form a `key:` at the right indent; otherwise treat as scalar.
   - For lines starting with `-`: confirm they are true sequence items at the current indent and not mis-indented content under a mapping.

3. Adjust `parse_mapping_with_tokens` and `parse_sequence_with_tokens` to trust this classification and return early with explicit errors when called in an impossible context.

4. Focus tests:
   - Use official cases like `236B`, `2CMS`, `4H7K`, `4HVU`, `4JVG`, `5U3A`, `6S55`, `7MNF`, `9C9N`, `9CWY`, `BD7L`, `C2SP`, `CXX2`, `D49Q`, `DK4H`, `GDY7`, `G5U8`, `G7JE`, `G9HC`, `GT5M`, `HU3P`, `KS4U`, `P2EQ`, `Q4CL`, `SU74`, `TD5N`, `U44R`, `ZCZ6`, `ZL4Z`, `YJV2`, `ZVH3` to verify structure errors are now rejected.

Success criteria:
   - All mapping/sequence/flow-structure error tests above either fail to parse with a clear structural error or are intentionally treated as spec-compliant successes (documented separately if divergent).

---

## 4. Directive and Tag Validation Layer

**Objective:** Treat malformed `%YAML`, `%TAG`, and tag/anchor usages as hard errors instead of silently accepting or ignoring them.

Planned steps:

1. Define a dedicated directive/tag validation layer on top of `DirectiveContext`:
   - Parse `%YAML` and `%TAG` lines via tokens and validate syntax strictly.
   - Validate that tag handles and prefixes are well-formed and consistent.

2. For tag application (`!tag`, `!!handle!suffix`, unknown tags):
   - Ensure invalid tag syntax or use of undefined handles yields a structured parse error.
   - Keep support for application-defined resolution, but separate that from syntax errors.

3. Use official tests tagged `tag`, `header`, `footer`, and `unknown-tag` (e.g., `H7J7`, `QLJ7`, `U99R`, `9HCY`, `CXX2`, `5TRB`, `EB22`, `RXY3`) to drive this behavior.

Success criteria:
   - All header/footer/tag/unknown-tag error tests produce deterministic, syntactically motivated errors rather than silently succeeding.

---

## 5. Unified Plain and Block Scalar Engine

**Objective:** Have a single, token-driven scalar engine that implements YAML 1.2 folding and indentation rules and is aware of when scalar-looking input should actually be rejected.

Planned steps:

1. Extract plain and block scalar parsing into a dedicated module (e.g. `parser/document/scalar.rs`):
   - Operate on `TokenStream` + `ParsingContext` rather than raw `ISource` lines.
   - Handle plain, single-quoted, double-quoted, literal, and folded styles with one state machine.

2. Encode error conditions highlighted by official tests:
   - Mis-indented literal/folded scalars (e.g. W9L4, X4QW, 5LLU, S4GJ, S98Z).
   - Scalars that are invalid because of where comments appear (e.g. SU5Z, GDY7, 8XDJ).
   - Cases where scalar content is indistinguishable from malformed structure and the spec says “this is an error”.

3. Replace current scalar-entry points in `parse_document_contents` and the token-based value parser with calls into this engine, so behavior is consistent across top-level and nested contexts.

Success criteria:
   - Scalar-related error tests (`scalar`, `literal`, `folded`, `comment` tags) move from "expected error, got success" to reliable syntax/structure errors.

---

## 6. Error Reporting and Messaging Alignment

**Objective:** Align parse error messages with both the internal error-handling system and the expectations of integration tests that assert on substrings like "Unexpected" or "Invalid".

Planned steps:

1. Reuse the enhanced error handling documented in `ERROR_HANDLING_IMPLEMENTATION.md` for structural and syntax errors raised from the areas above.

2. Standardize on a small set of message prefixes (e.g., "Unexpected ...", "Invalid ...", "YAML compliance error: ...") and use them consistently in the new validators.

3. Make sure tests in `integration_tests/error_handling_tests.rs` continue to pass by preserving or supersetting the substrings they match against.

Success criteria:
   - New structural errors from yaml-test-suite failures show up with consistent, diagnosable messages, and existing library error-handling tests stay green.

---

## 7. Iterative Tightening Strategy

**Objective:** Improve spec compliance without breaking existing users all at once.

Planned steps:

1. Introduce a configuration knob (feature flag or parser option) that can run in either:
   - **lenient** mode (current behavior, plus better error reporting when possible).
   - **strict** mode (enforce all new validation rules aligned with the official test suite).

2. Run internal and CI pipelines in strict mode, but allow downstream consumers to opt into lenient behavior temporarily while migrating.

3. Track pass rate in `run_yaml_test_suite` over time:
   - Target: raise from ~82.8% to ≥90%, then iterate towards ≥95%+.

---

## 8. Tracking and Documentation

**Objective:** Keep this plan live and measurable.

Planned steps:

1. For each failing yaml-test-suite ID, add a brief note (in this file or a companion file) explaining:
   - Root cause category (indent, mapping/sequence, scalar, directive/tag, flow, other).
   - Whether the library intentionally diverges from the spec for that case (if ever), with rationale.

2. Update this document as major chunks (Sections 2–5) are completed, marking which failure IDs have flipped from success to error in strict mode.

3. When `run_yaml_test_suite` exceeds the target pass rate, add a short summary of remaining divergences and whether they are deliberate.

---

## Status Summary (2026-01-12)

- Inventory of failing IDs and tags: **complete**.
- Central indent/whitespace validator:
   - **Implemented (initial)** via `validate_indentation_and_whitespace` and `validate_indentation_tokens`, wired into `parse_document_contents`.
   - Currently conservative, but extended to enforce a block-scalar blank-line rule that fixes `5LLU` while keeping valid shapes like `R4YG` and `Y79Y/001` parsing successfully.
- Token-first classifier for mapping/sequence/scalar:
   - **Implemented (initial)** as a `BlockHeadKind` classifier plus a parent-indent–aware sequence parser.
   - Integrated into `parse_document_contents` without yet flipping the bulk of mapping/sequence false positives (e.g., `236B`, `4HVU`), which remain as future work.
- Unified scalar engine:
   - **In progress**: scalar parsing is now centralized in `parser/document/scalar.rs` over `TokenStream`, with additional indentation checks for block scalars.
   - Used to fix `5LLU` (invalid block scalar indentation) and preserve success for spec examples like `R4YG` and tab-related case `Y79Y/001`.
- Directive/tag validation layer: **planned**.
- Error-message alignment and strict/lenient modes: **planned**.

Current yaml-test-suite quiet runner baseline (data-2022-01-17, limit 402):

- Passed: **337**
- Failed: **65** (all "expected: error, got: success")
- Pass rate: **83.8%**

Notable fixed IDs so far:

- `5LLU` (block scalar with wrong indented line after spaces only) now correctly fails to parse.
- `R4YG` (Spec Example 8.2. Block Indentation Indicator) parses successfully under the tightened rules.
- `Y79Y/001` (Tabs in various contexts) parses successfully, while still allowing future tabs-in-indentation tightening in other contexts.
- `W9L4` (Literal block scalar with more spaces in first line) now correctly fails to parse due to invalid indentation.
- `X4QW` (Comment without whitespace after block scalar indicator) now correctly fails to parse due to an invalid block scalar header.
- `S4GJ` (Folded block scalar with invalid text after the indicator on the header line) now correctly fails to parse due to an invalid block scalar header.

---

## What To Do Next (Concrete Suggestions)

1. **Lock in baseline:**
   - On the `yaml-test-suite-false-positives` branch, run `cargo test --package yaml_lib --lib` and `cargo test --test yaml_test_suite -- --nocapture` and snapshot the current failing YAML IDs and messages.

2. **Start with indentation/whitespace (Section 2):**
   - Implement the central indentation/whitespace validator and wire it into `parse_document_main_loop` and block scalar parsing.
   - Re-run the official suite and record which `indent`/`whitespace`/`folded`/`literal` tests flipped from success to error.

3. **Add the block-head classifier (Section 3):**
   - Introduce `classify_block_head` (or equivalent) and replace character-based mapping/sequence/scalar decisions in `parse_document_contents`.
   - Use a few representative mapping/sequence failures (e.g. 236B, 4HVU, 7MNF, BD7L, GT5M) as your inner loop while iterating.

4. **Iterate on directives/tags (Section 4):**
   - Implement strict parsing for `%YAML`, `%TAG`, and tag syntax, then run only the tests tagged `tag`, `header`, `footer`, `unknown-tag` until they match expectations.

5. **Plan the scalar engine extraction (Section 5):**
   - Sketch the API for a unified scalar module (`parser/document/scalar.rs`) and identify the minimum set of call sites to switch over in the first pass.

6. **Continuously update this doc:**
   - After each major parser change, update the "Status Summary" and, if helpful, add a short note under the relevant section listing which YAML IDs were fixed.
