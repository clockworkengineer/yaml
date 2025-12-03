# Remaining Work for Full Lexer Integration

1. Remove dead constants and legacy code (e.g., STR_FOLDED_BLOCK).
2. Refactor all parsing functions to consume tokens from the lexer, not raw characters.
3. Integrate lexer throughout all parser modules (mapping.rs, sequence.rs, inline.rs, etc.).
4. Update error handling to reference token positions.
5. Add/update tests for lexer-driven parsing and YAML features.
6. Remove deprecated/unused functions (manual whitespace skipping, direct character parsing).
7. Update documentation for lexer-driven architecture.
8. Profile performance and validate YAML compliance.
