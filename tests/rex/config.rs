// The configuration surface: settings, aliases, callbacks, and the
// spec_helper RSpec generates.

use super::{expected, run_spec, run_spec_with_env};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_aliases_hooks_settings_and_callbacks_behave_as_in_rspec() {
    assert_matches("config/surface", "documentation", &["-fd"]);
}

#[test]
fn test_every_rspec_setting_has_a_reader_and_a_writer() {
    assert_matches("config/accessors", "progress", &[]);
}

#[test]
fn test_the_spec_helper_rspec_generates_runs_as_under_rspec() {
    let directory =
        std::env::temp_dir().join(format!("metorex-rex-generated-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create the status directory");
    let status_file = directory.join("examples.txt");
    let run = run_spec_with_env(
        "config/generated",
        &["--seed", "7"],
        &[("STATUS_FILE", status_file.to_str().expect("utf-8 path"))],
    );
    std::fs::remove_dir_all(&directory).expect("remove the status directory");
    assert_eq!(
        run.transcript(),
        expected("config/generated", "expected_seeded.txt")
    );
}

#[test]
fn test_a_failure_prints_its_cause() {
    assert_matches("config/settings", "default", &[]);
}

#[test]
fn test_full_cause_backtrace_prints_every_frame_of_the_cause() {
    assert_matches(
        "config/settings",
        "full_cause",
        &["-r", "./spec/support/full_cause.rb"],
    );
}

#[test]
fn test_pending_failure_output_no_backtrace_leaves_out_the_backtrace() {
    assert_matches(
        "config/settings",
        "pending_no_backtrace",
        &["-r", "./spec/support/pending_no_backtrace.rb"],
    );
}

#[test]
fn test_pending_failure_output_skip_leaves_out_the_pending_list() {
    assert_matches(
        "config/settings",
        "pending_skip",
        &["-r", "./spec/support/pending_skip.rb"],
    );
}

#[test]
fn test_silenced_filter_announcements_and_a_default_formatter_apply() {
    assert_matches(
        "config/settings",
        "silence",
        &["-r", "./spec/support/silence.rb", "--tag", "~skip"],
    );
}

#[test]
fn test_fail_if_no_examples_fails_a_run_with_none() {
    assert_matches("config/no_examples", "filtered", &["--tag", "~slow"]);
}

#[test]
fn test_raise_on_warning_turns_a_warning_into_a_failure() {
    assert_matches("config/warnings", "progress", &[]);
}

// RSpec has no setting to give up its own constant, so this expectation is
// Rex's own.
#[test]
fn test_rspec_constant_false_leaves_rspec_for_the_suite_to_define() {
    assert_matches("config/rspec_constant", "documentation", &["-fd"]);
}

/// What the fast paths setting reads under `arguments`.
fn fast_paths_printed(arguments: &[&str]) -> String {
    let run = run_spec("config/fast_paths", arguments);
    run.output
        .lines()
        .find(|line| line.starts_with("fast paths: "))
        .unwrap_or_default()
        .to_string()
}

#[test]
fn test_fast_paths_are_on_by_default() {
    assert_eq!(fast_paths_printed(&[]), "fast paths: true");
}

#[test]
fn test_no_fast_paths_turns_them_off() {
    assert_eq!(
        fast_paths_printed(&["--no-fast-paths"]),
        "fast paths: false"
    );
}

#[test]
fn test_the_configuration_turns_fast_paths_off() {
    assert_eq!(
        fast_paths_printed(&["--require", "./spec/fast_paths_off.rb"]),
        "fast paths: false"
    );
}
