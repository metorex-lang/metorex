// `--jobs N`: groups run in forked workers and are reported in order.

use super::{expected, run_spec};

#[test]
fn test_a_parallel_run_reads_the_same_as_rspec_s_serial_run() {
    let run = run_spec("parallel/groups", &["-fd", "--jobs", "3"]);
    assert_eq!(
        run.transcript(),
        expected("parallel/groups", "expected_documentation.txt")
    );
}

#[test]
fn test_jobs_auto_uses_one_worker_per_processor() {
    let run = run_spec("parallel/groups", &["-fd", "--jobs", "auto"]);
    assert_eq!(
        run.transcript(),
        expected("parallel/groups", "expected_documentation.txt")
    );
}

#[test]
fn test_one_job_writes_the_same_bytes_as_a_serial_run() {
    let serial = run_spec("ordering/order", &["-fd", "--seed", "1234"]);
    let one_job = run_spec("ordering/order", &["-fd", "--seed", "1234", "--jobs", "1"]);
    assert_eq!(serial.output_with_times, one_job.output_with_times);
}

#[test]
fn test_a_shuffled_parallel_run_keeps_the_seed_s_order() {
    let run = run_spec("ordering/order", &["-fd", "--seed", "1234", "--jobs", "2"]);
    assert_eq!(
        run.transcript(),
        expected("ordering/order", "expected_seed.txt")
    );
}

// RSpec has no parallel runs, so this expectation is Rex's own.
#[test]
fn test_verify_parallel_warns_about_an_example_another_group_s_state_decides() {
    let run = run_spec(
        "parallel/shared_state",
        &["--jobs", "2", "--verify-parallel"],
    );
    assert_eq!(
        run.transcript(),
        expected("parallel/shared_state", "expected_verified.txt")
    );
}

#[test]
fn test_a_job_count_below_one_is_refused() {
    let run = run_spec("parallel/groups", &["--jobs", "0"]);
    assert_eq!(
        run.transcript(),
        "invalid argument: --jobs 0\nPlease use --help for a listing of valid options\nexit 1\n"
    );
}

#[test]
fn test_fail_fast_stops_a_parallel_run_where_a_serial_run_stops() {
    let run = run_spec("parallel/groups", &["-fd", "--fail-fast", "--jobs", "3"]);
    assert_eq!(
        run.transcript(),
        expected("parallel/groups", "expected_fail_fast.txt")
    );
}
