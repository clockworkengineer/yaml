use yaml_lib::test_helpers::assert_parse_error;

#[test]
fn inspect_td5n_node_shape() {
    let input = b"- item1\n- item2\ninvalid\n";
    assert_parse_error(input, "Unexpected plain scalar after top-level sequence");
}
