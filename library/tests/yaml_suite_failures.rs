yaml_suite_test!(test_zl4z, "---\r\na: 'b': c\r\n\n", true); // expected error
yaml_suite_test!(test_zvh3, "- key: value\r\n - item1\r\n\n", true); // expected error
yaml_suite_test!(test_zxt5, "[ \"key\"\r\n  :value ]\r\n\n", true); // expected error
yaml_suite_test!(test_td5n, "- item1\r\n- item2\r\ninvalid\r\n\n", true); // expected error
yaml_suite_test!(
    test_u44r,
    "map:\r\n  key1: \"quoted1\"\r\n   key2: \"bad indentation\"\r\n\n",
    true
); // expected error
yaml_suite_test!(test_u99r, "- !!str, xxx\r\n\n", true); // expected error
yaml_suite_test!(
    test_w9l4,
    "---\r\nblock scalar: |\r\n     \r\n  more spaces at the beginning\r\n  are invalid\r\n\n",
    true
); // expected error
yaml_suite_test!(test_zcz6, "a: b: c: d\r\n\n", true); // expected error
yaml_suite_test!(
    test_q4cl,
    "key1: \"quoted1\"\r\nkey2: \"quoted2\" trailing content\r\nkey3: \"quoted3\"\r\n\n",
    true
); // expected error
yaml_suite_test!(test_qb6e, "---\r\nquoted: \"a\r\nb\r\nc\"\r\n\n", true); // expected error
yaml_suite_test!(
    test_qlj7,
    "%TAG !prefix! tag:example.com,2011:\r\n--- !prefix!A\r\na: b\r\n--- !prefix!B\r\nc: d\r\n--- !prefix!C\r\ne: f\r\n\n",
    true
); // expected error
yaml_suite_test!(
    test_rzp5,
    "a: \"double\r\n  quotes\" # lala\r\nb: plain\r\n value  # lala\r\nc  : #lala\r\n  d\r\n? # lala\r\n - seq1\r\n: # lala\r\n - #lala\r\n  seq2\r\ne: &node # lala\r\n - x: y\r\nblock: > # lala\r\n  abcde\r\n\n",
    false
); // expected success
yaml_suite_test!(
    test_s98z,
    "empty block scalar: >\r\n \r\n  \r\n   \r\n # comment\r\n\n",
    true
); // expected error
yaml_suite_test!(
    test_ks4u,
    "---\r\n[\r\nsequence item\r\n]\r\ninvalid item\r\n\n",
    true
); // expected error
yaml_suite_test!(test_lhl4, "---\r\n!invalid{}tag scalar\r\n\n", true); // expected error
yaml_suite_test!(
    test_n4jp,
    "map:\r\n  key1: \"quoted1\"\r\n key2: \"bad indentation\"\r\n\n",
    true
); // expected error
yaml_suite_test!(test_p2eq, "---\r\n- { y: z }- invalid\r\n\n", true); // expected error
yaml_suite_test!(
    test_pw8x,
    "- &a\r\n- a\r\n-\r\n  &a : a\r\n  b: &b\r\n-\r\n  &c : &a\r\n-\r\n  ? &d\r\n-\r\n  ? &e\r\n  : &a\r\n\n",
    false
); // expected success
yaml_suite_test!(test_gdy7, "key: value\r\nthis is #not a: key\r\n\n", true); // expected error
yaml_suite_test!(test_gt5m, "- item1\r\n&node\r\n- item2\r\n\n", true); // expected error
yaml_suite_test!(test_h7j7, "key: &x\r\n!!map\r\n  a: b\r\n\n", true); // expected error
yaml_suite_test!(test_hu3p, "key:\r\n  word1 word2\r\n  no: key\r\n\n", true); // expected error
yaml_suite_test!(
    test_jy7z,
    "key1: \"quoted1\"\r\nkey2: \"quoted2\" no key: nor value\r\nkey3: \"quoted3\"\r\n\n",
    true
); // expected error
yaml_suite_test!(test_dmg6, "key:\r\n  ok: 1\r\n wrong: 2\r\n\n", true); // expected error
yaml_suite_test!(
    test_ehf6,
    "!!map {\r\n  k: !!seq\r\n  [ a, !!str b]\r\n}\r\n\n",
    false
); // expected success
yaml_suite_test!(test_ew3v, "k1: v1\r\n k2: v2\r\n\n", true); // expected error
yaml_suite_test!(
    test_f6mc,
    "---\r\na: >2\r\n   more indented\r\n  regular\r\nb: >2\r\n\r\n\r\n   more indented\r\n  regular\r\n\n",
    false
); // expected success
yaml_suite_test!(
    test_g9hc,
    "---\r\nseq:\r\n&anchor\r\n- a\r\n- b\r\n\n",
    true
); // expected error
yaml_suite_test!(test_bd7l, "- item1\r\n- item2\r\ninvalid: x\r\n\n", true); // expected error
yaml_suite_test!(test_bs4k, "word1  # comment\r\nword2\r\n\n", true); // expected error
yaml_suite_test!(test_c2sp, "[23\r\n]: 42\r\n\n", true); // expected error
yaml_suite_test!(test_dk4h, "---\r\n[ key\r\n  : value ]\r\n\n", true); // expected error
yaml_suite_test!(
    test_5llu_repeat,
    "block scalar: >\r\n \r\n  \r\n   \r\n invalid\r\n\n",
    true
); // expected error
yaml_suite_test!(
    test_6hb6,
    "  # Leading comment line spaces are\r\n   # neither content nor indentation.\r\n    \r\nNot indented:\r\n By one space: |\r\n    By four\r\n      spaces\r\n Flow style: [    # Leading spaces\r\n   By two,        # in flow style\r\n  Also by two,    # are neither\r\n  \tStill by two   # content nor\r\n    ]             # indentation.\r\n\n",
    false
); // expected success
yaml_suite_test!(
    test_6s55,
    "key:\r\n - bar\r\n - baz\r\n invalid\r\n\n",
    true
); // expected error
yaml_suite_test!(
    test_8kb6,
    "---\r\n- { single line, a: b}\r\n- { multi\r\n  line, a: b}\r\n\n",
    false
); // expected success
yaml_suite_test!(test_9c9n, "---\r\nflow: [a,\r\nb,\r\nc]\r\n\n", true); // expected error
/// Hardcoded tests for YAML test suite failures

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! yaml_suite_test {
        ($name:ident, $yaml:expr, $should_error:expr) => {
            #[test]
            fn $name() {
                use yaml_lib::{BufferSource, parse};
                let mut source = BufferSource::new($yaml.as_bytes());
                let result = parse(&mut source);
                if $should_error {
                    assert!(result.is_err(), "Expected error, got success");
                } else {
                    assert!(
                        result.is_ok(),
                        "Expected success, got error: {:?}",
                        result.err()
                    );
                }
            }
        };
    }
}

// Example: You must fill in the YAML content for each test below
// yaml_suite_test!(test_2CMS, "YAML_CONTENT_HERE", true); // expected error
// yaml_suite_test!(test_565N, "YAML_CONTENT_HERE", false); // expected success

// Repeat for all 52 failures, using the YAML from each test's in.yaml file
// ...
yaml_suite_test!(test_2cms, "this\r\n is\r\n  invalid: x\r\n\n", true); // expected error
yaml_suite_test!(
    test_565n,
    "canonical: !!binary \"\\r\n R0lGODlhDAAMAIQAAP//9/X17unp5WZmZgAAAOfn515eXvPz7Y6OjuDg4J+fn5\\r\n OTk6enp56enmlpaWNjY6Ojo4SEhP/++f/++f/++f/++f/++f/++f/++f/++f/+\\r\n +f/++f/++f/++f/++f/++SH+Dk1hZGUgd2l0aCBHSU1QACwAAAAADAAMAAAFLC\\r\n AgjoEwnuNAFOhpEMTRiggcz4BNJHrv/zCFcLiwMWYNG84BwwEeECcgggoBADs=\"\r\ngeneric: !!binary |\r\n R0lGODlhDAAMAIQAAP//9/X17unp5WZmZgAAAOfn515eXvPz7Y6OjuDg4J+fn5\r\n OTk6enp56enmlpaWNjY6Ojo4SEhP/++f/++f/++f/++f/++f/++f/++f/++f/+\r\n +f/++f/++f/++f/++f/++SH+Dk1hZGUgd2l0aCBHSU1QACwAAAAADAAMAAAFLC\r\n AgjoEwnuNAFOhpEMTRiggcz4BNJHrv/zCFcLiwMWYNG84BwwEeECcgggoBADs=\r\ndescription:\r\n The binary value above is a tiny arrow encoded as a gif image.\r\n\n",
    false
); // expected success
yaml_suite_test!(
    test_4hvu,
    "key:\r\n   - ok\r\n   - also ok\r\n  - wrong\r\n\n",
    true
); // expected error
yaml_suite_test!(
    test_5llu,
    "block scalar: >\r\n \r\n  \r\n   \r\n invalid\r\n\n",
    true
); // expected error
yaml_suite_test!(test_6ca3, "\t[\r\n\t]\r\n\n", false); // expected success
