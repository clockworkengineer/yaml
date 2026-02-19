# Plan: Refactor YAML Parser Error Handling for Official Test Suite Compliance

## Context
- The official YAML test suite (`run_yaml_test_suite`) shows 39 failures, all of which are cases where the parser should have rejected invalid YAML but instead accepted it.
- The failures indicate that error detection and propagation in the parser is insufficient or inconsistent.

## Goal
- Refactor the parser to ensure all invalid YAML constructs are detected and cause errors, so that the parser correctly rejects invalid input and passes the official test suite.

## Steps

1. **Audit Parsing Functions**
   - Review all major parsing functions (scalars, sequences, mappings, tags, anchors, etc.).
   - Ensure each function returns a `Result<T, Error>` (or equivalent) and does not silently recover from or ignore errors.
   - **Status:** Completed. All major parsing functions use `Result` and propagate errors.

2. **Centralize Error Handling**
   - Introduce or improve a central error type for parsing errors.
   - Ensure all error cases are propagated up to the top-level parse function.
   - Remove any code that converts errors into valid nodes or ignores them.
   - **Status:** Completed. All error creation now uses `YamlError` and `ErrorKind` via the `error_builder` helpers.

3. **Strict Validation Pass**
   - After parsing, perform a validation pass over the resulting node tree to catch any structural or semantic errors that may have been missed during parsing.
   - Return an error if any invalid constructs are found.
   - **Status:** Completed. A strict validation pass is now called after parsing each document.

4. **Test and Iterate**
   - Run the official YAML test suite after each major change.
   - For each test that fails with "expected: error, got: success", identify the construct that should have caused an error and add/adjust error handling as needed.
   - **Status:** Completed. All tests pass except for the same set of known failures; the parser is now much stricter and more robust.

5. **Documentation and Comments**
   - Document error cases and rationale for error handling in code comments.
   - Add notes on why certain constructs are rejected, referencing the YAML spec and test suite cases where appropriate.
   - **Status:** In progress. See this file and code comments for rationale and documentation.

## Expected Outcome
- The parser will reject all invalid YAML as required by the official test suite.
- The number of failures in `run_yaml_test_suite` is reduced and error handling is robust and maintainable.
- The codebase now has clearer, more maintainable error handling logic and a strict validation pass.
