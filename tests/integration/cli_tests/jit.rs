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
fn a_unit_whose_instructions_all_call_into_rust_is_left_to_the_interpreter() {
    let (printed, report) = run_example("jit/calls_only.rb", &[("METOREX_JIT", "1")]);
    assert_eq!(
        printed,
        "[\"<done>\", [\"done\", \"done\"], {\"done\" => \"done\"}]\n"
    );
    // The loop alone, and none of the three methods it calls.
    assert_eq!(units_compiled(&report), 1);
}

#[test]
fn a_unit_whose_instructions_mostly_call_into_rust_waits_longer_to_be_compiled() {
    let (printed, report) = run_example("jit/own_thresholds.rb", &[("METOREX_JIT", "1")]);
    assert_eq!(printed, "4950\n24\n");
    // The loop of Integer operations, and not the loop that builds a string.
    assert_eq!(units_compiled(&report), 1);
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
fn machine_code_calls_the_method_each_side_of_a_branch_names_as_the_interpreter_does() {
    let (interpreted, compiled) = interpreted_and_compiled("branching_calls");
    assert_eq!(interpreted, "100\n");
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_loads_symbol_literals_as_the_interpreter_makes_them() {
    let (interpreted, compiled) = interpreted_and_compiled("symbol_literals");
    assert_eq!(
        interpreted,
        "[:plain, :\"two words\", :café]\ntrue\n#<Encoding:UTF-8>\n1\n#<Encoding:BINARY (ASCII-8BIT)>\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_calls_what_a_bare_name_names_after_the_method_or_a_local_of_its_name_changes() {
    let (interpreted, compiled) = interpreted_and_compiled("bare_calls");
    assert_eq!(
        interpreted,
        "[\"hello\"]\n[\"hi\"]\n[\"hi\", \"hi\"]\n[\"hi\", \"hi\"]\n[\"own\"]\n[\"hi\"]\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_calls_what_a_built_in_value_answers_after_a_method_of_its_name_is_written() {
    let (interpreted, compiled) = interpreted_and_compiled("native_calls");
    assert_eq!(interpreted, "[3]\n[99]\n[3]\n[-1]\n[4]\n");
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_calls_what_a_top_level_definition_names_after_it_is_redefined() {
    let (interpreted, compiled) = interpreted_and_compiled("top_level_bare_calls");
    assert_eq!(interpreted, "60\n60\n[2]\n");
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_writes_instance_variables_as_the_interpreter_does_whatever_the_shape() {
    let (interpreted, compiled) = interpreted_and_compiled("slot_writes");
    assert_eq!(
        interpreted,
        "[[38, -38, nil], [39, -39, :odd]]\n[[-38, 38, nil], [-39, 39, :odd]]\n\
         [:@left, :@right]\n[:@left, :@right, :@__seen]\nFrozenError\n[1, 2, nil]\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_leaves_calls_on_other_values_to_the_interpreter_once_a_site_gives_up() {
    let (interpreted, compiled) = interpreted_and_compiled("given_up_calls");
    assert_eq!(interpreted, "[\"box\"]\n\"39\"\n[Object]\n70\n");
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_writes_through_writers_as_the_interpreter_calls_them() {
    let (interpreted, compiled) = interpreted_and_compiled("attribute_writes");
    assert_eq!(
        interpreted,
        "[49, -49, 49, 2]\n[49, 2]\n[]\n:kept\nNoMethodError\nFrozenError\n-490\n1049\n3\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_reads_through_readers_as_the_interpreter_calls_them() {
    let (interpreted, compiled) = interpreted_and_compiled("attribute_reads");
    assert_eq!(
        interpreted,
        "[700, :first]\n60\n[5550, :first]\n[350, :first]\n80\n1\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_calls_methods_on_main_as_the_interpreter_looks_them_up() {
    let (interpreted, compiled) = interpreted_and_compiled("resolved_calls");
    assert_eq!(
        interpreted,
        "100\n9900\n[:broke]\n117\n-39\n:singleton\n[:not_written, 39]\n780\n0\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_calls_the_native_methods_core_stubs_stand_for_as_the_interpreter_does() {
    let (interpreted, compiled) = interpreted_and_compiled("native_stubs");
    assert_eq!(
        interpreted,
        "70\n[:<<, :<<, :<<]\n\"private method '<<' called for an instance of String\"\n\
         70\n\"0!1!2!\"\n80\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_reads_closed_over_locals_as_the_interpreter_does_however_they_change() {
    let (interpreted, compiled) = interpreted_and_compiled("closed_over_locals");
    assert_eq!(
        interpreted,
        "80\n[20, 20, 20]\n[2, 20]\n[20, -1]\n[:p]\n78\n[-1, 5]\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn machine_code_closes_blocks_over_what_the_interpreter_does_when_one_scope_makes_several() {
    let (interpreted, compiled) = interpreted_and_compiled("kept_closures");
    assert_eq!(
        interpreted,
        "[39, [39, 78], [39, 79]]\n120\n[19, :held, 21]\n451\n"
    );
    assert_eq!(compiled, interpreted);
}

#[test]
fn an_integer_operand_that_turns_out_otherwise_returns_the_run_to_the_interpreter() {
    let (_, report) = run_example("jit/integer_operands.rb", &EVERY_UNIT);
    assert!(returns_to_the_interpreter(&report) > 0);
}

/// The count on the report's "returns from machine code" line.
fn returns_to_the_interpreter(report: &str) -> u64 {
    report
        .lines()
        .find_map(|line| line.strip_prefix("returns from machine code to the interpreter: "))
        .expect("a returns line")
        .parse()
        .unwrap()
}

#[test]
fn a_unit_that_returned_to_the_interpreter_waits_twice_as_long_to_be_compiled_again() {
    let (printed, waiting) = run_example("jit/backoff.rb", &[("METOREX_JIT", "1")]);
    assert_eq!(printed, "1600\n");
    // The add_ones method's own threshold, kept after each return.
    let (_, fixed) = run_example(
        "jit/backoff.rb",
        &[("METOREX_JIT", "1"), ("METOREX_JIT_CALL_THRESHOLD", "53")],
    );
    assert_eq!(returns_to_the_interpreter(&fixed), 4);
    assert_eq!(returns_to_the_interpreter(&waiting), 2);
}

#[test]
fn a_method_defined_with_no_operator_name_keeps_the_machine_code() {
    let (printed, report) = run_example("jit/unrelated_method_defined.rb", &[("METOREX_JIT", "1")]);
    assert_eq!(printed, "79800\n");
    assert_eq!(units_compiled(&report), 1);
    assert_eq!(returns_to_the_interpreter(&report), 0);
}

#[test]
fn recursion_past_the_depth_allowed_raises_as_the_interpreter_does() {
    let (interpreted, compiled) = interpreted_and_compiled("deep_recursion");
    assert_eq!(interpreted, "stack level too deep\n".repeat(5));
    assert_eq!(compiled, interpreted);
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

#[test]
fn a_bare_super_in_a_return_modifier_hands_on_the_arguments_once_compiled() {
    let (compiled, _) = run_example(
        "jit/zsuper_in_a_prepended_module_when_hot.rb",
        &[("METOREX_JIT", "1"), ("METOREX_JIT_CALL_THRESHOLD", "1")],
    );
    assert_eq!(compiled, "40000\n");
}

#[test]
fn a_tracepoint_hears_the_native_calls_of_compiled_code() {
    let (interpreted, _) = run_example(
        "stdlib_libraries/power_assert_method_chain.rb",
        &[("METOREX_JIT", "0")],
    );
    let (compiled, _) = run_example("stdlib_libraries/power_assert_method_chain.rb", &EVERY_UNIT);
    assert_eq!(compiled, interpreted);
}
