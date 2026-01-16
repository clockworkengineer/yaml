# False Positive Failure Categorization (YAML Test Suite)

| Test Case | Error Type (Inferred)         | Description/Pattern                        |
|-----------|------------------------------|--------------------------------------------|
| 236B      | Indentation/structure        | Invalid mapping/sequence structure         |
| 2CMS      | Indentation                  | Misaligned sequence item                   |
| 4H7K      | Structure                    | Extra closing bracket in flow sequence     |
| 4HVU      | Indentation                  | Misaligned sequence item                   |
| 5TRB      | Structure                    | Unterminated/invalid quoted scalar         |
| 5U3A      | Indentation                  | Sequence under mapping key misaligned      |
| 6S55      | Indentation                  | Invalid mapping/sequence structure         |
| 7LBH      | Scalar/structure             | Invalid multi-line quoted key              |
| 7MNF      | Structure                    | Mapping key without value                  |
| 9C9N      | Flow/structure               | Flow sequence with invalid line breaks     |
| 9CWY      | Indentation                  | Invalid mapping/sequence structure         |
| 9HCY      | Tag/directive/structure      | Tag/directive order/placement issue        |
| BD7L      | Structure                    | Sequence followed by invalid mapping       |
| BF9H      | Indentation/scalar           | Indented lines after plain scalar          |
| BS4K      | Structure                    | Multiple plain scalars on separate lines   |
| C2SP      | Structure                    | Invalid flow mapping syntax                |
| CXX2      | Anchor/structure             | Anchor in invalid position                 |
| D49Q      | Scalar/structure             | Invalid multi-line single-quoted key       |
| DK4H      | Structure                    | Invalid flow mapping syntax                |
| DMG6      | Indentation                  | Misaligned mapping key                     |
| EB22      | Directive/structure          | Directive after content                    |
| EW3V      | Indentation                  | Misaligned mapping key                     |
| F8F9      | Block scalar/comment         | Block scalar with comments, formatting     |
| G7JE      | Scalar/structure             | Invalid multi-line key                     |
| G9HC      | Anchor/structure             | Anchor in invalid position                 |
| GDY7      | Structure                    | Invalid mapping/comment                    |
| GT5M      | Anchor/structure             | Anchor in invalid position                 |
| H7TQ      | Directive/structure          | Invalid directive syntax                   |
| HU3P      | Structure                    | Mapping with invalid value                 |
| JKF3      | Structure                    | Sequence with unterminated quoted scalar   |
| JY7Z      | Structure                    | Mapping with trailing content              |
| KS4U      | Structure                    | Invalid flow sequence/mapping              |
| N4JP      | Indentation                  | Mapping with bad indentation               |
| P2EQ      | Structure                    | Invalid flow mapping syntax                |
| Q4CL      | Structure                    | Mapping with trailing content              |
| QB6E      | Scalar/structure             | Invalid multi-line quoted scalar           |
| QLJ7      | Tag/directive/structure      | Tag/directive order/placement issue        |
| RHX7      | Directive/structure          | Directive after content                    |
| RXY3      | Scalar/structure             | Invalid single-quoted scalar               |
| SY6V      | Anchor/structure             | Anchor in invalid position                 |
| TD5N      | Structure                    | Sequence followed by invalid mapping       |
| U44R      | Indentation                  | Mapping with bad indentation               |
| U99R      | Tag/structure                | Invalid tag usage                         |
| YJV2      | Structure                    | Invalid flow sequence                      |
| ZCZ6      | Structure                    | Mapping with too many colons               |
| ZL4Z      | Structure                    | Mapping with quoted key and value          |
| ZVH3      | Indentation/structure        | Sequence with misaligned mapping           |

**Patterns:**
- Majority: indentation or structure errors (misaligned keys, invalid flow/sequence/mapping)
- Some: YAML directives, anchors, or tags in invalid positions
- Few: invalid scalar formatting or multi-line keys

**Conclusion:**
Most false positives are due to insufficiently strict handling of indentation, mapping/sequence structure, and flow collection syntax. A smaller subset involves YAML directives, anchors, or tags.

## Targeted Refactor Strategies for False Positives

Based on the categorized failure summary, the following strategies are proposed:

### 1. Indentation and Structure Enforcement
- Refactor TokenStream and parser logic to strictly enforce YAML indentation rules.
- Add validation for misaligned keys, sequence items, and mapping entries.
- Ensure flow collections (sequences, mappings) reject invalid line breaks and extra/missing brackets.

### 2. Flow Collection Syntax
- Harden parsing of flow sequences and mappings to catch extra/missing delimiters and invalid content.
- Add error reporting for unterminated or malformed flow collections.

### 3. Scalar and Key Formatting
- Add checks for invalid multi-line keys and scalars.
- Ensure quoted and plain scalars are terminated and formatted per spec.

### 4. Directive, Tag, and Anchor Handling
- Validate placement and syntax of YAML directives, tags, and anchors.
- Ensure anchors/tags are only accepted in valid positions and contexts.

### 5. Error Reporting Granularity
- Improve error builder to attach precise line/column info for all parse errors.
- Categorize errors (indentation, structure, flow, scalar, directive, anchor) for better test suite mapping.

### 6. Test Harness Improvements
- Update test harness to assert error type and location for each failure.
- Map parser errors to test suite expectations for more actionable diagnostics.

These strategies directly address the most common failure patterns and will reduce false positives in the test suite.

## Prioritization of High-Impact Fixes

Based on the failure categorization and strategy proposals, the following priorities are recommended:

### Highest Impact (Most Failures)
1. **Indentation and Structure Enforcement**
   - Address misaligned keys, sequence items, and mapping entries.
   - Strictly validate indentation for all YAML constructs.
   - Fixes: 2CMS, 4HVU, 5U3A, 6S55, 9CWY, DMG6, EW3V, N4JP, U44R, ZVH3, etc.

2. **Flow Collection Syntax**
   - Catch extra/missing brackets, invalid line breaks, and malformed flow collections.
   - Fixes: 4H7K, 9C9N, C2SP, DK4H, KS4U, P2EQ, YJV2, etc.

### Medium Impact
3. **Scalar and Key Formatting**
   - Validate multi-line keys and scalars, quoted/plain scalar termination.
   - Fixes: 7LBH, D49Q, QB6E, RXY3, etc.

4. **Directive, Tag, and Anchor Handling**
   - Validate placement and syntax of directives, tags, anchors.
   - Fixes: 9HCY, CXX2, GT5M, QLJ7, RHX7, SY6V, U99R, etc.

### Lower Impact
5. **Error Reporting Granularity**
   - Improve error categorization and diagnostics for edge cases.
   - Fixes: BD7L, BF9H, BS4K, GDY7, HU3P, JKF3, JY7Z, Q4CL, ZCZ6, ZL4Z, etc.

**Recommendation:**
- Begin with indentation/structure and flow collection fixes, as these cover the majority of failures.
- Address scalar, directive, tag, and anchor issues next.
- Refine error reporting and diagnostics for remaining edge cases.

## Implementation Plan for Top Issues

### Phase 1: Indentation and Structure Enforcement
- Audit and refactor TokenStream and parser logic for strict indentation validation.
- Add checks for misaligned keys, sequence items, and mapping entries.
- Update error reporting for indentation/structure failures with precise line/column info.
- Write targeted tests for known indentation/structure false positives.

### Phase 2: Flow Collection Syntax
- Refactor flow sequence and mapping parsing to catch extra/missing brackets and invalid line breaks.
- Add error handling for unterminated/malformed flow collections.
- Expand test coverage for flow collection edge cases.

### Phase 3: Scalar, Directive, Tag, and Anchor Handling
- Harden scalar and key formatting validation (multi-line, quoted, plain).
- Validate placement/syntax of directives, tags, and anchors.
- Map parser errors to test suite expectations for these cases.

### Phase 4: Error Reporting and Harness Improvements
- Improve error builder to categorize and report all parse errors.
- Update test harness to assert error type/location for each failure.
- Document any spec ambiguities or intentional parser differences.

**Milestones:**
- Complete Phase 1 and rerun test suite to measure reduction in false positives.
- Iterate through Phases 2-4, updating summary and plan as failures are eliminated.
