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
fn counters_report_the_instructions_executed() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_COUNTERS", "1")
        .arg(format!(
            "{manifest_dir}/tests/_examples/cli_flags/engine_counted.rb"
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

/// What `cli_flags/blocks_in_a_loop.rb` reports for blocks built when it
/// makes `passes` passes.
fn blocks_built(passes: &str) -> u64 {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_COUNTERS", "1")
        .env("PASSES", passes)
        .arg(format!(
            "{manifest_dir}/tests/_examples/cli_flags/blocks_in_a_loop.rb"
        ))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let reported = String::from_utf8(output.stderr).unwrap();
    reported
        .lines()
        .find_map(|line| line.strip_prefix("blocks built: "))
        .and_then(|count| count.trim().parse().ok())
        .expect("a blocks built line")
}

#[test]
fn a_block_written_in_a_loop_is_built_once_however_many_passes_run_it() {
    assert_eq!(blocks_built("100"), blocks_built("200"));
}

/// The calls and the heap allocations `cli_flags/calls_in_a_loop.rb` reports
/// for the call path ending with `label`.
fn calls_in_a_loop(label: &str) -> (u64, u64) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_COUNTERS", "1")
        .env("PASSES", "1000")
        .arg(format!(
            "{manifest_dir}/tests/_examples/cli_flags/calls_in_a_loop.rb"
        ))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "1000\n");
    let reported = String::from_utf8(output.stderr).unwrap();
    let counts = counted(&reported, label);
    (counts[0], counts[1])
}

#[test]
fn a_call_of_a_method_returning_its_argument_makes_at_most_ten_heap_allocations() {
    let (calls, allocations) = calls_in_a_loop("Object#run -> Object#identity");
    assert_eq!(calls, 1000);
    assert!(
        allocations <= 10 * calls,
        "{allocations} allocations for {calls} calls"
    );
}

#[test]
fn fib_calls_prints_the_twentieth_fibonacci_number() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .arg(format!(
            "{manifest_dir}/tests/_examples/cli_flags/fib_calls.rb"
        ))
        .output()
        .expect("failed to execute");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "6765\n");
}

/// The names under the "libraries parsed from source" line of the report
/// a run of the example `script` writes.
fn libraries_parsed(script: &str) -> Vec<String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_COUNTERS", "1")
        .arg(format!("{manifest_dir}/tests/_examples/cli_flags/{script}"))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let report = String::from_utf8(output.stderr).unwrap();
    let mut lines = report
        .lines()
        .skip_while(|line| !line.starts_with("libraries parsed from source: "));
    let count: usize = lines
        .next()
        .and_then(|line| line.strip_prefix("libraries parsed from source: "))
        .expect("a libraries parsed line")
        .parse()
        .unwrap();
    lines
        .take(count)
        .map(|line| line.trim().to_string())
        .collect()
}

#[test]
fn a_default_start_parses_no_library_from_source() {
    assert_eq!(libraries_parsed("counted_calls.rb"), Vec::<String>::new());
}

#[test]
fn a_library_required_after_startup_is_parsed_from_source() {
    assert_eq!(libraries_parsed("requires_shellwords.rb"), ["shellwords"]);
}
