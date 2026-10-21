use super::run_example;

/// The expected output of both `struct/member_methods` variants, which differ
/// only in whether the calls are written with parentheses.
const MEMBER_METHODS_OUTPUT: &str = "42\n7\n[]\n[1, 2]\n2\n[\"[:x, 1]\", \"[:y, 2]\"]\n[2]\n[1]\n[1, 2]\n[1, 2, nil, nil]\n{x: 1}\n{x: 1, y: 2}\n{\"x\" => 10, \"y\" => 20}\nnil\nfalse\nempty::\nFrozenError\ntrue\nfalse\n";

#[test]
fn test_struct_member_methods_execution() {
    let output = run_example("struct/member_methods.rb");
    assert_eq!(output, MEMBER_METHODS_OUTPUT);
}

#[test]
fn test_struct_member_methods_no_parens_execution() {
    let output = run_example("struct/member_methods_no_parens.rb");
    assert_eq!(output, MEMBER_METHODS_OUTPUT);
}

/// The expected output of both `struct/shaped_by_initialize` variants.
const SHAPED_BY_INITIALIZE_OUTPUT: &str =
    "true\n[3, 4]\n[5, nil]\n\"struct size differs\"\n[\"held\", :tagged]\ntrue\n";

#[test]
fn test_struct_shaped_by_initialize_execution() {
    let output = run_example("struct/shaped_by_initialize.rb");
    assert_eq!(output, SHAPED_BY_INITIALIZE_OUTPUT);
}

#[test]
fn test_struct_shaped_by_initialize_parens_execution() {
    let output = run_example("struct/shaped_by_initialize_parens.rb");
    assert_eq!(output, SHAPED_BY_INITIALIZE_OUTPUT);
}

/// The expected output of both `struct/named_shapes` variants, which differ
/// only in whether the calls are written with parentheses.
const NAMED_SHAPES_OUTPUT: &str = concat!(
    "\"Struct::Waypoint\"\n",
    "\"summit\"\n",
    "[1, 2]\n",
    "[1, nil]\n",
    "[\"elefant\", 4]\n",
    "[\"mouse\", 4]\n",
    "duplicate member: foo\n",
    "NameError\n",
    "unknown keywords: missing\n"
);

#[test]
fn test_struct_named_shapes_execution() {
    let output = run_example("struct/named_shapes.rb");
    assert_eq!(output, NAMED_SHAPES_OUTPUT);
}

#[test]
fn test_struct_named_shapes_parens_execution() {
    let output = run_example("struct/named_shapes_parens.rb");
    assert_eq!(output, NAMED_SHAPES_OUTPUT);
}

#[test]
fn a_struct_member_reader_redefined_on_the_class_answers_in_place_of_the_member() {
    let output = run_example("struct/member_reader_redefined_with_define_method.rb");
    assert_eq!(output, ":baz\n:bar\n[:bar]\n");
}

#[test]
fn a_braceless_hash_after_positional_members_is_the_next_member() {
    let output = run_example("struct/braceless_hash_after_positionals.rb");
    assert_eq!(
        output,
        "[\"a\", \"b\"]\n[]\n{\"b\" => [\"a\"]}\nunknown keywords: b\n1\n"
    );
}

#[test]
fn a_subclass_of_an_anonymous_struct_inspects_as_a_struct() {
    let output = run_example("struct/subclass_of_an_anonymous_struct_inspect.rb");
    assert_eq!(
        output,
        "#<struct ResponseText code=nil, text=\"ready\">\n#<struct Response name=\"OK\", data=#<struct ResponseText code=nil, text=\"ready\">>\n#<struct ResponseText code=nil, text=\"\">\n#<struct Cell next_cell=#<struct Cell next_cell=#<struct Cell:...>>>\n"
    );
}

#[test]
fn struct_and_data_members_are_not_instance_variables() {
    let output = run_example("struct/members_are_not_instance_variables.rb");
    assert_eq!(output, "[]\n[:@label]\ntrue\n[]\n[:@tags]\ntrue\n");
}

#[test]
fn structs_compare_their_members_with_equals_through_super_and_nesting() {
    let output = run_example("struct/allocated_struct_equality.rb");
    assert_eq!(
        output,
        "true\ntrue\ntrue\ntrue\n[1, 2]\n[nil, nil]\ntrue\ntrue\n[\"invoice\"]\n[\"invoice\"]\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n"
    );
}

#[test]
fn keywords_beside_values_reach_a_struct_as_a_hash() {
    let output = run_example("struct/forwarding_initialize_through_super.rb");
    assert_eq!(
        output,
        "Fetched UIDs do not match (54321 != 22222)\n[7, nil]\n{\"UID\" => 5}\n{\"UID\" => 2}\n{\"UID\" => 2}\n{\"UID\" => 2}\n{uid: 2}\n{\"UID\" => 2}\n[[1], {\"UID\" => 2}]\n"
    );
}
