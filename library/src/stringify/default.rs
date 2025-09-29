use crate::nodes::node::*;
use crate::io::traits::IDestination;

pub fn stringify(node: &Node, destination: &mut dyn IDestination)-> Result<(), String> {
    match node {
        Node::None => destination.add_bytes("null"),
        // Node::Value(value) => destination.add_bytes(&format!("\"{}\"", value))?,
        Node::Boolean(b) => destination.add_bytes(&b.to_string()),
        Node::Str(s) => destination.add_bytes(&format!("\"{}\"", s)),
        Node::Comment(c) => destination.add_bytes(&format!("// {}", c)),
        Node::Documents(docs) => {
            destination.add_bytes("---\n");
            for (i, doc) in docs.iter().enumerate() {
                if i > 0 {
                    destination.add_bytes("\n---\n");
                }
                stringify(doc, destination)?;
            }
        },
        Node::Number(num) => match num {
            Numeric::Integer(i) => destination.add_bytes(&i.to_string()),
            Numeric::Float(f) => destination.add_bytes(&f.to_string()),
            _ => destination.add_bytes(&format!("{:?}", num)),
        },
        Node::Array(items) => {
            destination.add_bytes("[");
            let mut first = true;
            for item in items {
                if !first {
                    destination.add_bytes(", ");
                }
                stringify(item, destination)?;
                first = false;
            }
            destination.add_bytes("]");
        },
        Node::Dictionary(items) => {
            destination.add_bytes("{");
            let mut first = true;
            for (key, value) in items {
                if !first {
                    destination.add_bytes(", ");
                }
                destination.add_bytes(&format!("\"{}\": ", key));
                stringify(value, destination)?;
                first = false;
            }
            destination.add_bytes("}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;

    #[test]
    fn test_stringify_none() {
        let mut dest = Buffer::new();
        stringify(&Node::None, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "null");
    }

    #[test]
    fn test_stringify_boolean() {
        let mut dest = Buffer::new();
        stringify(&Node::Boolean(true), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "true");
    }

    #[test]
    fn test_stringify_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("test".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"test\"");
    }

    #[test]
    fn test_stringify_comment() {
        let mut dest = Buffer::new();
        stringify(&Node::Comment("test".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "// test");
    }

    #[test]
    fn test_stringify_numbers() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(42)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "42");

        dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(3.14)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "3.14");
    }

    #[test]
    fn test_stringify_array() {
        let mut dest = Buffer::new();
        let arr = vec![Node::Number(Numeric::Integer(1)), Node::Str("test".to_string())];
        stringify(&Node::Array(arr), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[1, \"test\"]");
    }

    #[test]
    fn test_stringify_dictionary() {
        let mut dest = Buffer::new();
        let mut dict = std::collections::HashMap::new();
        dict.insert("key".to_string(), Node::Str("value".to_string()));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_stringify_documents() {
        let mut dest = Buffer::new();
        let docs = vec![Node::Str("doc1".to_string()), Node::Str("doc2".to_string())];
        stringify(&Node::Documents(docs), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "---\n\"doc1\"\n---\n\"doc2\"");
    }

}
