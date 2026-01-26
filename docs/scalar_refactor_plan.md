# Refactor Plan for `scalar.rs` (Parser)

## Analysis
The current `scalar.rs` file contains a large, monolithic function `parse_scalar_with_tokens` that handles parsing of all YAML scalar types (single-quoted, double-quoted, plain, block, etc.) and their edge cases. The function is long, contains deeply nested logic, and mixes error handling, token stream management, and scalar construction. This makes it hard to maintain, test, and extend.

## DRY (Don't Repeat Yourself) Opportunities
- **Block Scalar Handling:** The logic for block scalar headers, indentation, and line collection is deeply nested and repeated in several places.
- **Chomping and Indentation:** Chomping logic and indentation validation are repeated for both literal and folded block scalars.
- **Token Stream Navigation:** The function repeatedly matches and advances tokens in similar patterns for different scalar types.
- **Scalar Construction:** The construction of `Node::Str`, `Node::Number`, and `Node::Boolean` is scattered and could be centralized.

## Refactor Plan
1. **Extract Block Scalar Parsing:**
   - Move block scalar header parsing, line collection, and chomping logic into a dedicated function (e.g., `parse_block_scalar`).
   - This function should handle both literal (`|`) and folded (`>`) styles, including indentation and chomping.
2. **Extract Plain Scalar Parsing:**
   - Move plain scalar parsing and multiline continuation logic into a separate function (e.g., `parse_plain_scalar`).
   - Handle YAML 1.1 boolean and null values in a helper.
3. **Extract Quoted Scalar Parsing:**
   - Move single-quoted and double-quoted scalar parsing into their own functions.
   - Unescaping logic for double-quoted strings should be handled in a utility.
4. **Centralize Error Handling:**
   - Use helper functions for common error cases (e.g., invalid indentation, invalid block header).
5. **Refactor Token Stream Navigation:**
   - Use helper methods or macros to advance and peek tokens, reducing boilerplate.
6. **Improve Testability:**
   - Write unit tests for each new helper function.
   - Ensure all edge cases from the current tests are covered.

## Steps
1. Create `parse_block_scalar`, `parse_plain_scalar`, `parse_single_quoted_scalar`, and `parse_double_quoted_scalar` functions.
2. Refactor `parse_scalar_with_tokens` to delegate to these helpers based on the current token.
3. Move repeated error messages and validation logic into helpers.
4. Add/expand unit tests for new helpers.
5. Document the new structure in code comments and update developer docs if needed.

---
This plan will make the parser more maintainable, testable, and DRY-compliant.
