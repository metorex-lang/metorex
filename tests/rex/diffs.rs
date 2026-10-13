// Diffs, the lines a failure quotes, and the expression it names.

use super::{expected, run_spec};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_diffs_pair_lines_and_split_hunks_as_rspec_does() {
    assert_matches("diffs/output", "progress", &[]);
}

#[test]
fn test_a_colored_diff_colors_each_line_as_rspec_does() {
    assert_matches("diffs/output", "color", &["--force-color"]);
}

#[test]
fn test_a_failure_quotes_every_line_of_the_failing_expression() {
    assert_matches("diffs/expressions", "progress", &[]);
}

// RSpec has neither setting the fixture turns on, so the expectation is
// Rex's own.
#[test]
fn test_source_expressions_name_the_value_and_context_lines_surround_it() {
    assert_matches("diffs/named", "progress", &[]);
}

// RSpec has no diff line limit and no `--no-diff`, so these expectations
// are Rex's own.
#[test]
fn test_a_diff_past_the_line_limit_is_left_out() {
    assert_matches("diffs/limits", "progress", &[]);
}

#[test]
fn test_no_diff_leaves_every_diff_out() {
    assert_matches("diffs/limits", "no_diff", &["--no-diff"]);
}
