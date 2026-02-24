# Parser Folder Restructuring Plan

## Goals
- Improve clarity and maintainability
- Group related modules and helpers
- Consolidate error handling
- Ensure consistent naming and documentation

## Steps

### 1. Consolidate Error Handling
- Move all error-related files from `document/` to `document/errors/`.
- Update `document/mod.rs` to reference errors only via `errors/mod.rs`.
- Remove direct `pub use` of individual error files in `document/mod.rs`.

### 2. Clarify Helper/Utility Placement
- Move generic helpers (e.g., `helpers.rs`) from `document/` to `utils/`.
- If helpers are document-specific, rename to `document_helpers.rs`.

### 3. Token Parsing Organization
- Keep `tokens/` for token-specific parsing.
- Merge small token files if tightly coupled.

### 4. Reduce mod.rs Complexity
- Use `pub mod ...` for error modules, not `pub use ...`.
- Group related modules (anchors, mappings, sequences) under subfolders if they grow large.

### 5. Naming Consistency
- Ensure all modules use `snake_case`.
- Rename ambiguous files (e.g., `contents.rs` → `document_contents.rs`).

### 6. Documentation
- Add module-level doc comments to each folder and file.

## Example Target Structure

```
parser/
  mod.rs
  config.rs
  directives.rs
  lexer.rs
  token_stream.rs
  document/
    mod.rs
    context.rs
    value.rs
    validate_tree.rs
    sequence.rs
    scalar.rs
    parse.rs
    node_utils.rs
    mapping.rs
    main_loop.rs
    loop_guards.rs
    inline_tokens.rs
    indentation.rs
    flow_punctuation.rs
    explicit_key.rs
    error_builder.rs
    errors/
      mod.rs
      anchor_errors.rs
      block_scalar_errors.rs
      comment_errors.rs
      directive_errors.rs
      indentation_errors.rs
      mapping_errors.rs
      token_errors.rs
    tokens/
      mod.rs
      inline.rs
      mapping.rs
      sequence.rs
      value.rs
  utils/
    mod.rs
    comments.rs
    error_helpers.rs
    indentation.rs
    token_scan.rs
    visit.rs
    whitespace.rs
    macros/
      mod.rs
      anchors_debug_macro.rs
      lexer_log_macro.rs
```

## Action Items
- [ ] Move error files as described
- [ ] Update mod.rs files for new structure
- [ ] Move/rename helpers
- [ ] Review token files for merging
- [ ] Update naming and add documentation
- [ ] Test build and update imports

---
This plan is ready for implementation. Update as progress is made.
