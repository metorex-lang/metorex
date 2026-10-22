
#[test]
fn a_percent_literal_interpolates_variables_written_after_a_hash() {
    let output = run_example("parser/percent_literal_with_caret_delimiter.rb");
    assert_eq!(output, "x=\"1\"\nplain \#@expanded_name\nx and 1\nx\nx\nx\n");
}
