// Metadata, filtering by it, and shared groups.

use super::{expected, run_spec};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_shared_examples_and_contexts_run_as_rspec_runs_them() {
    assert_matches(
        "metadata/shared",
        "documentation",
        &["-fd", "spec/shared_spec.rb"],
    );
}

#[test]
fn test_a_missing_shared_group_reports_the_load_error_rspec_reports() {
    assert_matches(
        "metadata/shared",
        "missing",
        &["spec/missing_shared_spec.rb"],
    );
}

#[test]
fn test_shared_groups_named_by_a_symbol_or_a_module_are_found() {
    assert_matches("metadata/shared_names", "documentation", &["-fd"]);
}

#[test]
fn test_apply_to_host_groups_gives_a_shared_context_metadata_to_its_host() {
    assert_matches("metadata/host_metadata", "documentation", &["-fd"]);
}

#[test]
fn test_a_tag_runs_only_the_examples_carrying_it() {
    assert_matches("metadata/filtering", "tag", &["-fd", "--tag", "fast"]);
}

#[test]
fn test_a_negated_tag_excludes_the_examples_carrying_it() {
    assert_matches(
        "metadata/filtering",
        "exclude_tag",
        &["-fd", "--tag", "~slow"],
    );
}

#[test]
fn test_a_tag_with_a_value_matches_that_value() {
    assert_matches(
        "metadata/filtering",
        "tag_value",
        &["-fd", "--tag", "priority:high"],
    );
}

#[test]
fn test_a_tag_with_a_number_matches_that_number() {
    assert_matches(
        "metadata/filtering",
        "tag_number",
        &["-fd", "--tag", "version:3"],
    );
}

#[test]
fn test_two_tags_run_the_examples_carrying_either() {
    assert_matches(
        "metadata/filtering",
        "two_tags",
        &["-fd", "--tag", "fast", "--tag", "slow"],
    );
}

#[test]
fn test_focused_examples_and_groups_run_alone() {
    assert_matches("metadata/filtering/focus", "documentation", &["-fd"]);
}

#[test]
fn test_configured_exclusion_filters_with_a_proc_skip_matching_examples() {
    assert_matches("metadata/filtering/configured", "documentation", &["-fd"]);
}
