
# DRY Refactor Plan for YAML Library (Concrete Checklist)

## ☑️ Common Patterns & Duplication Identified
- Error handling, string conversion, node validation, test helpers, and streaming logic are duplicated across modules.


## ☑️ Common Patterns & Duplication Identified
- Error handling, string conversion, node validation, test helpers, and streaming logic are duplicated across modules.

## 🔲 DRY Refactor Action Checklist

- [x] **Centralize Error Handling**
  - [x] Create a unified error type and conversion helpers (e.g., `YamlError`, `ParseResult<T>`)
  - [x] Provide a single function to convert errors to user-facing strings

- [x] **String Conversion Utilities**
  - [x] Implement shared traits/utilities for node/string conversion and cloning
  - [x] Refactor node construction, validation, and test code to use these utilities

- [x] **Node Validation & Construction**
  - [x] Move normalization, deduplication, and type-check helpers to a single module (e.g., `node_utils.rs`)
  - [x] Refactor all usages to call these helpers

- [ ] **Test Helper Module**
  - [ ] Create `test_helpers.rs` with common functions for parsing, stringifying, error assertions, and node comparison
  - [ ] Refactor all integration/unit tests to use these helpers

- [ ] **Streaming/Iteration**
  - [ ] Ensure all node traversal uses the shared streaming/iterator module
  - [ ] Remove or refactor any ad-hoc iteration logic

## ☑️ Map DRY Refactor to Test Suite Failure Areas

- [x] Analyze and map test suite failures to DRY refactor areas
- [ ] Refactor `official_suite_fixes.rs` and related integration tests to use unified error handling and test helpers
- [ ] Refactor set/mapping/round-trip tests to use centralized node normalization/deduplication
- [ ] Refactor property/fuzzing tests to use shared string conversion and error handling
- [ ] Refactor parser/validation tests to use common error and node utilities

---

## Progress Table

| Area                | DRY Action                                   | Modules/Files to Refactor                | Test Impacted                |
|---------------------|----------------------------------------------|------------------------------------------|------------------------------|
| Error Handling      | Centralize error types/conversion            | parser, validation, tests, examples      | error_handling, suite_fixes  |
| String Conversion   | Shared traits/utilities                      | node.rs, validation, stringify, tests    | round-trip, property, set    |
| Node Validation     | Centralize normalization/deduplication       | node_utils.rs, parser, validation        | set, mapping, suite_fixes    |
| Test Helpers        | Common test helper module                    | integration_tests, examples, property    | all integration/unit tests   |
| Streaming/Iteration | Use shared streaming/iterator module         | utils/streaming.rs, node traversal code  | streaming, traversal tests   |

---

**Check off each item as you complete it. Prioritize files with the most test failures.**
