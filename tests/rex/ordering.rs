// Ordering, seeds, fail-fast, dry runs, run persistence, and bisect.

use super::{EXAMPLES_DIR, expected, run_spec, run_spec_with_env};

fn assert_matches(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_the_defined_order_runs_examples_as_written() {
    assert_matches("ordering/order", "defined", &["-fd", "--order", "defined"]);
}

#[test]
fn test_a_seed_orders_examples_as_rspec_orders_them_for_that_seed() {
    assert_matches("ordering/order", "seed", &["-fd", "--seed", "1234"]);
}

#[test]
fn test_order_rand_with_a_seed_uses_that_seed() {
    assert_matches("ordering/order", "rand", &["-fd", "--order", "rand:42"]);
}

#[test]
fn test_order_random_with_a_seed_uses_that_seed() {
    assert_matches(
        "ordering/order",
        "random_seed",
        &["-fd", "--order", "random:5"],
    );
}

#[test]
fn test_a_configured_order_and_seed_shuffle_the_suite() {
    assert_matches("ordering/configured", "documentation", &["-fd"]);
}

#[test]
fn test_an_order_from_the_command_line_wins_over_the_configured_one() {
    assert_matches(
        "ordering/configured",
        "forced",
        &["-fd", "--order", "defined"],
    );
}

#[test]
fn test_registered_orderings_apply_by_metadata_and_globally() {
    assert_matches("ordering/custom", "documentation", &["-fd"]);
}

#[test]
fn test_a_seed_from_the_command_line_wins_over_a_registered_global_ordering() {
    assert_matches("ordering/custom", "forced", &["-fd", "--seed", "3"]);
}

#[test]
fn test_fail_fast_stops_after_the_first_failure() {
    assert_matches("ordering/failing", "fail_fast", &["--fail-fast"]);
}

#[test]
fn test_fail_fast_with_a_count_stops_after_that_many_failures() {
    assert_matches("ordering/failing", "fail_fast_two", &["--fail-fast=2"]);
}

#[test]
fn test_no_fail_fast_turns_an_earlier_fail_fast_off() {
    assert_matches(
        "ordering/failing",
        "no_fail_fast",
        &["--fail-fast", "--no-fail-fast"],
    );
}

#[test]
fn test_a_dry_run_reports_every_example_and_runs_no_hook_or_body() {
    assert_matches("ordering/failing", "dry_run", &["-fd", "--dry-run"]);
}

/// A directory of its own under the OS temp dir for one test's status file.
fn status_directory(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "metorex-rex-status-{}-{}",
        name,
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("create the status directory");
    directory
}

fn persistence_fixture(file: &str) -> String {
    format!(
        "{}/{}/rex/ordering/persistence/{}",
        env!("CARGO_MANIFEST_DIR"),
        EXAMPLES_DIR,
        file
    )
}

/// The status file with each cell trimmed, each run time written as
/// `N seconds`, and each divider as `-`, since run times change from run to
/// run and the column widths with them.
fn normalized_status_file(path: &std::path::Path) -> String {
    let text = std::fs::read_to_string(path).expect("read the status file");
    text.lines()
        .map(|line| {
            line.trim_end()
                .trim_end_matches('|')
                .split('|')
                .map(|cell| {
                    let cell = cell.trim();
                    if cell.ends_with(" seconds") || cell.ends_with(" second") {
                        "N seconds"
                    } else if !cell.is_empty() && cell.chars().all(|character| character == '-') {
                        "-"
                    } else {
                        cell
                    }
                })
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .map(|line| line + "\n")
        .collect()
}

/// Runs the persistence fixture with its status file starting as
/// `status_before.txt`, and answers the run and the status file it left.
fn run_with_status_file(name: &str, arguments: &[&str]) -> (String, String) {
    let directory = status_directory(name);
    let status_file = directory.join("examples.txt");
    std::fs::copy(persistence_fixture("status_before.txt"), &status_file)
        .expect("copy the status file");
    let run = run_spec_with_env(
        "ordering/persistence",
        arguments,
        &[("STATUS_FILE", status_file.to_str().expect("utf-8 path"))],
    );
    let status = normalized_status_file(&status_file);
    std::fs::remove_dir_all(&directory).expect("remove the status directory");
    (run.transcript(), status)
}

#[test]
fn test_the_first_run_writes_every_example_status() {
    let directory = status_directory("first");
    let status_file = directory.join("examples.txt");
    let run = run_spec_with_env(
        "ordering/persistence",
        &[],
        &[("STATUS_FILE", status_file.to_str().expect("utf-8 path"))],
    );
    let status = normalized_status_file(&status_file);
    std::fs::remove_dir_all(&directory).expect("remove the status directory");
    aggregate(&[
        (
            run.transcript(),
            expected("ordering/persistence", "expected_first.txt"),
        ),
        (
            status,
            expected("ordering/persistence", "expected_status_only_failures.txt"),
        ),
    ]);
}

#[test]
fn test_only_failures_runs_the_examples_that_failed_last_time() {
    let (transcript, status) = run_with_status_file("only", &["-fd", "--only-failures"]);
    aggregate(&[
        (
            transcript,
            expected("ordering/persistence", "expected_only_failures.txt"),
        ),
        (
            status,
            expected("ordering/persistence", "expected_status_only_failures.txt"),
        ),
    ]);
}

#[test]
fn test_next_failure_stops_at_the_first_example_that_failed_last_time() {
    let (transcript, status) = run_with_status_file("next", &["-fd", "--next-failure"]);
    aggregate(&[
        (
            transcript,
            expected("ordering/persistence", "expected_next_failure.txt"),
        ),
        (
            status,
            expected("ordering/persistence", "expected_status_next_failure.txt"),
        ),
    ]);
}

#[test]
fn test_only_failures_without_a_status_file_path_reports_how_to_set_one() {
    assert_matches(
        "ordering/persistence",
        "no_path_only_failures",
        &["--only-failures"],
    );
}

#[test]
fn test_bisect_narrows_an_order_dependent_failure_to_the_example_it_needs() {
    assert_matches("ordering/bisect", "bisect", &["--seed", "2", "--bisect"]);
}

#[test]
fn test_verbose_bisect_lists_each_run_and_each_set_of_examples() {
    assert_matches(
        "ordering/bisect",
        "verbose",
        &["--seed", "2", "--bisect=verbose"],
    );
}

#[test]
fn test_bisect_without_a_failure_reports_that_it_needs_one() {
    assert_matches(
        "ordering/bisect",
        "no_failures",
        &["--seed", "1", "--bisect"],
    );
}

/// Compares each pair and reports every mismatch together.
fn aggregate(pairs: &[(String, String)]) {
    let mismatches: Vec<String> = pairs
        .iter()
        .filter(|(actual, wanted)| actual != wanted)
        .map(|(actual, wanted)| format!("expected:\n{wanted}\nactual:\n{actual}"))
        .collect();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n---\n"));
}
