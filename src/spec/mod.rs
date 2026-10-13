// `metorex spec`: runs a suite with Rex, the spec framework Metorex carries.
// The runner is Rex's own Ruby, so this only recognizes the subcommand and
// names the program it runs.

use std::path::Path;

/// The program `metorex spec` runs, with the words after `spec` as ARGV.
pub(crate) const PROGRAM: &str = "require \"rex\"\nexit Rex::Core::Runner.run(ARGV)\n";

/// What `$0` and an error report call that program.
pub(crate) const PROGRAM_NAME: &str = "metorex spec";

/// Whether the command line asks for the spec runner: `spec` as the first
/// word, unless a script named `spec` is there to run, which `metorex spec`
/// then runs the way `ruby spec` would.
pub(crate) fn wanted(arguments: &[String]) -> bool {
    arguments.get(1).is_some_and(|word| word == "spec") && !Path::new("spec").is_file()
}
