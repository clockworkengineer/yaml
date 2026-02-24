# Suggested Restructure for `parser` Folder

## 1. Flatten or Group the `document` Submodules
- Move all error-related files into a new `document/errors/` subfolder:
  - `anchor_errors.rs`, `block_scalar_errors.rs`, `comment_errors.rs`, `directive_errors.rs`, `indentation_errors.rs`, `mapping_errors.rs`, `token_errors.rs`
- Merge tightly coupled parsing logic files into a single `core.rs` (or `parser.rs`) if appropriate:
  - `main_loop.rs`, `parse.rs`, `tokens.rs`, `inline_tokens.rs`
- Group utility/helper files into a single `helpers.rs` or `utils.rs`:
  - `helpers.rs`, `node_utils.rs`, `validate_tree.rs`

## 2. Clarify Public vs. Private API
- Use `pub mod` only for modules that are part of the public API.
- Internal modules should use `mod` only.
- Separate public interface from internal implementation.

## 3. Consolidate Utility Macros
- Move all parser-related macros (e.g., `lexer_log_macro.rs`, `anchors_debug_macro.rs`) into a `macros.rs` file or a `macros/` subfolder under `utils/`.

## 4. Documentation and Naming Consistency
- Ensure all modules have clear, consistent doc comments.
- Use consistent naming for error modules and helpers (e.g., always use `_errors.rs` for error-related files).

## 5. Example Structure

```
parser/
  mod.rs
  config.rs
  directives.rs
  lexer.rs
  token_stream.rs
  utils/
    mod.rs
    comments.rs
    error_helpers.rs
    indentation.rs
    token_scan.rs
    visit.rs
    whitespace.rs
    macros.rs
  document/
    mod.rs
    core.rs
    helpers.rs
    errors/
      mod.rs
      anchor_errors.rs
      block_scalar_errors.rs
      comment_errors.rs
      directive_errors.rs
      indentation_errors.rs
      mapping_errors.rs
      token_errors.rs
```

## Migration Plan
1. Create the new `errors/` subfolder under `document/` and move all error modules there.
2. Update all `mod` and `use` statements to reflect the new paths.
3. Merge or group files as suggested for `core.rs` and `helpers.rs` if appropriate.
4. Move macros into a single file or subfolder for clarity.
5. Review and update documentation and naming for consistency.

---
This plan will improve maintainability, clarity, and modularity of the parser codebase.
