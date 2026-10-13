// The expectation protocol: `expect`, the `should` syntax a suite turns on,
// and a defined matcher that calls `expect` inside its match block.

use super::{expected, run_spec};

#[test]
fn test_should_syntax_and_expect_inside_a_defined_matcher() {
    let run = run_spec("expectations/syntax", &["--format", "documentation"]);
    assert_eq!(
        run.transcript(),
        expected("expectations/syntax", "expected_documentation.txt")
    );
}

#[test]
fn test_should_runs_with_a_deprecation_until_a_suite_configures_the_syntax() {
    let run = run_spec(
        "expectations/default_syntax",
        &["--format", "documentation"],
    );
    assert_eq!(
        run.transcript(),
        expected("expectations/default_syntax", "expected_documentation.txt")
    );
}
