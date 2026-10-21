// `metorex bundle`: runs Bundler, which Metorex carries. The command is
// Bundler's own `bundle` executable, so this only recognizes the subcommand
// and names the program it runs.

use std::path::Path;

/// The program `metorex bundle` runs, with the words after `bundle` as ARGV.
pub(crate) const PROGRAM: &str = "require \"bundler/metorex/bundle\"\n";

/// What `$0` and an error report call that program.
pub(crate) const PROGRAM_NAME: &str = "bundle";

/// Whether the command line asks for Bundler: `bundle` as the first word,
/// unless a script named `bundle` is there to run, which `metorex bundle`
/// then runs the way `ruby bundle` would.
pub(crate) fn wanted(arguments: &[String]) -> bool {
    arguments.get(1).is_some_and(|word| word == "bundle") && !Path::new("bundle").is_file()
}
