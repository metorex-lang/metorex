// Code samples published on metorex.org.

use super::run_example;

#[test]
fn test_site_home_snippet_execution() {
    let expected = "#<struct Point x=4, y=6>\ny leads by 2\n";
    let output = run_example("site/home.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_install_hello_execution() {
    let expected = "Hello, world!\n[2, 4, 6]\n";
    let output = run_example("site/install_hello.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_meta_method_tree_execution() {
    let expected = "total takes 2 arguments\nBinaryOp +\n  BinaryOp *\n    Identifier price\n    Identifier quantity\n  IntLiteral 5\n";
    let output = run_example("site/meta.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_meta_parse_execution() {
    let expected = "\"+\"\n{\"type\" => \"BinaryOp\", \"op\" => \"*\", \"left\" => {\"type\" => \"IntLiteral\", \"value\" => 2}, \"right\" => {\"type\" => \"IntLiteral\", \"value\" => 3}}\n";
    let output = run_example("site/meta_parse.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_meta_block_execution() {
    let expected = "1\n{\"type\" => \"BinaryOp\", \"op\" => \"*\", \"left\" => {\"type\" => \"Identifier\", \"name\" => \"order\"}, \"right\" => {\"type\" => \"IntLiteral\", \"value\" => 2}}\n42\n";
    let output = run_example("site/meta_block.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_classes_execution() {
    let expected = "Ada: $1575.0\ntrue\n[SavingsAccount, Account, Object]\n";
    let output = run_example("site/language_classes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_blocks_execution() {
    let expected = "1 -> 4\n4 -> 9\n[[\"a\", \"b\"], [\"b\", \"c\"]]\n[1, 4, 9]\n17\n[\"APPLE\", \"BANANA\"]\n";
    let output = run_example("site/language_blocks.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_struct_execution() {
    let expected = "5.0\n[3, 4]\ntrue\n#<data Coordinate latitude=36.16, longitude=-86.78>\n#<data Coordinate latitude=40.71, longitude=-86.78>\n";
    let output = run_example("site/language_struct.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_enumerable_execution() {
    let expected = "464\n\"monitor\"\n[2, 1]\n{true => [\"keyboard\", \"cable\"], false => [\"monitor\"]}\n[4, 16, 36]\n{\"a\" => 0, \"b\" => 1, \"c\" => 2}\n";
    let output = run_example("site/language_enumerable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_pattern_matching_execution() {
    let expected = "no users\nAda and 1 more\nclient error 404\nserver error 503\n5432\n";
    let output = run_example("site/language_pattern_matching.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_language_exceptions_execution() {
    let expected =
        "payment of 120 declined (attempt 1)\ncharged 90\ndone after 2 attempts\n8080\nnil\n";
    let output = run_example("site/language_exceptions.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_differences_description_names_metorex_and_the_followed_release() {
    let output = run_example("site/differences_description.rb");
    assert!(
        output.starts_with("metorex 4.0.7 (ruby-compatible) [")
            && output.ends_with("]\n[\"ruby\", \"4.0.7\", \"4.0.7\"]\n"),
        "{output}"
    );
}

#[test]
fn test_site_differences_disasm_lists_instructions() {
    let expected = concat!(
        "== disasm: #<ISeq:<compiled>@<compiled>:1 (1,0)-(1,13)>\n",
        "0000 LoadInteger to: 0, value: 2            (   1)[Li]\n",
        "0001 LoadInteger to: 1, value: 3            (   1)\n",
        "0002 Add to: 0, left: 0, right: 1           (   1)\n",
        "0003 SetLocal from: 0, slot: 0  ; total     (   1)\n",
        "0004 Return from: 0                         (   1)\n",
    );
    let output = run_example("site/differences_disasm.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_differences_vm_stat_counts_compiled_units() {
    let output = run_example("site/differences_stat.rb");
    assert_eq!(output, "true\ntrue\ntrue\n");
}

#[test]
fn test_site_differences_gc_config_names_metorex() {
    let output = run_example("site/differences_gc_config.rb");
    assert_eq!(output, "\"metorex\"\n");
}

#[test]
fn test_site_differences_a_waiting_thread_has_no_native_id() {
    let output = run_example("site/differences_native_thread.rb");
    assert_eq!(output, "true\nNilClass\n");
}

#[test]
fn test_site_c_extensions_nkf_converts_shift_jis() {
    let expected = "16\n#<Encoding:Shift_JIS>\n\"こんにちは、世界\"\n\"かたかな\"\n";
    let output = run_example("site/c_extensions_nkf.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_differences_disable_gems_keeps_error_highlight_and_did_you_mean() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "--disable-gems",
            &format!(
                "{}/site/differences_disable_gems.rb",
                crate::common::EXAMPLES_DIR
            ),
        ])
        .output()
        .expect("run metorex");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "\"constant\"\n\"constant\"\n"
    );
}

#[test]
fn test_site_beyond_assertions_execution() {
    let output = run_example("site/beyond_assertions.rb");
    assert_eq!(output, "Expected 5, got 4\n");
}

#[test]
fn test_site_beyond_parse_execution() {
    let output = run_example("site/beyond_parse.rb");
    assert_eq!(output, "((rate * hours) + 40)\n(rate * (hours + 40))\n");
}

#[test]
fn test_site_versions_execution() {
    let output = run_example("site/versions.rb");
    assert_eq!(
        output,
        format!(
            "[\"ruby\", \"{0}\", \"{0}\"]\n3.13.6\n",
            metorex::DEFAULT_RUBY_VERSION
        )
    );
}

#[test]
fn test_site_versions_reports_the_release_metorex_ruby_version_names() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg(format!("{}/site/versions.rb", crate::common::EXAMPLES_DIR))
        .env("METOREX_RUBY_VERSION", "3.4.5")
        .output()
        .expect("failed to execute example");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "[\"ruby\", \"3.4.5\", \"3.4.5\"]\n3.13.6\n"
    );
}

#[test]
fn test_site_docs_nodes_execution() {
    let output = run_example("site/docs_nodes.rb");
    assert_eq!(
        output,
        concat!(
            "\"If\"\n",
            "{\"type\" => \"Identifier\", \"name\" => \"ready\"}\n",
            "\"total\"\n",
            "\"sum\"\n",
            "[{\"type\" => \"Symbol\", \"value\" => \"price\"}]\n",
            "nil\n",
            "{\"type\" => \"Unknown\"}\n"
        )
    );
}
