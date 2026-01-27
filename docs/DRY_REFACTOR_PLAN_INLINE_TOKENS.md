# DRY Refactor Plan for inline_tokens.rs

## Overview
The file `inline_tokens.rs` implements parsing helpers and logic for inline (flow) YAML collections, such as sequences (`[ ... ]`) and mappings (`{ ... }`). It contains logic for parsing, trivia skipping, error construction, and node creation. There are opportunities to further apply DRY (Don't Repeat Yourself) principles to reduce duplication, centralize logic, and improve maintainability.

## DRY Refactor Plan

### 1. Centralize Error Construction
- **Goal:** Ensure all error construction (syntax, structure, etc.) uses a single, consistent set of helpers or macros.
- **Actions:**
  - Audit all error creation (e.g., direct `Err(...)`, `syntax_err!`, `mapping_key_error_yaml`).
  - Refactor to use a unified error builder/helper for all error types.
  - Move any ad-hoc error formatting into the centralized helper or macro.

### 2. Deduplicate Trivia Skipping
- **Goal:** Ensure all whitespace and comment skipping uses a single entry point (e.g., `skip_inline_trivia`).
- **Actions:**
  - Audit all trivia skipping (whitespace, comments, newlines) in sequence and mapping parsing.
  - Refactor to use a single utility for trivia skipping.
  - Remove or deprecate redundant trivia skipping logic.

### 3. Unify Node Construction Helpers
- **Goal:** Use centralized helpers for constructing array, mapping, and set nodes.
- **Actions:**
  - Ensure all node construction uses `make_array_node`, `make_mapping_node`, or `make_set_node`.
  - Refactor any ad-hoc node construction to use these helpers.

### 4. Consolidate Inline Value Parsing
- **Goal:** Avoid duplication in logic that parses values or keys in inline collections.
- **Actions:**
  - Ensure all inline value parsing uses `parse_inline_value`.
  - Refactor any ad-hoc or duplicated value parsing logic to use this function.

### 5. Standardize Progress Checking
- **Goal:** Centralize logic for ensuring the token stream advances during parsing.
- **Actions:**
  - Ensure all progress checks use `ensure_progress`.
  - Refactor any duplicated or ad-hoc progress checking logic to use this function.

### 6. Document All Helpers and Entry Points
- **Goal:** Ensure all helpers have clear, consistent documentation and usage examples.
- **Actions:**
  - Add or update doc comments for each helper.
  - Provide usage notes and entry point annotations where ambiguity exists.

---

**Next Steps:**
- Review the file for each of the above opportunities.
- Refactor incrementally, running tests after each major change.
- Update this plan as new duplication or abstraction opportunities are discovered during refactor.
