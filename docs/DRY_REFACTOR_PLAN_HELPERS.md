# DRY Refactor Plan for helpers.rs

## Overview
The file `helpers.rs` contains a variety of parsing helpers, error utilities, token lookahead/classification, and whitespace/indentation logic for the YAML parser. While it is already modular, there are opportunities to further apply DRY (Don't Repeat Yourself) principles to reduce duplication, centralize logic, and improve maintainability.

## DRY Refactor Plan

### 1. Centralize Error Construction
- **Goal:** Ensure all error construction (syntax, structure, indentation, etc.) uses a single, consistent set of helpers (e.g., via `error_builder` or `error_helpers`).
- **Actions:**
  - Audit all error creation (e.g., `YamlError::new`, `parse_error_token`, direct `Err(...)` returns).
  - Refactor to use a unified error builder/helper for all error types.
  - Move any ad-hoc error formatting into the centralized helper.

### 2. Deduplicate TokenStream Setup and State Management
- **Goal:** Abstract repeated TokenStream setup, state save/restore, and trivia skipping into reusable helpers.
- **Actions:**
  - Identify all places where TokenStream is created with save/restore logic.
  - Create a helper for common TokenStream setup patterns (e.g., with trivia skip, with state snapshot).
  - Replace inline setup with calls to the helper.

### 3. Unify Whitespace and Comment Skipping
- **Goal:** Ensure all whitespace/comment skipping uses a single, consistent entry point.
- **Actions:**
  - Audit all whitespace/comment skipping (character-based and token-based).
  - Refactor to use a single utility (e.g., `skip_whitespace_and_comments`).
  - Remove or deprecate redundant/legacy skipping functions.

### 4. Consolidate Mapping Key Lookahead Logic
- **Goal:** Avoid duplication in logic that peeks ahead for mapping keys (colon detection, flow depth tracking).
- **Actions:**
  - Ensure all mapping key lookahead uses the same helper (e.g., `peek_ahead_for_mapping_key`).
  - Refactor any ad-hoc or character-based lookahead to use the token-based version.

### 5. Standardize Block Head Classification
- **Goal:** Centralize logic for classifying block head types (mapping, sequence, value, etc.).
- **Actions:**
  - Ensure all block head classification uses `classify_block_head`.
  - Refactor any duplicated or ad-hoc classification logic to use this function.

### 6. Centralize Indentation and Tab Validation
- **Goal:** Use a single entry point for indentation and tab validation.
- **Actions:**
  - Ensure all indentation/tab validation uses `validate_indentation_and_whitespace` or its token-based wrapper.
  - Remove or refactor legacy/duplicated validation code.

### 7. Unify Comment Parsing and Validation
**Complete.**
- **Goal:** Use a single helper for comment parsing and spacing validation.
- **Actions:**
  - All comment parsing now uses the single entry point: `parse_comment_token`.
  - All comment spacing validation uses the single entry point: `validate_comment_spacing_token`.
  - DRY notes have been added to the top of each function and the helpers.rs file header to enforce this pattern.
  - All duplicate or ad-hoc comment handling logic has been removed or refactored.

### 8. Document All Helpers and Entry Points
**Complete.**
- **Goal:** Ensure all helpers have clear, consistent documentation and usage examples.
- **Actions:**
  - All helpers in helpers.rs now have updated doc comments with clear purpose, usage, and entry point status.
  - Usage notes and DRY entry point annotations are present for each major helper.

---

**Next Steps:**
- Review the file for each of the above opportunities.
- Refactor incrementally, running tests after each major change.
- Update this plan as new duplication or abstraction opportunities are discovered during refactor.
