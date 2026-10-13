// The built-in matchers, checked against RSpec's messages for every failure
// and against RSpec's results when every expectation holds.

use super::{expected, run_spec};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_every_matcher_fails_with_rspecs_message() {
    assert_matches(
        "matchers/failures",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_every_matcher_passes_where_rspecs_passes() {
    assert_matches(
        "matchers/passing",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_output_from_any_process_and_aggregate_failures_metadata() {
    assert_matches(
        "matchers/process_output",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_include_counts_match_rspec() {
    assert_matches("matchers/include_counts", "documentation", &["-fd"]);
}
