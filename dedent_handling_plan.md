# Plan to Fix Dedent Handling for Mappings and Sequences

## 1. Mapping Parser
- [x] Track indentation levels using a stack.
- [x] When a new key is encountered:
  - [x] If its indentation is less than the current (top of stack), pop the stack until the parent level is reached.
  - [x] If its indentation is equal to the parent, close the nested mapping and add the key to the parent mapping.
- [x] Ensure that after closing a nested mapping, the next key/value pair is added to the correct parent.

## 2. Sequence Parser
- [x] Track indentation levels using a stack.
- [x] When a dash is encountered:
  - [x] If its indentation is less than the current (top of stack), pop the stack until the parent level is reached.
  - [x] If its indentation is equal to the parent, close the nested sequence and add the item to the parent sequence.
- [x] After closing a nested sequence, ensure the next item is added to the correct parent.

## 3. Shared Logic
- [x] Refactor both parsers to check indentation before processing each key/item.
- [x] On dedent, always pop the stack and attach the closed block to its parent.
- [x] On encountering a sibling (same indentation), close the current nested block and continue in the parent context.

## 4. Testing
- [x] Add/expand unit tests for:
  - [x] Sibling keys/items after nested blocks.
  - [x] Deeply nested structures with multiple dedents.
  - [x] Edge cases for explicit keys and set formats.

## 5. Review and Iterate
- [x] Validate with failing integration tests.
- [x] Refine stack pop/attach logic as needed for correct YAML structure.
