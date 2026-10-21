// The runner: discovery, option files, filters, formatters, and the exit
// status, each compared with what RSpec 3.13 prints for the same fixture.

use super::{expected, run_spec, run_spec_with_env};

fn assert_matches_rspec(fixture: &str, name: &str, arguments: &[&str]) {
    let run = run_spec(fixture, arguments);
    assert_eq!(
        run.transcript(),
        expected(fixture, &format!("expected_{name}.txt"))
    );
}

#[test]
fn test_spec_runs_the_spec_directory_with_the_progress_formatter() {
    assert_matches_rspec("runner/basic", "progress", &[]);
}

#[test]
fn test_spec_prints_the_documentation_formatter() {
    assert_matches_rspec(
        "runner/basic",
        "documentation",
        &["--format", "documentation"],
    );
}

#[test]
fn test_spec_reads_the_format_and_requires_from_dot_rspec() {
    assert_matches_rspec("runner/options", "default", &[]);
}

#[test]
fn test_spec_runs_only_the_files_under_a_named_directory() {
    assert_matches_rspec("runner/options", "directory", &["spec/models"]);
}

#[test]
fn test_spec_runs_the_example_at_a_file_and_line() {
    assert_matches_rspec("runner/options", "location", &["spec/money_spec.rb:7"]);
}

#[test]
fn test_spec_runs_every_example_in_the_group_at_a_file_and_line() {
    assert_matches_rspec(
        "runner/options",
        "location_group",
        &["spec/money_spec.rb:10"],
    );
}

#[test]
fn test_spec_runs_the_examples_at_several_lines_of_one_file() {
    assert_matches_rspec(
        "runner/options",
        "two_locations",
        &["spec/money_spec.rb:3:12"],
    );
}

#[test]
fn test_spec_runs_the_examples_whose_names_include_the_example_text() {
    assert_matches_rspec("runner/options", "example", &["-e", "formats"]);
}

#[test]
fn test_spec_runs_the_examples_whose_names_match_a_pattern() {
    assert_matches_rspec("runner/options", "example_matches", &["-E", "zero$"]);
}

#[test]
fn test_spec_loads_the_files_a_pattern_names() {
    assert_matches_rspec(
        "runner/options",
        "pattern",
        &["--pattern", "spec/**/*_check.rb"],
    );
}

#[test]
fn test_spec_leaves_out_the_files_an_exclude_pattern_names() {
    assert_matches_rspec(
        "runner/options",
        "exclude_pattern",
        &["--exclude-pattern", "**/models/*"],
    );
}

#[test]
fn test_a_format_on_the_command_line_replaces_the_one_in_dot_rspec() {
    assert_matches_rspec("runner/options", "progress", &["-f", "p"]);
}

#[test]
fn test_spec_exits_with_the_failure_exit_code_given() {
    assert_matches_rspec(
        "runner/options",
        "failure_exit_code",
        &["--failure-exit-code", "3"],
    );
}

#[test]
fn test_spec_adds_load_paths_and_requires_files() {
    assert_matches_rspec(
        "runner/options",
        "require",
        &["-I", "lib", "-r", "account", "spec/models"],
    );
}

#[test]
fn test_spec_opts_override_the_option_files() {
    let run = run_spec_with_env("runner/options", &[], &[("SPEC_OPTS", "--format progress")]);
    assert_eq!(
        run.transcript(),
        expected("runner/options", "expected_spec_opts.txt")
    );
}

#[test]
fn test_spec_exits_with_zero_when_every_example_passes() {
    assert_matches_rspec("runner/options", "passing", &["spec/money_spec.rb"]);
}

#[test]
fn test_spec_runs_the_default_path_given() {
    assert_matches_rspec(
        "runner/default_path",
        "default",
        &["--default-path", "checks"],
    );
}

#[test]
fn test_spec_reports_a_file_that_fails_to_load_and_exits_with_one() {
    assert_matches_rspec("errors/load_error", "progress", &[]);
}

#[test]
fn test_dot_rspec_local_overrides_dot_rspec() {
    assert_matches_rspec("runner/local_options", "default", &[]);
}

// RSpec loads spec_helper.rb only when an option file or a spec requires it.
// Rex loads it first whenever it is there, so this expectation is Rex's own.
#[test]
fn test_spec_loads_spec_helper_before_the_spec_files() {
    assert_matches_rspec("runner/spec_helper", "default", &[]);
}

/// `metorex FILE` run in the plain_script fixture, with the timing line
/// written with `N` and the status after it, as the expected file has them.
fn plain_script_transcript(file: &str) -> String {
    let directory = format!(
        "{}/{}/rex/runner/plain_script",
        env!("CARGO_MANIFEST_DIR"),
        super::EXAMPLES_DIR
    );
    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("exec \"$0\" \"$@\" 2>&1")
        .arg(env!("CARGO_BIN_EXE_metorex"))
        .arg(file)
        .current_dir(&directory)
        .env("SHELL", "/bin/zsh")
        .output()
        .expect("run metorex");
    let printed: String = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            if line.starts_with("Finished in ") && line.contains("(files took ") {
                "Finished in N seconds (files took N seconds to load)\n".to_string()
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    format!(
        "{}\nexit {}\n",
        printed.trim_end_matches('\n'),
        output.status.code().unwrap_or(-1)
    )
}

#[test]
fn test_a_script_requiring_rspec_autorun_runs_its_examples_as_it_ends() {
    assert_eq!(
        plain_script_transcript("ledger_autorun.rb"),
        expected("runner/plain_script", "expected_output.txt")
    );
}

#[test]
fn test_naming_rspec_in_a_script_loads_rex_and_runs_the_examples() {
    assert_eq!(
        plain_script_transcript("ledger_autoload.rb")
            .replace("ledger_autoload.rb", "ledger_autorun.rb"),
        expected("runner/plain_script", "expected_output.txt")
    );
}

#[test]
fn test_spec_init_writes_the_files_rspec_init_writes() {
    let project = std::env::temp_dir().join(format!("metorex_spec_init_{}", std::process::id()));
    std::fs::create_dir_all(&project).expect("make the project directory");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .args(["spec", "--init"])
        .current_dir(&project)
        .output()
        .expect("run metorex");
    let templates = format!(
        "{}/gems/rspec/rspec-core/lib/rspec/core/project_initializer",
        env!("CARGO_MANIFEST_DIR")
    );
    let written = |file: &str| {
        (
            std::fs::read_to_string(project.join(file)).expect("read the written file"),
            std::fs::read_to_string(format!("{templates}/{file}")).expect("read the template"),
        )
    };
    let (dot_rspec, rspec_dot_rspec) = written(".rspec");
    let (spec_helper, rspec_spec_helper) = written("spec/spec_helper.rb");
    std::fs::remove_dir_all(&project).expect("remove the project directory");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "  create   .rspec\n  create   spec/spec_helper.rb\n"
    );
    assert_eq!(dot_rspec, rspec_dot_rspec);
    assert_eq!(spec_helper, rspec_spec_helper);
}
