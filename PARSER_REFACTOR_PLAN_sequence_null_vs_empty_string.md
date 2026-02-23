# Parser Refactor Plan: Fix Null vs Empty String in Sequence Items

## Problem
- The parser currently produces `Node::Str("")` for YAML sequence items like `- ` or `-\n`, but the YAML spec expects these to be `Node::None` (null).

## Goals
- Ensure that sequence items with no explicit value (just a dash and whitespace/newline) are parsed as `Node::None`.
- Only produce `Node::Str("")` for explicit empty string scalars (`- ""` or `- ''`).

## Steps

### 1. Audit Sequence Parsing Logic
- Review the block sequence parser (likely in `parser/document/tokens/sequence.rs`).
- Identify where it decides between `Node::None` and `Node::Str("")` for sequence items.

### 2. Refactor Null Detection
- Refactor the logic so that:
  - If a dash is followed only by whitespace or a newline, produce `Node::None`.
  - If a dash is followed by an explicit quoted scalar, produce `Node::Str("")`.
- Centralize this logic in a helper function if possible.

### 3. Update Scalar Parsing (if needed)
- Ensure the scalar parser does not default to an empty string when the value is missing.

### 4. Add/Update Unit Tests
- Add or update unit tests for these YAML cases:
  - `-` (should be `Node::None`)
  - `- ` (should be `Node::None`)
  - `- ""` (should be `Node::Str("")`)
  - `- ''` (should be `Node::Str("")`)

### 5. Verify Integration
- Run the full test suite, especially `test_parse_empty_sequence_items` and related sequence tests.
- Confirm that the parser now produces the correct node types for all edge cases.

---

## Next Steps
- Implement the parser logic changes.
- Add/adjust tests.
- Verify all tests pass.
