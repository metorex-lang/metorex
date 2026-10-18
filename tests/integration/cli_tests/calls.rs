// What a method call costs, as METOREX_COUNTERS=1 reports it.

use super::metorex_cmd;

/// Run the example `calls/<name>.rb` with METOREX_COUNTERS=1, answering what
/// it printed and the report.
fn run_counted(name: &str) -> (String, String) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_COUNTERS", "1")
        .arg(format!("{manifest_dir}/tests/_examples/calls/{name}.rb"))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

/// The calls and the heap allocations on the report line for `path`.
fn calls_and_allocations(report: &str, path: &str) -> (u64, u64) {
    let line = report
        .lines()
        .find(|line| line.trim_end().ends_with(path))
        .unwrap_or_else(|| panic!("no line ends with {path:?} in:\n{report}"));
    let counts: Vec<u64> = line
        .split_whitespace()
        .map_while(|word| word.parse().ok())
        .collect();
    (counts[0], counts[1])
}

#[test]
fn a_bare_call_on_a_plain_instance_makes_at_most_six_heap_allocations() {
    let (printed, report) = run_counted("plain_instance_calls");
    assert_eq!(printed, "1000\n");
    let (calls, allocations) = calls_and_allocations(&report, "Counter#bare_call -> Counter#blank");
    assert_eq!(calls, 1000);
    assert!(allocations <= 6 * calls, "{allocations} allocations");
}

#[test]
fn a_call_with_self_on_a_plain_instance_makes_at_most_six_heap_allocations() {
    let (_, report) = run_counted("plain_instance_calls");
    let (calls, allocations) = calls_and_allocations(&report, "Counter#self_call -> Counter#blank");
    assert_eq!(calls, 1000);
    assert!(allocations <= 6 * calls, "{allocations} allocations");
}

#[test]
fn calls_leave_the_method_state_where_it_was() {
    let (printed, _) = run_counted("method_state_across_calls");
    assert_eq!(printed, "0\n");
}
