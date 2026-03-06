fn should_preserve_double_bang(tag_raw: &str) -> bool {
    if let Some(suffix) = tag_raw.strip_prefix("!!") {
        match suffix {
            // Core scalar and collection tags commonly preserved in tests
            "str" | "int" | "float" | "bool" | "null" | "timestamp" | "yaml" | "binary" | "map"
            | "seq" | "set" | "omap" | "pairs" => true,
            // Extended integer formats used in tests
            "int:hex" | "int:oct" => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Determine the tag string to store in `Node::Tagged`.
///
/// - `!!`-prefixed known core tags → preserve the raw `!!` form
/// - `!!`-prefixed unknown tags    → canonicalize to the resolved URI
/// - all other tags (custom/handle) → preserve the raw form verbatim
///
/// This consolidates two identical blocks that previously appeared in
/// `parse_value_with_tokens` (empty-decorated path and full-content path).
#[inline]
fn pick_tag_text(tag_raw: &str, resolved: String) -> String {
    if tag_raw.starts_with("!!") {
        if should_preserve_double_bang(tag_raw) {
            tag_raw.to_owned()
        } else {
            resolved
        }
    } else {
        tag_raw.to_owned()
    }
}

/// Try to coerce a value based on a tag
fn try_coerce_tag(tag: &str, node: Node) -> Option<Node> {
    match tag {
        "!!int:hex" => match node {
            Node::Str(s, _, _) => {
                if let Some(stripped) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    if let Ok(i) = i64::from_str_radix(stripped, 16) {
                        Some(Node::Number(Numeric::Integer(i)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!int:oct" => match node {
            Node::Str(s, _, _) => {
                if let Some(stripped) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
                    if let Ok(i) = i64::from_str_radix(stripped, 8) {
                        Some(Node::Number(Numeric::Integer(i)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!str" | "!str" | "tag:yaml.org,2002:str" => {
            let s = match node {
                Node::Str(s, _, _) => s,
                Node::Number(Numeric::Integer(i)) => i.to_string(),
                Node::Number(Numeric::Float(f)) => f.to_string(),
                Node::Boolean(b) => b.to_string(),
                Node::None => String::new(),
                _ => return None,
            };
            Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None))
        }
        "!!int" | "!int" | "tag:yaml.org,2002:int" => match node {
            Node::Number(Numeric::Integer(i)) => Some(Node::Number(Numeric::Integer(i))),
            Node::Str(s, _, _) => {
                // Accept both quoted and unquoted numbers, and also coerce floats that are integers
                if let Ok(i) = s.parse::<i64>() {
                    Some(Node::Number(Numeric::Integer(i)))
                } else if let Ok(f) = s.parse::<f64>() {
                    if f.fract() == 0.0 {
                        Some(Node::Number(Numeric::Integer(f as i64)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Node::Number(Numeric::Float(f)) => {
                if f.fract() == 0.0 {
                    Some(Node::Number(Numeric::Integer(f as i64)))
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!float" | "!float" | "tag:yaml.org,2002:float" => match node {
            Node::Number(Numeric::Float(f)) => Some(Node::Number(Numeric::Float(f))),
            Node::Number(Numeric::Integer(i)) => Some(Node::Number(Numeric::Float(i as f64))),
            Node::Str(s, _, _) => {
                if let Ok(f) = s.parse::<f64>() {
                    Some(Node::Number(Numeric::Float(f)))
                } else {
                    None
                }
            }
            _ => None,
        },
        "!!bool" | "!bool" | "tag:yaml.org,2002:bool" => match node {
            Node::Boolean(b) => Some(Node::Boolean(b)),
            Node::Str(s, _, _) => {
                let sl = s.trim().to_ascii_lowercase();
                match sl.as_str() {
                    "true" | "yes" | "on" => Some(Node::Boolean(true)),
                    "false" | "no" | "off" => Some(Node::Boolean(false)),
                    _ => None,
                }
            }
            Node::Number(Numeric::Integer(i)) => Some(Node::Boolean(i != 0)),
            Node::Number(Numeric::Float(f)) => Some(Node::Boolean(f != 0.0)),
            _ => None,
        },
        "!!set" | "!set" | "tag:yaml.org,2002:set" => match node {
            // Convert mapping with null values to a set (DRY)
            Node::Mapping(pairs) => {
                if let Some(set_items) =
                    crate::parser::utils::node_utils::pairs_to_set_items_if_all_none(&pairs)
                {
                    Some(Node::Set(set_items))
                } else {
                    None
                }
            }
            // Convert array to a set (remove duplicates)
            Node::Array(items) => {
                let mut unique_items = Vec::new();
                for item in items {
                    if !unique_items.contains(&item) {
                        unique_items.push(item);
                    }
                }
                Some(Node::Set(unique_items))
            }
            // FLATTEN: If already a set, flatten its items
            Node::Set(items) => {
                let mut flat = Vec::new();
                for item in items {
                    match item {
                        Node::Set(nested) => flat.extend(nested),
                        other => flat.push(other),
                    }
                }
                Some(Node::Set(flat))
            }
            // Single value becomes a set with one element
            _ => Some(Node::Set(vec![node])),
        },
        "!!omap" | "!omap" | "tag:yaml.org,2002:omap" => match node {
            // Ordered mapping - preserve as tagged array of single-key mappings
            Node::Array(items) => {
                // Validate that each item is a mapping with one key-value pair
                for item in &items {
                    match item {
                        Node::Mapping(pairs) if pairs.len() == 1 => {}
                        _ => return None, // Invalid omap format
                    }
                }
                Some(Node::Tagged(
                    Box::new(Node::Array(items)),
                    "tag:yaml.org,2002:omap".to_string(),
                ))
            }
            _ => None,
        },
        "!!pairs" | "!pairs" | "tag:yaml.org,2002:pairs" => match node {
            // Pairs - preserve as tagged array
            Node::Array(items) => Some(Node::Tagged(
                Box::new(Node::Array(items)),
                "tag:yaml.org,2002:pairs".to_string(),
            )),
            _ => None,
        },
        // Null coercion: map to Node::None
        "!!null" | "!null" | "tag:yaml.org,2002:null" => Some(Node::None),

        // Timestamp coercion: keep as plain string (tests expect string preservation)
        "!!timestamp" | "!timestamp" | "tag:yaml.org,2002:timestamp" => match node {
            Node::Str(s, _, _) => Some(Node::Str(s, QuoteType::Unquoted, BlockStyle::None)),
            // If numeric or boolean provided (unlikely), stringify
            Node::Number(Numeric::Integer(i)) => Some(Node::Str(
                i.to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            Node::Number(Numeric::Float(f)) => Some(Node::Str(
                f.to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            Node::Boolean(b) => Some(Node::Str(
                b.to_string(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            Node::None => Some(Node::Str(
                String::new(),
                QuoteType::Unquoted,
                BlockStyle::None,
            )),
            _ => None,
        },

        _ => None,
    }
}
