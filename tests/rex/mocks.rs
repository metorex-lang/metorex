// Doubles, stubs, message expectations, spies, and verifying doubles.

use super::{expected, run_spec};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_mock_failures_read_as_rspec_writes_them() {
    assert_matches(
        "mocks/failures",
        "documentation",
        &["--format", "documentation"],
    );
}

// The second examples of the class method, the instance method, and the
// constant check that the first example's stub was taken down.
#[test]
fn test_stubs_expectations_spies_and_verifying_doubles_pass_and_do_not_leak() {
    assert_matches(
        "mocks/passing",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_partial_doubles_verify_methods_and_arguments() {
    assert_matches(
        "mocks/verified_partials",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_should_receive_syntax_when_a_suite_turns_it_on() {
    assert_matches(
        "mocks/should_syntax",
        "documentation",
        &["--format", "documentation"],
    );
}

// RSpec reports this failure with its own internals as the backtrace. Rex
// points at the `expect_any_instance_of` line, so the expectation is Rex's.
#[test]
fn test_an_unmet_any_instance_expectation_points_at_its_line() {
    assert_matches(
        "mocks/any_instance",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_the_mocks_configuration_starts_where_rspecs_starts() {
    assert_matches(
        "mocks/default_configuration",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_any_instance_yields_the_receiver_and_stubs_message_chains() {
    assert_matches(
        "mocks/any_instance_features",
        "documentation",
        &["--format", "documentation", "--order", "defined"],
    );
}

#[test]
fn test_constants_are_stubbed_hidden_and_put_back_as_rspec_does() {
    assert_matches(
        "mocks/constants",
        "documentation",
        &["--format", "documentation", "--order", "defined"],
    );
}
