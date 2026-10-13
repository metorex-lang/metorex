// `metorex spec` run against fixture suites under tests/_examples/rex. Each
// expected file is the output of RSpec 3.13 for the same fixture, with the
// timing line written as `Finished in N seconds (files took N seconds to load)`.

pub mod common;

mod rex {
    mod config;
    mod coverage;
    mod diffs;
    mod dsl;
    mod errors;
    mod expectations;
    mod formatters;
    mod hooks;
    mod matchers;
    mod metadata;
    mod mocks;
    mod ordering;
    mod parallel;
    mod performance;
    mod runner;

    pub(crate) use crate::common::EXAMPLES_DIR;
    use std::process::Command;

    /// What one `metorex spec` run printed and the status it exited with.
    pub(crate) struct SpecRun {
        pub(crate) output: String,
        /// The output with only the `Finished in` line written with `N`.
        pub(crate) output_with_times: String,
        pub(crate) status: i32,
    }

    impl SpecRun {
        /// The run as scripts/rex_expected.sh writes RSpec's: the output with
        /// trailing newlines cut to one, then `exit STATUS`.
        pub(crate) fn transcript(&self) -> String {
            format!(
                "{}\nexit {}\n",
                self.output.trim_end_matches('\n'),
                self.status
            )
        }
    }

    /// Runs `metorex spec` with `arguments` in the fixture directory named
    /// by `fixture`, reading stdout and stderr as one stream, with the
    /// checkout's directory written as `ROOT`, as scripts/rex_expected.sh
    /// writes it.
    pub(crate) fn run_spec(fixture: &str, arguments: &[&str]) -> SpecRun {
        run_spec_with_env(fixture, arguments, &[])
    }

    /// `run_spec` with environment variables set for the run.
    pub(crate) fn run_spec_with_env(
        fixture: &str,
        arguments: &[&str],
        environment: &[(&str, &str)],
    ) -> SpecRun {
        let directory = format!(
            "{}/{}/rex/{}",
            env!("CARGO_MANIFEST_DIR"),
            EXAMPLES_DIR,
            fixture
        );
        // One pipe for stdout and stderr keeps their lines in the order
        // they were written, as the expected files have them.
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg("exec \"$0\" spec \"$@\" 2>&1")
            .arg(env!("CARGO_BIN_EXE_metorex"))
            .args(arguments)
            .current_dir(&directory)
            .env_remove("SPEC_OPTS")
            // RSpec quotes an example id for zsh and not for bash.
            .env("SHELL", "/bin/zsh")
            .envs(environment.iter().copied())
            .output()
            .expect("failed to run metorex spec");
        let text =
            String::from_utf8_lossy(&output.stdout).replace(env!("CARGO_MANIFEST_DIR"), "ROOT");
        SpecRun {
            output: normalize_timing(&text),
            output_with_times: text
                .lines()
                .map(|line| {
                    if line.starts_with("Finished in ") && line.contains("(files took ") {
                        "Finished in N seconds (files took N seconds to load)"
                    } else {
                        line
                    }
                })
                .map(|line| line.to_string() + "\n")
                .collect(),
            status: output.status.code().unwrap_or(-1),
        }
    }

    /// The expected output file `name` in the fixture directory.
    pub(crate) fn expected(fixture: &str, name: &str) -> String {
        let path = format!(
            "{}/{}/rex/{}/{}",
            env!("CARGO_MANIFEST_DIR"),
            EXAMPLES_DIR,
            fixture,
            name
        );
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing {path}"))
    }

    /// The timing line written as `Finished in N seconds (files took N
    /// seconds to load)`, each `--bisect` duration as `(N seconds)` or
    /// `in N seconds.`, and each object address as `0x`, since these change
    /// from run to run.
    fn normalize_timing(text: &str) -> String {
        text.lines()
            .map(|line| {
                if line.starts_with("Finished in ") && line.contains("(files took ") {
                    "Finished in N seconds (files took N seconds to load)".to_string()
                } else {
                    crate::without_html_durations(&crate::without_profile_times(
                        &crate::without_json_timings(&crate::without_bisect_durations(
                            &crate::without_addresses(line),
                        )),
                    ))
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + if text.ends_with('\n') { "\n" } else { "" }
    }
}

/// An HTML report line with each example duration and the total written
/// as `N`.
fn without_html_durations(line: &str) -> String {
    const SPANS: [(&str, &str); 3] = [
        ("<span class='duration'>", "s</span>"),
        ("<span class=\"duration\">", "s</span>"),
        ("<strong>", " seconds</strong>"),
    ];
    let mut out = line.to_string();
    for (open, close) in SPANS {
        if let Some(start) = out.find(open) {
            let after = start + open.len();
            if let Some(length) = out[after..].find(close)
                && length > 0
                && out[after..after + length]
                    .chars()
                    .all(|character| character.is_ascii_digit() || character == '.')
            {
                out.replace_range(after..after + length, "N");
            }
        }
    }
    out
}

/// A `--profile` line with its times and percentage written as `N`.
fn without_profile_times(line: &str) -> String {
    let is_number =
        |text: &str| !text.is_empty() && text.chars().all(|c| c.is_ascii_digit() || c == '.');
    if let Some(rest) = line.strip_prefix("Top ")
        && let Some((count, rest)) = rest.split_once(" slowest examples (")
        && let Some(rest) = rest.strip_suffix("% of total time):")
        && let Some((seconds, percent)) = rest.split_once(" seconds, ")
        && is_number(count)
        && is_number(seconds)
        && is_number(percent)
    {
        return format!("Top {count} slowest examples (N seconds, N% of total time):");
    }
    if let Some(rest) = line.strip_prefix("    ")
        && let Some((seconds, rest)) = rest.split_once(" seconds ")
        && is_number(seconds)
    {
        if rest.starts_with("./") {
            return format!("    N seconds {rest}");
        }
        if let Some(rest) = rest.strip_prefix("average (")
            && let Some((total, rest)) = rest.split_once(" seconds / ")
            && let Some((count, location)) = rest.split_once(") ")
            && is_number(total)
            && location.starts_with("./")
        {
            return format!("    N seconds average (N seconds / {count}) {location}");
        }
    }
    line.to_string()
}

/// `line` with each JSON duration and run time written as `N` and the
/// version as `V`.
fn without_json_timings(line: &str) -> String {
    const TIMED_KEYS: [&str; 6] = [
        "\"duration\":",
        "\"run_time\":",
        "\"slowest\":",
        "\"total\":",
        "\"average\":",
        "\"total_time\":",
    ];
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    'scan: while !rest.is_empty() {
        for key in TIMED_KEYS {
            if let Some(after) = rest.strip_prefix(key) {
                let digits = after
                    .find(|character: char| {
                        !(character.is_ascii_digit() || matches!(character, '.' | '-' | 'e'))
                    })
                    .unwrap_or(after.len());
                if digits > 0 {
                    out.push_str(key);
                    out.push('N');
                    rest = &after[digits..];
                    continue 'scan;
                }
            }
        }
        if let Some(after) = rest.strip_prefix("\"version\":\"")
            && let Some(close) = after.find('"')
        {
            out.push_str("\"version\":\"V\"");
            rest = &after[close + 1..];
            continue;
        }
        let character = rest.chars().next().expect("rest is not empty");
        out.push(character);
        rest = &rest[character.len_utf8()..];
    }
    out
}

/// `line` with each `(DURATION seconds)` written as `(N seconds)`, and a
/// closing ` in DURATION seconds.` as ` in N seconds.`.
fn without_bisect_durations(line: &str) -> String {
    let is_duration = |text: &str| {
        !text.is_empty()
            && text
                .chars()
                .all(|character| character.is_ascii_digit() || character == '.')
    };
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(open) = rest.find('(') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let replaced = after.find(')').and_then(|close| {
            let inside = &after[..close];
            let number = inside
                .strip_suffix(" seconds")
                .or_else(|| inside.strip_suffix(" second"))?;
            is_duration(number).then_some(close)
        });
        match replaced {
            Some(close) => {
                out.push_str("(N seconds)");
                rest = &after[close + 1..];
            }
            None => {
                out.push('(');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    let closing_duration = out
        .strip_suffix(" seconds.")
        .or_else(|| out.strip_suffix(" second."))
        .and_then(|stem| stem.rfind(" in ").map(|at| (stem, at)))
        .filter(|(stem, at)| is_duration(&stem[at + 4..]));
    if let Some((stem, at)) = closing_duration {
        return format!("{} in N seconds.", &stem[..at]);
    }
    out
}

/// `line` with each `0x` followed by eight to sixteen hex digits written as
/// `0x`.
fn without_addresses(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let bytes = line.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'0' && bytes.get(at + 1) == Some(&b'x') {
            let digits = bytes[at + 2..]
                .iter()
                .take_while(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                .count();
            if (8..=16).contains(&digits) {
                out.push_str("0x");
                at += 2 + digits;
                continue;
            }
        }
        out.push(bytes[at] as char);
        at += 1;
    }
    out
}

/// Every Ruby file of Rex's is registered with the libraries the binary
/// carries, so `require` finds each one without a directory on disk.
#[test]
fn test_every_rex_source_file_is_carried_in_the_binary() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/vm/stdlib");
    let mut pending = vec![root.join("rex")];
    let mut names = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("rex directory") {
            let path = entry.expect("rex entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rb") {
                let relative = path.strip_prefix(&root).expect("under stdlib");
                names.push(relative.with_extension("").to_string_lossy().to_string());
            }
        }
    }
    names.sort();
    let program = names
        .iter()
        // A file built on a gem, as the rake task is on rake, may find the
        // gem missing, which is not a file Rex failed to carry.
        .map(|name| {
            format!(
                "begin\n  require \"{name}\"\nrescue LoadError => error\n  raise if error.message.include?(\"rex/\")\nend"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .args([
            "-e",
            &format!("require \"rex\"\n{program}\nputs \"loaded\""),
        ])
        .output()
        .expect("run metorex");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "loaded\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The specs under spec/ describe interpreter behavior in Rex's own terms,
/// and each passes under MRI's rspec as well.
#[test]
fn test_the_specs_under_spec_pass_under_metorex_spec() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .args(["spec", "--no-color"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env_remove("SPEC_OPTS")
        .output()
        .expect("run metorex spec");
    let printed = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && printed.contains(" examples, 0 failures"),
        "{printed}{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `require "rspec"` and each of its siblings load Rex, which `RSpec` names.
/// `rspec/autorun` also runs the examples as the program ends, which the
/// plain_script fixture covers.
#[test]
fn test_the_rspec_names_load_rex() {
    let program = ["rspec", "rspec/core", "rspec/expectations", "rspec/mocks"]
        .iter()
        .map(|name| format!("p(require(\"{name}\") || true)"))
        .collect::<Vec<_>>()
        .join("\n");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .args(["-e", &format!("{program}\np(RSpec.equal?(Rex))")])
        .output()
        .expect("run metorex");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "true\ntrue\ntrue\ntrue\ntrue\n",
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
