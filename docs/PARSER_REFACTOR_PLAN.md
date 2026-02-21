# Parser Refactor Plan: Reducing False Positives in YAML Test Suite

## Goal
Reduce the 37 false positives in the official YAML test suite by enforcing strict YAML 1.2 compliance and improving error handling in the parser.

## Steps

1. **Enforce Strict Mode for Test Suite Runs**
   - Ensure all test suite executions use `ParserConfig::strict()`.
   - Update test helpers and test entrypoints to use strict configuration.

2. **Improve Duplicate Key Handling**
   - Refactor mapping parsing logic to reject duplicate keys when `strict_mode` is enabled.
   - Add explicit error messages for duplicate keys.

3. **Validate Anchors and Aliases**
   - Ensure all anchors are defined before use.
   - Reject unresolved aliases and provide clear error messages.

4. **Enforce Indentation and Tab Rules**
   - Disallow tabs for indentation in strict mode.
   - Add checks for inconsistent indentation and return errors.

5. **Document Marker and Directive Validation**
   - Require proper placement of `---` and `...` document markers.
   - Validate directives are followed by a document.

6. **Error Propagation and Reporting**
   - Refactor parser to return errors immediately for any non-compliant input.
   - Avoid partial node returns on parse errors.
   - Improve error messages for easier debugging.

7. **Test and Verify**
   - Run the official YAML test suite after each change.
   - Track reduction in false positives and adjust refactor as needed.

## Expected Outcome
- Parser rejects all non-compliant YAML inputs in strict mode.
- False positives in the test suite are significantly reduced.
- Error messages are clear and actionable for developers.

---

This plan targets the most common sources of false positives and provides actionable steps for parser refactoring.
