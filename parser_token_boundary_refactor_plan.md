# Plan: Improve Document and Collection Boundary Handling for Tokeniser Parser

## Goal
Reduce test failures and enable a full tokeniser-based YAML parser by making document and collection boundary detection fully token-driven.

## Steps


 - [x] **Audit Current Boundary Handling**
   - [x] Review how document start/end markers (`---`, `...`) and collection transitions (mapping vs sequence) are currently detected.

       **Current Detection Summary:**
       - Document start (`---`) and end (`...`) markers are detected via `Token::DocumentStart` and `Token::DocumentEnd` in the token stream. When these tokens are encountered in the main parsing loop (e.g., in `parse_sequence_with_tokens`), the parser ends the current sequence or document. There is no direct string matching for `---` or `...`; all detection is token-based.
       - For collection transitions:
          - The parser looks for `Token::Dash` to identify sequence items.
          - If a `Token::Indent(_)` follows a dash, it always parses as a mapping (calls `parse_mapping_with_tokens`).
          - If a `Token::Plain(_)` follows a dash and is followed by a colon (`Token::Colon`), it also parses as a mapping.
          - Otherwise, it parses as a value (calls `parse_value_with_tokens`).
       - Whitespace, comments, and indentation are skipped using token-based logic, ensuring clean boundaries.
       - No character-based or line-based detection remains in the sequence parser for these boundaries; all logic is token-driven.
   - [x] Identify any remaining character-based or indentation-based logic.

       **Remaining Character/Indentation-Based Logic:**
       - All whitespace skipping and indentation validation helpers (`skip_whitespace`, `skip_whitespace_with_context`, `validate_indentation`) have been refactored to use tokens (`TokenStream`, `Token::Indent`, etc.).
       - Indentation and whitespace are now handled exclusively via token-based logic. No character-based helpers remain in `helpers.rs`.

 - [x] **Refactor Document Boundary Detection**
   - [x] Ensure document start (`---`) and end (`...`) markers are handled strictly via tokens, not by scanning lines or characters.
   - [x] On encountering a document marker token, always start/end a document, regardless of whitespace or comments.

 - [x] **Refactor Collection (Mapping/Sequence) Detection**
   - [x] Ensure that a sequence dash (`-`) at the correct indentation is never treated as a mapping key.
   - [x] Use the token stream to distinguish between mapping and sequence entries, especially in nested and mixed-content cases.
   - [x] Remove any fallback to manual lookahead or indentation checks.

- [ ] **Update/Expand Tests**
   - [ ] Add/expand tests for edge cases:
      - [ ] Documents with trailing/leading whitespace or comments.
      - [ ] Nested mappings and sequences with ambiguous boundaries.
      - [ ] Multi-document YAML with mixed content.

- [ ] **Validate Against YAML Test Suite**
   - [ ] Run the full YAML test suite and integration tests.
   - [ ] Identify and fix any remaining failures related to document or collection boundaries.

- [ ] **Document the New Logic**
   - [ ] Update developer docs to describe the new, token-driven approach for document and collection boundary detection.

## Expected Impact
- Fewer test failures related to document splitting and collection parsing.
- More robust handling of complex/nested YAML documents.
- Parser code is simpler, more maintainable, and ready for further tokeniser-based improvements.
