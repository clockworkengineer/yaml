//! Tests for directive parsing and tag resolution

use crate::io::sources::buffer::Buffer as BufferSource;
use crate::parse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_tag_prefix_5tym() {
        // Test from 5TYM: Local tag prefix with multiple documents
        let yaml = b"%TAG !m! !my-\n--- # Bulb here\n!m!light fluorescent\n...\n%TAG !m! !my-\n--- # Color here\n!m!light green\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        if let Err(ref e) = result {
            // Helpful context if this regression ever reappears
            eprintln!("5TYM parse error: {}", e);
        }

        assert!(
            result.is_ok(),
            "Should parse TAG directive with local prefix"
        );
    }

    #[test]
    fn test_primary_tag_handle_6wlz() {
        // Test from 6WLZ: Primary tag handle (!)
        let yaml = b"# Private\n---\n!foo \"bar\"\n...\n# Global\n%TAG ! tag:example.com,2000:app/\n---\n!foo \"bar\"\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        #[cfg(feature = "debug-trace")]
        println!("6WLZ Result: {:?}", result);

        assert!(result.is_ok(), "Should parse primary TAG handle directive");
    }

    #[test]
    fn test_yaml_version_directive() {
        let yaml = b"%YAML 1.2\n---\ntest: value\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        assert!(result.is_ok(), "Should parse YAML version directive");
    }

    #[test]
    fn test_tag_directive_simple() {
        let yaml = b"%TAG !e! tag:example.com,2000:\n---\n!e!type value\n";
        let mut source = BufferSource::new(yaml);
        let result = parse(&mut source);

        #[cfg(feature = "debug-trace")]
        println!("Simple TAG Result: {:?}", result);

        assert!(result.is_ok(), "Should parse simple TAG directive");
    }
}
