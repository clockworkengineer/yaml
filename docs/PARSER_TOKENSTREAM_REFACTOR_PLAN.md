# Parser TokenStream Refactor Plan (2025-12-17)

## Goal
Migrate as much of the YAML parser as possible from character-based (ISource) parsing to token-based (TokenStream) parsing for improved correctness, maintainability, and spec compliance.

---

## 1. Inventory: Identify All Char-Based Parsing
- [ ] List all parser modules/functions still using ISource directly (outside helpers.rs)

### Inventory of Char-Based Parser Functions (excluding helpers.rs)

#### document/error_builder.rs
	- pub fn context(mut self, source: &mut dyn ISource) -> Self
		- Can be converted: Yes. Should use TokenStream or token context for error reporting.
	- pub fn syntax_error(source: &mut dyn ISource, message: &str) -> String
		- Can be converted: Yes. Should use TokenStream for richer error context.
	- pub fn indentation_error(source: &mut dyn ISource, message: &str) -> String
		- Can be converted: Yes. Should use TokenStream for error context.
	- pub fn structure_error(source: &mut dyn ISource, message: &str) -> String
		- Can be converted: Yes. Should use TokenStream for error context.
	- pub fn expected_error(source: &mut dyn ISource, expected: &str) -> String
		- Can be converted: Yes. Should use TokenStream for error context.
	- pub fn unexpected_error(source: &mut dyn ISource, found: &str) -> String
		- Can be converted: Yes. Should use TokenStream for error context.
	- pub fn forbidden_error(source: &mut dyn ISource, what: &str, where_forbidden: &str) -> String
		- Can be converted: Yes. Should use TokenStream for error context.

#### document/value.rs
	- pub(crate) fn parse_value(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should be refactored to use TokenStream for value parsing.

#### document/sequence.rs
	- pub(crate) fn parse_sequence(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should be refactored to use TokenStream for sequence parsing.
	- fn parse_sequence_inner(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should be refactored to use TokenStream for sequence parsing.

#### document/parse.rs
	- fn parse_document_markers(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should use TokenStream for document marker detection.
	- fn parse_document_end_marker(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should use TokenStream for end marker detection.
	- fn check_explicit_directives(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should use TokenStream for directive parsing.
	- pub fn parse(source: &mut dyn ISource) -> Result<Node, String>
		- Can be converted: Yes. Should use TokenStream as the main entry point for parsing.

#### document/mapping.rs
	- pub(crate) fn parse_mapping(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should be refactored to use TokenStream for mapping parsing.

#### document/main_loop.rs
	- fn parse_document_main_loop(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should use TokenStream for main loop parsing.
	- pub fn parse_document(source: &mut dyn ISource, ...)
		- Can be converted: Yes. Should use TokenStream for document parsing.

#### document/inline.rs
	- (No direct ISource usage in function signatures, but check for internal usage)
		- Can be converted: Already uses TokenStream for inline parsing. No action needed unless indirect char-based helpers are found.

#### document/explicit_key.rs
	- fn parse_and_normalize_explicit_key(source: &mut dyn ISource) -> Result<Node, String>
		- Can be converted: Yes. Should use TokenStream for explicit key parsing.
	- fn should_continue_explicit_key_loop(source: &mut dyn ISource, indent_level: usize) -> bool
		- Can be converted: Yes. Should use TokenStream for loop control.

> Note: All listed functions can be converted to TokenStream-based parsing. No technical blockers identified, but some may require significant refactoring. Inline.rs appears to already use TokenStream, but indirect usage of char-based helpers should be checked and migrated if found.
- [ ] For each, note if/why it cannot be converted

## 2. Scalar Parsing
- [ ] Refactor plain scalar parsing to use TokenStream
- [ ] Refactor quoted scalar parsing to use TokenStream
- [ ] Refactor block scalar parsing to use TokenStream
- [ ] Add/convert tests to cover token-based scalar parsing

## 3. Block/Flow Collection Parsing
- [ ] Refactor block sequence/mapping parsing to use TokenStream
- [ ] Refactor flow sequence/mapping parsing to use TokenStream
- [ ] Ensure context (block/flow, indentation) is handled via tokens
- [ ] Add/convert tests for token-based collection parsing

## 4. Comment Handling
- [ ] Refactor parse_comment to consume Comment tokens
 - [x] Refactor validate_comment_spacing to use token context
- [ ] Add/convert tests for token-based comment handling

## 5. Error Reporting
- [ ] Refactor parse_error to use TokenStream position/context
- [ ] Ensure all error messages include token context

## 6. Node Construction
- [ ] Refactor node construction to build from tokens, not chars
- [ ] Ensure all node types (Array, Mapping, Scalar, etc.) are covered

## 7. Test Coverage
- [ ] Update all tests to use TokenStream where possible
- [ ] Add new tests for edge cases only possible with token-based parsing

## 8. Cleanup
- [ ] Remove/deprecate char-based helpers once all usages are migrated
- [ ] Document any remaining char-based code and why it cannot be converted

---

## Progress Checklist
- [ ] Inventory complete
- [ ] Scalar parsing migrated
- [ ] Block/flow parsing migrated
- [ ] Comment handling migrated
- [ ] Error reporting migrated
- [ ] Node construction migrated
- [ ] Test coverage updated
- [ ] Cleanup complete

---

## Notes
- Prioritize correctness and spec compliance over minimal diff.
- If a function cannot be migrated, document the reason in this file.
- Update this plan as progress is made.
