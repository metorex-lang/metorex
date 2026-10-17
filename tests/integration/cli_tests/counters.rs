// What METOREX_COUNTERS=1 reports at exit.

use super::metorex_cmd;

/// Run `cli_flags/counted_calls.rb` with the environment given, answering
/// what it wrote to standard output and to standard error.
fn run_counted_calls(environment: &[(&str, &str)]) -> (String, String) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/tests/_examples/cli_flags/counted_calls.rb");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .envs(environment.iter().copied())
        .arg(path)
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

/// The count on the report line that ends with `label`.
fn counted(report: &str, label: &str) -> Vec<u64> {
    let line = report
        .lines()
        .find(|line| line.trim_end().ends_with(label))
        .unwrap_or_else(|| panic!("no line ends with {label:?} in:\n{report}"));
    line.split_whitespace()
        .map_while(|word| word.parse().ok())
        .collect()
}

#[test]
fn counters_report_nothing_unless_asked() {
    let (printed, reported) = run_counted_calls(&[]);
    assert_eq!(printed, "250\n");
    assert_eq!(reported, "");
}

#[test]
fn counters_report_each_call_path_with_its_count() {
    let (printed, reported) = run_counted_calls(&[("METOREX_COUNTERS", "1")]);
    assert_eq!(printed, "250\n");
    assert_eq!(counted(&reported, "<main> -> Tally#bump")[0], 250);
    assert_eq!(counted(&reported, "<main> -> Tally.start")[0], 251);
}

#[test]
fn counters_report_the_statements_and_expressions_evaluated() {
    // The tree walker counts statements and expressions, where the bytecode
    // engine counts instructions.
    let (_, reported) = run_counted_calls(&[("METOREX_COUNTERS", "1"), ("METOREX_ENGINE", "tree")]);
    assert!(reported.starts_with("metorex counters, process "));
    assert!(counted(&reported, "  While")[0] >= 1);
    assert!(counted(&reported, "  MethodCall")[0] >= 500);
}

#[test]
fn counters_report_heap_allocations_for_each_call_path() {
    let (_, reported) = run_counted_calls(&[("METOREX_COUNTERS", "1")]);
    let total: u64 = reported
        .lines()
        .find_map(|line| line.strip_prefix("heap allocations: "))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|count| count.parse().ok())
        .expect("a heap allocations line");
    assert!(total > 0);
    assert!(counted(&reported, "<main> -> Tally#bump")[1] > 0);
}

#[test]
fn counters_stay_off_when_set_to_zero() {
    let (_, reported) = run_counted_calls(&[("METOREX_COUNTERS", "0")]);
    assert_eq!(reported, "");
}

#[test]
fn counters_report_the_instructions_executed_on_the_bytecode_engine() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_COUNTERS", "1")
        .env("METOREX_ENGINE", "bytecode")
        .arg(format!(
            "{manifest_dir}/tests/_examples/cli_flags/engine_units.rb"
        ))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let reported = String::from_utf8(output.stderr).unwrap();
    assert!(reported.contains("\ninstructions executed: "), "{reported}");
    assert!(counted(&reported, "LoadNil")[0] >= 1);
    assert!(counted(&reported, "Return")[0] >= 1);
    assert!(counted(&reported, "Define")[0] >= 1);
}
