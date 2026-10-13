// The example group DSL: groups, examples, aliases, pending and skipped
// examples, and the descriptions built from them.

use super::{expected, run_spec};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_groups_examples_and_pending_examples_document_as_rspec_documents_them() {
    assert_matches(
        "dsl/groups",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_groups_examples_and_pending_examples_report_as_rspec_reports_them() {
    assert_matches("dsl/groups", "progress", &[]);
}

// Rex's own names have no RSpec counterpart, so this expectation is Rex's.
#[test]
fn test_rex_names_the_framework_rspec_names() {
    assert_matches(
        "dsl/rex_names",
        "documentation",
        &["--format", "documentation"],
    );
}
