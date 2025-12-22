# YAML Parser Refactor Plan (for test failures)

## 1. Explicit Sequence Keys Splitting Documents
- Refactor sequence parsing logic to avoid splitting documents on dashes when inside explicit key or flow context. **DONE**
- Add logic to check parser context before treating a dash as a new document start. **DONE**
- Add/expand unit tests for explicit sequence key edge cases.

## 2. Nested Mapping Key Placement
- Track current indentation level for each mapping using a stack.
- On encountering a new key at the same indentation as a parent, close the nested mapping and add the key to the parent.
- Refactor mapping parser to trigger dedent-based mapping closure.
- Add/expand unit tests for nested mapping dedent cases.

## 3. Nested Sequence Item Placement
- Track indentation level for each sequence using a stack.
- On encountering a dash at the same indentation as the parent, close the nested sequence and add the item to the parent.
- Refactor sequence parser to trigger dedent-based sequence closure.
- Add/expand unit tests for nested sequence dedent cases.

## General
- Use a context/indentation stack for both mappings and sequences.
- Ensure context transitions (push/pop) are correct for all nested/sibling structures.
- Add focused unit tests for all identified edge cases.
