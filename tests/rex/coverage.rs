// `--coverage`: which lines and branches of the loaded files ran. RSpec has
// no coverage of its own, so these expectations are Rex's own.

use super::{expected, run_spec};

const FIXTURE: &str = "coverage/project";

#[test]
fn test_coverage_summarizes_lines_and_branches_per_file() {
    let run = run_spec(FIXTURE, &["--coverage"]);
    assert_eq!(run.transcript(), expected(FIXTURE, "expected_summary.txt"));
}

#[test]
fn test_a_parallel_run_reports_the_same_coverage() {
    let run = run_spec(FIXTURE, &["--coverage", "--jobs", "2"]);
    assert_eq!(run.transcript(), expected(FIXTURE, "expected_summary.txt"));
}

#[test]
fn test_coverage_below_the_threshold_fails_the_run() {
    let run = run_spec(FIXTURE, &["--coverage", "--coverage-threshold", "90"]);
    assert_eq!(
        run.transcript(),
        expected(FIXTURE, "expected_threshold.txt")
    );
}

#[test]
fn test_an_excluded_file_is_left_out_of_the_report() {
    let run = run_spec(
        FIXTURE,
        &["--coverage", "--coverage-exclude", "lib/fees.rb"],
    );
    assert_eq!(run.transcript(), expected(FIXTURE, "expected_exclude.txt"));
}

#[test]
fn test_lcov_and_json_files_are_written_to_the_coverage_directory() {
    let directory =
        std::env::temp_dir().join(format!("metorex-rex-coverage-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("create the coverage directory");
    let directory_text = directory.to_str().expect("utf-8 path");
    run_spec(
        FIXTURE,
        &[
            "--coverage",
            "--coverage-format",
            "lcov",
            "--coverage-format",
            "json",
            "--coverage-dir",
            directory_text,
        ],
    );
    let project = format!(
        "{}/{}/rex/{}/",
        env!("CARGO_MANIFEST_DIR"),
        crate::common::EXAMPLES_DIR,
        FIXTURE
    );
    let lcov = std::fs::read_to_string(directory.join("lcov.info"))
        .expect("read lcov.info")
        .replace(&project, "");
    let json =
        std::fs::read_to_string(directory.join("coverage.json")).expect("read coverage.json");
    std::fs::remove_dir_all(&directory).expect("remove the coverage directory");
    assert_eq!(
        (lcov, json),
        (
            expected(FIXTURE, "expected_lcov.info"),
            expected(FIXTURE, "expected_coverage.json"),
        )
    );
}
