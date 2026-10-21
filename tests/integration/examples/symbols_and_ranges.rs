use super::run_example;

/// The expected output of both `symbols_and_ranges/names_and_walks` variants,
/// which differ only in whether the calls are written with parentheses.
const NAMES_AND_WALKS_OUTPUT: &str = ":GLARK\n:glark\n:Hello_world\n:aBc\n:b\n:AA\n\"ruby\"\n\"ruby\"\n:ruby\n4\ntrue\n-1\n:a2b\n\"Symbol\"\n\"àBc\"\n:àbc\n\"Über\"\n4\n\"a b\"\n\"HI\"\n\"A\"\n\"あ\"\n\"ba\"\n\"aaa\"\n\"b0\"\n\"2.0\"\n\"Hello world\"\n\"aBc\"\n1234.5\n1234.5\n0.0\n5.0\n[1, 2, 3, 4, 5]\n[\"A\", \"B\", \"C\", \"D\"]\n[]\n[\"ax\", \"ay\", \"az\", \"ba\", \"bb\", \"bc\", \"bd\"]\n[:A, :B, :C, :D]\n\"1...4\"\n\"1...4\"\n\"1..\"\n\"..3\"\ntrue\ntrue\nfalse\n3\nInfinity\n[\"a\", \"b\", \"c\"]\ncannot convert endless range to an array\n[1, 2, 3]\n[1, 2]\n";

#[test]
fn test_symbol_names_and_walks_execution() {
    let output = run_example("symbols_and_ranges/names_and_walks.rb");
    assert_eq!(output, NAMES_AND_WALKS_OUTPUT);
}

#[test]
fn test_symbol_names_and_walks_no_parens_execution() {
    let output = run_example("symbols_and_ranges/names_and_walks_no_parens.rb");
    assert_eq!(output, NAMES_AND_WALKS_OUTPUT);
}

/// The expected output of both `symbols_and_ranges/binary_search` variants,
/// which differ only in whether the calls are written with parentheses.
const BINARY_SEARCH_OUTPUT: &str = concat!(
    "4\nnil\n3\nnil\n3.0\n7\n2\nEnumerator\n",
    "can't do binary search for String\n",
    "wrong argument type Object (must be numeric, true, false or nil)\n"
);

#[test]
fn test_symbols_and_ranges_binary_search_execution() {
    let output = run_example("symbols_and_ranges/binary_search.rb");
    assert_eq!(output, BINARY_SEARCH_OUTPUT);
}

#[test]
fn test_symbols_and_ranges_binary_search_parens_execution() {
    let output = run_example("symbols_and_ranges/binary_search_parens.rb");
    assert_eq!(output, BINARY_SEARCH_OUTPUT);
}

/// The expected output of both `symbols_and_ranges/reverse_walks` variants,
/// which differ only in whether the calls are written with parentheses.
const REVERSE_WALKS_OUTPUT: &str = "[3, 2, 1]\n[2, 1]\n[\"D\", \"C\", \"B\", \"A\"]\n[:D, :C, :B, :A]\n[5, 4, 3]\n3\nnil\nInfinity\n\"can't iterate from NilClass\"\n\"can't iterate from Float\"\n0\nnil\n";

#[test]
fn test_symbols_and_ranges_reverse_walks_execution() {
    let output = run_example("symbols_and_ranges/reverse_walks.rb");
    assert_eq!(output, REVERSE_WALKS_OUTPUT);
}

#[test]
fn test_symbols_and_ranges_reverse_walks_parens_execution() {
    let output = run_example("symbols_and_ranges/reverse_walks_parens.rb");
    assert_eq!(output, REVERSE_WALKS_OUTPUT);
}

/// The expected output of both `symbols_and_ranges/range_identity` variants,
/// which differ only in whether the calls are written with parentheses.
const RANGE_IDENTITY_OUTPUT: &str =
    "true\nfalse\ntrue\ntrue\nfalse\ntrue\n\"a\"\n\"z\"\ntrue\nfalse\n";

#[test]
fn test_symbols_and_ranges_range_identity_execution() {
    let output = run_example("symbols_and_ranges/range_identity.rb");
    assert_eq!(output, RANGE_IDENTITY_OUTPUT);
}

#[test]
fn test_symbols_and_ranges_range_identity_parens_execution() {
    let output = run_example("symbols_and_ranges/range_identity_parens.rb");
    assert_eq!(output, RANGE_IDENTITY_OUTPUT);
}

/// The expected output of both `symbols_and_ranges/allocated_range` variants,
/// which differ only in whether the calls are written with parentheses.
const ALLOCATED_RANGE_OUTPUT: &str = "3\n7\ntrue\nbad value for range\nFrozenError\n";

#[test]
fn test_symbols_and_ranges_allocated_range_execution() {
    let output = run_example("symbols_and_ranges/allocated_range.rb");
    assert_eq!(output, ALLOCATED_RANGE_OUTPUT);
}

#[test]
fn test_symbols_and_ranges_allocated_range_parens_execution() {
    let output = run_example("symbols_and_ranges/allocated_range_parens.rb");
    assert_eq!(output, ALLOCATED_RANGE_OUTPUT);
}

/// The expected output of both `symbols_and_ranges/stepped_walks` variants,
/// which differ only in whether the calls are written with parentheses.
const STEPPED_WALKS_OUTPUT: &str = concat!(
    "[1, 4, 7, 10]\n",
    "Enumerator::ArithmeticSequence\n",
    "[1.0, 19.2, 37.4, 55.599999999999994]\n",
    "4\n",
    "[\"A\", \"C\", \"E\", \"G\"]\n",
    "[\"A\", \"B\", \"C\", \"D\", \"E\"]\n",
    "[:a, :c, :e]\n",
    "[\"A\"]\n",
    "[\"A\", \"AA\", \"AAA\"]\n",
    "Enumerator\n",
    "nil\n",
    "[]\n",
    "no implicit conversion of Array into String\n",
    "#step for non-numeric beginless ranges is meaningless\n",
    "step is required for non-numeric ranges\n",
    "[1, 3, 5]\n",
    "Enumerator\n",
    "Object can't be coerced into Integer\n"
);

#[test]
fn test_symbols_and_ranges_stepped_walks_execution() {
    let output = run_example("symbols_and_ranges/stepped_walks.rb");
    assert_eq!(output, STEPPED_WALKS_OUTPUT);
}

#[test]
fn test_symbols_and_ranges_stepped_walks_parens_execution() {
    let output = run_example("symbols_and_ranges/stepped_walks_parens.rb");
    assert_eq!(output, STEPPED_WALKS_OUTPUT);
}

/// The expected output of both `symbols_and_ranges/literal_range_ends` variants.
const LITERAL_RANGE_ENDS_OUTPUT: &str = "[true, true, true]\n[false, false]\n";

#[test]
fn test_symbols_and_ranges_literal_range_ends_execution() {
    let output = run_example("symbols_and_ranges/literal_range_ends.rb");
    assert_eq!(output, LITERAL_RANGE_ENDS_OUTPUT);
}

#[test]
fn test_symbols_and_ranges_literal_range_ends_no_parens_execution() {
    let output = run_example("symbols_and_ranges/literal_range_ends_no_parens.rb");
    assert_eq!(output, LITERAL_RANGE_ENDS_OUTPUT);
}

/// The expected output of both `symbols_and_ranges/range_cover_and_include` variants.
const RANGE_COVER_AND_INCLUDE_OUTPUT: &str = "true\nfalse\ntrue\nfalse\nfalse\ntrue\nfalse\nfalse\ntrue\nfalse\ntrue\nfalse\nfalse\ntrue\n\"can't convert Object to String (Object#to_str gives Integer)\"\ntrue\n[:aa, :ab, :ac]\ntrue\nfalse\n\"cannot determine inclusion in beginless/endless ranges\"\n\"cannot determine inclusion in beginless/endless ranges\"\n\"#step iteration for beginless ranges is meaningless\"\n\"#step for non-numeric beginless ranges is meaningless\"\n";

#[test]
fn test_symbols_and_ranges_range_cover_and_include_execution() {
    let output = run_example("symbols_and_ranges/range_cover_and_include.rb");
    assert_eq!(output, RANGE_COVER_AND_INCLUDE_OUTPUT);
}

#[test]
fn test_symbols_and_ranges_range_cover_and_include_no_parens_execution() {
    let output = run_example("symbols_and_ranges/range_cover_and_include_no_parens.rb");
    assert_eq!(output, RANGE_COVER_AND_INCLUDE_OUTPUT);
}

/// The expected output of both `symbols_and_ranges/ascii_symbols_one_object`
/// variants.
const ASCII_SYMBOLS_ONE_OBJECT_OUTPUT: &str =
    "true\ntrue\ntrue\n[#<Encoding:US-ASCII>, #<Encoding:US-ASCII>, #<Encoding:US-ASCII>]\ntrue\n";

#[test]
fn test_symbols_and_ranges_ascii_symbols_one_object_execution() {
    let output = run_example("symbols_and_ranges/ascii_symbols_one_object.rb");
    assert_eq!(output, ASCII_SYMBOLS_ONE_OBJECT_OUTPUT);
}

#[test]
fn test_symbols_and_ranges_ascii_symbols_one_object_no_parens_execution() {
    let output = run_example("symbols_and_ranges/ascii_symbols_one_object_no_parens.rb");
    assert_eq!(output, ASCII_SYMBOLS_ONE_OBJECT_OUTPUT);
}

#[test]
fn rational_ranges_step_and_write_their_ends_with_to_s() {
    let output = run_example("symbols_and_ranges/rational_range_steps.rb");
    assert_eq!(
        output,
        "11\n[(0/1), (1/10), (1/5)]\n(1/1)\n[(0/1), (1/10), (1/5), (3/10), (2/5), (1/2)]\n(1/2)\n(2/5)\n((0/1..1/1).%((1/10)))\n[(0/1), (1/4), (1/2), (3/4), (1/1)]\n[[(0/1), (1/10)], [(1/10), (1/5)]]\n((1/1..).%(2))\n((1.0...3.5).step(0.5))\n(1/1.step(3, (1/2)))\n(1.step(10, (3/1)))\n\"1/1..2/1\"\n\"1..\"\n\"1/1..\"\n\"..2/1\"\n\"1/1..2/1\"\n\"1.5..2\"\n"
    );
}
