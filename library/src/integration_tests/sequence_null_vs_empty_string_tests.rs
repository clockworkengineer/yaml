use crate::{BufferSource, Node, parse};

#[test]
fn test_sequence_null_vs_empty_string_cases() {
    let cases = vec![
        (
            b"-\n-\n" as &[u8],
            vec![Node::None, Node::None],
            "dash + newline",
        ),
        (
            b"- \n- \n",
            vec![Node::None, Node::None],
            "dash + space + newline",
        ),
        (
            b"- ''\n- \"\"\n",
            vec![
                Node::Str(
                    String::new(),
                    crate::nodes::node::QuoteType::Single,
                    crate::nodes::node::BlockStyle::None,
                ),
                Node::Str(
                    String::new(),
                    crate::nodes::node::QuoteType::Double,
                    crate::nodes::node::BlockStyle::None,
                ),
            ],
            "explicit empty strings",
        ),
    ];
    for (yaml, expected, label) in cases {
        let node = {
            let mut source = BufferSource::new(yaml);
            parse(&mut source).expect(label)
        };
        let arr = match &node {
            Node::Document(items) => {
                if let Some(Node::Array(arr)) = items.first() {
                    arr
                } else {
                    panic!(
                        "{}: Expected Array as first document element, got: {:#?}",
                        label, node
                    )
                }
            }
            Node::Documents(docs) => {
                if let Some(Node::Document(items)) = docs.first() {
                    if let Some(Node::Array(arr)) = items.first() {
                        arr
                    } else {
                        panic!(
                            "{}: Expected Array as first document element, got: {:#?}",
                            label, node
                        )
                    }
                } else {
                    panic!(
                        "{}: Expected Document as first element in Documents, got: {:#?}",
                        label, node
                    )
                }
            }
            _ => panic!(
                "{}: Expected Document or Documents at root, got: {:#?}",
                label, node
            ),
        };
        assert_eq!(
            arr.len(),
            expected.len(),
            "{}: Array length mismatch",
            label
        );
        for (i, (item, exp)) in arr.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                item, exp,
                "{}: Item {} mismatch: got {:#?}, expected {:#?}",
                label, i, item, exp
            );
        }
    }
}
