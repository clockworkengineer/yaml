# Next Steps - YAML Parser Improvement Roadmap

## Current Status Summary

✅ **Completed:** All 402 tests run without hanging
- **Pass Rate:** 271/402 (67.4%)
- **Failed:** 131 tests (32.6%)
- **Skipped:** 0
- **Hangs:** 0

## Failure Analysis

The 131 failing tests break down into two categories:

### Category 1: Parser Too Strict (70 tests)
**Expected: success, got: error**

These are valid YAML documents that the parser incorrectly rejects. Fixing these will improve compliance.

**High Priority Issues:**
1. **Flow mapping edge cases** (8 tests)
   - 5C5M: Trailing commas in flow mappings
   - 5KJE: Flow sequence formatting
   - 7ZZ5: Empty flow collections
   - DFF7: Flow mapping entries
   - FRK4: Completely empty flow nodes
   - Q88A: Flow content variations
   - SBG9: Flow sequences in flow mappings
   - MXS3: Flow mappings in block sequences

2. **Tab handling issues** (10+ tests)
   - 01, 04: Leading tabs in double quoted strings
   - DC7X-02, DC7X-03: Trailing tabs in double quoted
   - DK95-00: Tabs that look like indentation
   - 002: Tabs in various contexts
   - K858-01: Inline tabs in double quoted

3. **Multi-line scalars** (12 tests)
   - 4CQQ: Multi-line flow scalars
   - 36F6: Multiline plain scalar with empty line
   - 229Q: Sequence of mappings
   - 26DV: Whitespace around colon
   - 565N: Construct binary
   - 6HB6: Indentation spaces
   - A984: Multiline scalar in mapping
   - JTV5: Block mapping with multiline scalars
   - NB6Z: Multiline plain value with tabs
   - NJ66: Multiline plain flow mapping key

4. **Empty keys and implicit keys** (8 tests)
   - CFD4: Empty implicit key in single pair flow
   - 8KB6: Multiline plain flow mapping key without value
   - 9BXH: Multiline doublequoted flow mapping key without value
   - 9MMW: Single pair implicit entries
   - 9SA2: Multiline double quoted flow mapping key
   - NKF9: Empty keys in block and flow mapping
   - UDM2: Plain URL in flow mapping

5. **Anchors and tags** (15 tests)
   - 6BFJ: Mapping, key and flow sequence item anchors
   - 6M2F: Aliases in explicit block mapping
   - 7BMT: Node and mapping key anchors
   - 93JH: Block mappings in block sequence
   - E76Z: Aliases in implicit block mapping
   - EHF6: Tags for flow objects
   - FH7J: Tags on empty scalars
   - PW8X: Anchors on empty scalars
   - U3XV: Node and mapping key anchors
   - V55R: Aliases in block sequence
   - X38W: Aliases in flow objects
   - ZWK4: Key with anchor after missing explicit mapping value

6. **Indentation and whitespace** (10 tests)
   - 5GBF: Empty lines
   - 5T43: Colon at beginning of adjacent flow scalar
   - 735Y: Block node types
   - CUP7: Node property indicators
   - DBG4: Plain characters
   - F6MC: More indented lines at beginning of folded block scalars
   - H2RW: Blank lines
   - M6YH: Block sequence indentation
   - KK5P: Various combinations of explicit block mappings

7. **Other parsing issues** (7 tests)
   - BEC7: YAML directive
   - JHB9: Two documents in a stream
   - JR7V: Question marks in scalars
   - LE5A: Flow nodes
   - LP6E: Whitespace after scalars in flow
   - M???: Question mark edge cases
   - RZP5, RZT7: Trailing comments
   - S4JQ: Non-specific tags
   - XLQ9: Multiline scalar that looks like YAML directive
   - XV9V: Empty lines [1.3]
   - XW4D: Various trailing comments
   - YD5X: Sequence of sequences
   - Z9M4: Global tag prefix

### Category 2: Parser Too Lenient (61 tests)
**Expected: error, got: success**

These are invalid YAML documents that the parser incorrectly accepts. Fixing these improves robustness.

**High Priority Issues:**
1. **Invalid flow collections** (12 tests)
   - 9MAG: Flow sequence with invalid comma at beginning
   - CTN5: Flow sequence with invalid extra comma
   - CVW2: Invalid comment after comma
   - C2SP: Flow mapping key on two lines
   - T833: Flow mapping missing separating comma
   - KS4U: Invalid item after end of flow sequence
   - YJV2: Dash in flow sequence

2. **Indentation violations** (10 tests)
   - 4HVU: Wrong indentation in sequence
   - 5LLU: Block scalar with wrong indented line
   - 9C9N: Wrong indented flow sequence
   - DMG6: Wrong indentation in map
   - EW3V: Wrong indentation in mapping
   - N4JP: Bad indentation in mapping
   - U44R: Bad indentation in mapping (2)
   - ZVH3: Wrong indented sequence item

3. **Invalid structure after content** (8 tests)
   - 5TRB: Invalid document-start marker in doublequoted string
   - 6S55: Invalid scalar at end of sequence
   - 9CWY: Invalid scalar at end of mapping
   - 9JBA: Invalid comment after end of flow sequence
   - BD7L: Invalid mapping after sequence
   - P2EQ: Invalid sequence item on same line as previous item
   - TD5N: Invalid scalar after sequence
   - ZL4Z: Invalid nested mapping

4. **Document structure errors** (8 tests)
   - 9KBC: Mapping starting at --- line
   - 9MMA: Directive by itself with no document
   - B63P: Directive without document
   - CXX2: Mapping with anchor on document start line
   - RHX7: YAML directive without document end marker
   - RXY3: Invalid document-end marker in single quoted string
   - SF5V: Duplicate YAML directive
   - N782: Invalid document markers in flow style

5. **Comment and trailing content issues** (8 tests)
   - BS4K: Comment between plain scalar lines
   - DK4H: Implicit key followed by newline
   - GDY7: Comment that looks like mapping key
   - JY7Z: Trailing content that looks like mapping
   - Q4CL: Trailing content after quoted value
   - QB6E: Wrong indented multiline quoted scalar
   - X4QW: Comment without whitespace after block scalar indicator
   - SU5Z: Comment without whitespace after doublequoted scalar

6. **Invalid anchor/alias/tag usage** (8 tests)
   - G9HC: Invalid anchor in zero indented sequence
   - GT5M: Node anchor in sequence
   - H7J7: Node anchor not indented
   - SR86: Anchor plus alias
   - SU74: Anchor and alias as mapping key
   - SY6V: Anchor before sequence entry on same line
   - LHL4: Invalid tag
   - U99R: Invalid comma in tag

7. **Directive and tag errors** (4 tests)
   - H7TQ: Extra words on %YAML directive
   - MJS9-00: Directive variants
   - QLJ7: Tag shorthand used in documents but only defined in first
   - S98Z: Block scalar with more spaces than first content line

8. **Tab-related errors** (3 tests)
   - 000: Tabs in various contexts
   - 004, 005, 007, 009: Tabs in various contexts
   - DK95-01: Tabs that look like indentation

## Recommended Approach

### Phase 1: Quick Wins (Target +10-15% pass rate)
**Focus on Category 1 - Parser Too Strict**

1. **Fix tab handling** (Est. +10 tests)
   - Review tab rules in YAML 1.2 spec
   - Allow tabs in quoted strings (currently rejected)
   - File: `library/src/parser/document/scalar.rs`

2. **Fix empty flow collections** (Est. +3 tests)
   - Allow `{}`, `[]`, `{:}` patterns
   - File: `library/src/parser/document/inline.rs`

3. **Fix trailing commas in flow collections** (Est. +2 tests)
   - Allow optional trailing comma in flow mappings/sequences
   - File: `library/src/parser/document/inline.rs`

### Phase 2: Medium Complexity (Target +10-15% pass rate)
**Focus on multiline scalars and implicit keys**

1. **Improve multiline plain scalar handling** (Est. +8 tests)
   - Better line folding logic
   - Handle empty lines in multiline scalars
   - Files: `library/src/parser/document/scalar.rs`, `library/src/utils/mod.rs`

2. **Fix empty key handling** (Est. +5 tests)
   - Allow empty keys in mappings where spec permits
   - File: `library/src/parser/document/mapping.rs`

3. **Improve implicit key detection** (Est. +5 tests)
   - Multiline implicit keys
   - Complex implicit keys
   - File: `library/src/parser/document/mapping.rs`

### Phase 3: Validation Improvements (Target +5-10% pass rate)
**Focus on Category 2 - Parser Too Lenient**

1. **Add indentation validation** (Est. +10 tests)
   - Stricter indentation checks for block collections
   - Validate sequence item alignment
   - Files: `library/src/parser/document/sequence.rs`, `library/src/parser/document/mapping.rs`

2. **Add flow collection validation** (Est. +8 tests)
   - Reject invalid comma positions
   - Reject invalid structure after closing brackets
   - File: `library/src/parser/document/inline.rs`

3. **Improve document structure validation** (Est. +8 tests)
   - Check for directives without documents
   - Validate document marker positions
   - File: `library/src/parser/document/mod.rs`

### Phase 4: Complex Features (Target +5-10% pass rate)
**Anchor, alias, and tag improvements**

1. **Improve anchor/alias handling** (Est. +10 tests)
   - Validate anchor positions
   - Fix alias resolution in complex structures
   - Files: `library/src/parser/document/anchor.rs`, anchor handling code

2. **Fix tag processing** (Est. +5 tests)
   - Tag validation
   - Tag scope handling
   - File: `library/src/parser/document/tag.rs`

## Expected Outcomes

Following this roadmap:
- **Phase 1:** 67.4% → 77-80% (10-15 tests fixed)
- **Phase 2:** 77-80% → 87-92% (15-20 tests fixed)
- **Phase 3:** 87-92% → 92-97% (15-18 tests fixed)
- **Phase 4:** 92-97% → 95-99% (10-15 tests fixed)

## Implementation Strategy

1. **Pick one category at a time**
2. **Create a focused test file** with just those failing tests
3. **Fix the parser code** incrementally
4. **Verify with full test suite** after each fix
5. **Commit often** with descriptive messages
6. **Document** any spec ambiguities encountered

## Getting Started

To start with Phase 1, pick one of these quick wins:

```powershell
# Run specific failing tests to understand the issue
cd c:\Projects\yaml\library
cargo test --test yaml_test_suite -- 7ZZ5 --nocapture  # Empty flow collections
cargo test --test yaml_test_suite -- 5C5M --nocapture  # Trailing comma in flow mapping
cargo test --test yaml_test_suite -- 01 --nocapture    # Leading tabs in double quoted
```

Then examine the test input files:
```powershell
cat C:\Projects\yaml\tests\yaml-test-suite\7ZZ5\in.yaml
cat C:\Projects\yaml\tests\yaml-test-suite\5C5M\in.yaml
```

And start fixing the parser code!
