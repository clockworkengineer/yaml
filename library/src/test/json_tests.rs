#[cfg(test)]
mod tests {
    use crate::io::traits::IDestination;
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{
        BufferDestination, Node, Numeric
    };

#[test]
fn test_json_basic_string_and_number() {
    let mut buf = BufferDestination::new();
    let n = Node::Str("hi".to_string(), QuoteType::Unquoted, BlockStyle::None);
    crate::stringify::json::stringify(&n, &mut buf).expect("json stringify failed");
    assert_eq!(buf.to_string(), "\"hi\"");

    buf.clear();
    let ni = Node::Number(Numeric::Integer(123));
    crate::stringify::json::stringify(&ni, &mut buf).expect("json stringify failed");
    assert_eq!(buf.to_string(), "123");
}

#[test]
fn test_json_array_and_object() {
    let mut buf = BufferDestination::new();
    let arr = Node::Array(vec![Node::from(1), Node::from("two")]);
    crate::stringify::json::stringify(&arr, &mut buf).expect("json stringify failed");
    assert_eq!(buf.to_string(), "[1,\"two\"]");

    buf.clear();
    let obj = Node::Mapping(vec![
        (Node::from("a"), Node::from(1)),
        (Node::from("b"), Node::from(2)),
    ]);
    crate::stringify::json::stringify(&obj, &mut buf).expect("json stringify failed");
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
    crate::stringify::json::stringify_pretty(&obj, &mut buf, 2)
        .expect("json pretty stringify failed");
    let expected = "{\n  \"a\": 1,\n  \"b\": [\n    2,\n    3\n  ]\n}";
    assert_eq!(buf.to_string(), expected);
}
}
