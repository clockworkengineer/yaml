# DRY Refactor Plan for lexer.rs

## Overview
The lexer.rs file contains the implementation of the YAML lexer, responsible for tokenizing YAML input. The file is relatively large and contains several methods with overlapping logic, repeated patterns, and opportunities for abstraction. Applying DRY (Don't Repeat Yourself) principles will improve maintainability, reduce bugs, and make the codebase easier to extend.

## Key DRY Issues Identified
1. **Whitespace and Indentation Handling**
	- Multiple methods handle whitespace, indentation, and tab/space logic with similar code blocks.
	- The logic for peeking ahead, saving/restoring state, and consuming whitespace is repeated.

2. **Token Scanning Patterns**
	- Methods for scanning tags, anchors, aliases, and scalars share similar character consumption and termination logic.
	- Error handling for empty names and invalid characters is duplicated.

3. **Flow Context and Newline Handling**
	- Flow context checks and newline suppression logic are scattered and repeated.
	- The handling of line breaks, especially in flow context, is similar across several methods.

4. **Escape Sequence Parsing**
	- The double-quoted scalar scanner contains a large match block for escape sequences, which could be factored out.

5. **State Management**
	- Saving and restoring lexer state is performed in several places with similar patterns.

## Concrete Refactor Plan

### 1. Abstract Whitespace and Indentation Logic
- Create utility methods for consuming horizontal whitespace, peeking next non-whitespace character, and handling indentation.
- Replace repeated whitespace/tab/space handling with calls to these utilities.

### 2. Unify Token Scanning Patterns
- Create a generic method for scanning a token until a set of terminator characters is reached.
- Parameterize the method for tag, anchor, alias, and plain scalar scanning.
- Centralize error handling for empty names and invalid characters.

### 3. Centralize Flow Context and Newline Handling
- Abstract flow context checks and newline suppression into helper methods.
- Use these helpers in all relevant places (e.g., handle_newline, scan_plain_scalar).

### 4. Extract Escape Sequence Parsing
- Move escape sequence parsing logic from scan_double_quoted into a dedicated method.
- This method should handle all escape types and error reporting.

### 5. Consolidate State Management
- Provide utility methods for saving/restoring state and peeking ahead.
- Use these consistently instead of duplicating the logic.

### 6. Improve Test Coverage for Refactored Utilities
- Add or update unit tests to cover the new utility methods and ensure no regressions.

## Refactor Steps
1. Identify and extract all whitespace/indentation handling into utility functions.
2. Refactor tag, anchor, alias, and scalar scanning to use a shared scanning utility.
3. Move escape sequence parsing to a dedicated function and update scan_double_quoted.
4. Replace repeated state management code with utility calls.
5. Update all call sites to use the new abstractions.
6. Run and expand tests to verify correctness.

## Expected Benefits
- Reduced code duplication and improved clarity.
- Easier to maintain and extend lexer logic.
- Fewer bugs due to centralized error handling and state management.

---

**Author:** GitHub Copilot
**Date:** 2026-01-26

### 8. Testing and Validation
- Ensure all existing tests pass after refactoring.
- Add new tests for generalized helpers if needed.

## Deliverables
- Refactored `inline_tokens.rs` adhering to DRY principles.
- Updated documentation/comments in the file.
- This plan (docs/INLINE_TOKENS_DRY_PLAN.md).

## Next Steps
1. Review `inline_tokens.rs` for duplicated logic.
2. Implement refactoring steps as outlined.
3. Validate with tests and update documentation.
