//! Test for roundtrip property

use yaml_lib::testing::property::properties;
use yaml_lib::Node;
use yaml_lib::PropertyResult;

#[test]
fn test_roundtrip_simple() {
    let node = Node::from("test");
    let result = properties::roundtrip_preserves_structure(&node);
    if !matches!(result, PropertyResult::Pass | PropertyResult::Skip(_)) {
        eprintln!("Roundtrip failed: {:?}", result);
    }
    assert!(matches!(
        result,
        PropertyResult::Pass | PropertyResult::Skip(_)
    ));
}