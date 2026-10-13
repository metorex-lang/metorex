// The spec suite metorex.org's Rex page shows, with the output it prints
// there: RSpec's own for the documentation format, and Rex's for coverage.

use super::{expected, run_spec};

#[test]
fn test_the_site_ledger_suite_prints_what_rspec_prints() {
    let run = run_spec("site/ledger", &["--format", "documentation"]);
    assert_eq!(
        run.transcript(),
        expected("site/ledger", "expected_documentation.txt")
    );
}

#[test]
fn test_the_site_ledger_suite_reports_its_coverage() {
    let run = run_spec("site/ledger", &["--coverage"]);
    assert_eq!(
        run.transcript(),
        expected("site/ledger", "expected_coverage.txt")
    );
}
