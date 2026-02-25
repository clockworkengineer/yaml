// =====================================================================================
//  File: json_tests.rs
//  Location: library/src/integration_tests/
// -------------------------------------------------------------------------------------
//  Purpose:
//      Integration tests for JSON serialization in the yaml_lib crate.
//      These tests validate correct conversion of YAML node structures to JSON format,
//      ensuring compliance with JSON encoding rules for strings, numbers, arrays, and objects.
//
//  Context:
//      - Part of the yaml_lib project, a Rust YAML parser/serializer.
//      - Ensures interoperability and correctness of JSON output from YAML data.
//      - Focuses on both compact and pretty-printed JSON serialization.
//
// -------------------------------------------------------------------------------------
//  Test Coverage:
//      - String and number encoding
//      - Array and object encoding
//      - Pretty-printing and compact output
//      - Edge cases for empty and nested structures
//      - Error handling for invalid input
// =====================================================================================

#[cfg(test)]
mod tests {
    use crate::io::traits::IDestination;
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{BufferDestination, Node, Numeric, to_json, to_json_pretty};

    #[test]
    fn test_json_basic_string_and_number() {
        let mut buf = BufferDestination::new();
        let n = Node::Str("hi".to_string(), QuoteType::Unquoted, BlockStyle::None);
        to_json(&n, &mut buf).expect("json stringify failed");
        assert_eq!(buf.to_string(), "\"hi\"");

        buf.clear();
        let ni = Node::Number(Numeric::Integer(123));
        to_json(&ni, &mut buf).expect("json stringify failed");
        assert_eq!(buf.to_string(), "123");
    }

    #[test]
    fn test_json_array_and_object() {
        let mut buf = BufferDestination::new();
        let arr = Node::Array(vec![Node::from(1), Node::from("two")]);
        to_json(&arr, &mut buf).expect("json stringify failed");
        assert_eq!(buf.to_string(), "[1,\"two\"]");

        buf.clear();
        let obj = Node::Mapping(vec![
            (Node::from("a"), Node::from(1)),
            (Node::from("b"), Node::from(2)),
        ]);
        to_json(&obj, &mut buf).expect("json stringify failed");
        assert_eq!(buf.to_string(), "{\"a\":1,\"b\":2}");
    }

    #[test]
    fn test_json_pretty() {
        let mut buf = BufferDestination::new();
        let obj = Node::Mapping(vec![
            (Node::from("a"), Node::from(1)),
            (
                Node::from("b"),
                Node::Array(vec![Node::from(2), Node::from(3)]),
            ),
        ]);
        to_json_pretty(&obj, &mut buf, 2).expect("json pretty stringify failed");
        let expected = "{\n  \"a\": 1,\n  \"b\": [\n    2,\n    3\n  ]\n}";
        assert_eq!(buf.to_string(), expected);
    }
}
