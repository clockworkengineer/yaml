//! YAML parser implementation that converts YAML text into Node structures
//! Provides functions for parsing different YAML data types including mappings,
//! sequences, strings, numbers, boolean and null values.

use crate::nodes::node::Node;
use crate::nodes::node::Numeric;
use std::collections::HashMap;
use crate::io::traits::ISource;
// use crate::error::messages::*;

fn skip_whitespace(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if !c.is_whitespace() {
            break;
        }
        source.next();
    }
}

fn parse_scalar(value: &str) -> Node {
    if value == "null" || value == "~" {
        Node::None
    } else if value == "true" {
        Node::Boolean(true)
    } else if value == "false" {
        Node::Boolean(false)
    } else if let Ok(i) = value.parse::<i64>() {
        Node::Number(Numeric::Integer(i))
    } else if let Ok(f) = value.parse::<f64>() {
        Node::Number(Numeric::Float(f))
    } else {
        Node::Str(value.to_string())
    }
}

fn parse_sequence(source: &mut dyn ISource) -> Result<Node, String> {
    let mut items = Vec::new();
    while let Some(c) = source.current() {
        if c == '-' {
            source.next();
            skip_whitespace(source);
            let mut value = String::new();
            while let Some(c) = source.current() {
                if c == '\n' { break; }
                value.push(c);
                source.next();
            }
            items.push(parse_scalar(value.trim()));
        } else {
            break;
        }
        source.next();
    }
    Ok(Node::Array(items))
}

fn parse_mapping(source: &mut dyn ISource) -> Result<Node, String> {
    let mut map = HashMap::new();
    while let Some(c) = source.current() {
        if c.is_alphanumeric() {
            let mut key = String::new();
            while let Some(c) = source.current() {
                if c == ':' { break; }
                key.push(c);
                source.next();
            }
            source.next(); // Skip ':'
            skip_whitespace(source);

            let mut value = String::new();
            while let Some(c) = source.current() {
                if c == '\n' { break; }
                value.push(c);
                source.next();
            }

            map.insert(key.trim().to_string(), parse_scalar(value.trim()));
        }
        source.next();
    }
    Ok(Node::Dictionary(map))
}


pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    skip_whitespace(source);

    match source.current() {
        None => Ok(Node::None),
        Some('-') => parse_sequence(source),
        Some(c) if c.is_alphanumeric() => parse_mapping(source),
        Some(c) => Err(format!("Unexpected character: {}", c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_parse_scalar() {
        assert_eq!(parse_scalar("null"), Node::None);
        assert_eq!(parse_scalar("~"), Node::None);
        assert_eq!(parse_scalar("true"), Node::Boolean(true));
        assert_eq!(parse_scalar("false"), Node::Boolean(false));
        assert_eq!(parse_scalar("42"), Node::Number(Numeric::Integer(42)));
        assert_eq!(parse_scalar("3.14"), Node::Number(Numeric::Float(3.14)));
        assert_eq!(parse_scalar("hello"), Node::Str("hello".to_string()));
    }

    #[test]
    fn test_parse_sequence() {
        let mut source = Buffer::new(b"- 1\n- 2\n- 3");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::Array(vec![
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Integer(2)),
            Node::Number(Numeric::Integer(3))
        ]));
    }

    #[test]
    fn test_parse_mapping() {
        let mut source = Buffer::new(b"key1: value1\nkey2: 42");
        let result = parse(&mut source).unwrap();
        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), Node::Str("value1".to_string()));
        expected.insert("key2".to_string(), Node::Number(Numeric::Integer(42)));
        assert_eq!(result, Node::Dictionary(expected));
    }

    #[test]
    fn test_parse_empty() {
        let mut source = Buffer::new(b"");
        let result = parse(&mut source).unwrap();
        assert_eq!(result, Node::None);
    }

}

