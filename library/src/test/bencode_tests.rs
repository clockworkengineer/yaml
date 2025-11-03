#[cfg(test)]
mod tests {
    use crate::io::traits::IDestination;
    use crate::nodes::node::{BlockStyle, QuoteType};
    use crate::{
        BufferDestination, Node, Numeric
    };

#[test]
fn test_bencode_string_and_integer() {
    let mut buf = BufferDestination::new();
    let n = Node::Str("hello".to_string(), QuoteType::Unquoted, BlockStyle::None);
    crate::stringify::bencode::stringify(&n, &mut buf).expect("bencode stringify failed");
    assert_eq!(buf.to_string(), "5:hello");

    buf.clear();
    let ni = Node::Number(Numeric::Integer(42));
    crate::stringify::bencode::stringify(&ni, &mut buf).expect("bencode stringify failed");
    assert_eq!(buf.to_string(), "i42e");
}

#[test]
fn test_bencode_list_and_map_sorting() {
    let mut buf = BufferDestination::new();
    let list = Node::Array(vec![Node::from(1), Node::from("two")]);
    crate::stringify::bencode::stringify(&list, &mut buf).expect("bencode stringify failed");
    assert_eq!(buf.to_string(), "li1e3:twoe");

    buf.clear();
    let mapping = Node::Mapping(vec![
        (Node::from("b"), Node::from("beta")),
        (Node::from("a"), Node::from("alpha")),
    ]);
    crate::stringify::bencode::stringify(&mapping, &mut buf).expect("bencode stringify failed");
    // keys should be sorted lexicographically: 'a' then 'b'
    assert_eq!(buf.to_string(), "d1: a5:alpha1: b4:betae".replace(" ", ""));
}

#[test]
fn test_bencode_float_and_boolean() {
    let mut buf = BufferDestination::new();
    let nf = Node::Number(Numeric::Float(3.14));
    crate::stringify::bencode::stringify(&nf, &mut buf).expect("bencode stringify failed");
    assert_eq!(buf.to_string(), "4:3.14");

    buf.clear();
    let nb = Node::Boolean(true);
    crate::stringify::bencode::stringify(&nb, &mut buf).expect("bencode stringify failed");
    assert_eq!(buf.to_string(), "4:true");
}

#[test]
fn test_bencode_nested_mapping() {
    let mut buf = BufferDestination::new();
    let mapping = Node::Mapping(vec![(
        Node::from("list"),
        Node::Array(vec![Node::from(1), Node::from(2)]),
    )]);
    crate::stringify::bencode::stringify(&mapping, &mut buf).expect("bencode stringify failed");
    // Expect: d4:listl i1e i2e e  -> without spaces: d4:listli1ei2ee
    assert_eq!(buf.to_string(), "d4:listli1ei2eee");
}
}
