//! Counting how often each line of a loaded file runs, which is what
//! `Coverage` reports.
//!
//! A file is measured only when it is loaded while measurement is on, the way
//! Ruby does it: the counting is set up as the file is read, so a file already
//! loaded is never counted.

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;

/// One measurement run: the lines of each file it has seen, counted where a
/// line carries something to run and left out where it does not.
#[derive(Default)]
pub(crate) struct CoverageRun {
    /// Whether the run reports each mode under its own name, which is what
    /// naming any mode outright asks for.
    by_mode: bool,
    /// The modes the run reports, in the order Ruby lists them.
    modes: Vec<String>,
    /// Whether code handed to `eval` under a measured file's name is counted
    /// into that file.
    pub(crate) eval_too: bool,
    /// Each file measured, by the path it was loaded under, with one entry per
    /// line of the file.
    files: Vec<(String, Vec<Option<u64>>)>,
    /// Every method defined in a measured file while the run was on, with the
    /// span it was written across and how often it has been called.
    methods: Vec<MethodSpan>,
    /// The branch sites of each file measured, by the path it was loaded
    /// under.
    branches: Vec<(String, FileBranches)>,
    /// For each file measured, the lines a statement starts on that Ruby
    /// counts on another line, where an assignment's value starts below it.
    moved: Vec<(String, std::collections::HashMap<usize, usize>)>,
}

/// A method as the `methods` mode reports it: where it sits, and how often it
/// has run.
/// The end a method made from a block is recorded with until `Coverage`
/// reads where the block closes.
const UNKNOWN_END: usize = usize::MAX;

/// A recorded end, or nil where it is not known yet.
fn end_place(place: usize) -> Object {
    if place == UNKNOWN_END {
        Object::Nil
    } else {
        Object::Int(place as i64)
    }
}

struct MethodSpan {
    file: String,
    owner: Object,
    name: String,
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
    calls: u64,
}

impl VirtualMachine {
    /// Start measuring, forgetting whatever an earlier run counted.
    pub(crate) fn coverage_start(&mut self, by_mode: bool, modes: Vec<String>, eval_too: bool) {
        // Machine code compiled while nothing was measured skips the
        // branch counts.
        crate::vm::jit::assumptions_changed();
        self.coverage = Some(CoverageRun {
            by_mode,
            modes,
            eval_too,
            files: Vec::new(),
            methods: Vec::new(),
            branches: Vec::new(),
            moved: Vec::new(),
        });
    }

    /// Whether coverage is being measured.
    pub(crate) fn coverage_running(&self) -> bool {
        self.coverage.is_some()
    }

    pub(crate) fn coverage_stop(&mut self) {
        self.coverage = None;
    }

    /// Whether code handed to `eval` is counted into the file it names.
    pub(crate) fn coverage_counts_eval(&self) -> bool {
        self.coverage.as_ref().is_some_and(|run| run.eval_too)
    }

    /// Begin counting a file that is being loaded now. Every line carrying
    /// something to run starts at zero, and every other line is left out.
    pub(crate) fn coverage_note_file(&mut self, path: &str, source: &str, body: &[Statement]) {
        if self.coverage.is_none() {
            return;
        }
        let counted = source.lines().count();
        let mut lines = vec![None; counted];
        FROZEN_STRINGS.set(crate::lexer::freezes_string_literals(source));
        let moved = mark(body, &mut lines);
        let mut sites = FileBranches::default();
        register_branches(body, &mut sites);
        if let Some(run) = self.coverage.as_mut() {
            run.files.retain(|(held, _)| held != path);
            run.files.push((path.to_string(), lines));
            run.branches.retain(|(held, _)| held != path);
            run.branches.push((path.to_string(), sites));
            run.moved.retain(|(held, _)| held != path);
            run.moved.push((path.to_string(), moved));
        }
    }

    /// Add the lines of code handed to `eval` under a file name, which `eval`
    /// coverage counts into that file. A name the run has not seen before
    /// becomes a file of its own, the way Ruby reports `eval("...", b, name)`.
    /// Note the lines of code evaluated into `path`, which runs through
    /// `last_line`, the line its last line is read as: the counts reach
    /// that far, whether or not a statement starts there, as a file's do.
    pub(crate) fn coverage_note_eval(&mut self, path: &str, body: &[Statement], last_line: usize) {
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        if !run.eval_too {
            return;
        }
        let mut wanted = Vec::new();
        collect(body, &mut wanted);
        let Some(highest) = wanted.iter().max().copied() else {
            return;
        };
        let highest = highest.max(last_line);
        if !run.files.iter().any(|(held, _)| held == path) {
            run.files.push((path.to_string(), Vec::new()));
        }
        let Some((_, lines)) = run.files.iter_mut().find(|(held, _)| held == path) else {
            return;
        };
        if lines.len() < highest {
            lines.resize(highest, None);
        }
        for line in wanted {
            if line >= 1 && lines[line - 1].is_none() {
                lines[line - 1] = Some(0);
            }
        }
    }

    /// Count one run of the line at `line` in the file being run now.
    pub(crate) fn coverage_count(&mut self, line: usize) {
        if line == 0 {
            return;
        }
        let Some(path) = self.current_source_file.clone() else {
            return;
        };
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        let line = run
            .moved
            .iter()
            .find(|(held, _)| *held == path)
            .and_then(|(_, moved)| moved.get(&line).copied())
            .unwrap_or(line);
        let Some((_, lines)) = run.files.iter_mut().find(|(held, _)| *held == path) else {
            return;
        };
        if let Some(Some(held)) = lines.get_mut(line - 1) {
            *held += 1;
        }
    }

    /// Count one run of arm `arm` of the branch site written at `position` in
    /// the file being run now.
    pub(crate) fn coverage_branch(&mut self, position: Position, arm: usize) {
        if self.coverage.is_none() {
            return;
        }
        let Some(path) = self.current_source_file.clone() else {
            return;
        };
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        let Some((_, file)) = run.branches.iter_mut().find(|(held, _)| *held == path) else {
            return;
        };
        let Some(&at) = file.by_offset.get(&(position.offset, position.line)) else {
            return;
        };
        if let Some(held) = file.sites[at].arms.get_mut(arm) {
            held.count += 1;
        }
    }

    /// Every branch site counted so far: for each file, each site's kind
    /// and line with each arm's kind, line, and count. Rex reads this to
    /// report branch coverage.
    pub(crate) fn coverage_branch_report(&self) -> Object {
        let Some(run) = self.coverage.as_ref() else {
            return Object::array(Vec::new());
        };
        Object::array(
            run.branches
                .iter()
                .map(|(path, file)| {
                    let sites = file
                        .sites
                        .iter()
                        .map(|site| {
                            let arms = site
                                .arms
                                .iter()
                                .map(|arm| {
                                    Object::array(vec![
                                        Object::symbol(arm.kind.to_string()),
                                        Object::Int(arm.line as i64),
                                        Object::Int(arm.count as i64),
                                    ])
                                })
                                .collect();
                            Object::array(vec![
                                Object::symbol(site.kind.to_string()),
                                Object::Int(site.line as i64),
                                Object::array(arms),
                            ])
                        })
                        .collect();
                    Object::array(vec![Object::string(path.clone()), Object::array(sites)])
                })
                .collect(),
        )
    }

    /// Record a method written in a measured file, which the `methods` mode
    /// reports whether or not it is ever called.
    pub(crate) fn coverage_note_method(
        &mut self,
        owner: Object,
        name: &str,
        start: crate::lexer::Position,
        end: crate::lexer::Position,
    ) {
        let Some(path) = self.current_source_file.clone() else {
            return;
        };
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        if !run.files.iter().any(|(held, _)| *held == path) {
            return;
        }
        run.methods.push(MethodSpan {
            file: path,
            owner,
            name: name.to_string(),
            start_line: (start.line as usize),
            // Ruby counts columns from zero, and the span runs to just past
            // the `end` that closes the definition.
            start_column: (start.column.saturating_sub(1) as usize),
            end_line: (end.line as usize),
            end_column: (end.column.saturating_sub(1) as usize) + "end".len(),
            calls: 0,
        });
    }

    /// Record a `def` written in a class or module body, keyed on the class
    /// or, for `def self.name`, on its singleton class.
    pub(crate) fn coverage_note_definition(
        &mut self,
        class: &std::rc::Rc<crate::class::Class>,
        on_singleton: bool,
        name: &str,
        start: crate::lexer::Position,
        end: crate::lexer::Position,
    ) {
        if self.coverage.is_none() {
            return;
        }
        let held = if class.is_module() {
            Object::Module(std::rc::Rc::clone(class))
        } else {
            Object::Class(std::rc::Rc::clone(class))
        };
        let owner = if on_singleton {
            Object::Class(self.singleton_class_of(&held))
        } else {
            held
        };
        self.coverage_note_method(owner, name, start, end);
    }

    /// Record a method `define_method` made from a block written in a
    /// measured file. The block's place is where it opens, and where it
    /// closes is left for `Coverage` to read from the file.
    pub(crate) fn coverage_note_block_method(
        &mut self,
        owner: Object,
        name: &str,
        file: &str,
        line: usize,
        column: usize,
    ) {
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        if !run.files.iter().any(|(held, _)| held == file) {
            return;
        }
        run.methods.push(MethodSpan {
            file: file.to_string(),
            owner,
            name: name.to_string(),
            start_line: line,
            start_column: column,
            end_line: UNKNOWN_END,
            end_column: UNKNOWN_END,
            calls: 0,
        });
    }

    /// Count one call of a method the run is watching.
    pub(crate) fn coverage_count_method(&mut self, file: &str, name: &str, line: usize) {
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        if let Some(held) = run
            .methods
            .iter_mut()
            .find(|held| held.file == file && held.name == name && held.start_line == line)
        {
            held.calls += 1;
        }
    }

    /// What the run has counted so far, as the Hash `Coverage.result` hands
    /// back. `clear` sets every count back to zero without forgetting which
    /// lines are countable.
    pub(crate) fn coverage_report(&mut self, clear: bool) -> Result<Object, MetorexError> {
        let Some(run) = self.coverage.as_mut() else {
            return Ok(Object::dict(Default::default()));
        };
        let by_mode = run.by_mode;
        let modes = run.modes.clone();
        let counts: Vec<(String, Vec<Option<u64>>)> = run.files.clone();
        let defined: Vec<(String, Object, String, [usize; 4], u64)> = run
            .methods
            .iter()
            .map(|held| {
                (
                    held.file.clone(),
                    held.owner.clone(),
                    held.name.clone(),
                    [
                        held.start_line,
                        held.start_column,
                        held.end_line,
                        held.end_column,
                    ],
                    held.calls,
                )
            })
            .collect();
        if clear {
            for (_, lines) in run.files.iter_mut() {
                for held in lines.iter_mut().filter(|held| held.is_some()) {
                    *held = Some(0);
                }
            }
            for held in run.methods.iter_mut() {
                held.calls = 0;
            }
            for (_, file) in run.branches.iter_mut() {
                for site in file.sites.iter_mut() {
                    for arm in site.arms.iter_mut() {
                        arm.count = 0;
                    }
                }
            }
        }

        let mut answer: crate::object::HashEntries = Default::default();
        for (path, lines) in &counts {
            let counted = Object::array(
                lines
                    .iter()
                    .map(|held| match held {
                        Some(times) => Object::Int(*times as i64),
                        None => Object::Nil,
                    })
                    .collect(),
            );
            let value = if by_mode {
                let mut held: crate::object::HashEntries = Default::default();
                for mode in &modes {
                    let named = format!(":{mode}");
                    match mode.as_str() {
                        "lines" => held.insert(named.into(), counted.clone()),
                        "methods" => held.insert(named.into(), self.methods_in(&defined, path)?),
                        _ => held.insert(named.into(), Object::dict(Default::default())),
                    };
                }
                Object::dict(held)
            } else {
                counted
            };
            answer.insert(path.clone().into(), value);
        }
        Ok(Object::dict(answer))
    }

    /// The methods written in one file, keyed the way the `methods` mode names
    /// them: owner, name, and the span the definition covers.
    fn methods_in(
        &mut self,
        defined: &[(String, Object, String, [usize; 4], u64)],
        path: &str,
    ) -> Result<Object, MetorexError> {
        let mut entries: crate::object::HashEntries = Default::default();
        let mut key_objects: crate::object::HashEntries = Default::default();
        for (_, owner, name, span, calls) in defined.iter().filter(|held| held.0 == path) {
            let key = Object::array(vec![
                owner.clone(),
                Object::symbol(name.clone()),
                Object::Int(span[0] as i64),
                Object::Int(span[1] as i64),
                end_place(span[2]),
                end_place(span[3]),
            ]);
            let slot = self.dict_slot_in(&entries, &key, false, Position::new(0, 0, 0))?;
            key_objects.insert(slot.clone(), key);
            entries.insert(slot, Object::Int(*calls as i64));
        }
        if !key_objects.is_empty() {
            // An Array key is not read back from its text, so the key object
            // itself travels beside the entry.
            entries.insert(
                "__MX_KEY_OBJECTS__".into(),
                Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(key_objects))),
            );
        }
        Ok(Object::dict(entries))
    }
}

/// The lines `Coverage.line_stub` answers for `source`: 0 where a line
/// carries something to run and nil elsewhere, read without running it.
/// None where the source does not parse.
pub(crate) fn line_stub(source: &str) -> Option<Vec<Option<u64>>> {
    if crate::vm::VirtualMachine::prism_refuses_program(source, "") {
        return None;
    }
    let tokens = crate::lexer::Lexer::new(source).tokenize();
    let body = crate::parser::Parser::new(tokens).parse().ok()?;
    let mut lines = vec![None; source.lines().count()];
    mark(&body, &mut lines);
    Some(lines)
}

/// Mark every line of `body` that carries something to run, leaving a line
/// already marked at the count it has.
fn mark(body: &[Statement], lines: &mut [Option<u64>]) -> std::collections::HashMap<usize, usize> {
    let mut wanted = Vec::new();
    let mut moved = std::collections::HashMap::new();
    collect_used(body, &mut wanted, true, &mut moved);
    for line in wanted {
        if line >= 1 && line <= lines.len() && lines[line - 1].is_none() {
            lines[line - 1] = Some(0);
        }
    }
    moved
}

/// The lines of `body` that carry something to run.
fn collect(body: &[Statement], found: &mut Vec<usize>) {
    collect_used(body, found, true, &mut std::collections::HashMap::new());
}

/// The lines of `body` that carry something to run. `used` says whether
/// the value of the body's last statement is used, which decides whether a
/// literal standing there runs at all: Ruby drops a literal whose value goes
/// nowhere, and it is no line of its own then.
fn collect_used(
    body: &[Statement],
    found: &mut Vec<usize>,
    used: bool,
    moved: &mut std::collections::HashMap<usize, usize>,
) {
    let last = body.len().saturating_sub(1);
    for (at, statement) in body.iter().enumerate() {
        let value_used = used && at == last;
        // `begin` is not a line of its own in Ruby's counting: what runs is
        // the body inside it.
        if !matches!(statement, Statement::Begin { .. } | Statement::Block { .. })
            && (value_used || !is_plain_literal(statement))
        {
            // An assignment whose value starts on a line below it is
            // counted on the value's line.
            let line = statement.position().line as usize;
            let counted_on = assigned_value_line(statement).filter(|held| *held > line);
            if let Some(below) = counted_on {
                moved.insert(line, below);
            }
            found.push(counted_on.unwrap_or(line));
        }
        // An `elsif` line counts as its condition is tested.
        if let Statement::If { elsif_branches, .. } = statement {
            found.extend(
                elsif_branches
                    .iter()
                    .map(|branch| branch.position.line as usize),
            );
        }
        // A method, class or module body hands its last value on, and a
        // loop's body hands nothing on. Any other body's value is used as
        // far as the statement holding it is.
        let inner_used = match statement {
            Statement::ClassDef { .. }
            | Statement::ModuleDef { .. }
            | Statement::MethodDef { .. }
            | Statement::FunctionDef { .. } => true,
            Statement::While { .. } | Statement::DoWhile { .. } | Statement::For { .. } => false,
            _ => value_used,
        };
        for inner in nested_bodies(statement) {
            collect_used(inner, found, inner_used, moved);
        }
        let mut arm_lines = Vec::new();
        for inner in nested_blocks(statement, &mut arm_lines) {
            collect_used(inner, found, true, moved);
        }
        found.extend(arm_lines);
    }
}

thread_local! {
    /// Whether the file being read freezes its string literals, which
    /// decides whether an array or hash holding strings is copied whole.
    static FROZEN_STRINGS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The line the value of an assignment, or of an expression standing as a
/// statement, starts on.
fn assigned_value_line(statement: &Statement) -> Option<usize> {
    match statement {
        // An attribute or index target reads what it is set on first.
        Statement::Assignment { target, value, .. } => match &**target {
            Expression::MethodCall { .. } | Expression::Index { .. } => {
                Some(start_line(target).min(start_line(value)))
            }
            _ => Some(start_line(value)),
        },
        Statement::MultipleAssignment { values, .. } => values.first().map(start_line),
        Statement::Expression { expression, .. } => Some(start_line(expression)),
        _ => None,
    }
}

/// Whether Ruby builds a value once, ahead of time, and copies it where it
/// stands: a literal, or an array or hash of them. A string is one only
/// where the file freezes its string literals, while a hash's string keys
/// always are.
fn is_static_value(expression: &Expression) -> bool {
    match expression {
        Expression::StringLiteral { .. } => FROZEN_STRINGS.get(),
        // A string literal in a file that freezes them.
        Expression::MethodCall { receiver, method, .. } if method == "__frozen_literal__" => {
            matches!(**receiver, Expression::StringLiteral { .. })
        }
        Expression::Array { elements, .. } => elements.iter().all(is_static_value),
        Expression::Dictionary { entries, .. } => entries.iter().all(|(key, value)| {
            (matches!(key, Expression::StringLiteral { .. }) || is_static_value(key))
                && is_static_value(value)
        }),
        _ => is_literal_expression(expression),
    }
}

/// The line an expression starts on, which for a call or an operator is
/// where what it is made on starts.
fn start_line(expression: &Expression) -> usize {
    let own = expression.position().line as usize;
    let inner = match expression {
        Expression::MethodCall { receiver, .. } => Some(start_line(receiver)),
        Expression::BinaryOp { left, .. } => Some(start_line(left)),
        Expression::Index { array, .. } => Some(start_line(array)),
        Expression::Range { start, .. } => Some(start_line(start)),
        // An array or hash literal holding only literals is copied whole
        // where it starts. Any other is counted from its first element,
        // which is where the work of building it starts.
        Expression::Array { .. } | Expression::Dictionary { .. }
            if is_static_value(expression) =>
        {
            None
        }
        Expression::Array { elements, .. } => {
            return elements.first().map_or(own, start_line);
        }
        Expression::Dictionary { entries, .. } => {
            return entries.first().map_or(own, |(key, _)| start_line(key));
        }
        _ => None,
    };
    inner.map_or(own, |held| held.min(own))
}

/// Whether a statement is nothing but a literal, which Ruby compiles to
/// nothing where its value is not used.
fn is_plain_literal(statement: &Statement) -> bool {
    let Statement::Expression { expression, .. } = statement else {
        return false;
    };
    is_literal_expression(expression)
}

fn is_literal_expression(expression: &Expression) -> bool {
    match expression {
        Expression::IntLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BoolLiteral { .. }
        | Expression::NilLiteral { .. }
        | Expression::Symbol { .. }
        | Expression::BigIntLiteral { .. } => true,
        Expression::MethodCall { receiver, method, .. } if method == "__frozen_literal__" => {
            matches!(**receiver, Expression::StringLiteral { .. })
        }
        Expression::Array { elements, .. } => elements.iter().all(is_literal_expression),
        Expression::Dictionary { entries, .. } => entries
            .iter()
            .all(|(key, value)| is_literal_expression(key) && is_literal_expression(value)),
        _ => false,
    }
}

/// The statement bodies a statement holds, which carry lines of their own.
pub(crate) fn nested_bodies(statement: &Statement) -> Vec<&[Statement]> {
    match statement {
        Statement::ClassDef { body, .. }
        | Statement::ModuleDef { body, .. }
        | Statement::MethodDef { body, .. }
        | Statement::FunctionDef { body, .. } => vec![&body[..]],
        Statement::While { body, .. }
        | Statement::DoWhile { body, .. }
        | Statement::For { body, .. } => vec![body.as_slice()],
        Statement::Block { statements, .. } => vec![statements.as_slice()],
        Statement::If {
            then_branch,
            elsif_branches,
            else_branch,
            ..
        } => {
            let mut held: Vec<&[Statement]> = vec![then_branch.as_slice()];
            for branch in elsif_branches {
                held.push(branch.body.as_slice());
            }
            if let Some(otherwise) = else_branch {
                held.push(otherwise.as_slice());
            }
            held
        }
        Statement::Unless {
            then_branch,
            else_branch,
            ..
        } => {
            let mut held: Vec<&[Statement]> = vec![then_branch.as_slice()];
            if let Some(otherwise) = else_branch {
                held.push(otherwise.as_slice());
            }
            held
        }
        Statement::Match { cases, .. } | Statement::CaseIn { cases, .. } => {
            cases.iter().map(|case| case.body.as_slice()).collect()
        }
        Statement::Begin {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            ..
        } => {
            let mut held: Vec<&[Statement]> = vec![body.as_slice()];
            for clause in rescue_clauses {
                held.push(clause.body.as_slice());
            }
            if let Some(otherwise) = else_clause {
                held.push(otherwise.as_slice());
            }
            if let Some(ensured) = ensure_block {
                held.push(ensured.as_slice());
            }
            held
        }
        _ => Vec::new(),
    }
}

/// The bodies of blocks written on a statement's own expressions, which run
/// line by line the same way.
fn nested_blocks<'a>(statement: &'a Statement, arm_lines: &mut Vec<usize>) -> Vec<&'a [Statement]> {
    let mut found = Vec::new();
    for expression in own_expressions(statement) {
        gather_blocks(expression, &mut found, arm_lines);
    }
    found
}

pub(crate) fn own_expressions(statement: &Statement) -> Vec<&Expression> {
    match statement {
        Statement::Expression { expression, .. } => vec![expression],
        Statement::Match { expression, .. } => vec![expression],
        Statement::Assignment { target, value, .. } => vec![&**target, &**value],
        Statement::MultipleAssignment {
            targets, values, ..
        } => targets.iter().chain(values.iter()).collect(),
        Statement::Return {
            value: Some(held), ..
        }
        | Statement::Raise {
            exception: Some(held),
            ..
        } => vec![held],
        _ => Vec::new(),
    }
}

fn gather_blocks<'a>(
    expression: &'a Expression,
    found: &mut Vec<&'a [Statement]>,
    arm_lines: &mut Vec<usize>,
) {
    match expression {
        Expression::Lambda {
            parameter_defaults,
            body,
            ..
        } => {
            for (_, default) in parameter_defaults.iter() {
                gather_blocks(default, found, arm_lines);
            }
            found.push(&body[..]);
        }
        Expression::SingletonClass { target, body, .. } => {
            gather_blocks(target, found, arm_lines);
            found.push(&body[..]);
        }
        Expression::BeginRescue {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            ..
        } => {
            found.push(body.as_slice());
            for clause in rescue_clauses {
                found.push(clause.body.as_slice());
            }
            for held in else_clause.iter().chain(ensure_block.iter()) {
                found.push(held.as_slice());
            }
        }
        Expression::BinaryOp { left, right, .. } => {
            gather_blocks(left, found, arm_lines);
            gather_blocks(right, found, arm_lines);
        }
        Expression::UnaryOp { operand, .. } => gather_blocks(operand, found, arm_lines),
        Expression::PatternTest { value, .. } => gather_blocks(value, found, arm_lines),
        Expression::Call {
            callee,
            arguments,
            trailing_block,
            ..
        } => {
            gather_blocks(callee, found, arm_lines);
            for argument in arguments {
                gather_blocks(argument, found, arm_lines);
            }
            if let Some(block) = trailing_block {
                gather_blocks(block, found, arm_lines);
            }
        }
        Expression::MethodCall {
            receiver,
            arguments,
            trailing_block,
            ..
        } => {
            gather_blocks(receiver, found, arm_lines);
            for argument in arguments {
                gather_blocks(argument, found, arm_lines);
            }
            if let Some(block) = trailing_block {
                gather_blocks(block, found, arm_lines);
            }
        }
        Expression::Super {
            arguments,
            trailing_block,
            ..
        } => {
            for argument in arguments {
                gather_blocks(argument, found, arm_lines);
            }
            if let Some(block) = trailing_block {
                gather_blocks(block, found, arm_lines);
            }
        }
        Expression::Yield { arguments, .. }
        | Expression::Array {
            elements: arguments,
            ..
        } => {
            for argument in arguments {
                gather_blocks(argument, found, arm_lines);
            }
        }
        Expression::Index { array, index, .. } => {
            gather_blocks(array, found, arm_lines);
            gather_blocks(index, found, arm_lines);
        }
        Expression::Dictionary { entries, .. } => {
            for (key, value) in entries {
                gather_blocks(key, found, arm_lines);
                gather_blocks(value, found, arm_lines);
            }
        }
        Expression::Grouped { expression, .. }
        | Expression::Splat { expression, .. }
        | Expression::KeywordSplat { expression, .. }
        | Expression::BlockArg { expression, .. }
        | Expression::Defined { expression, .. } => gather_blocks(expression, found, arm_lines),
        Expression::Range { start, end, .. } => {
            gather_blocks(start, found, arm_lines);
            gather_blocks(end, found, arm_lines);
        }
        Expression::InterpolatedString { parts, .. } => {
            for part in parts {
                if let crate::ast::InterpolationPart::Expression(held) = part {
                    gather_blocks(held, found, arm_lines);
                }
            }
        }
        Expression::If {
            condition,
            then_branch,
            elsif_branches,
            else_branch,
            ..
        } => {
            gather_blocks(condition, found, arm_lines);
            found.push(then_branch.as_slice());
            for branch in elsif_branches.iter() {
                gather_blocks(&branch.condition, found, arm_lines);
                found.push(branch.body.as_slice());
            }
            if let Some(body) = else_branch {
                found.push(body.as_slice());
            }
        }
        Expression::Unless {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            gather_blocks(condition, found, arm_lines);
            found.push(then_branch.as_slice());
            if let Some(body) = else_branch {
                found.push(body.as_slice());
            }
        }
        Expression::Case {
            expression,
            cases,
            else_case,
            ..
        } => {
            gather_blocks(expression, found, arm_lines);
            // An arm written as one expression is a line of its own.
            for case in cases {
                if let Some(guard) = &case.guard {
                    gather_blocks(guard, found, arm_lines);
                }
                if !matches!(case.body, Expression::BeginRescue { .. }) {
                    arm_lines.push(case.body.position().line as usize);
                }
                gather_blocks(&case.body, found, arm_lines);
            }
            if let Some(held) = else_case {
                if !matches!(**held, Expression::BeginRescue { .. }) {
                    arm_lines.push(held.position().line as usize);
                }
                gather_blocks(held, found, arm_lines);
            }
        }
        _ => {}
    }
}

/// One place a run can go more than one way: an `if`, `unless`, `case`,
/// `while`, or `&.`, with how often each of its arms ran.
#[derive(Clone)]
pub(crate) struct BranchSite {
    pub(crate) kind: &'static str,
    pub(crate) line: usize,
    pub(crate) arms: Vec<BranchArm>,
}

#[derive(Clone)]
pub(crate) struct BranchArm {
    pub(crate) kind: &'static str,
    pub(crate) line: usize,
    pub(crate) count: u64,
}

/// The branch sites written in one file, keyed by where each starts.
#[derive(Default, Clone)]
pub(crate) struct FileBranches {
    pub(crate) sites: Vec<BranchSite>,
    pub(crate) by_offset: std::collections::HashMap<(u32, u32), usize>,
}

impl FileBranches {
    fn add(&mut self, kind: &'static str, position: Position, arms: Vec<(&'static str, usize)>) {
        let key = (position.offset, position.line);
        if self.by_offset.contains_key(&key) {
            return;
        }
        self.by_offset.insert(key, self.sites.len());
        self.sites.push(BranchSite {
            kind,
            line: (position.line as usize),
            arms: arms
                .into_iter()
                .map(|(kind, line)| BranchArm {
                    kind,
                    line,
                    count: 0,
                })
                .collect(),
        });
    }
}

/// The line an arm's first statement is written on, or `fallback` for an
/// arm with nothing in it.
fn first_line(body: &[Statement], fallback: usize) -> usize {
    body.first()
        .map(|statement| statement.position().line as usize)
        .unwrap_or(fallback)
}

fn if_arms(
    then_branch: &[Statement],
    elsif_branches: &[crate::ast::ElsifBranch],
    else_branch: &Option<Vec<Statement>>,
    line: usize,
) -> Vec<(&'static str, usize)> {
    let mut arms = vec![("then", first_line(then_branch, line))];
    for branch in elsif_branches {
        arms.push((
            "elsif",
            first_line(&branch.body, branch.position.line as usize),
        ));
    }
    arms.push((
        "else",
        else_branch
            .as_deref()
            .map_or(line, |body| first_line(body, line)),
    ));
    arms
}

/// Every branch site in `body`, at any depth.
pub(crate) fn register_branches(body: &[Statement], found: &mut FileBranches) {
    for statement in body {
        branch_statement(statement, found);
    }
}

fn branch_statement(statement: &Statement, found: &mut FileBranches) {
    match statement {
        Statement::Expression { expression, .. } => branch_expression(expression, found),
        Statement::Assignment { target, value, .. } => {
            branch_expression(target, found);
            branch_expression(value, found);
        }
        Statement::MultipleAssignment {
            targets, values, ..
        } => {
            for held in targets.iter().chain(values) {
                branch_expression(held, found);
            }
        }
        Statement::While {
            condition,
            body,
            position,
        }
        | Statement::DoWhile {
            condition,
            body,
            position,
        } => {
            found.add(
                "while",
                *position,
                vec![("body", first_line(body, position.line as usize))],
            );
            branch_expression(condition, found);
            register_branches(body, found);
        }
        Statement::For { iterable, body, .. } => {
            branch_expression(iterable, found);
            register_branches(body, found);
        }
        Statement::BeginBlock { body, .. } => register_branches(body, found),
        Statement::FunctionDef {
            parameters, body, ..
        }
        | Statement::MethodDef {
            parameters, body, ..
        } => {
            for parameter in parameters {
                if let Some(default) = &parameter.default_value {
                    branch_expression(default, found);
                }
            }
            register_branches(body, found);
        }
        Statement::ClassDef {
            namespace,
            superclass_expression,
            body,
            ..
        } => {
            for held in namespace.iter().chain(superclass_expression.iter()) {
                branch_expression(held, found);
            }
            register_branches(body, found);
        }
        Statement::ModuleDef { body, .. } => register_branches(body, found),
        Statement::Block {
            statements: body, ..
        } => register_branches(body, found),
        Statement::If {
            condition,
            then_branch,
            elsif_branches,
            else_branch,
            position,
        } => {
            found.add(
                "if",
                *position,
                if_arms(
                    then_branch,
                    elsif_branches,
                    else_branch,
                    position.line as usize,
                ),
            );
            branch_expression(condition, found);
            register_branches(then_branch, found);
            for branch in elsif_branches {
                branch_expression(&branch.condition, found);
                register_branches(&branch.body, found);
            }
            if let Some(body) = else_branch {
                register_branches(body, found);
            }
        }
        Statement::Unless {
            condition,
            then_branch,
            else_branch,
            position,
        } => {
            found.add(
                "unless",
                *position,
                vec![
                    ("then", first_line(then_branch, position.line as usize)),
                    (
                        "else",
                        else_branch
                            .as_deref()
                            .map_or(position.line as usize, |body| {
                                first_line(body, position.line as usize)
                            }),
                    ),
                ],
            );
            branch_expression(condition, found);
            register_branches(then_branch, found);
            if let Some(body) = else_branch {
                register_branches(body, found);
            }
        }
        Statement::Match {
            expression,
            cases,
            position,
        }
        | Statement::CaseIn {
            expression,
            cases,
            position,
        } => {
            let pattern_kind = if matches!(statement, Statement::Match { .. }) {
                "when"
            } else {
                "in"
            };
            let mut arms: Vec<(&'static str, usize)> = cases
                .iter()
                .map(|case| {
                    if matches!(case.pattern, crate::ast::MatchPattern::Wildcard) {
                        ("else", first_line(&case.body, case.position.line as usize))
                    } else {
                        (
                            pattern_kind,
                            first_line(&case.body, case.position.line as usize),
                        )
                    }
                })
                .collect();
            let has_else = cases
                .last()
                .is_some_and(|case| matches!(case.pattern, crate::ast::MatchPattern::Wildcard));
            if !has_else {
                arms.push(("else", (position.line as usize)));
            }
            found.add("case", *position, arms);
            branch_expression(expression, found);
            for case in cases {
                if let Some(guard) = &case.guard {
                    branch_expression(guard, found);
                }
                register_branches(&case.body, found);
            }
        }
        Statement::Return {
            value: Some(held), ..
        }
        | Statement::Raise {
            exception: Some(held),
            ..
        } => branch_expression(held, found),
        Statement::Break {
            value: Some(held), ..
        }
        | Statement::Continue {
            value: Some(held), ..
        } => branch_expression(held, found),
        Statement::Begin {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            ..
        } => {
            register_branches(body, found);
            for clause in rescue_clauses {
                register_branches(&clause.body, found);
            }
            for held in else_clause.iter().chain(ensure_block.iter()) {
                register_branches(held, found);
            }
        }
        _ => {}
    }
}

fn branch_expression(expression: &Expression, found: &mut FileBranches) {
    match expression {
        Expression::BinaryOp { left, right, .. } => {
            branch_expression(left, found);
            branch_expression(right, found);
        }
        Expression::UnaryOp { operand, .. } => branch_expression(operand, found),
        Expression::PatternTest { value, .. } => branch_expression(value, found),
        Expression::Call {
            callee,
            arguments,
            trailing_block,
            ..
        } => {
            branch_expression(callee, found);
            for argument in arguments {
                branch_expression(argument, found);
            }
            if let Some(block) = trailing_block {
                branch_expression(block, found);
            }
        }
        Expression::MethodCall {
            receiver,
            method,
            arguments,
            trailing_block,
            position,
        } => {
            if method == crate::parser::SAFE_CALL {
                found.add(
                    "&.",
                    *position,
                    vec![
                        ("then", (position.line as usize)),
                        ("else", (position.line as usize)),
                    ],
                );
            }
            branch_expression(receiver, found);
            for argument in arguments {
                branch_expression(argument, found);
            }
            if let Some(block) = trailing_block {
                branch_expression(block, found);
            }
        }
        Expression::Super {
            arguments,
            trailing_block,
            ..
        } => {
            for argument in arguments {
                branch_expression(argument, found);
            }
            if let Some(block) = trailing_block {
                branch_expression(block, found);
            }
        }
        Expression::Yield { arguments, .. }
        | Expression::Array {
            elements: arguments,
            ..
        } => {
            for argument in arguments {
                branch_expression(argument, found);
            }
        }
        Expression::Index { array, index, .. } => {
            branch_expression(array, found);
            branch_expression(index, found);
        }
        Expression::Dictionary { entries, .. } => {
            for (key, value) in entries {
                branch_expression(key, found);
                branch_expression(value, found);
            }
        }
        Expression::Lambda {
            parameter_defaults,
            body,
            ..
        } => {
            for (_, default) in parameter_defaults.iter() {
                branch_expression(default, found);
            }
            register_branches(body, found);
        }
        Expression::Grouped { expression, .. }
        | Expression::Splat { expression, .. }
        | Expression::KeywordSplat { expression, .. }
        | Expression::BlockArg { expression, .. }
        | Expression::Defined { expression, .. } => branch_expression(expression, found),
        Expression::ScopeResolution { namespace, .. } => branch_expression(namespace, found),
        Expression::SingletonClass { target, body, .. } => {
            branch_expression(target, found);
            register_branches(body, found);
        }
        Expression::Range { start, end, .. } => {
            branch_expression(start, found);
            branch_expression(end, found);
        }
        Expression::InterpolatedString { parts, .. } => {
            for part in parts {
                if let crate::ast::InterpolationPart::Expression(held) = part {
                    branch_expression(held, found);
                }
            }
        }
        Expression::BeginRescue {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            ..
        } => {
            register_branches(body, found);
            for clause in rescue_clauses {
                register_branches(&clause.body, found);
            }
            for held in else_clause.iter().chain(ensure_block.iter()) {
                register_branches(held, found);
            }
        }
        Expression::If {
            condition,
            then_branch,
            elsif_branches,
            else_branch,
            position,
        } => {
            found.add(
                "if",
                *position,
                if_arms(
                    then_branch,
                    elsif_branches,
                    else_branch,
                    position.line as usize,
                ),
            );
            branch_expression(condition, found);
            register_branches(then_branch, found);
            for branch in elsif_branches.iter() {
                branch_expression(&branch.condition, found);
                register_branches(&branch.body, found);
            }
            if let Some(body) = else_branch {
                register_branches(body, found);
            }
        }
        Expression::Unless {
            condition,
            then_branch,
            else_branch,
            position,
        } => {
            found.add(
                "unless",
                *position,
                vec![
                    ("then", first_line(then_branch, position.line as usize)),
                    (
                        "else",
                        else_branch
                            .as_deref()
                            .map_or(position.line as usize, |body| {
                                first_line(body, position.line as usize)
                            }),
                    ),
                ],
            );
            branch_expression(condition, found);
            register_branches(then_branch, found);
            if let Some(body) = else_branch {
                register_branches(body, found);
            }
        }
        Expression::Case {
            expression,
            cases,
            else_case,
            position,
        } => {
            let mut arms: Vec<(&'static str, usize)> = cases
                .iter()
                .map(|case| ("when", case.body.position().line as usize))
                .collect();
            arms.push((
                "else",
                else_case
                    .as_ref()
                    .map_or(position.line as usize, |held| held.position().line as usize),
            ));
            found.add("case", *position, arms);
            branch_expression(expression, found);
            for case in cases {
                if let Some(guard) = &case.guard {
                    branch_expression(guard, found);
                }
                branch_expression(&case.body, found);
            }
            if let Some(held) = else_case {
                branch_expression(held, found);
            }
        }
        _ => {}
    }
}
