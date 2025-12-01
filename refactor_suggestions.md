# Test Suite Failure Refactor Suggestions

## 1. Indentation and Structure Errors
- Strengthen indentation validation in block mappings and sequences.
- Use ParsingContext to enforce consistent indentation and tab rules.
- Improve error reporting for indentation issues using ErrorBuilder::indentation_error.

## 2. Block Scalar Handling
- Enforce single-digit indentation indicators (1-9) and reject 0 or double digits.
- Refactor parse_block_scalar and parse_scalar to validate indentation and chomping indicators.
- Add more tests for edge cases in block scalars.

## 3. Sequence and Mapping Alignment
- Ensure nested collections validate alignment and indentation.
- Prevent sequences starting on the same line as mapping keys.
- Improve error messages for alignment issues.

## 4. Flow-style Collection Parsing
- Refactor flow collection parsers to better handle empty collections, trailing commas, and nested structures.
- Validate tab usage in flow collections and reject tabs as indentation.
- Add/expand tests for edge cases in flow collections.

## 5. Error Reporting and Spec Compliance
- Use EnhancedError and ErrorBuilder for all parser errors to provide error codes, context, and suggestions.
- Integrate error recovery strategies to allow parsing to continue after recoverable errors.
- Ensure error messages include line/column, context, and hints for fixing issues.

## 6. General Suggestions
- Centralize error creation using ErrorBuilder and EnhancedError for consistency.
- Add more tests for edge cases, especially for block scalars, flow collections, and indentation.
- Integrate recovery strategies to collect multiple errors and continue parsing when possible.
