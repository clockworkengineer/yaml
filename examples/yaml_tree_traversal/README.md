# Tree Traversal and Visitor Pattern Example

This example demonstrates the powerful tree traversal and visitor pattern APIs for navigating and transforming YAML documents.

## Features Demonstrated

1. **children()** - Iterator over immediate child nodes
2. **visit()** - Recursive immutable traversal with depth tracking
3. **visit_mut()** - Recursive mutable traversal for transformations
4. **find_all()** - Search for nodes matching a predicate
5. **find_first()** - Find first matching node with early termination
6. **count_nodes()** - Count all nodes in tree
7. **max_depth()** - Find maximum tree depth

## Usage

```bash
cargo run --example yaml_tree_traversal
```

## What You'll Learn

- How to iterate through document trees efficiently
- Implementing custom traversal logic with visitors
- Searching and filtering nodes with predicates
- Modifying documents during traversal
- Collecting statistics about tree structure

## Real-World Applications

- **Document Validation** - Walk tree and validate all nodes
- **Data Transformation** - Modify values during traversal
- **Search & Filter** - Find specific nodes or patterns
- **Statistics Gathering** - Count node types, measure depth
- **Schema Enforcement** - Ensure document structure compliance
