#[cfg(test)]
mod tests {
    use crate::io::sources::buffer::Buffer;
    use crate::parser::document::parse;

    #[test]
    #[ignore]
    fn test_su5z_comment_after_quoted_scalar() {
        let input = b"key: \"value\"# invalid comment";
        let mut source = Buffer::new(input);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "SU5Z: Should error on comment without space after quoted scalar"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Comment") || err.contains("whitespace"),
            "Error message: {}",
            err
        );
    }

    #[test]
    #[ignore]
    fn test_x4qw_comment_after_block_scalar_indicator() {
        let input = b"block: ># comment\n  scalar\n";
        let mut source = Buffer::new(input);
        let result = parse(&mut source);
        if result.is_ok() {
            #[cfg(feature = "debug-trace")]
            eprintln!("X4QW parsed successfully (WRONG): {:?}", result);
        }
        assert!(
            result.is_err(),
            "X4QW: Should error on comment without space after block scalar indicator"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Comment") || err.contains("whitespace"),
            "Error message: {}",
            err
        );
    }

    #[test]
    #[ignore] // TODO: Need better logic to distinguish misaligned vs nested sequences
    fn test_zvh3_wrong_indented_sequence_item() {
        let input = b"- key: value\n - item1\n";
        let mut source = Buffer::new(input);
        let result = parse(&mut source);
        assert!(
            result.is_err(),
            "ZVH3: Should error on inconsistent sequence indentation"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Inconsistent") || err.contains("indentation"),
            "Error message: {}",
            err
        );
    }
}
