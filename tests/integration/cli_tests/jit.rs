// The JIT, which METOREX_JIT=0 turns off.

use super::metorex_cmd;

/// Run the example `script` under `environment`, and none of the JIT's
/// settings this process was run with, answering what it printed and the
/// report METOREX_COUNTERS wrote.
fn run_example(script: &str, environment: &[(&str, &str)]) -> (String, String) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_COUNTERS", "1")
        .env_remove("METOREX_JIT")
        .env_remove("METOREX_JIT_CALL_THRESHOLD")
        .envs(environment.iter().copied())
        .arg(format!("{manifest_dir}/tests/_examples/{script}"))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    (
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

/// The count on the report's "units compiled to machine code" line.
fn units_compiled(report: &str) -> u64 {
    report
        .lines()
        .find_map(|line| line.strip_prefix("units compiled to machine code: "))
        .expect("a units compiled line")
        .parse()
        .unwrap()
}

const EVERY_UNIT: [(&str, &str); 2] = [("METOREX_JIT", "1"), ("METOREX_JIT_CALL_THRESHOLD", "1")];

#[test]
fn the_jit_is_on_unless_turned_off() {
    let (_, report) = run_example("cli_flags/fib_calls.rb", &[]);
    assert!(units_compiled(&report) >= 1);
}

#[test]
fn without_the_jit_no_unit_is_compiled_to_machine_code() {
    let (_, report) = run_example("cli_flags/fib_calls.rb", &[("METOREX_JIT", "0")]);
    assert_eq!(units_compiled(&report), 0);
}

#[test]
fn the_jit_compiles_every_unit_on_its_first_run_when_the_threshold_is_one() {
    let (printed, report) = run_example("cli_flags/fib_calls.rb", &EVERY_UNIT);
    assert_eq!(printed, "6765\n");
    assert!(units_compiled(&report) > 100);
}

#[test]
fn the_jit_leaves_a_unit_run_fewer_times_than_the_threshold_to_the_interpreter() {
    let (_, every) = run_example("cli_flags/fib_calls.rb", &EVERY_UNIT);
    let (_, hot) = run_example(
        "cli_flags/fib_calls.rb",
        &[("METOREX_JIT", "1"), ("METOREX_JIT_CALL_THRESHOLD", "1000")],
    );
    assert!(units_compiled(&hot) >= 1);
    assert!(units_compiled(&hot) < units_compiled(&every));
}

#[test]
fn machine_code_raises_rescues_breaks_and_jumps_as_the_interpreter_does() {
    let (interpreted, _) = run_example("jit/control_flow.rb", &[("METOREX_JIT", "0")]);
    let (compiled, _) = run_example("jit/control_flow.rb", &EVERY_UNIT);
    assert_eq!(compiled, interpreted);
}

/// The output of the example `jit/<name>.rb` under the interpreter and with
/// the JIT compiling every unit on its first run.
fn interpreted_and_compiled(name: &str) -> (String, String) {
    let script = format!("jit/{name}.rb");
    let (interpreted, _) = run_example(&script, &[("METOREX_JIT", "0")]);
    let (compiled, _) = run_example(&script, &EVERY_UNIT);
    (interpreted, compiled)
}

#[test]
fn machine_code_applies_integer_operations_and_leaves_other_operands_to_the_interpreter() {
    let (interpreted, compiled) = interpreted_and_compiled("integer_operands");
    assert_eq!(compiled, interpreted);
}

#[test]
fn an_integer_operand_that_turns_out_otherwise_returns_the_run_to_the_interpreter() {
    let (_, report) = run_example("jit/integer_operands.rb", &EVERY_UNIT);
    let returned: u64 = report
        .lines()
        .find_map(|line| line.strip_prefix("returns from machine code to the interpreter: "))
        .expect("a returns line")
        .parse()
        .unwrap();
    assert!(returned > 0);
}

#[test]
fn a_method_redefined_while_machine_code_runs_is_called_from_then_on() {
    let (interpreted, compiled) = interpreted_and_compiled("method_redefined");
    assert_eq!(interpreted, "2028\n");
    assert_eq!(compiled, interpreted);
}

#[test]
fn a_constant_set_while_machine_code_runs_is_read_from_then_on() {
    let (interpreted, compiled) = interpreted_and_compiled("constant_set");
    assert_eq!(interpreted, "[3, 4, 102, 103]\n");
    assert_eq!(compiled, interpreted);
}

#[test]
fn an_instance_variable_read_follows_a_change_of_shape() {
    let (interpreted, compiled) = interpreted_and_compiled("shape_changed");
    assert_eq!(compiled, interpreted);
}

#[test]
fn a_tracepoint_enabled_while_machine_code_runs_sees_each_line_from_then_on() {
    let (interpreted, compiled) = interpreted_and_compiled("tracepoint_enabled");
    assert_eq!(compiled, interpreted);
}

#[test]
fn a_trace_func_set_while_machine_code_runs_sees_each_line_from_then_on() {
    let (interpreted, compiled) = interpreted_and_compiled("trace_func_set");
    assert_eq!(compiled, interpreted);
}

#[test]
fn a_local_set_through_a_binding_is_seen_by_machine_code() {
    let (interpreted, compiled) = interpreted_and_compiled("binding_taken");
    assert_eq!(interpreted, "37\n");
    assert_eq!(compiled, interpreted);
}
