Parser module layout

This directory contains the YAML parser implementation, organized for clarity between character-driven parsing and token-driven helpers.

- config.rs: Parser configuration and builder (fluent API).
- directives.rs: YAML directive handling and context.
- lexer.rs: Lightweight tokenizer used by the parser/token stream.
- token_stream.rs: High-level token cursor consumed by token-based routines.
- document/: All document-level parsing logic.
  - anchors.rs, scalar.rs, mapping.rs, sequence.rs, inline.rs: Character-driven parsers for core YAML constructs.
  - contents.rs, main_loop.rs, bridge.rs, helpers.rs, error_builder.rs: Document flow, orchestration, and utilities.
  - tokens/: Token-driven parsing helpers used where lookahead is complex (inline, mapping, sequence, value).

Notes
- contents.rs is the canonical entry for document body parsing. A previous duplicate (document_contents.rs) has been removed.
- tokens/* provide robust boundaries when decorators or complex keys make character-based lookahead brittle.
- Tests live under library/tests and exercise the public parse API.

Contributing tips
- Prefer small utilities in tokens/ when you need non-trivial lookahead state.
- Keep character-driven parsers minimal and delegate to tokens/ where it reduces complexity or infinite-loop risks.