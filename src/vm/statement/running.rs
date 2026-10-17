// Running one statement, and the statements of a scope.

use super::*;
use crate::object::{OptionView, View};

/// How long a thread runs before it hands the turn to the others, which is
/// how often MRI's timer marks a turn as spent.
const TIME_SLICE: std::time::Duration = std::time::Duration::from_millis(100);
/// How many statements run between two reads of the clock for the time slice.
const STATEMENTS_BETWEEN_CLOCK_READS: u64 = 64;

impl VirtualMachine {
    /// Counts a statement, and hands the turn over once the thread has held
    /// it for a time slice.
    pub(crate) fn pass_checkpoint(
        &mut self,
        position: crate::lexer::Position,
    ) -> Result<(), MetorexError> {
        self.statements_run = self.statements_run.wrapping_add(1);
        // Reading the clock costs more than a statement does, so the time
        // slice is checked every so many statements rather than at each.
        if self
            .statements_run
            .is_multiple_of(STATEMENTS_BETWEEN_CLOCK_READS)
            && self.turn_started.elapsed() >= TIME_SLICE
        {
            self.turn_started = std::time::Instant::now();
            self.share_the_turn(position)?;
        }
        if !self.timeout_limits.is_empty() {
            self.raise_expired_timeout(position)?;
        }
        Ok(())
    }

    /// What happens before each statement runs: a `:line`
    /// event for whatever is tracing, which costs a check on an empty list
    /// when nothing is, then signals, a collection when one is due, the
    /// turn passed to another thread when this one's time is up, and the
    /// line's coverage.
    pub(crate) fn begin_statement(
        &mut self,
        position: crate::lexer::Position,
        fires_line_event: bool,
    ) -> Result<(), MetorexError> {
        if fires_line_event && !self.tracepoints.is_empty() {
            self.fire_line_event(position)?;
        }
        self.deliver_pending_signals(position)?;
        if crate::object::live::collection_due() {
            self.collect_garbage(position)?;
        }
        self.pass_checkpoint(position)?;
        let line = position.line;
        if self.coverage_skip_line.take() != Some(line) && self.coverage.is_some() {
            self.coverage_count(line);
        }
        Ok(())
    }

    /// Run a statement alone, compiled, answering how it ended.
    pub(crate) fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, MetorexError> {
        let flow = self.run_statement(statement)?;
        // A local assigned inside a branch or a loop is a local from there
        // on whether or not the assignment ran, so a block written later
        // closes over it.
        if matches!(
            statement,
            Statement::If { .. }
                | Statement::Unless { .. }
                | Statement::While { .. }
                | Statement::DoWhile { .. }
                | Statement::For { .. }
                | Statement::Match { .. }
                | Statement::CaseIn { .. }
                | Statement::Begin { .. }
        ) {
            for name in crate::ast::collect_assigned_locals(std::slice::from_ref(statement)) {
                self.environment_mut().unhoist(&name);
            }
        }
        Ok(flow)
    }

    /// Define what a `def`, `alias`, `include`, or `extend` statement names,
    /// which evaluates nothing the program wrote: a method's body runs when
    /// the method is called.
    pub(crate) fn run_definition(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, MetorexError> {
        match statement {
            Statement::FunctionDef {
                name,
                parameters,
                body,
                position,
                singleton_class,
                end_position,
            } => self.execute_function_def(
                name,
                parameters,
                body,
                *position,
                singleton_class.as_deref(),
                end_position.line,
            ),
            Statement::Include {
                module_name,
                position,
            } => self.execute_include(module_name, *position),
            Statement::Extend {
                module_name,
                position,
            } => self.execute_extend(module_name, *position),
            Statement::Alias {
                new_name,
                old_name,
                position,
            } => self.execute_alias(new_name, old_name, *position),
            Statement::MethodDef {
                name,
                parameters,
                body,
                is_class_method,
                position,
                ..
            } => {
                // Run from a method, such as in a default value, it installs
                // on the class the method was written in.
                let from_method_body = self.running_method_def_scope();
                let Some(target) = from_method_body
                    .clone()
                    .or_else(|| self.def_scope_stack.last().cloned())
                    .or_else(|| {
                        self.method_nesting_stack
                            .last()
                            .and_then(|nesting| nesting.first().cloned())
                    })
                else {
                    return Err(unimplemented_statement_error(statement));
                };
                let _ = (parameters, body);
                // The definition runs as the one statement of a class body
                // would, with no class variable home of the code around it.
                if self.coverage.is_some() {
                    let line = statement.position().line;
                    self.coverage_count(line);
                    self.coverage_skip_line = Some(line);
                }
                let held_home = std::mem::take(&mut self.class_var_home);
                let mut last_value = Object::Nil;
                let defined =
                    self.class_body_statement(&target, statement, *position, &mut last_value);
                self.class_var_home = held_home;
                defined?;
                // Run from a method body, it is public whatever visibility
                // the class body left in force.
                if from_method_body.is_some() && !is_class_method {
                    target.clear_method_visibility(name);
                }
                Ok(ControlFlow::Value(Object::symbol(name.clone())))
            }
            other => Err(unimplemented_statement_error(other)),
        }
    }

    /// A `return` written at the top level ends the file, and the value
    /// written after it goes nowhere, which Ruby says so.
    pub(crate) fn warn_top_level_return_value(&mut self, position: crate::lexer::Position) {
        if !self.call_stack.is_empty() {
            return;
        }
        let written_in = self
            .current_source_file
            .clone()
            .or_else(|| {
                self.reported_current_file()
                    .map(|path| path.display().to_string())
            })
            .unwrap_or_else(|| "-".to_string());
        let notice = format!(
            "{}: warning: argument of top-level return is ignored",
            written_in
        );
        self.emit_warning_to_stderr(&notice, position);
    }

    /// Execute statements within a new lexical scope.
    /// Ruby reports a constant given a second value, naming the class or
    /// module the constant is bound on.
    pub(crate) fn warn_already_initialized(
        &mut self,
        owner: &Rc<crate::class::Class>,
        name: &str,
        position: crate::lexer::Position,
    ) {
        // `$VERBOSE = nil` silences warnings of every kind.
        if matches!(self.globals().get("VERBOSE").view(), Some(View::Nil)) {
            return;
        }
        let named = owner.ruby_name();
        let named = if named.is_empty() {
            owner.inspect_name()
        } else {
            named
        };
        // A constant at the top level is bound on Object, which Ruby leaves
        // out of the name it reports.
        let message = if named == "Object" {
            format!("warning: already initialized constant {}", name)
        } else {
            format!("warning: already initialized constant {}::{}", named, name)
        };
        self.emit_warning_to_stderr(&message, position);
    }
}
