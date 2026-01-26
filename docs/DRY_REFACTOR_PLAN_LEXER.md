# DRY Refactor Plan for lexer.rs

## Overview
The `lexer.rs` file implements a YAML lexer with a variety of token types and a large number of methods for tokenization, whitespace handling, and context management. The code is functional but contains several areas where DRY (Don't Repeat Yourself) principles can be applied to reduce duplication, improve maintainability, and clarify intent.

## Key DRY Refactor Opportunities

### 1. Whitespace and Indentation Handling
- **Duplication:** There are multiple methods and code blocks for consuming, skipping, and peeking whitespace (e.g., `consume_horizontal_whitespace`, `skip_horizontal_whitespace`, `peek_next_non_whitespace`, and repeated whitespace checks in token scanning).
- **Plan:** Consolidate whitespace logic into a single utility module or trait. Use helper functions to handle all whitespace-related operations, reducing repeated code in token scanning and indentation logic.

### 2. Token Scanning Patterns
- **Duplication:** The scanning of tags, anchors, aliases, and scalars follows similar patterns: consume a leading character, collect content until a delimiter, handle errors, and return a token. Each has its own method with similar structure.
- **Plan:** Abstract common scanning logic into a generic function or macro that takes the leading character, delimiter set, and token constructor as parameters. Specialize only where necessary (e.g., for escape sequences in quoted strings).

### 3. Error Handling
- **Duplication:** Error construction and reporting is repeated in many places, especially for syntax errors and forbidden patterns.
- **Plan:** Create helper functions or macros for common error patterns, such as forbidden indentation, unterminated strings, and invalid escape sequences. This will reduce boilerplate and centralize error message formatting.

### 4. Flow Context and Indentation Checks
- **Duplication:** Logic for handling flow context (e.g., in `emit_indentation_token_if_any`, `scan_indentation`, and token scanning) is scattered and sometimes repeated.
- **Plan:** Encapsulate flow context checks and indentation validation in dedicated helper functions. Use clear, descriptive names to clarify when flow context affects behavior.

### 5. Token Emission Logging
- **Duplication:** Logging for token emission is repeated with similar patterns and feature flags.
- **Plan:** Use a macro or helper function for conditional logging, reducing repeated `#[cfg(feature = "debug-trace")]` blocks and string formatting.

### 6. Test Utilities
- **Duplication:** Test setup for creating a buffer and lexer is repeated in each test.
- **Plan:** Move common test setup into a test utility function or module to reduce repetition in test cases.

## Next Steps
1. Refactor whitespace and indentation handling into shared helpers.
2. Abstract common token scanning logic.
3. Centralize error handling and logging.
4. Encapsulate flow context logic.
5. Refactor test setup for DRYness.

---
This plan provides a roadmap for refactoring `lexer.rs` to follow DRY principles, improving maintainability and clarity.
