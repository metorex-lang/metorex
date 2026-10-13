// Code samples published on metorex.org.

use super::run_example;

#[test]
fn test_site_home_snippet_execution() {
    let expected = "#<struct Point x=4, y=6>\ny leads by 2\n";
    let output = run_example("site/home.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_install_hello_execution() {
    let expected = "Hello, world!\n[2, 4, 6]\n";
    let output = run_example("site/install_hello.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_meta_method_tree_execution() {
    let expected = "total takes 2 arguments\nBinaryOp +\n  BinaryOp *\n    Identifier price\n    Identifier quantity\n  IntLiteral 5\n";
    let output = run_example("site/meta.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_meta_parse_execution() {
    let expected = "\"+\"\n{\"type\" => \"BinaryOp\", \"op\" => \"*\", \"left\" => {\"type\" => \"IntLiteral\", \"value\" => 2}, \"right\" => {\"type\" => \"IntLiteral\", \"value\" => 3}}\n";
    let output = run_example("site/meta_parse.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_meta_block_execution() {
    let expected = "1\n{\"type\" => \"BinaryOp\", \"op\" => \"*\", \"left\" => {\"type\" => \"Identifier\", \"name\" => \"order\"}, \"right\" => {\"type\" => \"IntLiteral\", \"value\" => 2}}\n42\n";
    let output = run_example("site/meta_block.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_classes_execution() {
    let expected = "Ada: $1575.0\ntrue\n[SavingsAccount, Account, Object]\n";
    let output = run_example("site/language_classes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_blocks_execution() {
    let expected = "1 -> 4\n4 -> 9\n[[\"a\", \"b\"], [\"b\", \"c\"]]\n[1, 4, 9]\n17\n[\"APPLE\", \"BANANA\"]\n";
    let output = run_example("site/language_blocks.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_struct_execution() {
    let expected = "5.0\n[3, 4]\ntrue\n#<data Coordinate latitude=36.16, longitude=-86.78>\n#<data Coordinate latitude=40.71, longitude=-86.78>\n";
    let output = run_example("site/language_struct.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_enumerable_execution() {
    let expected = "464\n\"monitor\"\n[2, 1]\n{true => [\"keyboard\", \"cable\"], false => [\"monitor\"]}\n[4, 16, 36]\n{\"a\" => 0, \"b\" => 1, \"c\" => 2}\n";
    let output = run_example("site/language_enumerable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_pattern_matching_execution() {
    let expected = "no users\nAda and 1 more\nclient error 404\nserver error 503\n5432\n";
    let output = run_example("site/language_pattern_matching.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_exceptions_execution() {
    let expected =
        "payment of 120 declined (attempt 1)\ncharged 90\ndone after 2 attempts\n8080\nnil\n";
    let output = run_example("site/language_exceptions.rb");
    assert_eq!(output, expected);
}
