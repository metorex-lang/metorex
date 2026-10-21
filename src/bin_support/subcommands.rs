// `metorex help`, `metorex version`, `metorex repl`, and `metorex test`: the
// words a command line can start with in place of a script, as `metorex spec`
// does.

use super::*;
use clap::CommandFactory;

/// The directory `metorex test` runs when no other is named.
const DEFAULT_TEST_DIRECTORY: &str = "test";

pub(crate) enum Subcommand {
    Help,
    Version,
    Repl,
    Test(String),
}

/// The subcommand the first word names, unless a script by that name is there
/// to run, which then runs the way `ruby help` would run it.
pub(crate) fn subcommands_wanted(arguments: &[String]) -> Option<Subcommand> {
    let word = arguments.get(1)?;
    if Path::new(word).is_file() {
        return None;
    }
    match word.as_str() {
        "help" => Some(Subcommand::Help),
        "version" => Some(Subcommand::Version),
        "repl" => Some(Subcommand::Repl),
        "test" => Some(Subcommand::Test(
            arguments
                .get(2)
                .cloned()
                .unwrap_or_else(|| DEFAULT_TEST_DIRECTORY.to_string()),
        )),
        _ => None,
    }
}

pub(crate) fn run_subcommand(subcommand: Subcommand) -> ! {
    match subcommand {
        Subcommand::Help => {
            let _ = Cli::command().print_long_help();
            process::exit(0);
        }
        Subcommand::Version => {
            println!("metorex {}", env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }
        Subcommand::Repl => {
            run_repl();
            process::exit(0);
        }
        Subcommand::Test(directory) => {
            match test_discovery::run_test_discovery(Path::new(&directory)) {
                Ok(result) if result.all_passed() => process::exit(0),
                Ok(_) => process::exit(1),
                Err(err) => {
                    eprintln!("Test discovery error: {}", err);
                    process::exit(1);
                }
            }
        }
    }
}

/// Start the REPL and exit with status 1 if it cannot start or fails.
pub(crate) fn run_repl() {
    match Repl::new() {
        Ok(mut repl) => {
            if let Err(err) = repl.run() {
                eprintln!("REPL error: {}", err);
                process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("Failed to initialize REPL: {}", err);
            process::exit(1);
        }
    }
}
