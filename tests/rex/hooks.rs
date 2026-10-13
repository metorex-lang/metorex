// Hooks, helper modules, and memoized values.

use super::{expected, run_spec};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

// The fixture records the order every hook ran in and asserts it, so the
// run passing proves the order matches RSpec's.
#[test]
fn test_hooks_lets_subjects_and_helper_modules_run_as_rspec_runs_them() {
    assert_matches(
        "hooks/order",
        "documentation",
        &["--format", "documentation"],
    );
}

// RSpec's message names the example's address, which changes from run to
// run, and has no list of helpers, so this expectation is Rex's own.
#[test]
fn test_an_undefined_name_lists_the_lets_declared_nearby() {
    assert_matches("hooks/undefined_let", "progress", &[]);
}
