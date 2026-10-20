use super::EXAMPLES_DIR;
use std::process::Command;

fn run_primitive(name: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_metorex"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg(format!("{EXAMPLES_DIR}/rex/primitives/{name}.rb"))
        .output()
        .expect("failed to execute example");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout was not utf8")
}

#[test]
fn rex_source_reads_the_statement_a_line_falls_in_and_the_value_handed_to_expect() {
    assert_eq!(
        run_primitive("source_lines"),
        concat!(
            "[18, [18, 18]]\n[23, [23, 23]]\n[3, [3, 5]]\n[11, [9, 11]]\n",
            "[10, [10, 10]]\n[17, [15, 17]]\n[26, [26, 26]]\n[7, nil]\n",
            "\"cart.total\"\nnil\nnil\n"
        )
    );
}
