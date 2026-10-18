// The JUnit report scripts/spec_junit.rb writes of a run of
// scripts/run_ruby_spec.sh, read under the Ruby that runs mspec.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A directory laid out as the spec runner leaves one pass: a manifest of
/// two runs, the first passing and the second failing, and each run's output
/// and exit status.
fn work_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("metorex-spec-report-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("manifest"),
        "0\t/specs/core/array/push_spec.rb\n1\t/specs/core/string/plus_spec.rb\t/specs/core/string/times_spec.rb\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("0.out"),
        "$ mspec core/array/push_spec.rb\n...\n\n1 file, 3 examples, 3 expectations, 0 failures, 0 errors, 0 tagged\n",
    )
    .unwrap();
    std::fs::write(dir.join("0.code"), "0\n").unwrap();
    std::fs::write(
        dir.join("1.out"),
        "$ mspec core/string/plus_spec.rb\n.F\n\n1)\nString#+ returns a String & keeps <encoding> FAILED\nExpected \"a\" == \"b\"\n  core/string/plus_spec.rb:12\n\nFinished in 0.1 seconds\n\n2 files, 2 examples, 2 expectations, 1 failure, 0 errors, 0 tagged\n",
    )
    .unwrap();
    std::fs::write(dir.join("1.code"), "1\n").unwrap();
    dir
}

fn report_of(dir: &Path) -> String {
    let report = dir.join("report.xml");
    let status = Command::new("ruby")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scripts/spec_junit.rb"
        ))
        .arg(&report)
        .arg("ruby/spec")
        .arg(dir)
        .arg("/specs")
        .status()
        .expect("ruby runs");
    assert!(status.success());
    std::fs::read_to_string(report).unwrap()
}

#[test]
fn a_passing_run_is_a_test_case_with_no_failure() {
    let report = report_of(&work_dir("passing"));
    assert!(report.contains(r#"<testcase classname="core.array" name="push_spec.rb"/>"#));
}

#[test]
fn a_failing_run_is_a_failure_holding_what_mspec_printed() {
    let report = report_of(&work_dir("failing"));
    assert!(
        report
            .contains(r#"<testcase classname="core.string" name="plus_spec.rb + times_spec.rb">"#)
    );
    assert!(report.contains(
        r#"<failure message="2 files, 2 examples, 2 expectations, 1 failure, 0 errors, 0 tagged">"#
    ));
    assert!(report.contains("String#+ returns a String &amp; keeps &lt;encoding&gt; FAILED"));
    assert!(report.contains("  core/string/plus_spec.rb:12"));
    assert!(!report.contains("Finished in"));
}

#[test]
fn the_suite_counts_the_runs_and_the_failures() {
    let report = report_of(&work_dir("counts"));
    assert!(report.contains(r#"<testsuite name="ruby/spec" tests="2" failures="1" errors="0">"#));
}
