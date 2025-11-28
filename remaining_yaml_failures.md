# Remaining YAML Test Failure Categories

1. Indentation and Structure Errors
   - Parser accepts invalid indentation (should error).
   - Parser rejects valid indentation (should succeed).

2. Block Scalar Handling
   - Block scalars with incorrect indentation are not always rejected.
   - Edge cases in folded/literal scalars may fail.

3. Sequence and Mapping Alignment
   - Sequence items under mappings are not always validated for correct alignment.
   - Nested sequences/mappings may have inconsistent handling.

4. Binary and Tag Support
   - Some binary values (`!!binary`) are not parsed or validated correctly.
   - Tag handling for other types may be incomplete.

5. Flow-style Collection Parsing
   - Empty or nested flow-style sequences/mappings (`[ ]`, `{ }`) may fail.
   - Edge cases with comments, whitespace, or trailing commas.

6. Error Reporting and Spec Compliance
   - Error messages may not match test expectations.
   - Some YAML spec edge cases are not fully supported.
