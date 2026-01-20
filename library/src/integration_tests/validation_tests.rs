//! Integration tests for validation and error detection

use crate::io::sources::buffer::Buffer;
use crate::parser::document::parse;

#[cfg(test)]
mod tests {
    use super::*;

    // ...existing code...

    #[test]
    fn test_4ejs_tabs_forbidden_as_indentation() {
        // Tabs as indentation in a mapping should be rejected
        let yaml = b"key1: value1\nkey2:\n\tvalue2"; // Tab before 'value2'
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject tabs as indentation: {:?}",
            result
        );
        if let Err(e) = result {
            let e_str = e.to_string().to_lowercase();
            assert!(
                e_str.contains("tab"),
                "Error should mention tabs: {}",
                e_str
            );
        }
    }

    #[test]
    fn test_tabs_allowed_in_quoted_strings() {
        // Tabs inside quoted strings should be allowed
        let yaml = b"key: \"value\twith\ttab\"";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_ok(),
            "Tabs inside quoted strings should be allowed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_tabs_allowed_in_comments() {
        // Tabs in comments should be allowed
        let yaml = b"key: value  #\tcomment\twith\ttabs";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_ok(),
            "Tabs in comments should be allowed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_tabs_forbidden_in_flow_indentation() {
        // Tabs as indentation in flow collections should be rejected per YAML 1.2 spec
        let yaml = b"[\n\titem\n]";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject tabs as indentation in flow collections"
        );
        if let Err(e) = result {
            let e_str = e.to_string().to_lowercase();
            assert!(
                e_str.contains("tab"),
                "Error should mention tabs: {}",
                e_str
            );
        }
    }

    // ...existing code...

    // ...existing code...

    #[test]
    fn test_2g84_00_block_scalar_indent_zero() {
        let yaml = b"--- |0\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject block scalar with indentation indicator 0"
        );
        if let Err(e) = result {
            let e_str = e.to_string();
            assert!(
                e_str.contains("indentation indicator") && e_str.contains("1-9"),
                "Error: {}",
                e_str
            );
        }
    }

    #[test]
    fn test_2g84_01_block_scalar_indent_ten() {
        let yaml = b"--- |10\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject block scalar with indentation indicator 10"
        );
        if let Err(e) = result {
            let e_str = e.to_string();
            assert!(
                e_str.contains("single digit") && e_str.contains("1-9"),
                "Error: {}",
                e_str
            );
        }
    }

    #[test]
    fn test_g5u8_invalid_flow_dash_entries() {
        let yaml = b"---\n- [-, -]\n";
        let mut source = Buffer::new(yaml);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "Should reject flow sequence entries that are bare '-' indicators"
        );
    }
}
