
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
