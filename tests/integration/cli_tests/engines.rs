// What METOREX_ENGINE chooses.

use super::metorex_cmd;

/// Run `cli_flags/counted_calls.rb` with `METOREX_ENGINE` set to `engine`.
fn run_on(engine: &str) -> std::process::Output {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/tests/_examples/cli_flags/counted_calls.rb");
    metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_ENGINE", engine)
        .arg(path)
        .output()
        .expect("failed to execute")
}

#[test]
fn the_tree_engine_prints_the_programs_output() {
    let output = run_on("tree");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "250\n");
}

#[test]
fn the_bytecode_engine_prints_the_programs_output() {
    let output = run_on("bytecode");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "250\n");
}

#[test]
fn an_unknown_engine_ends_the_run_naming_the_two_allowed() {
    let output = run_on("jit");
    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "metorex: METOREX_ENGINE must be tree or bytecode, not \"jit\"\n"
    );
}

/// What `METOREX_ENGINE_REPORT` wrote for `cli_flags/<example>` run on the
/// bytecode engine: the units on bytecode, the units on the tree walker,
/// and the count of each node type a unit fell back for.
fn engine_report(example: &str) -> (u64, u64, std::collections::HashMap<String, u64>) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let report_path = std::env::temp_dir().join(format!(
        "metorex_engine_report_{}_{example}.txt",
        std::process::id()
    ));
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_ENGINE", "bytecode")
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
    let fallbacks = report
        .lines()
        .skip_while(|line| *line != "fallbacks:")
        .skip(1)
        .filter_map(|line| {
            let (count, node) = line.trim().split_once(' ')?;
            Some((node.to_string(), count.parse().ok()?))
        })
        .collect();
    (
        count_after("units on bytecode:"),
        count_after("units on the tree walker:"),
        fallbacks,
    )
}

#[test]
fn the_engine_report_counts_a_compiled_method_and_one_that_falls_back() {
    let (bytecode_before, tree_before, fallbacks_before) =
        engine_report("engine_units_uncalled.rb");
    let (bytecode_after, tree_after, fallbacks_after) = engine_report("engine_units.rb");
    assert_eq!(bytecode_after - bytecode_before, 1);
    assert_eq!(tree_after - tree_before, 1);
    let while_loops = |fallbacks: &std::collections::HashMap<String, u64>| {
        fallbacks.get("While").copied().unwrap_or(0)
    };
    assert_eq!(
        while_loops(&fallbacks_after) - while_loops(&fallbacks_before),
        1
    );
}

#[test]
fn an_engine_report_named_by_a_directory_is_written_there_under_the_process_id() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let directory =
        std::env::temp_dir().join(format!("metorex_engine_reports_{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let child = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_ENGINE", "bytecode")
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
    assert!(written.starts_with("units on bytecode: "));
}

/// What `disassembly/<example>` wrote to standard error and standard output
/// run with `METOREX_DISASM` set to `wanted` on `engine`.
fn disassembly_of(example: &str, engine: &str, wanted: &str) -> (String, String) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .env("METOREX_ENGINE", engine)
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
    let (written, printed) = disassembly_of("locals_and_calls.rb", "bytecode", "1");
    assert_eq!(printed, "");
    assert_eq!(
        written,
        concat!(
            "== tests/_examples/disassembly/locals_and_calls.rb\n",
            "        0000  LoadNil { to: 0 }\n",
            "        0001  Return { from: 0 }\n",
            "== tests/_examples/disassembly/locals_and_calls.rb\n",
            "    2 |   product = width * height\n",
            "        0000  GetIdentifier { to: 1, name: 0 }  ; width\n",
            "        0001  GetIdentifier { to: 2, name: 1 }  ; height\n",
            "        0002  Multiply { to: 0, left: 1, right: 2 }\n",
            "        0003  SetLocal { from: 0, slot: 0 }  ; product\n",
            "    3 |   product.to_s\n",
            "        0004  GetLocal { to: 1, slot: 0 }  ; product\n",
            "        0005  Call { base: 1, site: 0 }  ; to_s/0\n",
            "        0006  Move { to: 0, from: 1 }\n",
            "        0007  Return { from: 0 }\n",
        )
    );
}

#[test]
fn disassembly_is_written_only_when_asked_for() {
    assert_eq!(
        disassembly_of("locals_and_calls.rb", "bytecode", "0"),
        (String::new(), String::new())
    );
}

#[test]
fn the_tree_walker_compiles_nothing_to_disassemble() {
    assert_eq!(
        disassembly_of("locals_and_calls.rb", "tree", "1"),
        (String::new(), String::new())
    );
}
