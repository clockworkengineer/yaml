# SOLID Refactoring Plan for `YAML_lib`

This document details the architectural plan to refactor `YAML_lib` into a completely SOLID-compliant architecture.

---

## Executive Summary

An architectural analysis of `YAML_lib` identified key refactoring opportunities to improve single responsibility, modularity, open/closed trait extensibility, interface segregation, and dependency inversion across the core parser, AST nodes, format converters, and I/O traits.

---

## SOLID Architectural Improvements

### 1. Single Responsibility Principle (SRP)
- **Problem**: [`nodes/node.rs`](file:///c:/Projects/yaml/library/src/nodes/node.rs) (92KB) combines AST data structures, builder logic, string formatting, tree traversal, visitor functions, and node searching into one file.
- **Solution**: Break `nodes/node.rs` into specialized submodules:
  - `nodes/core.rs`: Core `Node` and `Numeric` data types.
  - `nodes/query.rs`: Tree search functions (`find_all`, `find_first`, `contains_key`).
  - `nodes/traversal.rs`: Tree visitors and recursive traversal.
  - `nodes/builders/`: Modular builders (`ArrayBuilder`, `MappingBuilder`, `SetBuilder`).
  - `nodes/display.rs`: `Display` and `Debug` formatting implementations.
- **Problem**: `misc/mod.rs` acts as a catch-all module.
- **Solution**: Relocate document functions into `parser::document` and version info into `constants`, eliminating `misc/mod.rs`.

---

### 2. Open/Closed Principle (OCP)
- **Problem**: Format converters in `stringify/` (`json.rs`, `toml.rs`, `xml.rs`, `bencode.rs`, `default.rs`) do not share a common serializer trait contract. Adding a new output format requires modifying module exports and writing standalone functions.
- **Solution**: Define a public `NodeSerializer` trait:
  ```rust
  pub trait NodeSerializer {
      fn serialize(&self, node: &Node, dest: &mut dyn IDestination) -> crate::error::Result<()>;
      fn serialize_pretty(&self, node: &Node, dest: &mut dyn IDestination) -> crate::error::Result<()> {
          self.serialize(node, dest)
      }
  }
  ```
  Implement `NodeSerializer` for each format converter (`YamlSerializer`, `JsonSerializer`, `TomlSerializer`, `XmlSerializer`, `BencodeSerializer`).

---

### 3. Liskov Substitution Principle (LSP)
- **Problem**: Stream reading implementations (`BufferSource` vs `FileSource`) must handle EOF, position snapshots (`SaveState`), and character buffering with identical semantics.
- **Solution**: Enforce strict contract alignment and error handling parity across `BufferSource` and `FileSource`.

---

### 4. Interface Segregation Principle (ISP)
- **Problem**: [`ISource`](file:///c:/Projects/yaml/library/src/io/traits.rs) combines character reading, state save/restore, whitespace checks, and indentation level queries into a single large trait.
- **Solution**: Segregate `ISource` into focused interface traits:
  - `ICharStream`: `next()`, `current()`, `more()`
  - `IStatefulStream`: `save_state()`, `restore_state()`
  - `IIndentationAware`: `get_current_indent_level()`
  - Define `ISource` as a composite super-trait (`ICharStream + IStatefulStream + IIndentationAware`) for backwards compatibility.

---

### 5. Dependency Inversion Principle (DIP)
- **Problem**: High-level parsing and serialization helper functions should rely on stream and destination abstractions (`ISource`, `IDestination`, `NodeSerializer`) rather than concrete buffer or file operations.
- **Solution**: Route all high-level `to_<format>` functions to consume `&mut dyn IDestination` through the `NodeSerializer` abstraction interface.

---

## Verification Plan
1. Run `cargo test --all-targets` to verify all 1090+ unit and integration tests pass.
2. Run `cargo test --test yaml_test_suite` to verify official YAML 1.2 specification compliance remains 100%.
3. Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
