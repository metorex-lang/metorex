// Formatters: the built-in ones, color, profiling, several outputs, and a
// user's own formatter class.

use super::{expected, run_spec};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_progress_prints_a_character_per_example() {
    assert_matches("formatters/outcomes", "progress", &[]);
}

#[test]
fn test_documentation_prints_each_example_indented_by_its_group() {
    assert_matches("formatters/outcomes", "documentation", &["-fd"]);
}

#[test]
fn test_json_writes_the_run_as_rspec_shapes_it() {
    assert_matches("formatters/outcomes", "json", &["--format", "json"]);
}

#[test]
fn test_html_writes_the_run_as_rspec_lays_it_out() {
    assert_matches("formatters/outcomes", "html", &["--format", "html"]);
}

#[test]
fn test_failures_lists_each_failure_location_and_description() {
    assert_matches("formatters/outcomes", "failures", &["--format", "failures"]);
}

#[test]
fn test_force_color_colors_output_that_is_not_a_terminal() {
    assert_matches("formatters/outcomes", "force_color", &["--force-color"]);
}

#[test]
fn test_documentation_with_force_color_colors_each_example_by_outcome() {
    assert_matches(
        "formatters/outcomes",
        "documentation_color",
        &["-fd", "--force-color"],
    );
}

#[test]
fn test_color_leaves_output_that_is_not_a_terminal_plain() {
    assert_matches("formatters/outcomes", "color", &["--color"]);
}

#[test]
fn test_force_color_and_no_color_together_end_the_run() {
    assert_matches(
        "formatters/outcomes",
        "no_color",
        &["--force-color", "--no-color"],
    );
}

#[test]
fn test_two_formats_both_write_to_the_output() {
    assert_matches("formatters/outcomes", "two_formats", &["-fp", "-fd"]);
}

#[test]
fn test_out_sends_the_format_before_it_to_a_file() {
    let directory = std::env::temp_dir().join(format!("metorex-rex-out-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create the output directory");
    let json_path = directory.join("report.json");
    let json_argument = json_path.to_str().expect("utf-8 path");
    let run = run_spec(
        "formatters/outcomes",
        &[
            "--format",
            "progress",
            "--format",
            "json",
            "--out",
            json_argument,
        ],
    );
    let json = std::fs::read_to_string(&json_path).expect("read the JSON report");
    std::fs::remove_dir_all(&directory).expect("remove the output directory");
    assert_eq!(
        (run.transcript(), crate::without_json_timings(&json) + "\n"),
        (
            expected("formatters/outcomes", "expected_out.txt"),
            expected("formatters/outcomes", "expected_out_file.txt"),
        )
    );
}

#[test]
fn test_profile_lists_the_slowest_examples() {
    assert_matches("formatters/profile", "profile", &["--profile"]);
}

#[test]
fn test_profile_with_json_adds_the_profile_to_the_document() {
    assert_matches(
        "formatters/profile",
        "profile_json",
        &["--profile", "--format", "json"],
    );
}

// The fixture replaces Rex's clock, so the times are fixed and the ranking
// is Rex's own expectation rather than RSpec's output.
#[test]
fn test_profile_ranks_examples_and_groups_by_time_taken() {
    let run = run_spec("formatters/profile_ranking", &["--profile", "2"]);
    assert_eq!(
        format!(
            "{}\nexit {}\n",
            run.output_with_times.trim_end_matches('\n'),
            run.status
        ),
        expected("formatters/profile_ranking", "expected_profile.txt")
    );
}

#[test]
fn test_a_formatter_added_in_the_configuration_hears_the_events_it_registered() {
    assert_matches("formatters/custom", "configured", &[]);
}

#[test]
fn test_a_formatter_named_on_the_command_line_is_loaded_from_its_file() {
    assert_matches(
        "formatters/custom",
        "by_name",
        &["--format", "SilentFormatter"],
    );
}

#[test]
fn test_a_formatter_instance_writes_to_the_string_io_it_was_given() {
    assert_matches("formatters/captured", "documentation", &[]);
}
