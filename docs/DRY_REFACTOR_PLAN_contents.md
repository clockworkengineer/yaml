# DRY Refactor Plan for contents.rs

## Analysis
The file `contents.rs` contains several parsing functions for YAML document contents, including:
- `parse_plain_multiline_scalar`: Parses multi-line plain scalars, handling paragraph folding and document markers.
- `token_dispatch`: Dispatches parsing to mapping/sequence handlers based on token stream.
- `is_doc_end`: Checks for document end marker.
- `handle_multiple_explicit_keys`: Handles multiple explicit keys at the same indentation.
- `parse_document_contents`: Main entry for parsing document contents, with context-aware logic for different YAML constructs.

### Observed DRY Violations
- Repeated logic for handling token streams and restoring source state.
- Multiple places where whitespace/comments are skipped.
- Similar error handling and indentation validation patterns.
- Mapping/sequence parsing logic is duplicated in both character and token-based branches.
- Scalar and value parsing logic is scattered and sometimes duplicated.

## Concrete Refactor Plan
1. **Centralize Token Stream Handling**
   - Create utility functions for initializing and restoring token streams.
   - Use these utilities in all places where token streams are created/restored.

2. **Unify Whitespace/Comment Skipping**
   - Move whitespace/comment skipping to a single helper function.
   - Ensure all entry points use this helper before parsing.

3. **Consolidate Error Handling**
   - Create a macro or helper for common error patterns (indentation, unexpected char, etc.).
   - Replace repeated error construction with this macro/helper.

4. **Abstract Indentation Validation**
   - Move indentation validation to a single function.
   - Call this function from all relevant parsing branches.

5. **Merge Mapping/Sequence Parsing Logic**
   - Refactor mapping/sequence parsing to use a shared function for both character and token-based paths.
   - Pass context to determine which parsing mode to use.

6. **Centralize Scalar/Value Parsing**
   - Create a unified entry point for scalar/value parsing.
   - Use this entry point in all branches that parse scalars/values.

7. **Document All Helper Functions**
   - Add clear documentation to all new helpers/utilities for maintainability.

## Expected Benefits
- Reduced code duplication and improved maintainability.
- Easier to update parsing logic in one place.
- More consistent error handling and validation.
- Improved readability and testability.

---
**Next Steps:**
- Identify all places where token streams, whitespace skipping, and error handling are duplicated.
- Implement the above refactor plan incrementally, starting with token stream utilities and whitespace skipping.
- Add unit tests for new helpers to ensure correctness.
