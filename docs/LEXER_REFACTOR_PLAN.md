# Lexer-Driven Parser Refactor Plan

1. Review current lexer API and capabilities
   - Document available lexer methods, token types, and error handling.
   
   **Lexer API and Capabilities**
   - **Token Types:**
     - Tag (`!tag`, `!!tag`), Anchor (`&name`), Alias (`*name`)
     - Flow collection indicators (`{`, `}`, `[`, `]`, `,`)
     - Colon, Dash, QuestionMark
     - Quoted scalars (`'single'`, `"double"`), Plain scalars
     - Newline, Indent, Comment
     - Document start/end (`---`, `...`), Directive (`%YAML`, `%TAG`), End of stream (Eof)
   - **Lexer API:**
     - `Lexer::new(source: &mut dyn ISource) -> Self`
     - `current() -> Option<&Token>`: Get current token without consuming.
     - `next() -> Result<Option<Token>, String>`: Advance to next token.
     - `peek() -> Result<Option<&Token>, String>`: Peek at next token.
     - `indent_level() -> usize`: Get current indentation level (for error reporting).
   - **Tokenization Logic:**
     - Handles whitespace, indentation, line starts, decorators, scalars, flow collections, document markers.
     - Error handling for invalid tokens (empty tag/anchor/alias, tabs for indentation, unclosed quotes).
   - **Integration:**
     - Used directly or via `TokenStream` for higher-level parser operations.
     - `TokenStream` wraps Lexer, provides decorator extraction, manages token sequences.
   - **Error Handling:**
     - Returns `Result<Option<Token>, String>` for all tokenization steps.
     - Specific error messages for invalid YAML constructs.

2. Identify parser areas using manual char parsing
   - List functions/modules where character-by-character parsing is still used.
   - **Manual character-by-character parsing is still used in:**
     - `parse_value` (value.rs)
     - `parse_mapping` (mapping.rs)
     - `parse_scalar` (scalar.rs)
     - `parse_block_scalar` (block_scalar.rs)
     - `parse_sequence` (sequence.rs)
     - `parse_mapping_key`, `parse_quoted_scalar`, `parse_comment`, and related helpers (helpers.rs)
     - `parse_explicit_mapping_entry` and explicit key logic (explicit_key.rs)
     - `collect_until`, `skip_whitespace`, and similar utility functions
   - **Common patterns:**
     - `while let Some(c) = source.current()` loops
     - Frequent use of `source.next()`, `save_state()`, `restore_state()`
     - Direct char checks for indentation, newlines, and YAML syntax
   - **Modules most affected:**
     - `document/value.rs`, `document/mapping.rs`, `document/scalar.rs`, `document/block_scalar.rs`, `document/sequence.rs`, `document/helpers.rs`, `document/explicit_key.rs`

3. Map parser logic to lexer-driven workflow
   - **Outline for rewriting parser functions to consume tokens:**
     - `parse_value` (value.rs):
       - Replace char-based loops with token-driven logic using `TokenStream`.
       - Switch on token type (Plain, Quoted, Alias, Tag, Flow indicators) to dispatch parsing.
       - Use token boundaries to handle decorators, scalars, and collections.
     - `parse_mapping` (mapping.rs):
       - Iterate over tokens, expecting key/value pairs separated by Colon tokens.
       - Use Indent/Newline tokens for block structure, and error on unexpected tokens.
       - Handle anchors/tags as separate tokens before keys.
     - `parse_scalar` (scalar.rs):
       - Accept only scalar tokens (Plain, SingleQuoted, DoubleQuoted) from the lexer.
       - Remove manual char escaping and folding logic; rely on lexer output.
     - `parse_block_scalar` (block_scalar.rs):
       - Use Indent/Newline tokens to manage block content and folding.
       - Parse block indicators and content as token streams.
     - `parse_sequence` (sequence.rs):
       - Iterate over Dash tokens for sequence items.
       - Use token boundaries to detect item starts/ends and nested collections.
     - `parse_mapping_key`, `parse_quoted_scalar`, `parse_comment` (helpers.rs):
       - Refactor to accept tokens directly, removing char-by-char parsing.
       - Use token types to distinguish between key, value, and comment.
     - `parse_explicit_mapping_entry` (explicit_key.rs):
       - Use QuestionMark and Colon tokens to identify explicit keys.
       - Handle anchors/tags as tokens before keys.
     - Utility functions (`collect_until`, `skip_whitespace`, etc.):
       - Replace with token-based equivalents or remove if redundant.
   - **Detailed breakdown for refactoring to token-driven mode:**
     - `parse_value` (value.rs):
       1. Initialize a `TokenStream` at the start of the function.
       2. Use `current()` and `next()` to retrieve tokens.
       3. Match on token type:
          - `Plain`, `SingleQuoted`, `DoubleQuoted`: construct scalar nodes.
          - `Tag`, `Anchor`, `Alias`: store decorators, apply to next value.
          - `FlowMappingStart`, `FlowSequenceStart`: dispatch to collection parsers.
          - `Dash`: start sequence item.
          - `Colon`: error if not in mapping context.
          - `Newline`, `Indent`: manage block structure.
          - `Comment`: skip or attach to node.
          - `DocumentStart`, `DocumentEnd`, `Directive`, `Eof`: handle document boundaries.
       4. Centralize error handling for unexpected tokens.
     - `parse_mapping` (mapping.rs):
       1. Initialize a `TokenStream`.
       2. Loop over tokens, expecting key tokens followed by `Colon` and value tokens.
       3. Use `Indent`/`Newline` tokens to manage block structure and nesting.
       4. Handle decorators (`Tag`, `Anchor`, `Alias`) before keys.
       5. On `FlowMappingStart`, dispatch to inline mapping parser.
       6. Error on unexpected tokens (e.g., missing colon, invalid indentation).
     - `parse_scalar` (scalar.rs):
       1. Accept only scalar tokens from the lexer.
       2. Remove manual char escaping/folding; rely on lexer output.
       3. Error on non-scalar tokens.
     - `parse_block_scalar` (block_scalar.rs):
       1. Use `Indent`/`Newline` tokens to manage block content and folding.
       2. Parse block indicators and content as token streams.
       3. Error on invalid block structure.
     - `parse_sequence` (sequence.rs):
       1. Loop over `Dash` tokens for sequence items.
       2. Use token boundaries to detect item starts/ends and nested collections.
       3. Handle decorators before items.
       4. Error on unexpected tokens.
     - `parse_mapping_key`, `parse_quoted_scalar`, `parse_comment` (helpers.rs):
       1. Accept tokens directly, removing char-by-char parsing.
       2. Use token types to distinguish between key, value, and comment.
       3. Centralize error handling for invalid key/value/comment tokens.
     - `parse_explicit_mapping_entry` (explicit_key.rs):
       1. Use `QuestionMark` and `Colon` tokens to identify explicit keys.
       2. Handle decorators as tokens before keys.
       3. Error on missing or misplaced tokens.
     - Utility functions:
       1. Replace with token-based equivalents or remove if redundant.
       2. Use token stream for whitespace, comment, and boundary management.
   - **General workflow for all functions:**
     1. Initialize `TokenStream` at function start.
     2. Use token-driven loops and state machines instead of char-based logic.
     3. Match on token type for all parser decisions.
     4. Centralize error handling for unexpected or invalid tokens.
     5. Document token expectations and error cases in function comments.
4. Refactor mapping and value parsing to use lexer tokens
   - Update mapping and value parsing logic to use lexer tokens instead of raw chars.
5. Centralize error handling for lexer/token errors
   - Ensure all lexer/token errors are handled consistently and reported clearly.
6. Run YAML test suite and analyze improvements
   - Execute the test suite, compare results, and document improvements/failures.
