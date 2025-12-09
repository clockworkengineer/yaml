# Token Parser Refactor TODOs (Consolidated)

Date: 2025-12-09

- [x] Directive boundary validation
- [x] Missing-colon detection hardening
- [x] Single-anchor per node enforcement
- [x] Explicit key parsing via tokens
- [x] Indented values after decorators
- [x] Mapping edge cases: empty/multiline
- [ ] Roundtrip stability tests
- [ ] Property/fuzz test harness
- [ ] Serializer: preserve raw tag handles
- [ ] TokenStream reuse/skip optimizations
- [ ] yaml-test-suite sweep
- [ ] Docs: update refactor guides

Upcoming Work (Detailed)
- [x] Block scalar tokens: folded/literal, chomping, indent
- [ ] Full tokenization of block mappings/sequences
- [ ] Merge key handling (<<) during parse
- [ ] Error spans via token boundaries
- [ ] Performance pass on TokenStream skip operations
