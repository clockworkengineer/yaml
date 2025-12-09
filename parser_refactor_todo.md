# Parser Refactor Todo List

- [X] Refactor mapping key detection to use token stream only
- [X] Remove character-based lookahead and state restoration in mapping parsing
- [X] Update `peek_ahead_for_mapping_key` to operate on tokens
- [X] Unify explicit and implicit key handling in token-based logic
- [ ] Add/expand tests for mapping keys with anchors, tags, whitespace
- [ ] Verify improved mapping parsing against YAML test suite
