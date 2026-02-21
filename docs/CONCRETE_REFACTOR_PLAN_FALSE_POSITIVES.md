# Concrete Refactor Plan: Reducing False Positives in YAML Test Suite

## Goal
Reduce the 37+ false positives in the official YAML 1.2 test suite by enforcing strict indentation and structure validation in the parser.

## Background
Analysis of [FALSE_POSITIVE_FAILURE_SUMMARY.md](docs/FALSE_POSITIVE_FAILURE_SUMMARY.md) and test failures shows most false positives are due to:
- Insufficiently strict indentation checks (misaligned keys, mapping/sequence items)
- Structure errors (invalid mapping/sequence nesting, flow collection syntax)

## Step-by-Step Plan

### 1. Audit Current Parser Logic
- Review `TokenStream` and parser modules responsible for indentation and structure handling.
- Identify locations where indentation is checked or assumed.
- List all places where mapping/sequence entry alignment is validated.

### 2. Implement Strict Indentation Checks
- Ensure every mapping key/value and sequence item is checked for correct indentation relative to its parent.
- Add explicit error reporting for:
  - Misaligned mapping keys/values
  - Sequence items at wrong indentation
  - Mixed mapping/sequence at same indentation
- Update error messages to include line/column info for easier debugging.

### 3. Enforce Structure and Flow Collection Rules
- Refactor flow collection (bracketed) parsing to:
  - Detect extra/missing brackets
  - Reject invalid line breaks inside flow collections
  - Catch unterminated or malformed flow collections
- Add tests for edge cases in flow collections.

### 4. Expand and Update Test Coverage
- Add/expand unit tests for known false positive cases (see [official_suite_fixes.rs](library/src/integration_tests/official_suite_fixes.rs)).
- Ensure all known problematic YAML snippets are covered.

### 5. Rerun and Analyze Test Suite
- Run the full official YAML test suite (`cargo test` or test harness).
- Record the new count of false positives and regressions.
- If regressions are found, iterate on the above steps.

### 6. Document and Commit
- Document changes and rationale in `docs/DRY_REFACTOR_PLAN_LIBRARY.md` and `docs/FALSE_POSITIVE_FAILURE_SUMMARY.md`.
- Commit changes with a summary of improvements and remaining issues.

## References
- [FALSE_POSITIVE_FAILURE_SUMMARY.md](docs/FALSE_POSITIVE_FAILURE_SUMMARY.md)
- [DRY_REFACTOR_PLAN_LIBRARY.md](docs/DRY_REFACTOR_PLAN_LIBRARY.md)
- [COMPLIANCE.md](docs/COMPLIANCE.md)
- [official_suite_fixes.rs](library/src/integration_tests/official_suite_fixes.rs)

---
This plan targets the most common root causes and provides a clear, actionable path to reducing false positives in the YAML test suite.
