# Refactor Plan: Parsing Functions to Use Lexer

1. Identify all parsing functions that currently use raw character access (e.g., source.current(), manual char checks).
2. Update function signatures to accept a token stream from the lexer instead of a raw character source.
3. Replace all direct character comparisons with token kind checks (e.g., token.kind == TokenKind::Colon).
4. Refactor loops and conditionals to advance and consume tokens, not characters.
5. Update error handling to reference token positions and types.
6. Remove legacy character-based logic and helpers.
7. Test each refactored function and update tests for lexer-driven parsing.

Recommended: Proceed module by module (mapping.rs, sequence.rs, inline.rs, scalar.rs, etc.) for maintainability and correctness.