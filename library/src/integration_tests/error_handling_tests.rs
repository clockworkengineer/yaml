///
/// Error handling tests: parsing errors, invalid syntax, edge cases.
///
#[cfg(test)]
mod tests {
    use crate::{BufferSource, parse};

    #[test]
    fn test_parse_undefined_alias_errors() {
        let mut source = BufferSource::new(b"---\nvalue: *nope\n");
        let res = parse(&mut source);
        assert!(res.is_err());
    }

    #[test]
    fn test_error_on_empty_alias_name() {
        use crate::error::messages::ERR_EMPTY_ALIAS_NAME;
        let mut source = BufferSource::new(b"---\nvalue: *\n");
        let res = parse(&mut source);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains(ERR_EMPTY_ALIAS_NAME));
    }

    #[test]
    fn test_error_on_empty_anchor_name() {
        use crate::error::messages::ERR_EMPTY_ANCHOR_NAME;
        let mut source = BufferSource::new(b"---\nroot: &\n  nested: 1\n");
        let res = parse(&mut source);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains(ERR_EMPTY_ANCHOR_NAME));
    }

    #[test]
    fn test_error_on_duplicate_anchor() {
        use crate::error::messages::ERR_DUPLICATE_ANCHOR_PREFIX;
        let yaml = b"---\na: &dup\n  x: 1\nb: &dup\n  y: 2\n";
        let mut source = BufferSource::new(yaml);
        let res = parse(&mut source);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains(ERR_DUPLICATE_ANCHOR_PREFIX));
    }
}
