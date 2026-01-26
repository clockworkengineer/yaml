# Refactor Plan for `parse.rs` (YAML Document Parser)

## Analysis
The current `parse.rs` file implements the top-level YAML document parsing logic, including handling document start/end markers, directives, and the main document loop. The file contains several large functions with deeply nested logic, repeated patterns for token and source navigation, and error handling that could be more consistent. This makes the code harder to maintain, test, and extend.

## DRY (Don't Repeat Yourself) Opportunities
- **Document Marker Handling:** Logic for processing document start (`---`) and end (`...`) markers is repeated and could be unified.
- **Whitespace/Comment Skipping:** Skipping whitespace and comments is performed in multiple places and could be centralized.
- **Token Stream Navigation:** Patterns for peeking, advancing, and checking tokens are repeated.
- **Error Handling:** Error construction is scattered and could use centralized helpers for consistency.
- **Directive Handling:** Parsing and merging directives is verbose and could be encapsulated.

## Refactor Plan
1. **Extract Marker Handling:**
   - Move document start/end marker logic into dedicated helpers (e.g., `handle_document_start`, `handle_document_end`).
   - Ensure all marker validation and error reporting is consistent.
2. **Centralize Whitespace/Comment Skipping:**
   - Use a single utility for skipping whitespace/comments throughout the parser.
3. **Refactor Token Stream Navigation:**
   - Use helper methods for advancing, peeking, and checking tokens to reduce boilerplate.
4. **Centralize Error Handling:**
   - Use error builder helpers for all error construction.
5. **Encapsulate Directive Handling:**
   - Move directive parsing/merging into a helper to reduce clutter in the main loop.
6. **Improve Testability:**
   - Add/expand unit tests for new helpers and edge cases.
   - Ensure all error paths are covered by tests.
7. **Document the New Structure:**
   - Add code comments and update developer docs to describe the modular structure.

## Steps
1. Create helpers for document marker handling, whitespace/comment skipping, and directive management.
2. Refactor the main parse loop to use these helpers, reducing nesting and repetition.
3. Replace inline error construction with centralized error builder helpers.
4. Add/expand unit tests for helpers and edge cases.
5. Document the new structure in code comments and developer docs.

---
This plan will make the document parser more maintainable, DRY, and testable.
