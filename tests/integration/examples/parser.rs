
#[test]
fn a_percent_literal_interpolates_variables_written_after_a_hash() {
    let output = run_example("parser/percent_literal_with_caret_delimiter.rb");
    assert_eq!(output, "x=\"1\"\nplain \#@expanded_name\nx and 1\nx\nx\nx\n");
}

#[test]
fn a_bare_super_ends_at_the_colon_of_a_ternary() {
    let output = run_example("parser/super_in_a_ternary.rb");
    assert_eq!(output, "base\nnone\n");
}

#[test]
fn an_array_literal_nested_a_hundred_deep_parses() {
    let output = run_example("parser/deeply_nested_array_literal.rb");
    assert_eq!(output, "100\n[[1, {2 => 3}], [{a: [4]}]]\n");
}

#[test]
fn an_assignment_can_be_the_right_operand_of_an_operator() {
    let output = run_example("parser/assignment_as_right_operand.rb");
    assert_eq!(
        output,
        concat!(
            "[10, 100, 1000]\n1000\n3\n2\n:yes\n:yes\n-1\n4\n",
            "[\"first\"]\n\"first\"\n8\n4\n8\n3\n",
        )
    );
}

#[test]
fn a_bare_yield_ends_at_a_comment_and_a_bare_raise_at_the_end_of_an_interpolation() {
    let output = run_example("parser/bare_yield_and_raise_before_a_comment_or_brace.rb");
    assert_eq!(output, ":answered\n:rescued\nvalue=given\nRuntimeError\n");
}

#[test]
fn and_and_or_take_a_ternary_as_their_right_side_and_read_from_the_left() {
    let output = run_example("parser/or_keyword_before_a_ternary.rb");
    assert_eq!(output, "5\n12\nnil\n:ternary\nnil\nfalse\ntrue\nfalse\n");
}

#[test]
fn a_pattern_can_open_a_method_body_after_its_parameters() {
    let output = run_example("parser/regex_after_def_parameters.rb");
    assert_eq!(output, "/([ab])/n\n/([^c])/n\n5\n/xy/\n[\"a\", \"b\"]\n");
}

#[test]
fn a_parameter_default_can_start_on_the_next_line() {
    let output = run_example("parser/parameter_default_on_the_next_line.rb");
    assert_eq!(
        output,
        "[\"a\", \"reserved\", \"\"]\n[\"a\", \"b\", \"c\"]\n3\n"
    );
}

#[test]
fn not_takes_an_assignment_and_a_rescue_modifier_ends_at_a_comment() {
    let output = run_example("parser/not_assignment_and_rescue_return.rb");
    assert_eq!(output, "[:missing, nil]\n[:loaded, 3]\nnil\n:fallback\n");
}

#[test]
fn super_with_arguments_can_be_a_bare_argument() {
    let output = run_example("parser/super_as_a_bare_argument.rb");
    assert_eq!(output, "[:base, 1]\n[2]\n");
}
