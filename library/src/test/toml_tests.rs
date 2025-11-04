//! Module: test/toml_tests.rs

#[cfg(test)]
mod tests {
    use crate::io::traits::IDestination;
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{BufferDestination, Node, Numeric, to_toml};

    #[test]
    fn test_toml_basic_string_and_number() {
        let mut buf = BufferDestination::new();
        let n = Node::Str("hi".to_string(), QuoteType::Unquoted, BlockStyle::None);
        to_toml(&n, &mut buf).expect("toml stringify failed");
        assert_eq!(buf.to_string(), "\"hi\"");

        buf.clear();
        let ni = Node::Number(Numeric::Integer(123));
        to_toml(&ni, &mut buf).expect("toml stringify failed");
        assert_eq!(buf.to_string(), "123");
    }

    #[test]
    fn test_toml_mapping_and_nested_table() {
        let mut buf = BufferDestination::new();
        let m = Node::Mapping(vec![(
            Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::from(1),
        )]);
        to_toml(&m, &mut buf).expect("toml stringify failed");
        assert_eq!(buf.to_string(), "a = 1");

        buf.clear();
        let nested = Node::Mapping(vec![(
            Node::Str("parent".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![(
                Node::Str("child".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::from(2),
            )]),
        )]);
        to_toml(&nested, &mut buf).expect("toml stringify failed");
        assert_eq!(buf.to_string(), "[parent]\nchild = 2");
    }

    #[test]
    fn test_toml_array_of_tables() {
        let mut buf = BufferDestination::new();

        let people = Node::Array(vec![
            Node::Mapping(vec![
                (
                    Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("Alice".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
                (
                    Node::Str("age".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::from(30),
                ),
            ]),
            Node::Mapping(vec![
                (
                    Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::Str("Bob".to_string(), QuoteType::Unquoted, BlockStyle::None),
                ),
                (
                    Node::Str("age".to_string(), QuoteType::Unquoted, BlockStyle::None),
                    Node::from(25),
                ),
            ]),
        ]);

        let top = Node::Mapping(vec![(
            Node::Str("people".to_string(), QuoteType::Unquoted, BlockStyle::None),
            people,
        )]);

        to_toml(&top, &mut buf).expect("toml stringify failed");

        let expected =
            "[[people]]\nname = \"Alice\"\nage = 30\n[[people]]\nname = \"Bob\"\nage = 25";
        assert_eq!(buf.to_string(), expected);
    }
}
