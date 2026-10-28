// Metorex CLI
// Command-line interface for the Metorex programming language

use clap::Parser as ClapParser;

mod bin_support;
mod bundle;
mod spec;

use bin_support::*;

/// The stack a deeply nested program such as mspec runs in.
const PROGRAM_STACK_BYTES: usize = 64 * 1024 * 1024;

fn main() {
    metorex::standard_streams::close_the_ones_closed_at_start();
    metorex::interpreter_path::interpreter_path();
    run_with_program_stack(real_main);
}

/// On Linux the program stays on the first thread, since a second thread
/// makes glibc's `setgroups` and `setuid` signal it to repeat the call, and
/// that signal crashed the process now and then. The first thread's stack is
/// sized by the stack limit, so a limit below what the program needs is
/// raised and the binary started again under it. Raising it without starting
/// again is not enough under Rosetta, which sizes the stack at start.
#[cfg(target_os = "linux")]
fn run_with_program_stack(program: fn()) {
    use std::os::unix::process::CommandExt;

    let mut limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let wanted = PROGRAM_STACK_BYTES as libc::rlim_t;
    // SAFETY: both calls read or write the one struct passed to them.
    let raised = unsafe {
        libc::getrlimit(libc::RLIMIT_STACK, &mut limit) == 0
            && limit.rlim_cur != libc::RLIM_INFINITY
            && limit.rlim_cur < wanted
            && limit.rlim_cur < limit.rlim_max
            && {
                limit.rlim_cur = wanted.min(limit.rlim_max);
                libc::setrlimit(libc::RLIMIT_STACK, &limit) == 0
            }
    };
    if raised && let Ok(binary) = std::env::current_exe() {
        let mut arguments = std::env::args_os();
        let mut command = std::process::Command::new(binary);
        if let Some(name) = arguments.next() {
            command.arg0(name);
        }
        // `exec` returns only when the binary could not be started again, and
        // the program then runs in the stack it has.
        let _ = command.args(arguments).exec();
    }
    program();
}

/// macOS fixes the first thread's stack when the process starts, so the
/// program runs on a thread of its own.
#[cfg(not(target_os = "linux"))]
fn run_with_program_stack(program: fn()) {
    std::thread::Builder::new()
        .stack_size(PROGRAM_STACK_BYTES)
        .spawn(program)
        .expect("Failed to spawn main thread")
        .join()
        .expect("Main thread panicked");
}

fn real_main() {
    metorex::vm::counters::enable_from_environment();
    metorex::vm::engine_report::enable_from_environment();
    metorex::vm::memory_report::enable_from_environment();
    // Ruby lets `-r`, `-I`, and `-W` carry their value attached (`-rfoo`),
    // which the argument parser only understands as two words.
    let mut arguments = interpreter_words(std::env::args_os().map(argument_text).collect());
    if spec::wanted(&arguments) {
        let mut cli = Cli::parse_from([arguments[0].clone()]);
        cli.file = arguments.split_off(2);
        run_inline(&cli, spec::PROGRAM, spec::PROGRAM_NAME);
    }
    if bundle::wanted(&arguments) {
        let mut cli = Cli::parse_from([arguments[0].clone()]);
        cli.file = arguments.split_off(2);
        run_inline(&cli, bundle::PROGRAM, bundle::PROGRAM_NAME);
    }
    if let Some(subcommand) = subcommands_wanted(&arguments) {
        run_subcommand(subcommand);
    }
    refuse_a_second_internal_encoding(&arguments);
    // RUBYOPT is read as though its words had been written on the command
    // line, ahead of what was, so a flag written there is overridden by the
    // same flag on the line itself.
    let from_rubyopt = rubyopt_arguments(&arguments);
    let required_by_rubyopt = from_rubyopt.iter().filter(|word| *word == "-r").count();
    arguments.splice(1..1, from_rubyopt);
    let mut cli = Cli::parse_from(arguments);
    // The libraries the command line names are required before those
    // RUBYOPT names.
    cli.require_libs.rotate_left(required_by_rubyopt);
    let features = Features::read(&cli);

    // Without a magic comment of its own, a source takes the setting the
    // program was started with. Said here rather than with the rest of the
    // flags, since a source is read before a VM is built for it.
    match features.frozen_string_literal {
        Some(true) => metorex::lexer::set_literal_default(metorex::lexer::LiteralDefault::Frozen),
        Some(false) => metorex::lexer::set_literal_default(metorex::lexer::LiteralDefault::Mutable),
        None => {}
    }
    // `-K` names the encoding every source is read as, so it has to be known
    // before any of them is read.
    if let Some(written) = &cli.source_encoding
        && let Some(named) = source_encoding_letter(written)
    {
        metorex::lexer::set_default_source_encoding(named);
    }

    // `-C` and `-X` both name the directory the rest of the run works from.
    if let Some(directory) = cli
        .working_directory
        .as_ref()
        .or(cli.working_directory_x.as_ref())
        && let Err(err) = std::env::set_current_dir(directory)
    {
        eprintln!("Error changing directory to '{}': {}", directory, err);
        process::exit(1);
    }

    // Ruby-compatible version output. `-v` also turns `$VERBOSE` on, so a
    // script or `-e` written alongside it still runs.
    if cli.ruby_version {
        println!("{}", metorex::ruby_description());
        if cli.file.is_empty() && cli.execute.is_empty() {
            return;
        }
    }

    // With no script named and no terminal to talk to, the program is what
    // arrives on standard input, the way `ruby < script.rb` runs it.
    if cli.file.is_empty()
        && cli.execute.is_empty()
        && !std::io::IsTerminal::is_terminal(&std::io::stdin())
    {
        let mut bytes = Vec::new();
        if let Err(err) = std::io::Read::read_to_end(&mut std::io::stdin(), &mut bytes) {
            eprintln!("Error reading standard input: {}", err);
            process::exit(1);
        }
        let Some(code) = metorex::file_loader::source_text(&bytes) else {
            eprintln!("-:1: invalid multibyte char (UTF-8)");
            process::exit(1);
        };
        run_inline(&cli, &code, "-");
    }

    // Evaluate inline code. Ruby joins the `-e` strings with newlines.
    if !cli.execute.is_empty() {
        let code = cli.execute.join("\n");
        run_inline(&cli, &code, "-e");
    }

    real_main_after_inline(cli);
}

/// Run code given on the command line or on standard input, under the name
/// a report and `__FILE__` give it, and exit with its status.
fn run_inline(cli: &Cli, code: &str, named: &str) -> ! {
    {
        if cli.check_syntax {
            check_syntax(code, named, cli);
        }
        // A magic comment names the encoding the code's literals are
        // written in. Without one, code typed at the terminal is taken to be
        // in the locale's encoding.
        // `-K` names one too, ahead of the locale's.
        let source_encoding = metorex::lexer::named_source_encoding(code)
            .or_else(metorex::lexer::command_line_source_encoding)
            .or_else(metorex::vm::locale_encoding_name);
        let lexer = Lexer::new(code).with_source_encoding(source_encoding.clone());
        let tokens = lexer.tokenize();
        let parsed = Parser::new(tokens).parse();
        let prism_refused = prism_refuses_main_script(code, cli);
        let mut vm = VirtualMachine::new();
        apply_cli_flags(&mut vm, cli);
        let errors = parsed.as_ref().err().cloned().unwrap_or_default();
        if let Some(messages) =
            refused_program_messages(&mut vm, code, named, prism_refused, &errors)
        {
            for message in messages {
                eprintln!("{}: {}", named, vm.syntax_error_report(&message, named));
            }
            process::exit(vm.run_at_exit_handlers(1, None));
        }
        let program = parsed.unwrap_or_default();
        // `--ast` dumps code given with `-e` the way it dumps a file.
        if cli.ast {
            for statement in &program {
                println!("{:#?}", statement);
            }
            process::exit(0);
        }
        // Code given on the command line is named `-e`, and code read from
        // standard input `-`, which is what a report and `__FILE__` say of it.
        vm.set_current_file(std::path::PathBuf::from(named));
        vm.set_source_encoding(source_encoding);
        vm.set_script_path(
            std::path::PathBuf::from(named),
            std::path::PathBuf::from(named),
        );
        // Names written after the code are the program's arguments, which is
        // where ARGF looks for the files to read.
        let named = if cli.switch_globals {
            take_switch_globals(&mut vm, cli.file.clone())
        } else {
            cli.file.clone()
        };
        vm.set_argv(named);
        if let Err(err) = run_program(&mut vm, &program, &line_loop_from(cli)) {
            finish_with_error(&mut vm, &err);
        }
        process::exit(vm.run_at_exit_handlers(0, None));
    }
}

/// Run what the command line names once inline code is ruled out: the REPL
/// or a script.
fn real_main_after_inline(cli: Cli) {
    // With no script named, the program is typed at the REPL.
    if cli.file.is_empty() {
        run_repl();
        return;
    }

    let named = &cli.file[0];
    let found = if cli.script_search {
        script_on_search_path(named)
    } else {
        None
    };
    let filename = found.as_ref().unwrap_or(named);
    let script_args: Vec<String> = cli.file[1..].to_vec();

    // Convert filename to absolute path
    let absolute_path = match fs::canonicalize(filename) {
        Ok(path) => path,
        // Ruby names a script it cannot open as a LoadError, with what the
        // operating system said about it.
        Err(err) => {
            let reason = err.to_string();
            let reason = reason.split(" (os error").next().unwrap_or(&reason);
            eprintln!(
                "{}: {reason} -- {filename} (LoadError)",
                bin_support::program_name()
            );
            process::exit(1);
        }
    };

    // Read the source file. Bytes that spell no character in the encoding
    // the file names are what Ruby reports as an invalid multibyte char,
    // naming the file and the first line.
    let source = match fs::read(&absolute_path) {
        Ok(bytes) => match metorex::file_loader::source_text(&bytes) {
            Some(content) => content,
            None => {
                eprintln!(
                    "{}:1: invalid multibyte char (UTF-8)",
                    absolute_path.display()
                );
                process::exit(1);
            }
        },
        Err(err) => {
            eprintln!("Error reading file '{}': {}", absolute_path.display(), err);
            process::exit(1);
        }
    };

    // `-x` runs the script inside another file, which opens at the first
    // line naming ruby. A file whose first line names another interpreter is
    // read the same way, whether `-x` was written or not.
    let source = match embedded_script(&source, cli.strip_leading_text) {
        Ok(held) => held,
        Err(message) => {
            eprintln!("{}: {}", filename, message);
            process::exit(1);
        }
    };

    // A line reading `__END__` closes the code, and the main script's data
    // after it is what the `DATA` constant reads.
    let (code, data_offset) = metorex::lexer::source_before_data_section(&source);
    let source = code.to_string();

    if cli.check_syntax {
        check_syntax(&source, filename, &cli);
    }

    if cli.debug {
        eprintln!("[debug] File: {}", absolute_path.display());
        eprintln!("[debug] Source length: {} bytes", source.len());
        eprintln!("[debug] Tokens: {}", Lexer::new(&source).tokenize().len());
    }

    // Prism checks the script before the VM comes up and the script is
    // parsed, so the chunks its check maps come on top of little more than
    // the source.
    let prism_refused = prism_refuses_main_script(&source, &cli);

    // The VM comes up before the script is parsed so that a library named by
    // `-r` is loaded either way, and the `at_exit` handlers it registered run
    // even when the script itself does not parse.
    let mut vm = VirtualMachine::new();
    apply_cli_flags(&mut vm, &cli);

    // Parse
    let mut parser = Parser::lexing(Lexer::new(&source));
    parser.hold_method_bodies();
    let parsed = parser.parse();
    let warnings = parser.default_warnings().to_vec();
    drop(parser);
    let errors = parsed.as_ref().err().cloned().unwrap_or_default();
    if let Some(messages) =
        refused_program_messages(&mut vm, &source, filename, prism_refused, &errors)
    {
        for message in messages {
            let report = vm.syntax_error_report(&message, filename);
            eprintln!("{}: {}", filename, report);
        }
        process::exit(vm.run_at_exit_handlers(1, None));
    }
    vm.report_parse_warnings(filename, &warnings);
    let program = parsed.unwrap_or_default();

    if cli.debug {
        eprintln!("[debug] Statements: {}", program.len());
    }

    // AST dump mode
    if cli.ast {
        for stmt in &program {
            println!("{:#?}", stmt);
        }
        return;
    }

    // Set the current file path and mark it as loaded
    vm.set_current_file(absolute_path.clone());
    // The main script's literals are in the encoding its magic comment names,
    // or else the one `-K` names.
    vm.set_source_encoding(
        metorex::lexer::named_source_encoding(&source)
            .or_else(metorex::lexer::command_line_source_encoding),
    );
    // `__FILE__` reports the path the script was named by on the command
    // line, while everything that resolves a path uses the canonical one.
    vm.set_script_path(absolute_path.clone(), std::path::PathBuf::from(filename));
    vm.mark_file_loaded(absolute_path.clone());
    let script_args = if cli.switch_globals {
        take_switch_globals(&mut vm, script_args)
    } else {
        script_args
    };
    vm.set_argv(script_args);
    if let Some(offset) = data_offset {
        define_data_constant(&mut vm, &absolute_path, offset);
    }

    if let Err(err) = run_program(&mut vm, &program, &line_loop_from(&cli)) {
        vm.trace_uncaught_error(&err);
        // `abort` and `exit` raise SystemExit: it ends the program with the
        // status it carries, having already reported anything it wanted to.
        if let metorex::error::MetorexError::UncaughtException { exception, .. } = &err
            && let metorex::object::View::Exception(exc) = exception.view()
            && exc.borrow().is_system_exit()
        {
            let status = exc.borrow().status.unwrap_or(0) as i32;
            let ending = exception.clone();
            process::exit(vm.run_at_exit_handlers(status, Some(ending)));
        }
        // A SignalException nothing caught ends the program the way the
        // signal itself would have, so the exit status names the signal
        // rather than a plain failure.
        if let metorex::error::MetorexError::UncaughtException { exception, .. } = &err
            && let metorex::object::View::Exception(exc) = exception.view()
            && let Some(number) = metorex::vm::signals::signal_number_of(&exc.borrow())
        {
            let ending = err_exception(&err);
            vm.run_at_exit_handlers(1, ending);
            // SAFETY: the default disposition is restored and the signal is
            // sent to this process, which then ends before returning.
            unsafe {
                libc::signal(number, libc::SIG_DFL);
                libc::raise(number);
            }
        }
        // The `at_exit` handlers run before the error is reported, so one
        // that calls `exit!` replaces both the report and the status.
        let status = vm.run_at_exit_handlers(1, err_exception(&err));
        report_error(&mut vm, &err);
        if let metorex::error::MetorexError::RuntimeError { stack_trace, .. } = &err
            && !stack_trace.is_empty()
        {
            eprintln!("Stack trace:");
            for frame in stack_trace {
                eprintln!("{}", frame);
            }
        }
        process::exit(status);
    }
    process::exit(vm.run_at_exit_handlers(0, None));
}

/// End the program the way the failure calls for: `exit` leaves with the
/// status it carries, a signal nothing caught ends the process the way the
/// signal itself would, and everything else is reported and leaves with 1.
fn finish_with_error(vm: &mut VirtualMachine, err: &metorex::error::MetorexError) -> ! {
    vm.trace_uncaught_error(err);
    if let metorex::error::MetorexError::UncaughtException { exception, .. } = err
        && let metorex::object::View::Exception(exc) = exception.view()
        && exc.borrow().is_system_exit()
    {
        let status = exc.borrow().status.unwrap_or(0) as i32;
        let ending = exception.clone();
        process::exit(vm.run_at_exit_handlers(status, Some(ending)));
    }
    let signalled = match err {
        metorex::error::MetorexError::UncaughtException { exception, .. }
            if let metorex::object::View::Exception(exc) = exception.view() =>
        {
            metorex::vm::signals::signal_number_of(&exc.borrow())
        }
        _ => None,
    };
    if let Some(number) = signalled {
        let ending = err_exception(err);
        vm.run_at_exit_handlers(1, ending);
        report_error(vm, err);
        // SAFETY: the default disposition is restored and the signal is sent
        // to this process, which then ends before returning.
        unsafe {
            libc::signal(number, libc::SIG_DFL);
            libc::raise(number);
        }
    }
    let status = vm.run_at_exit_handlers(1, err_exception(err));
    report_error(vm, err);
    process::exit(status);
}

/// Report a failure the way Ruby reports one: an exception nothing rescued
/// names where it was raised, what it says, and the class it is, and every
/// other failure is reported as itself.
fn report_error(vm: &mut VirtualMachine, err: &metorex::error::MetorexError) {
    if let Some(exception) = err_exception(err)
        && let Some(report) = vm.uncaught_report(&exception)
    {
        eprint!("{}", report);
        return;
    }
    eprintln!("Runtime error: {}", err);
}

/// The exception an error carries, which the `at_exit` handlers are told
/// about so one of them can report or replace it.
fn err_exception(err: &metorex::error::MetorexError) -> Option<metorex::object::Object> {
    match err {
        metorex::error::MetorexError::UncaughtException { exception, .. } => {
            Some(exception.clone())
        }
        _ => None,
    }
}

/// End the run when the command line names the internal encoding twice,
/// through `-U` and `-E`, as two different encodings. RUBYOPT is read apart
/// from this, so a `-U` written there gives way to `-E`.
fn refuse_a_second_internal_encoding(arguments: &[String]) {
    let mut internal: Option<String> = None;
    let mut words = arguments.iter().skip(1);
    while let Some(word) = words.next() {
        if word == "--" || word == "-" || !word.starts_with('-') {
            break;
        }
        let named = if word == "-U" {
            Some("UTF-8".to_string())
        } else if word == "-E" || word == "--encoding" {
            words
                .next()
                .and_then(|pair| pair.split_once(':'))
                .map(|(_, after)| after.to_string())
        } else if let Some(pair) = word.strip_prefix("--encoding=") {
            pair.split_once(':').map(|(_, after)| after.to_string())
        } else {
            if FLAGS_TAKING_A_VALUE.contains(&word.as_str()) {
                words.next();
            }
            None
        };
        let Some(named) = named.filter(|named| !named.is_empty()) else {
            continue;
        };
        if let Some(set) = &internal
            && !set.eq_ignore_ascii_case(&named)
        {
            eprintln!(
                "{}: default_internal already set to {} (RuntimeError)",
                program_name(),
                set
            );
            process::exit(1);
        }
        internal = Some(named);
    }
}

/// A command-line argument as text. One that is not UTF-8, such as code in
/// an encoding of its own passed to `-e`, holds its other bytes escaped.
fn argument_text(argument: std::ffi::OsString) -> String {
    match argument.into_string() {
        Ok(text) => text,
        Err(raw) => {
            use std::os::unix::ffi::OsStrExt;
            metorex::file_loader::escaped_source_text(raw.as_bytes())
        }
    }
}

/// The flags that take their value as the next word when it is not written
/// attached.
const FLAGS_TAKING_A_VALUE: &[&str] = &[
    "-e",
    "-r",
    "-I",
    "-C",
    "-X",
    "-F",
    "-E",
    "-K",
    "--encoding",
    "--external-encoding",
    "--internal-encoding",
    "--enable",
    "--disable",
    "--backtrace-limit",
];

/// The command line with each interpreter flag written the way the argument
/// parser reads it. The interpreter's flags end at `--` or at the first word
/// that is neither a flag nor a flag's value, which is the program's name,
/// and every word from there on is the program's own, left as written.
fn interpreter_words(raw: Vec<String>) -> Vec<String> {
    let mut written = Vec::with_capacity(raw.len());
    let mut words = raw.into_iter();
    written.extend(words.next());
    let mut takes_next_word = false;
    while let Some(word) = words.next() {
        if takes_next_word {
            written.push(word);
            takes_next_word = false;
            continue;
        }
        if word == "--" || word == "-" || !word.starts_with('-') {
            written.push(word);
            written.extend(words);
            break;
        }
        // `-x` written with a directory attached works from that directory.
        if let Some(directory) = word.strip_prefix("-x")
            && !directory.is_empty()
        {
            written.extend(["-x".to_string(), "-X".to_string(), directory.to_string()]);
            continue;
        }
        for piece in spelled_out_flag(word)
            .into_iter()
            .flat_map(metorex::split_short_flags)
        {
            takes_next_word = FLAGS_TAKING_A_VALUE.contains(&piece.as_str());
            written.push(piece);
        }
    }
    written
}
