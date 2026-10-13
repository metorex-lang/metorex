// The runner: discovery, option files, filters, formatters, and the exit
// status, each compared with what RSpec 3.13 prints for the same fixture.

use super::{expected, run_spec, run_spec_with_env};

fn assert_matches_rspec(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_spec_runs_the_spec_directory_with_the_progress_formatter() {
    assert_matches_rspec("runner/basic", "progress", &[]);
}

#[test]
fn test_spec_prints_the_documentation_formatter() {
    assert_matches_rspec(
        "runner/basic",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_spec_reads_the_format_and_requires_from_dot_rspec() {
    assert_matches_rspec("runner/options", "default", &[]);
}

#[test]
fn test_spec_runs_only_the_files_under_a_named_directory() {
    assert_matches_rspec("runner/options", "directory", &["spec/models"]);
}

#[test]
fn test_spec_runs_the_example_at_a_file_and_line() {
    assert_matches_rspec("runner/options", "location", &["spec/money_spec.rb:7"]);
}

#[test]
fn test_spec_runs_every_example_in_the_group_at_a_file_and_line() {
    assert_matches_rspec(
        "runner/options",
        "location_group",
        &["spec/money_spec.rb:10"],
    );
}

#[test]
fn test_spec_runs_the_examples_at_several_lines_of_one_file() {
    assert_matches_rspec(
        "runner/options",
        "two_locations",
        &["spec/money_spec.rb:3:12"],
    );
}

#[test]
fn test_spec_runs_the_examples_whose_names_include_the_example_text() {
    assert_matches_rspec("runner/options", "example", &["-e", "formats"]);
}

#[test]
fn test_spec_runs_the_examples_whose_names_match_a_pattern() {
    assert_matches_rspec("runner/options", "example_matches", &["-E", "zero$"]);
}

#[test]
fn test_spec_loads_the_files_a_pattern_names() {
    assert_matches_rspec(
        "runner/options",
        "pattern",
        &["--pattern", "spec/**/*_check.rb"],
    );
}

#[test]
fn test_spec_leaves_out_the_files_an_exclude_pattern_names() {
    assert_matches_rspec(
        "runner/options",
        "exclude_pattern",
        &["--exclude-pattern", "**/models/*"],
    );
}

#[test]
fn test_a_format_on_the_command_line_replaces_the_one_in_dot_rspec() {
    assert_matches_rspec("runner/options", "progress", &["-f", "p"]);
}

#[test]
fn test_spec_exits_with_the_failure_exit_code_given() {
    assert_matches_rspec(
        "runner/options",
        "failure_exit_code",
        &["--failure-exit-code", "3"],
    );
}

#[test]
fn test_spec_adds_load_paths_and_requires_files() {
    assert_matches_rspec(
        "runner/options",
        "require",
        &["-I", "lib", "-r", "account", "spec/models"],
    );
}

#[test]
fn test_spec_opts_override_the_option_files() {
    let run = run_spec_with_env("runner/options", &[], &[("SPEC_OPTS", "--format progress")]);
    assert_eq!(
        run.transcript(),
        expected("runner/options", "expected_spec_opts.txt")
    );
}

#[test]
fn test_spec_exits_with_zero_when_every_example_passes() {
    assert_matches_rspec("runner/options", "passing", &["spec/money_spec.rb"]);
}

#[test]
fn test_spec_runs_the_default_path_given() {
    assert_matches_rspec(
        "runner/default_path",
        "default",
        &["--default-path", "checks"],
    );
}

#[test]
fn test_spec_reports_a_file_that_fails_to_load_and_exits_with_one() {
    assert_matches_rspec("errors/load_error", "progress", &[]);
}

#[test]
fn test_dot_rspec_local_overrides_dot_rspec() {
    assert_matches_rspec("runner/local_options", "default", &[]);
}

// RSpec loads spec_helper.rb only when an option file or a spec requires it.
// Rex loads it first whenever it is there, so this expectation is Rex's own.
#[test]
fn test_spec_loads_spec_helper_before_the_spec_files() {
    assert_matches_rspec("runner/spec_helper", "default", &[]);
}
