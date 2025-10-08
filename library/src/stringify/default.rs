use crate::nodes::node::*;
use crate::io::traits::IDestination;

pub fn stringify_document(node: &Node, destination: &mut dyn IDestination)-> Result<(), String> {
    match node {
        Node::None => destination.add_bytes("null"),
        // Node::Value(value) => destination.add_bytes(&format!("\"{}\"", value))?,
        Node::Boolean(b) => destination.add_bytes(&b.to_string()),
        Node::Str(s) => destination.add_bytes(&format!("\"{}\"", s)),
        Node::Comment(c) => destination.add_bytes(&format!("# {}", c)),
        Node::Number(num) => match num {
            Numeric::Integer(i) => destination.add_bytes(&i.to_string()),
            Numeric::Float(f) => destination.add_bytes(&f.to_string()),
            _ => destination.add_bytes(&format!("{:?}", num)),
        },
        Node::Array(items) => {
            for (_i, item) in items.iter().enumerate() {
                destination.add_bytes("- ");
                stringify_document(item, destination)?;
                destination.add_bytes("\n");
            }
        },
        Node::Dictionary(items) => {
            for (key, value) in items {
                destination.add_bytes(&format!("\"{}\": ", key));
                stringify_document(value, destination)?;
                destination.add_bytes("\n");
            }
        },
        Node::Document(nodes) => {
            for node in nodes {
                stringify_document(node, destination)?;
            }
        }
        _ => { return Err("Unsupported node type".to_string()); }
    }
    Ok(())  
}

pub fn stringify(node: &Node, destination: &mut dyn IDestination)-> Result<(), String> {
    match node {
        Node::Documents(docs) => {
            for doc in docs {
                destination.add_bytes("---\n");
                stringify_document(doc, destination)?;
                destination.add_bytes("...\n");
            }
          
        },
        _ => { stringify_document(node, destination)?; }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{parse, BufferSource};
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
        assert_eq!(dest.to_string(), "# test");
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
        assert_eq!(dest.to_string(), "- 1\n- \"test\"\n");
    }

    #[test]
    fn test_stringify_dictionary() {
        let mut dest = Buffer::new();
        let mut dict = std::collections::HashMap::new();
        dict.insert("key".to_string(), Node::Str("value".to_string()));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"key\": \"value\"\n");
    }

    #[test]
    fn test_stringify_documents() {
        let mut dest = Buffer::new();
        let docs = vec![Node::Str("doc1".to_string()), Node::Str("doc2".to_string())];
        stringify(&Node::Documents(docs), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "---\n\"doc1\"...\n---\n\"doc2\"...\n");
    }

    #[test]
    fn test_stringify_integer_sequence() {
        let mut dest = Buffer::new();
        let mut source  = BufferSource::new("---\n- 1\n- 2\n- 3\n...\n".as_bytes());
        let  node  = parse(&mut source).unwrap();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "---\n- 1\n- 2\n- 3\n...\n");
    }

}
