// Errors in hooks, backtrace filtering, deprecations, and interrupts.

use super::{expected, run_spec};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_errors_in_hooks_are_reported_as_rspec_reports_them() {
    assert_matches("errors/hooks", "documentation", &["-fd"]);
}

#[test]
fn test_a_backtrace_keeps_project_and_vendored_frames() {
    assert_matches("errors/backtrace", "default", &[]);
}

#[test]
fn test_filter_gems_from_backtrace_drops_a_gem_s_frames() {
    assert_matches(
        "errors/backtrace",
        "filter_gem",
        &["-r", "./spec/support/filter_gem.rb"],
    );
}

#[test]
fn test_an_exclusion_pattern_drops_matching_frames() {
    assert_matches(
        "errors/backtrace",
        "exclude_lib",
        &["-r", "./spec/support/exclude_lib.rb"],
    );
}

#[test]
fn test_an_inclusion_pattern_keeps_frames_an_exclusion_would_drop() {
    assert_matches(
        "errors/backtrace",
        "include_vendor",
        &["-r", "./spec/support/include_vendor.rb"],
    );
}

// RSpec's full backtrace lists its own frames and Rex's lists Rex's, so this
// checks for Rex's frames rather than comparing with RSpec.
#[test]
fn test_backtrace_keeps_the_frames_the_filter_drops() {
    let run = run_spec(
        "errors/backtrace",
        &["--backtrace", "spec/inventory_spec.rb:4"],
    );
    let transcript = run.transcript();
    assert!(
        transcript.contains("# ./spec/inventory_spec.rb:5:in 'block (2 levels) in <top (required)>'\n     # <metorex>/rex/core/example.rb:"),
        "{transcript}"
    );
}

#[test]
fn test_deprecations_are_listed_once_each_after_the_run() {
    assert_matches("errors/deprecations", "progress", &[]);
}

#[test]
fn test_raise_errors_for_deprecations_fails_the_example_that_uses_one() {
    assert_matches("errors/deprecations_raise", "progress", &[]);
}

#[test]
fn test_deprecation_out_writes_the_deprecations_to_a_file() {
    let directory =
        std::env::temp_dir().join(format!("metorex-rex-deprecations-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create the deprecation directory");
    let path = directory.join("deprecations.txt");
    let path_text = path.to_str().expect("utf-8 path");
    let run = run_spec("errors/deprecations", &["--deprecation-out", path_text]);
    let written = std::fs::read_to_string(&path).expect("read the deprecations file");
    std::fs::remove_dir_all(&directory).expect("remove the deprecation directory");
    assert_eq!(
        (
            run.transcript().replace(path_text, "DEPRECATIONS_FILE"),
            written
        ),
        (
            expected("errors/deprecations", "expected_out_file.txt"),
            expected("errors/deprecations", "expected_out_file_contents.txt"),
        )
    );
}

#[test]
fn test_an_interrupt_finishes_the_running_example_and_prints_the_summary() {
    assert_matches("errors/interrupt", "documentation", &["-fd"]);
}
