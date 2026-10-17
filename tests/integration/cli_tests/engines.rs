// The engine report, disassembly, and what a body too deep to compile does.

use super::metorex_cmd;

/// What `METOREX_ENGINE_REPORT` wrote for `cli_flags/<example>`: the units
/// compiled, the units refused, and the count of each node type a unit was
/// refused for.
fn engine_report(example: &str) -> (u64, u64, std::collections::HashMap<String, u64>) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let report_path = std::env::temp_dir().join(format!(
        "metorex_engine_report_{}_{example}.txt",
        std::process::id()
    ));
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_ENGINE_REPORT", &report_path)
        .arg(format!(
            "{manifest_dir}/tests/_examples/cli_flags/{example}"
        ))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let report = std::fs::read_to_string(&report_path).expect("no report written");
    std::fs::remove_file(&report_path).unwrap();
    let count_after = |label: &str| -> u64 {
        report
            .lines()
            .find_map(|line| line.strip_prefix(label))
            .and_then(|count| count.trim().parse().ok())
            .unwrap_or_else(|| panic!("no {label:?} line in:\n{report}"))
    };
    let refusals = report
        .lines()
        .skip_while(|line| *line != "refusals:")
        .skip(1)
        .filter_map(|line| {
            let (count, node) = line.trim().split_once(' ')?;
            Some((node.to_string(), count.parse().ok()?))
        })
        .collect();
    (
        count_after("units compiled:"),
        count_after("units refused:"),
        refusals,
    )
}

#[test]
fn the_engine_report_counts_compiled_methods_and_a_refused_file() {
    let (compiled_before, refused_before, refusals_before) =
        engine_report("engine_units_uncalled.rb");
    let (compiled_after, refused_after, refusals_after) = engine_report("engine_units.rb");
    // The two methods and the method of the prelude `load` runs.
    assert_eq!(compiled_after - compiled_before, 3);
    assert_eq!(refused_after - refused_before, 1);
    let powers = |refusals: &std::collections::HashMap<String, u64>| {
        refusals.get("BinaryOp").copied().unwrap_or(0)
    };
    assert_eq!(powers(&refusals_after) - powers(&refusals_before), 1);
}

#[test]
fn an_engine_report_named_by_a_directory_is_written_there_under_the_process_id() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let directory =
        std::env::temp_dir().join(format!("metorex_engine_reports_{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let child = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_ENGINE_REPORT", &directory)
        .arg(format!(
            "{manifest_dir}/tests/_examples/cli_flags/engine_units.rb"
        ))
        .spawn()
        .expect("failed to execute");
    let process_id = child.id();
    assert!(child.wait_with_output().unwrap().status.success());
    let written = std::fs::read_to_string(directory.join(format!("metorex-{process_id}.txt")))
        .expect("no report written under the process id");
    std::fs::remove_dir_all(&directory).unwrap();
    assert!(written.starts_with("units compiled: "));
}

/// What `disassembly/<example>` wrote to standard error and standard output
/// run with `METOREX_DISASM` set to `wanted`.
fn disassembly_of(example: &str, wanted: &str) -> (String, String) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_DISASM", wanted)
        .arg(format!("tests/_examples/disassembly/{example}"))
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    (
        String::from_utf8(output.stderr).unwrap(),
        String::from_utf8(output.stdout).unwrap(),
    )
}

#[test]
fn disassembly_writes_each_unit_under_its_source_lines_to_standard_error() {
    let (written, printed) = disassembly_of("locals_and_calls.rb", "1");
    assert_eq!(printed, "");
    assert_eq!(
        written,
        concat!(
            "== tests/_examples/disassembly/locals_and_calls.rb\n",
            "    1 | def area(width, height)\n",
            "        0000  Define { to: 0, definition: 0 }\n",
            "    6 | def nothing\n",
            "        0001  Define { to: 0, definition: 1 }\n",
            "    9 | nothing\n",
            "        0002  GetIdentifier { to: 0, name: 0 }  ; nothing\n",
            "   10 | area(2, 3)\n",
            "        0003  ResolveCall { site: 0 }\n",
            "        0004  LoadInteger { to: 2, value: 2 }\n",
            "        0005  LoadInteger { to: 3, value: 3 }\n",
            "        0006  CallResolved { base: 1, site: 0 }\n",
            "        0007  Move { to: 0, from: 1 }\n",
            "        0008  Return { from: 0 }\n",
            "== tests/_examples/disassembly/locals_and_calls.rb\n",
            "        0000  LoadNil { to: 0 }\n",
            "        0001  Return { from: 0 }\n",
            "== tests/_examples/disassembly/locals_and_calls.rb\n",
            "    2 |   product = width * height\n",
            "        0000  Unhoist { slot: 0 }  ; product\n",
            "        0001  GetIdentifier { to: 0, name: 0 }  ; width\n",
            "        0002  GetIdentifier { to: 1, name: 1 }  ; height\n",
            "        0003  Multiply { to: 0, left: 0, right: 1 }\n",
            "        0004  SetLocal { from: 0, slot: 0 }  ; product\n",
            "    3 |   product.to_s\n",
            "        0005  GetLocal { to: 0, slot: 0 }  ; product\n",
            "        0006  Call { base: 0, site: 0 }  ; to_s/0\n",
            "        0007  Return { from: 0 }\n",
        )
    );
}

#[test]
fn disassembly_is_written_only_when_asked_for() {
    assert_eq!(
        disassembly_of("locals_and_calls.rb", "0"),
        (String::new(), String::new())
    );
}

#[test]
fn code_handed_to_eval_instance_eval_class_eval_and_a_binding_is_compiled_as_a_unit_of_its_own() {
    let (written, _) = disassembly_of("code_from_strings.rb", "1");
    let evaluated: Vec<&str> = written
        .lines()
        .filter(|line| line.starts_with("== (eval"))
        .collect();
    assert_eq!(
        evaluated,
        [
            "== (eval at tests/_examples/disassembly/code_from_strings.rb:3)",
            "== (eval at tests/_examples/disassembly/code_from_strings.rb:4)",
            "== (eval at tests/_examples/disassembly/code_from_strings.rb:5)",
            "== (eval at tests/_examples/disassembly/code_from_strings.rb:6)",
        ]
    );
}

#[test]
fn a_body_nested_deeper_than_an_instruction_has_registers_for_raises_system_stack_error() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .arg("tests/_examples/cli_flags/engine_units_refused.rb")
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("stack level too deep (SystemStackError)")
    );
}
