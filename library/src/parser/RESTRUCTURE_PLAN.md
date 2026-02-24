# Parser Folder Restructure Plan

## Objective
Restructure the `parser` folder for improved maintainability, clarity, and scalability by grouping related files, centralizing utilities, and flattening deep nesting.

---

## Step-by-Step Migration Plan

### 1. Move Error Modules
- ~~Move all files from `parser/document/errors/` to `parser/errors/`.~~ ✅
- ~~Update all imports referencing `parser::document::errors::*` to `parser::errors::*`.~~ ✅
- ~~Remove the now-empty `parser/document/errors/` folder.~~ ✅

### 2. Move Token Modules
- ~~Move all files from `parser/document/tokens/` to `parser/tokens/`.~~ ✅
- ~~Update all imports referencing `parser::document::tokens::*` to `parser::tokens::*`.~~ ✅
- ~~Remove the now-empty `parser/document/tokens/` folder.~~ ✅

### 3. Centralize Utilities
- Move all utility files from `parser/document/` and `parser/utils/` into `parser/utils/`.
  - If a utility is only used by `document`, consider a `parser/document/utils/` submodule.
- Update all imports referencing moved utilities.
- Remove redundant or empty utility folders.

### 4. Review and Relocate Macros
- If macros in `parser/utils/macros/` are only used in one module, move them to that module.
- Otherwise, keep them in `parser/utils/macros/`.
- Update macro imports as needed.

### 5. Flatten and Group Document Submodules
- In `parser/document/`, merge small files (e.g., `context.rs`, `helpers.rs`, `node_utils.rs`, `error_builder.rs`) into a single `core.rs` if possible.
- Keep main logic files (`parse.rs`, `main_loop.rs`, `mapping.rs`, `sequence.rs`, `scalar.rs`, `value.rs`, `validate_tree.rs`, `anchors.rs`, `contents.rs`, `explicit_key.rs`, `flow_punctuation.rs`, `indentation.rs`, `inline_tokens.rs`, `loop_guards.rs`) in `parser/document/`.
- Remove or merge files as appropriate.

### 6. Update `mod.rs` Files
- Update all `mod.rs` files to reflect the new structure and re-export modules as needed.
- Ensure all modules are properly declared and public interfaces are maintained.

### 7. Update All Imports
- Search and replace all old import paths in the codebase to match the new structure.
- Run `cargo check` to catch any missed references or errors.

### 8. Test Thoroughly
- Run all tests to ensure nothing is broken by the restructure.
- Fix any issues that arise.

---

## Example New Structure

```
parser/
  config.rs
  directives.rs
  lexer.rs
  token_stream.rs
  mod.rs
  errors/
    anchor_errors.rs
    block_scalar_errors.rs
    ...
    mod.rs
  tokens/
    inline.rs
    mapping.rs
    ...
    mod.rs
  utils/
    comments.rs
    error_helpers.rs
    indentation.rs
    macros.rs
    ...
    mod.rs
  document/
    core.rs
    parse.rs
    main_loop.rs
    mapping.rs
    sequence.rs
    scalar.rs
    value.rs
    validate_tree.rs
    anchors.rs
    contents.rs
    explicit_key.rs
    flow_punctuation.rs
    indentation.rs
    inline_tokens.rs
    loop_guards.rs
    ...
```

---

## Notes
- Perform the restructure in small, testable steps.
- Commit after each major step for easier rollback.
- Document any module interface changes in the codebase.
