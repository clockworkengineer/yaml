// =====================================================================================
//  File: directive_tests.rs
//  Location: library/src/integration_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Integration tests for YAML directive parsing and handling in the yaml_lib crate.
//      These tests validate correct recognition and processing of YAML directives such as
//      %TAG and %YAML, ensuring compliance with the YAML specification for document structure
//      and tag resolution.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Directives control parsing behavior, tag resolution, and document versioning.
//      - Tests ensure robust handling of directives and edge cases in multi-document streams.
//
//  Authors:      (Add your name or contributors here)
//  Created:      (Add creation date if known)
//  Last Updated: 2026-02-23
// -------------------------------------------------------------------------------------
//  Test Coverage:
//      - %TAG and %YAML directive parsing
//      - Local and global tag handles
//      - Multi-document streams with directives
//      - Edge cases for directive placement and errors
// =====================================================================================

#[cfg(test)]
mod tests {
    // ...existing code...

    #[test]
    fn test_local_tag_prefix_5tym() {
        // Test from 5TYM: Local tag prefix with multiple documents
        let yaml = b"%TAG !m! !my-\n--- # Bulb here\n!m!light fluorescent\n...\n%TAG !m! !my-\n--- # Color here\n!m!light green\n";
        let result = crate::test_helpers::parse_yaml(yaml);

        // parse_yaml returns Node, so just check it's a Node::Documents
        assert!(
            matches!(result, crate::Node::Documents(_)),
            "Should parse TAG directive with local prefix"
        );
    }

    #[test]
    fn test_primary_tag_handle_6wlz() {
        // Test from 6WLZ: Primary tag handle (!)
        let yaml = b"# Private\n---\n!foo \"bar\"\n...\n# Global\n%TAG ! tag:example.com,2000:app/\n---\n!foo \"bar\"\n";
        let result = crate::test_helpers::parse_yaml(yaml);

        #[cfg(feature = "debug-trace")]
        println!("6WLZ Result: {:?}", result);
        assert!(
            matches!(result, crate::Node::Documents(_)),
            "Should parse primary TAG handle directive"
        );
    }

    #[test]
    fn test_yaml_version_directive() {
        let yaml = b"%YAML 1.2\n---\ntest: value\n";
        let result = crate::test_helpers::parse_yaml(yaml);
        assert!(
            matches!(result, crate::Node::Documents(_)),
            "Should parse YAML version directive"
        );
    }

    #[test]
    fn test_tag_directive_simple() {
        let yaml = b"%TAG !e! tag:example.com,2000:\n---\n!e!type value\n";
        let result = crate::test_helpers::parse_yaml(yaml);
        #[cfg(feature = "debug-trace")]
        println!("Simple TAG Result: {:?}", result);
        assert!(
            matches!(result, crate::Node::Documents(_)),
            "Should parse simple TAG directive"
        );
    }
}
