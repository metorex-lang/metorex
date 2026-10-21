// What METOREX_MEMORY_REPORT=1 reports at exit.

use super::metorex_cmd;

/// Run `cli_flags/memory_reported.rb` with the environment given, answering
/// what it wrote to standard output and to standard error.
fn run_memory_reported(environment: &[(&str, &str)]) -> (String, String) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .envs(environment.iter().copied())
        .arg(format!(
            "{manifest_dir}/tests/_examples/cli_flags/memory_reported.rb"
        ))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

/// The megabytes on the report line that ends with `label`, one for each
/// column.
fn megabytes(report: &str, label: &str) -> Vec<f64> {
    let line = report
        .lines()
        .find(|line| line.trim_end().ends_with(label))
        .unwrap_or_else(|| panic!("no line ends with {label:?} in:\n{report}"));
    line.split("MB")
        .filter_map(|part| part.split_whitespace().last()?.parse().ok())
        .collect()
}

#[test]
fn memory_report_writes_nothing_unless_asked() {
    let (printed, reported) = run_memory_reported(&[]);
    assert_eq!(printed, "50000\n");
    assert_eq!(reported, "");
}

#[test]
fn memory_report_stays_off_when_set_to_zero() {
    let (_, reported) = run_memory_reported(&[("METOREX_MEMORY_REPORT", "0")]);
    assert_eq!(reported, "");
}

#[test]
fn memory_report_leaves_what_the_program_prints_alone() {
    let (printed, _) = run_memory_reported(&[("METOREX_MEMORY_REPORT", "1")]);
    assert_eq!(printed, "50000\n");
}

#[test]
fn memory_report_accounts_for_the_peak_resident_set_size_within_ten_percent() {
    let (_, reported) = run_memory_reported(&[("METOREX_MEMORY_REPORT", "1")]);
    let percent: f64 = reported
        .lines()
        .find_map(|line| line.strip_prefix("accounted for: "))
        .and_then(|rest| rest.split(", ").nth(1))
        .and_then(|share| share.split('%').next())
        .and_then(|share| share.parse().ok())
        .unwrap_or_else(|| panic!("no accounted for line in:\n{reported}"));
    assert!((90.0..=110.0).contains(&percent), "{reported}");
}

#[test]
fn memory_report_counts_the_core_library_syntax_tree_under_startup() {
    let (_, reported) = run_memory_reported(&[("METOREX_MEMORY_REPORT", "1")]);
    let [startup, _program] = megabytes(&reported, "the syntax tree")[..] else {
        panic!("two columns on the syntax tree line in:\n{reported}");
    };
    assert!(startup > 1.0, "{reported}");
}

#[test]
fn memory_report_counts_the_strings_the_program_keeps_under_the_program() {
    let (_, reported) = run_memory_reported(&[("METOREX_MEMORY_REPORT", "1")]);
    let [_startup, program] = megabytes(&reported, "everything else on the heap")[..] else {
        panic!("two columns on the everything else line in:\n{reported}");
    };
    assert!(program > 1.0, "{reported}");
}

#[test]
fn memory_report_lists_the_objects_alive_by_class() {
    let (_, reported) = run_memory_reported(&[("METOREX_MEMORY_REPORT", "1")]);
    let strings = reported
        .lines()
        .find(|line| line.trim_end().ends_with("  String"))
        .unwrap_or_else(|| panic!("no String row in:\n{reported}"));
    let count: u64 = strings.split_whitespace().next().unwrap().parse().unwrap();
    assert!(count >= 10_000, "{reported}");
    assert!(
        reported.lines().any(|line| line.ends_with("  Ledger")),
        "{reported}"
    );
}
