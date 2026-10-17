//! Compiling a body's statements to a `Unit`. A body needing more registers
//! than an instruction names is refused with the type name of the node that
//! ran out.

use crate::ast::{BinaryOp, Expression, InterpolationPart, Statement, UnaryOp};
use crate::bytecode::instruction::{
    BINARY_OPERATORS, FIRST_CONSTANT_OPERAND, Instruction, RANGE_EXCLUSIVE, RANGE_LITERAL_ENDS,
};
use crate::bytecode::unit::{
    CallSite, CaseArm, Definition, ErrorSpot, Handler, HandlerKind, HashEntry, HashShape,
    MultipleAssignment, Name, OpenAtHandler, PatternArm, Piece, Receiver, RegexLiteral,
    RescueClause, StartKind, Target, Unit,
};
use crate::lexer::Position;
use crate::object::Object;

/// The register the body's value is left in.
const ANSWER: usize = 0;

/// The registers an instruction can name without an `Extend`.
const REGISTER_LIMIT: usize = 256;

/// How many elements or pairs of a long Array or Hash literal are built at
/// a time.
const LITERAL_CHUNK: usize = 64;

/// How many holes of a long interpolated String are joined at a time.
const INTERPOLATION_CHUNK: usize = 32;

/// What a register or table index past what an operand holds refuses with,
/// which the node being compiled is reported in place of.
const OVER_LIMIT: &str = "over limit";

/// Report `node` for a refusal over a limit.
fn naming(node: &'static str) -> impl Fn(&'static str) -> &'static str {
    move |refused| if refused == OVER_LIMIT { node } else { refused }
}

/// Compile `body`, or name the first node type it holds that the compiler
/// does not handle.
pub fn compile(body: &[Statement]) -> Result<Unit, &'static str> {
    compile_as(body, UnitKind::Plain)
}

/// What a body is the body of, which decides how some of its statements
/// run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    /// A block's body or a file's top level.
    Plain,
    /// A method's body: its last statement answers the method's value, an
    /// `if` or `unless` there running as an `if` written as an expression.
    Method,
    /// A `class` body: its statements follow the class body's rules.
    ClassBody,
    /// A `module` body: its statements follow the module body's rules.
    ModuleBody,
}

/// Compile `body`, the body of what `kind` names.
pub fn compile_as(body: &[Statement], kind: UnitKind) -> Result<Unit, &'static str> {
    let of_a_method = kind == UnitKind::Method;
    let mut compiler = Compiler::for_body(body);
    let end = match body.last() {
        Some(last) => last.position(),
        None => Position::default(),
    };
    // Every `BEGIN` body runs before the rest of the unit, in the order they
    // were written, and shares the unit's own scope.
    for statement in body {
        if let Statement::BeginBlock { body, position } = statement {
            compiler
                .begin_block(body, *position)
                .map_err(naming(statement.kind_name()))?;
        }
    }
    match body.split_last() {
        Some((last, before)) if of_a_method && answers_its_own_value(last) => {
            if !before.is_empty() {
                compiler.statements(before, ANSWER, true)?;
            }
            let start = compiler.unit.instructions.len();
            compiler
                .terminal_statement(last, ANSWER)
                .map_err(naming(last.kind_name()))?;
            compiler.unit.start_statement(start, last.position());
            compiler.unit.set_start_kind(
                start,
                StartKind::Terminal {
                    line_event: !matches!(last, Statement::Begin { .. }),
                },
            );
        }
        _ if matches!(kind, UnitKind::ClassBody | UnitKind::ModuleBody) => {
            if body.is_empty() {
                compiler.emit(Instruction::LoadNil { to: ANSWER as u8 }, end);
            }
            for statement in body {
                let start = compiler.unit.instructions.len();
                let held = compiler.next_register;
                let runs = compiler
                    .body_statement(statement, kind == UnitKind::ModuleBody)
                    .map_err(naming(statement.kind_name()))?;
                compiler.next_register = held;
                compiler.unit.start_statement(start, statement.position());
                compiler
                    .unit
                    .set_start_kind(start, StartKind::BodyStatement { runs });
            }
        }
        _ => compiler.statements(body, ANSWER, true)?,
    }
    compiler.emit(Instruction::Return { from: ANSWER as u8 }, end);
    Ok(compiler.unit)
}

/// Compile `expression` alone, as code that runs where something other
/// than a unit's own statements asks for its value: a parameter's default,
/// a value a pattern compares against. It starts no statement of its own.
pub fn compile_expression(expression: &Expression) -> Result<Unit, &'static str> {
    let position = expression.position();
    let mut compiler = Compiler::for_body(&[Statement::Expression {
        expression: expression.clone(),
        position,
    }]);
    compiler.expression(expression, ANSWER)?;
    compiler.emit(Instruction::Return { from: ANSWER as u8 }, position);
    Ok(compiler.unit)
}

/// Compile `statement` alone, as a unit of one statement.
pub fn compile_statement(statement: &Statement) -> Result<Unit, &'static str> {
    let body = std::slice::from_ref(statement);
    let mut compiler = Compiler::for_body(body);
    compiler.statements(body, ANSWER, true)?;
    compiler.emit(
        Instruction::Return { from: ANSWER as u8 },
        statement.position(),
    );
    Ok(compiler.unit)
}

struct Compiler {
    unit: Unit,
    /// The first register no expression being compiled holds.
    next_register: usize,
    /// The error spot of each expression being compiled, the innermost last.
    spots: Vec<Option<ErrorSpot>>,
    /// The locals the body assigns, which the scope reserves on entry.
    assigned: Vec<String>,
    /// The loops being compiled, the innermost last.
    loops: Vec<Loop>,
    /// What the code compiled so far has opened and not yet closed.
    pending: Pending,
    /// The regions being compiled that something has to run on the way out
    /// of, the innermost last.
    protections: Vec<Protection>,
    /// How many `begin`s the code being compiled is inside.
    begins: usize,
}

/// What a run has open at a place in the code: argument lists, call routes,
/// scopes `case` arms and `Block` statements opened, and loops. A jump out of
/// the place drops them.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Pending {
    lists: usize,
    routes: usize,
    scopes: usize,
    loops: usize,
}

/// A region of code that something has to run on the way out of, by a jump
/// or an exit from the unit as much as by an error.
#[derive(Debug, Clone)]
enum Protection {
    /// The statements of an `ensure` clause.
    Ensure {
        body: Vec<Statement>,
        pending: Pending,
    },
    /// The body of a rescue clause, which puts `$!` back to what the
    /// register `standing` holds as it is left, and whose `retry` runs its
    /// `begin` again from `retry_target`.
    RescueBody {
        standing: usize,
        retry_target: usize,
        pending: Pending,
    },
    /// An `ensure` clause running while an error is on its way out, in the
    /// expression form of `begin`: leaving it by a jump drops the error, and
    /// `$!` goes back to what `standing` holds.
    DroppedError { standing: usize, pending: Pending },
    /// The arms of a `case`/`in`, which let go of the value they asked for
    /// its elements as they are left.
    Deconstructing { pending: Pending },
}

/// A loop being compiled.
struct Loop {
    /// The register the loop answers in, which a `break` writes.
    to: usize,
    /// Where `redo` goes: the start of the body.
    redo_target: usize,
    /// The jumps `break` makes, to the end of the loop.
    breaks: Vec<usize>,
    /// The jumps `next` makes, to where the condition is read.
    nexts: Vec<usize>,
    /// What was open where the loop began.
    pending: Pending,
    /// How many protections stood where the loop began.
    protections: usize,
}

impl Compiler {
    /// A compiler for `body`, with a slot for each local it assigns.
    fn for_body(body: &[Statement]) -> Compiler {
        let mut compiler = Compiler {
            unit: Unit::new(),
            next_register: ANSWER + 1,
            spots: Vec::new(),
            assigned: crate::ast::collect_assigned_locals(body),
            loops: Vec::new(),
            pending: Pending::default(),
            protections: Vec::new(),
            begins: 0,
        };
        compiler.unit.use_registers(ANSWER + 1);
        for name in compiler.assigned.clone() {
            compiler.unit.local(&name);
        }
        compiler
    }

    /// Add an instruction, with the spot of the innermost expression being
    /// compiled that has one.
    fn emit(&mut self, instruction: Instruction, position: Position) -> usize {
        match instruction {
            Instruction::StartArguments => self.pending.lists += 1,
            Instruction::EnterLoop => self.pending.loops += 1,
            Instruction::LeaveLoop => self.pending.loops -= 1,
            Instruction::ArgumentsToArray { .. } => self.pending.lists -= 1,
            Instruction::ResolveCall { .. } | Instruction::ResolveScopedCall { .. } => {
                self.pending.routes += 1;
            }
            _ => {}
        }
        if let Instruction::Call { site, .. }
        | Instruction::CallResolved { site, .. }
        | Instruction::CallNativeName { site, .. }
        | Instruction::CallBlock { site, .. }
        | Instruction::CallSuper { site, .. } = instruction
        {
            if self.unit.call_sites[usize::from(site)].arguments_in_list {
                self.pending.lists -= 1;
            }
            if matches!(instruction, Instruction::CallResolved { .. }) {
                self.pending.routes -= 1;
            }
        }
        let at = self.unit.emit(instruction, position);
        self.unit.error_spots[at] = self.spots.iter().rev().flatten().next().cloned();
        at
    }

    /// A register no expression being compiled holds.
    fn temporary(&mut self) -> Result<usize, &'static str> {
        let register = self.next_register;
        if register >= REGISTER_LIMIT {
            return Err(OVER_LIMIT);
        }
        self.next_register += 1;
        self.unit.use_registers(self.next_register);
        Ok(register)
    }

    /// A register to compute into ahead of the registers after it: `to`
    /// itself when it is the last one held, so a chain such as `a.b.c`
    /// takes no more registers for each call, or else a new one.
    fn register_after(&mut self, to: usize) -> Result<usize, &'static str> {
        match to + 1 == self.next_register {
            true => Ok(to),
            false => self.temporary(),
        }
    }

    /// Compile a statement of a class body, or of a module body when
    /// `module`, by the rules the body runs it with, answering whether it
    /// runs as a statement of its own, with a line event, rather than being
    /// taken up by the body.
    fn body_statement(
        &mut self,
        statement: &Statement,
        module: bool,
    ) -> Result<bool, &'static str> {
        let unchanged = self.temporary()?;
        let position = statement.position();
        if let Statement::Expression {
            expression: Expression::Identifier { name, .. },
            ..
        } = statement
            && (matches!(
                name.as_str(),
                "private" | "public" | "protected" | "module_function"
            ) || (!module && name == "freeze"))
        {
            self.body_statement_as_written(statement, ANSWER)?;
            return Ok(false);
        }
        match statement {
            Statement::MethodDef { .. } | Statement::Include { .. } | Statement::Extend { .. } => {
                self.body_statement_as_written(statement, ANSWER)?;
                Ok(false)
            }
            Statement::Alias { new_name, .. } if module && !new_name.starts_with('$') => {
                self.body_statement_as_written(statement, ANSWER)?;
                Ok(false)
            }
            Statement::Assignment {
                target: target @ Expression::Identifier { name, .. },
                value,
                ..
            } if starts_uppercase(name) => {
                let decided = self.assigned_value(target, value, ANSWER)?;
                let name = self.name(name)?;
                self.emit(
                    Instruction::BodyConstant {
                        from: ANSWER as u8,
                        name,
                    },
                    position,
                );
                if let Some(decided) = decided {
                    self.jump_here(decided)?;
                }
                Ok(false)
            }
            _ if module => self.module_body_statement(statement, unchanged),
            _ => self.class_body_statement(statement, unchanged),
        }
    }

    /// The statements of a class body that follow the class body's own
    /// rules, past those it shares with a module body.
    fn class_body_statement(
        &mut self,
        statement: &Statement,
        unchanged: usize,
    ) -> Result<bool, &'static str> {
        let position = statement.position();
        match statement {
            Statement::FunctionDef { .. }
            | Statement::Expression {
                expression: Expression::InstanceVariable { .. },
                ..
            } => {
                self.body_statement_as_written(statement, ANSWER)?;
                Ok(false)
            }
            Statement::Alias { new_name, .. } if !new_name.starts_with('$') => {
                self.body_statement_as_written(statement, ANSWER)?;
                Ok(false)
            }
            // `class << target` opens the target's singleton class whatever
            // the target is written as, and the body answers what it did.
            Statement::Expression {
                expression:
                    Expression::SingletonClass {
                        target,
                        body,
                        position: opened_at,
                        ..
                    },
                ..
            } => {
                let held = self.temporary()?;
                self.expression(target, held)?;
                self.define(
                    Definition::SingletonClass {
                        target: held,
                        body: body.clone(),
                        position: *opened_at,
                    },
                    ANSWER,
                    *opened_at,
                )?;
                Ok(false)
            }
            // A `Block` in a class body defines the accessors of its
            // `attr_accessor` statements, and nothing else in it runs.
            Statement::Block { statements, .. } => {
                for inner in statements {
                    if let Statement::AttrAccessor { attributes, .. } = inner {
                        let names = self.attribute_names(attributes, position)?;
                        self.emit(
                            Instruction::BodyBlockAccessors { names: names as u8 },
                            position,
                        );
                    }
                }
                Ok(false)
            }
            Statement::AttrReader { attributes, .. }
            | Statement::AttrWriter { attributes, .. }
            | Statement::AttrAccessor { attributes, .. } => {
                let kind = match statement {
                    Statement::AttrReader { .. } => 0,
                    Statement::AttrWriter { .. } => 1,
                    _ => 2,
                };
                let names = self.attribute_names(attributes, position)?;
                self.emit(
                    Instruction::BodyAttributes {
                        to: ANSWER as u8,
                        names: names as u8,
                        kind,
                    },
                    position,
                );
                Ok(false)
            }
            Statement::Assignment {
                target: Expression::InstanceVariable { name, .. },
                value,
                ..
            } => {
                self.expression(value, unchanged)?;
                let name = self.name(name)?;
                self.emit(
                    Instruction::BodyInstanceVariable {
                        from: unchanged as u8,
                        name,
                    },
                    position,
                );
                Ok(false)
            }
            Statement::Assignment {
                target: Expression::ClassVariable { name, .. },
                value,
                ..
            } => {
                self.expression(value, unchanged)?;
                let name = self.name(name)?;
                self.emit(
                    Instruction::BodyClassVariable {
                        from: unchanged as u8,
                        name,
                    },
                    position,
                );
                Ok(false)
            }
            Statement::Expression {
                expression:
                    Expression::Call {
                        callee,
                        arguments,
                        trailing_block,
                        ..
                    },
                ..
            } if matches!(callee.as_ref(), Expression::Identifier { name, .. }
                if (name == "alias_method" && arguments.len() == 2)
                    || (name == "define_method" && trailing_block.is_some())
                    || name == "refine") =>
            {
                let Expression::Identifier { name, .. } = callee.as_ref() else {
                    unreachable!("matched an identifier above");
                };
                match name.as_str() {
                    "alias_method" => {
                        let first = self.temporary()?;
                        self.expression(&arguments[0], first)?;
                        let second = self.temporary()?;
                        self.expression(&arguments[1], second)?;
                        self.emit(
                            Instruction::BodyAliasMethod { first: first as u8 },
                            position,
                        );
                    }
                    "define_method" => {
                        // Each argument is evaluated on its own, and then
                        // the block.
                        let block = self.temporary()?;
                        for argument in arguments {
                            let register = self.temporary()?;
                            self.expression(argument, register)?;
                        }
                        if let Some(written) = trailing_block {
                            self.expression(written, block)?;
                        }
                        let count = u8::try_from(arguments.len()).map_err(|_| OVER_LIMIT)?;
                        self.emit(
                            Instruction::BodyDefineMethod {
                                to: ANSWER as u8,
                                block: block as u8,
                                count,
                            },
                            position,
                        );
                    }
                    _ => {
                        let receiver = Expression::SelfExpr { position };
                        self.method_call(
                            &receiver,
                            "refine",
                            arguments,
                            trailing_block.as_deref(),
                            unchanged,
                            position,
                        )?;
                    }
                }
                Ok(false)
            }
            // A bare expression is evaluated, and a name the class defines a
            // method of is the call of that method.
            Statement::Expression { expression, .. } => {
                self.expression(expression, ANSWER)?;
                if matches!(expression, Expression::Identifier { .. }) {
                    self.emit(
                        Instruction::InvokeIfMethod {
                            register: ANSWER as u8,
                        },
                        position,
                    );
                }
                Ok(false)
            }
            other => {
                self.statement(other, ANSWER)?;
                Ok(true)
            }
        }
    }

    /// The statements of a module body that follow the module body's own
    /// rules, past those it shares with a class body.
    fn module_body_statement(
        &mut self,
        statement: &Statement,
        unchanged: usize,
    ) -> Result<bool, &'static str> {
        let position = statement.position();
        match statement {
            // `class << target` opens the target's singleton class whatever
            // the target is written as.
            Statement::Expression {
                expression:
                    Expression::SingletonClass {
                        target,
                        body,
                        position: opened_at,
                        ..
                    },
                ..
            } => {
                let held = self.temporary()?;
                self.expression(target, held)?;
                self.define(
                    Definition::SingletonClass {
                        target: held,
                        body: body.clone(),
                        position: *opened_at,
                    },
                    unchanged,
                    *opened_at,
                )?;
                Ok(false)
            }
            Statement::Block { statements, .. } => {
                for inner in statements {
                    let start = self.unit.instructions.len();
                    match inner {
                        Statement::AttrReader { attributes, .. }
                        | Statement::AttrWriter { attributes, .. }
                        | Statement::AttrAccessor { attributes, .. } => {
                            let names = self.attribute_names(attributes, position)?;
                            let statement = wide(Some(self.unit.body_statement(inner.clone())))?;
                            self.emit(
                                Instruction::ModuleBlockStatement {
                                    names: names as u8,
                                    statement,
                                },
                                position,
                            );
                        }
                        Statement::MethodDef { .. } => {
                            let statement = wide(Some(self.unit.body_statement(inner.clone())))?;
                            self.emit(
                                Instruction::ModuleBlockStatement {
                                    names: unchanged as u8,
                                    statement,
                                },
                                position,
                            );
                        }
                        other => {
                            self.statement(other, unchanged)?;
                            let live = self.next_register;
                            self.unit
                                .start_inner_statement(start, other.position(), live);
                        }
                    }
                }
                Ok(false)
            }
            Statement::ClassDef { .. } | Statement::ModuleDef { .. } => {
                self.statement(statement, unchanged)?;
                Ok(false)
            }
            other => {
                self.statement(other, ANSWER)?;
                Ok(true)
            }
        }
    }

    /// Hand a statement of a class or module body to the body's own rules,
    /// which evaluate nothing the program wrote, leaving what the body then
    /// answers in register `to`.
    fn body_statement_as_written(
        &mut self,
        statement: &Statement,
        to: usize,
    ) -> Result<(), &'static str> {
        let index = wide(Some(self.unit.body_statement(statement.clone())))?;
        self.emit(
            Instruction::BodyStatement {
                to: to as u8,
                statement: index,
            },
            statement.position(),
        );
        Ok(())
    }

    /// Compile the last statement of a method body, which answers the
    /// method's value without passing through the statement runner: an
    /// assignment there is evaluated as an expression would be, and an `if`
    /// or `unless` runs as one written as an expression.
    fn terminal_statement(&mut self, statement: &Statement, to: usize) -> Result<(), &'static str> {
        match statement {
            Statement::Assignment {
                target,
                value,
                position,
            } => self.assignment(target, value, to, *position),
            Statement::Begin {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                position,
            } => self.begin(
                body,
                rescue_clauses,
                else_clause.as_deref(),
                ensure_block.as_deref(),
                to,
                *position,
                true,
            ),
            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position,
            } => self.branches(
                condition,
                false,
                then_branch,
                elsif_branches,
                else_branch.as_deref(),
                to,
                *position,
                true,
            ),
            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                position,
            } => self.branches(
                condition,
                true,
                then_branch,
                &[],
                else_branch.as_deref(),
                to,
                *position,
                true,
            ),
            other => self.statement(other, to),
        }
    }

    /// Compile a list of statements, leaving the value of the last in
    /// register `to`, or nil when there are none. `of_the_body` is set for
    /// the body's own statements, which a continuation can restart at.
    fn statements(
        &mut self,
        list: &[Statement],
        to: usize,
        of_the_body: bool,
    ) -> Result<(), &'static str> {
        self.statements_in(list, to, of_the_body, None)
    }

    /// Compile a list of statements as `statements` does. The statements of
    /// a branch of an `if` written as an expression say `enclosing_line`,
    /// the line the branch belongs to.
    fn statements_in(
        &mut self,
        list: &[Statement],
        to: usize,
        of_the_body: bool,
        enclosing_line: Option<usize>,
    ) -> Result<(), &'static str> {
        if list.is_empty() {
            let at = list.first().map(Statement::position).unwrap_or_default();
            self.emit(Instruction::LoadNil { to: to as u8 }, at);
        }
        for statement in list {
            let start = self.unit.instructions.len();
            let held = self.next_register;
            self.statement(statement, to)
                .map_err(naming(statement.kind_name()))?;
            self.next_register = held;
            if of_the_body {
                self.unit.start_statement(start, statement.position());
            } else {
                let live = self.next_register;
                self.unit
                    .start_inner_statement(start, statement.position(), live);
            }
            // A `begin` is not a line event: the statements inside it are.
            if let Statement::Begin { .. } = statement {
                self.unit
                    .set_start_kind(start, StartKind::Statement { line_event: false });
            }
            if let (Some(enclosing_line), Statement::Expression { .. }) =
                (enclosing_line, statement)
            {
                self.unit
                    .set_start_kind(start, StartKind::InBranch { enclosing_line });
            }
        }
        Ok(())
    }

    fn statement(&mut self, statement: &Statement, to: usize) -> Result<(), &'static str> {
        let to_register = to as u8;
        match statement {
            Statement::Expression { expression, .. } => self.expression(expression, to),
            Statement::Assignment {
                target,
                value,
                position,
            } => {
                // An error the assignment raises with no spot of its own is
                // pointed at the setter or `[]=` it called.
                self.spots
                    .push(
                        crate::vm::error_spots::assignment_spot(target).map(|(start, name)| {
                            ErrorSpot {
                                start,
                                name: Some(name),
                            }
                        }),
                    );
                // Ruby's parser introduces the local where the assignment is
                // written, ahead of the value, so a block on the right-hand
                // side closes over the name being assigned. That is what
                // lets `walk = -> { walk.call }` call itself.
                if let Expression::Identifier { name, .. } = target
                    && self.assigned.contains(name)
                {
                    let slot = wide(Some(self.unit.local(name)))?;
                    self.emit(Instruction::Unhoist { slot }, *position);
                }
                let compiled = self.assignment(target, value, to, *position);
                self.spots.pop();
                compiled
            }
            Statement::MultipleAssignment {
                targets,
                values,
                position,
            } => self.multiple_assignment(targets, values, to, *position),
            Statement::DeclareLocals { names, position } => {
                for name in names {
                    let slot = wide(Some(self.unit.local(name)))?;
                    self.emit(Instruction::DeclareLocal { slot }, *position);
                }
                self.emit(Instruction::LoadNil { to: to_register }, *position);
                Ok(())
            }
            Statement::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position,
            } => {
                self.branches(
                    condition,
                    false,
                    then_branch,
                    elsif_branches,
                    else_branch.as_deref(),
                    to,
                    *position,
                    false,
                )?;
                self.unhoist_assigned(statement)
            }
            Statement::Unless {
                condition,
                then_branch,
                else_branch,
                position,
            } => {
                self.branches(
                    condition,
                    true,
                    then_branch,
                    &[],
                    else_branch.as_deref(),
                    to,
                    *position,
                    false,
                )?;
                self.unhoist_assigned(statement)
            }
            Statement::While {
                condition,
                body,
                position,
            } => {
                self.while_loop(condition, body, to, *position)?;
                self.unhoist_assigned(statement)
            }
            Statement::DoWhile {
                condition,
                body,
                position,
            } => {
                self.do_while_loop(condition, body, to, *position)?;
                self.unhoist_assigned(statement)
            }
            Statement::For {
                variable,
                iterable,
                body,
                position,
            } => {
                // `for name in iterable` is a call of `each` with a block
                // that assigns `name`.
                let looped = crate::ast::for_loop::for_over_each(
                    vec![Expression::Identifier {
                        name: variable.clone(),
                        position: *position,
                    }],
                    false,
                    iterable.clone(),
                    body.clone(),
                    *position,
                );
                // The desugared loop is a `begin` with no clauses around the
                // names the loop binds and the call of `each`, which runs its
                // statements in the scope holding the loop.
                let Statement::Expression {
                    expression: Expression::BeginRescue { body: looped, .. },
                    ..
                } = &looped
                else {
                    unreachable!("for_over_each answers a begin around the loop");
                };
                self.emit(Instruction::EnterLoop, *position);
                self.statements(looped, to, false)?;
                self.emit(Instruction::LeaveLoop, *position);
                self.unhoist_assigned(statement)
            }
            Statement::Break { value, position } => {
                let into = self.loops.last().map_or(to, |inner| inner.to);
                self.optional_value(value.as_deref(), into, *position)?;
                match self.loops.len() {
                    0 => {
                        // A `break` whose block's call has returned raises
                        // where it stands, so a `begin` around it can rescue
                        // the LocalJumpError.
                        if self.begins > 0 {
                            self.emit(Instruction::BreakCheck { from: into as u8 }, *position);
                        }
                        self.leave_protections(0, *position)?;
                        self.emit(Instruction::ExitBreak { from: into as u8 }, *position);
                    }
                    _ => {
                        self.leave_to_loop(*position)?;
                        let jump = self.emit(Instruction::Jump { target: 0 }, *position);
                        self.innermost_loop().breaks.push(jump);
                    }
                }
                Ok(())
            }
            Statement::Continue { value, position } => {
                self.optional_value(value.as_deref(), to, *position)?;
                match self.loops.len() {
                    0 => {
                        self.leave_protections(0, *position)?;
                        self.emit(Instruction::ExitNext { from: to_register }, *position);
                    }
                    _ => {
                        self.leave_to_loop(*position)?;
                        let jump = self.emit(Instruction::Jump { target: 0 }, *position);
                        self.innermost_loop().nexts.push(jump);
                    }
                }
                Ok(())
            }
            Statement::Retry { position } => {
                let rescued = self
                    .protections
                    .iter()
                    .rposition(|protection| matches!(protection, Protection::RescueBody { .. }));
                match rescued {
                    None => {
                        self.leave_protections(0, *position)?;
                        self.emit(Instruction::ExitRetry, *position);
                    }
                    Some(at) => {
                        let Protection::RescueBody { retry_target, .. } = self.protections[at]
                        else {
                            unreachable!("found a rescue body above");
                        };
                        self.leave_protections(at, *position)?;
                        let target = wide(Some(retry_target))?;
                        self.emit(Instruction::Jump { target }, *position);
                    }
                }
                Ok(())
            }
            Statement::Match {
                expression,
                cases,
                position,
            } => {
                self.case_when(expression, cases, to, *position)?;
                self.unhoist_assigned(statement)
            }
            Statement::CaseIn {
                expression,
                cases,
                position,
            } => {
                self.case_in(expression, cases, to, *position)?;
                self.unhoist_assigned(statement)
            }
            Statement::Raise {
                exception,
                position,
            } => {
                if let Some(written) = exception {
                    self.expression(written, to)?;
                }
                self.emit(
                    Instruction::RaiseStatement {
                        from: to_register,
                        written: u8::from(exception.is_some()),
                    },
                    *position,
                );
                Ok(())
            }
            Statement::Begin {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                position,
            } => {
                self.begin(
                    body,
                    rescue_clauses,
                    else_clause.as_deref(),
                    ensure_block.as_deref(),
                    to,
                    *position,
                    false,
                )?;
                self.unhoist_assigned(statement)
            }
            Statement::Return { value, position } => {
                self.optional_value(value.as_ref(), to, *position)?;
                self.leave_protections(0, *position)?;
                self.emit(
                    Instruction::ExitReturn {
                        from: to_register,
                        written: u8::from(value.is_some()),
                    },
                    *position,
                );
                Ok(())
            }
            // A block of statements runs in a scope of its own.
            Statement::Block {
                statements,
                position,
            } => {
                self.emit(Instruction::PushScope, *position);
                self.pending.scopes += 1;
                self.statements(statements, to, false)?;
                self.pending.scopes -= 1;
                self.emit(Instruction::PopScope, *position);
                Ok(())
            }
            Statement::FunctionDef { position, .. }
            | Statement::MethodDef { position, .. }
            | Statement::Include { position, .. }
            | Statement::Extend { position, .. }
            | Statement::Alias { position, .. } => {
                let definition = Definition::Statement(statement.clone());
                self.define(definition, to, *position)
            }
            Statement::AttrReader {
                attributes,
                position,
            }
            | Statement::AttrWriter {
                attributes,
                position,
            }
            | Statement::AttrAccessor {
                attributes,
                position,
            } => {
                let method = match statement {
                    Statement::AttrReader { .. } => "attr_reader",
                    Statement::AttrWriter { .. } => "attr_writer",
                    _ => "attr_accessor",
                };
                let names = self.attribute_names(attributes, *position)?;
                self.define(
                    Definition::Attributes {
                        method,
                        names,
                        position: *position,
                    },
                    to,
                    *position,
                )
            }
            Statement::ClassDef {
                namespace,
                superclass_expression,
                position,
                ..
            } => {
                let namespace = self.held_value(namespace.as_deref())?;
                let superclass = self.held_value(superclass_expression.as_deref())?;
                self.define(
                    Definition::Class {
                        statement: statement.clone(),
                        namespace,
                        superclass,
                    },
                    to,
                    *position,
                )
            }
            Statement::ModuleDef {
                namespace,
                position,
                ..
            } => {
                let namespace = self.held_value(namespace.as_deref())?;
                self.define(
                    Definition::Module {
                        statement: statement.clone(),
                        namespace,
                    },
                    to,
                    *position,
                )
            }
            // A `BEGIN` body already ran before the rest of its code unit, so
            // reaching it where it was written does nothing.
            Statement::BeginBlock { position, .. } => {
                let unused = self.temporary()?;
                self.emit(Instruction::LoadNil { to: unused as u8 }, *position);
                Ok(())
            }
            Statement::Redo { position } => {
                match self.loops.last().map(|inner| inner.redo_target) {
                    None => {
                        self.leave_protections(0, *position)?;
                        self.emit(Instruction::ExitRedo, *position);
                    }
                    Some(target) => {
                        self.leave_to_loop(*position)?;
                        let target = wide(Some(target))?;
                        self.emit(Instruction::Jump { target }, *position);
                    }
                }
                Ok(())
            }
        }
    }

    /// Emit the definition, leaving what it answers in register `to`. The
    /// registers holding what it needed evaluated are kept until it is made.
    fn define(
        &mut self,
        definition: Definition,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let definition = wide(Some(self.unit.definition(definition)))?;
        self.emit(
            Instruction::Define {
                to: to as u8,
                definition,
            },
            position,
        );
        Ok(())
    }

    /// Evaluate `expression`, when there is one, into a register kept until
    /// the statement using it is done, answering the register.
    fn held_value(
        &mut self,
        expression: Option<&Expression>,
    ) -> Result<Option<usize>, &'static str> {
        let Some(expression) = expression else {
            return Ok(None);
        };
        let register = self.temporary()?;
        self.expression(expression, register)?;
        Ok(Some(register))
    }

    /// Evaluate the names an `attr_*` statement is given into an Array, in
    /// a register kept until the statement is done, each splat spread into
    /// the names its list holds. Parentheses leave a splat inside a group.
    fn attribute_names(
        &mut self,
        attributes: &[Expression],
        position: Position,
    ) -> Result<usize, &'static str> {
        let names = self.temporary()?;
        self.emit(
            Instruction::NewArray {
                to: names as u8,
                first: 0,
                count: 0,
            },
            position,
        );
        for attribute in attributes {
            let mut attribute = attribute;
            while let Expression::Grouped { expression, .. } = attribute {
                attribute = expression;
            }
            let register = self.temporary()?;
            let add = match attribute {
                Expression::Splat { expression, .. } => {
                    self.expression(expression, register)?;
                    Instruction::AttributeSplat {
                        array: names as u8,
                        from: register as u8,
                    }
                }
                _ => {
                    self.expression(attribute, register)?;
                    Instruction::AppendArray {
                        array: names as u8,
                        first: register as u8,
                        count: 1,
                    }
                }
            };
            self.emit(add, position);
            self.next_register = register;
        }
        Ok(names)
    }

    /// Store the value in register `from` through `target`, as an
    /// assignment of it would.
    fn store_register(
        &mut self,
        target: &Expression,
        from: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let from = from as u8;
        let instruction = match target {
            Expression::Identifier { name, .. } if self.assigned.contains(name) => {
                Instruction::SetLocal {
                    from,
                    slot: wide(Some(self.unit.local(name)))?,
                }
            }
            Expression::Identifier { name, .. } if starts_uppercase(name) => {
                Instruction::SetConstant {
                    from,
                    name: self.name(name)?,
                }
            }
            Expression::InstanceVariable { name, .. } => Instruction::SetInstanceVariable {
                from,
                name: self.name(name)?,
            },
            Expression::GlobalVariable { name, .. } => Instruction::SetGlobal {
                from,
                name: self.name(name)?,
            },
            Expression::ClassVariable { name, .. } => Instruction::SetClassVariable {
                from,
                name: self.name(name)?,
            },
            other => {
                let shape = self.prepare(other)?;
                Instruction::StoreTarget {
                    from,
                    target: wide(Some(self.unit.target(shape)))?,
                }
            }
        };
        self.emit(instruction, position);
        Ok(())
    }

    /// Run a `BEGIN` body the first time its unit runs.
    fn begin_block(&mut self, body: &[Statement], position: Position) -> Result<(), &'static str> {
        let held = self.next_register;
        let runs = self.temporary()?;
        self.emit(Instruction::BeginBlockOnce { to: runs as u8 }, position);
        let ran = self.emit(
            Instruction::JumpIfFalse {
                test: runs as u8,
                target: 0,
            },
            position,
        );
        self.statements(body, runs, false)?;
        self.jump_here(ran)?;
        self.next_register = held;
        Ok(())
    }

    /// The loop being compiled innermost.
    fn innermost_loop(&mut self) -> &mut Loop {
        self.loops.last_mut().expect("a loop is being compiled")
    }

    /// Before a jump out of the middle of the innermost loop's body, run
    /// what the regions inside the loop that the jump leaves have to run, and
    /// drop what was opened since the loop began.
    fn leave_to_loop(&mut self, position: Position) -> Result<(), &'static str> {
        let (at_loop, protections) = {
            let inner = self.innermost_loop();
            (inner.pending, inner.protections)
        };
        self.leave_protections(protections, position)?;
        self.unwind_to(at_loop, position);
        Ok(())
    }

    /// Drop what was opened past `target`: argument lists, call routes,
    /// scopes, and loops.
    fn unwind_to(&mut self, target: Pending, position: Position) {
        let now = self.pending;
        if (now.lists, now.routes, now.scopes) != (target.lists, target.routes, target.scopes) {
            self.emit(
                Instruction::Unwind {
                    lists: (now.lists - target.lists) as u8,
                    routes: (now.routes - target.routes) as u8,
                    scopes: (now.scopes - target.scopes) as u8,
                },
                position,
            );
        }
        for _ in target.loops..now.loops {
            self.emit(Instruction::LeaveLoop, position);
        }
        // The code after the jump is reached with what was open before it.
        self.pending = now;
    }

    /// On the way out of the protected regions from the one numbered
    /// `down_to` on, run what each has to run, the innermost first: an
    /// `ensure` clause's statements, and the putting back of `$!` a rescue
    /// body does.
    fn leave_protections(
        &mut self,
        down_to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.pending;
        for at in (down_to..self.protections.len()).rev() {
            match self.protections[at].clone() {
                Protection::Ensure { body, pending } => {
                    self.unwind_to(pending, position);
                    // Inside the clause, the region it protects is left.
                    let outer = self.protections.split_off(at);
                    let value = self.temporary()?;
                    let compiled = self.statements(&body, value, false);
                    self.next_register = value;
                    self.protections.extend(outer);
                    compiled?;
                }
                Protection::Deconstructing { pending } => {
                    self.unwind_to(pending, position);
                    self.emit(Instruction::PopDeconstructed, position);
                }
                Protection::RescueBody {
                    standing, pending, ..
                }
                | Protection::DroppedError { standing, pending } => {
                    self.unwind_to(pending, position);
                    self.emit(
                        Instruction::RestoreCurrentException {
                            from: standing as u8,
                        },
                        position,
                    );
                }
            }
        }
        self.pending = held;
        Ok(())
    }

    /// Compile `begin`, with its rescue clauses, `else`, and `ensure`,
    /// leaving its value in register `to`. `expression_form` is set where the
    /// value is used as an expression, and for a method body's last
    /// statement, which answers the method's value as an expression does.
    #[allow(clippy::too_many_arguments)]
    fn begin(
        &mut self,
        body: &[Statement],
        rescue_clauses: &[crate::ast::RescueClause],
        else_clause: Option<&[Statement]>,
        ensure_block: Option<&[Statement]>,
        to: usize,
        position: Position,
        expression_form: bool,
    ) -> Result<(), &'static str> {
        self.begins += 1;
        let compiled = self.begin_in_place(
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            to,
            position,
            expression_form,
        );
        self.begins -= 1;
        compiled
    }

    #[allow(clippy::too_many_arguments)]
    fn begin_in_place(
        &mut self,
        body: &[Statement],
        rescue_clauses: &[crate::ast::RescueClause],
        else_clause: Option<&[Statement]>,
        ensure_block: Option<&[Statement]>,
        to: usize,
        position: Position,
        expression_form: bool,
    ) -> Result<(), &'static str> {
        let held = self.next_register;
        let standing = self.temporary()?;
        let retry_target = self.unit.instructions.len();
        self.emit(
            Instruction::SaveCurrentException { to: standing as u8 },
            position,
        );
        let region_start = self.unit.instructions.len();
        let region_open = self.pending;
        if let Some(ensure) = ensure_block {
            self.protections.push(Protection::Ensure {
                body: ensure.to_vec(),
                pending: self.pending,
            });
        }
        let body_start = self.unit.instructions.len();
        let body_open = self.pending;
        self.clause_statements(body, to)?;
        let body_end = self.unit.instructions.len();
        let mut done = Vec::new();
        if let Some(else_clause) = else_clause {
            self.clause_statements(else_clause, to)?;
        }
        done.push(self.emit(Instruction::Jump { target: 0 }, position));
        if !rescue_clauses.is_empty() {
            let exception = self.temporary()?;
            let slot = self.unit.error_slot();
            self.unit.handler(Handler {
                start: body_start,
                end: body_end,
                target: self.unit.instructions.len(),
                register: exception,
                kind: HandlerKind::Rescue { slot },
                open: open_at(body_open),
            });
            for clause in rescue_clauses {
                let base = self.temporary()?;
                self.temporary()?;
                self.emit(
                    Instruction::Move {
                        to: base as u8,
                        from: exception as u8,
                    },
                    clause.position,
                );
                for splatted in &clause.splatted_types {
                    let register = self.temporary()?;
                    self.expression(splatted, register)?;
                }
                let index = wide(Some(self.unit.rescue_clause(RescueClause {
                    clause: clause.clone(),
                    expression_form,
                })))?;
                self.emit(
                    Instruction::RescueMatches {
                        register: base as u8,
                        clause: index,
                    },
                    clause.position,
                );
                let next_clause = self.emit(
                    Instruction::JumpIfFalse {
                        test: base as u8 + 1,
                        target: 0,
                    },
                    clause.position,
                );
                self.next_register = base;
                self.emit(
                    Instruction::EnterRescue {
                        from: exception as u8,
                        clause: index,
                    },
                    clause.position,
                );
                if let Some(target) = &clause.variable_target {
                    self.store_register(target, exception, clause.position)?;
                }
                self.protections.push(Protection::RescueBody {
                    standing,
                    retry_target,
                    pending: self.pending,
                });
                let rescue_start = self.unit.instructions.len();
                let compiled = self.clause_statements(&clause.body, to);
                let rescue_end = self.unit.instructions.len();
                self.protections.pop();
                compiled?;
                self.unit.handler(Handler {
                    start: rescue_start,
                    end: rescue_end,
                    target: 0,
                    register: 0,
                    kind: HandlerKind::RescueBody {
                        handled: exception,
                        standing,
                        expression_form,
                    },
                    open: open_at(self.pending),
                });
                // A handled exception leaves `$!` naming whatever it was
                // before, so an outer handler still names what it handles.
                self.emit(
                    Instruction::RestoreCurrentException {
                        from: standing as u8,
                    },
                    clause.position,
                );
                done.push(self.emit(Instruction::Jump { target: 0 }, clause.position));
                self.jump_here(next_clause)?;
            }
            // No clause takes the exception, which goes on as it was raised.
            let slot = wide(Some(slot))?;
            self.emit(Instruction::Reraise { slot }, position);
        }
        for jump in done {
            self.jump_here(jump)?;
        }
        if let Some(ensure) = ensure_block {
            self.protections.pop();
            let region_end = self.unit.instructions.len();
            let slot = self.unit.error_slot();
            let value = self.temporary()?;
            self.statements(ensure, value, false)?;
            let finished = self.emit(Instruction::Jump { target: 0 }, position);
            self.unit.handler(Handler {
                start: region_start,
                end: region_end,
                target: self.unit.instructions.len(),
                register: 0,
                kind: HandlerKind::Ensure { slot },
                open: open_at(region_open),
            });
            if expression_form {
                self.protections.push(Protection::DroppedError {
                    standing,
                    pending: self.pending,
                });
            }
            let compiled = self.statements(ensure, value, false);
            if expression_form {
                self.protections.pop();
            }
            compiled?;
            let slot = wide(Some(slot))?;
            self.emit(Instruction::Reraise { slot }, position);
            self.jump_here(finished)?;
        }
        self.next_register = held;
        Ok(())
    }

    /// Compile `case subject; when ...; end` written as a statement, leaving
    /// the value of the arm taken in register `to`, nil when none is.
    fn case_when(
        &mut self,
        subject: &Expression,
        cases: &[crate::ast::MatchCase],
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.next_register;
        let value = self.temporary()?;
        let matched = self.temporary()?;
        self.expression(subject, value)?;
        let list = wide(Some(self.unit.when_list(cases.to_vec(), position)))?;
        self.emit(Instruction::WarnDuplicatedWhen { list }, position);
        let mut done = Vec::new();
        for (index, case) in cases.iter().enumerate() {
            let arm = wide(Some(self.unit.pattern_arm(PatternArm {
                pattern: case.pattern.clone(),
                guard: case.guard.clone(),
                case_position: position,
                index,
            })))?;
            self.emit(
                Instruction::WhenArm {
                    subject: value as u8,
                    arm,
                },
                case.position,
            );
            let next_arm = self.emit(
                Instruction::JumpIfFalse {
                    test: matched as u8,
                    target: 0,
                },
                case.position,
            );
            self.statements_in(&case.body, to, false, Some(position.line))?;
            done.push(self.emit(Instruction::Jump { target: 0 }, case.position));
            self.jump_here(next_arm)?;
        }
        let arm = u16::try_from(cases.len()).map_err(|_| OVER_LIMIT)?;
        self.emit(Instruction::CoverBranch { arm }, position);
        self.emit(Instruction::LoadNil { to: to as u8 }, position);
        for jump in done {
            self.jump_here(jump)?;
        }
        self.next_register = held;
        Ok(())
    }

    /// Compile `case subject; in pattern; ...; end`, leaving the value of the
    /// arm taken in register `to`, and raising NoMatchingPatternError when no
    /// arm is. The value is asked for its elements once for the whole `case`.
    fn case_in(
        &mut self,
        subject: &Expression,
        cases: &[crate::ast::MatchCase],
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.next_register;
        let value = self.temporary()?;
        let matched = self.temporary()?;
        self.expression(subject, value)?;
        self.emit(
            Instruction::PushDeconstructed { from: value as u8 },
            position,
        );
        let slot = self.unit.error_slot();
        let region_start = self.unit.instructions.len();
        let region_open = self.pending;
        self.protections.push(Protection::Deconstructing {
            pending: self.pending,
        });
        let mut done = Vec::new();
        for (index, case) in cases.iter().enumerate() {
            let arm = wide(Some(self.unit.pattern_arm(PatternArm {
                pattern: case.pattern.clone(),
                guard: None,
                case_position: position,
                index,
            })))?;
            self.emit(
                Instruction::InArm {
                    subject: value as u8,
                    arm,
                },
                case.position,
            );
            let mut next_arm = vec![self.emit(
                Instruction::JumpIfFalse {
                    test: matched as u8,
                    target: 0,
                },
                case.position,
            )];
            // The pattern's names are locals of the scope the `case` is in,
            // so the guard reads them there.
            if let Some(guard) = &case.guard {
                let test = self.temporary()?;
                self.expression(guard, test)?;
                let passed = self.emit(
                    Instruction::JumpIfTrue {
                        test: test as u8,
                        target: 0,
                    },
                    case.position,
                );
                self.emit(Instruction::GuardFailed, case.position);
                next_arm.push(self.emit(Instruction::Jump { target: 0 }, case.position));
                self.jump_here(passed)?;
                self.next_register = test;
            }
            let arm_number = u16::try_from(index).map_err(|_| OVER_LIMIT)?;
            self.emit(Instruction::CoverBranch { arm: arm_number }, position);
            self.statements_in(&case.body, to, false, Some(position.line))?;
            done.push(self.emit(Instruction::Jump { target: 0 }, case.position));
            for jump in next_arm {
                self.jump_here(jump)?;
            }
        }
        // A `case` with one `in` clause and no `else` has nowhere to go when
        // the clause fails, so Ruby reports what went wrong rather than the
        // value alone.
        self.emit(
            Instruction::NoMatchingPattern {
                subject: value as u8,
                single: u8::from(cases.len() == 1),
            },
            position,
        );
        self.protections.pop();
        let region_end = self.unit.instructions.len();
        for jump in done {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::PopDeconstructed, position);
        let finished = self.emit(Instruction::Jump { target: 0 }, position);
        self.unit.handler(Handler {
            start: region_start,
            end: region_end,
            target: self.unit.instructions.len(),
            register: 0,
            kind: HandlerKind::Cleanup { slot },
            open: open_at(region_open),
        });
        self.emit(Instruction::PopDeconstructed, position);
        let slot = wide(Some(slot))?;
        self.emit(Instruction::Reraise { slot }, position);
        self.jump_here(finished)?;
        self.next_register = held;
        Ok(())
    }

    /// Compile the statements of a `begin` clause, leaving the value of the
    /// last in register `to`. The last answers it the way a method body's
    /// last statement does.
    fn clause_statements(&mut self, list: &[Statement], to: usize) -> Result<(), &'static str> {
        let Some((last, before)) = list.split_last() else {
            self.emit(Instruction::LoadNil { to: to as u8 }, Position::default());
            return Ok(());
        };
        if !before.is_empty() {
            self.statements(before, to, false)?;
        }
        if !answers_its_own_value(last) {
            return self.statements(std::slice::from_ref(last), to, false);
        }
        let start = self.unit.instructions.len();
        let held = self.next_register;
        self.terminal_statement(last, to)
            .map_err(naming(last.kind_name()))?;
        self.next_register = held;
        self.unit
            .start_inner_statement(start, last.position(), self.next_register);
        self.unit.set_start_kind(
            start,
            StartKind::Terminal {
                line_event: !matches!(last, Statement::Begin { .. }),
            },
        );
        Ok(())
    }

    /// Compile the value written after `break` or `next`, nil when none was.
    fn optional_value(
        &mut self,
        value: Option<&Expression>,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        match value {
            Some(value) => self.expression(value, to),
            None => {
                self.emit(Instruction::LoadNil { to: to as u8 }, position);
                Ok(())
            }
        }
    }

    /// A local assigned inside a branch or a loop is a local from there on
    /// whether or not the assignment ran, so a block written later closes
    /// over it.
    fn unhoist_assigned(&mut self, statement: &Statement) -> Result<(), &'static str> {
        for name in crate::ast::collect_assigned_locals(std::slice::from_ref(statement)) {
            let slot = wide(Some(self.unit.local(&name)))?;
            self.emit(Instruction::Unhoist { slot }, statement.position());
        }
        Ok(())
    }

    /// Compile `if`, with its `elsif`s and `else`, or `unless` when
    /// `negated`, leaving the value of the branch taken in register `to`, or
    /// nil when none was.
    #[allow(clippy::too_many_arguments)]
    fn branches(
        &mut self,
        condition: &Expression,
        negated: bool,
        then_branch: &[Statement],
        elsif_branches: &[crate::ast::ElsifBranch],
        else_branch: Option<&[Statement]>,
        to: usize,
        position: Position,
        as_expression: bool,
    ) -> Result<(), &'static str> {
        let condition_line = as_expression.then_some(condition.position().line);
        let test = to as u8;
        let mut done = Vec::new();
        self.condition(condition, to)?;
        let skip = if negated {
            Instruction::JumpIfTrue { test, target: 0 }
        } else {
            Instruction::JumpIfFalse { test, target: 0 }
        };
        let mut not_taken = self.emit(skip, position);
        self.emit(Instruction::CoverBranch { arm: 0 }, position);
        self.statements_in(then_branch, to, false, condition_line)?;
        done.push(self.emit(Instruction::Jump { target: 0 }, position));
        for (index, elsif) in elsif_branches.iter().enumerate() {
            self.jump_here(not_taken)?;
            // Ruby counts an `elsif` line each time its condition is tested.
            self.emit(Instruction::CoverLine, elsif.position);
            self.condition(&elsif.condition, to)?;
            not_taken = self.emit(Instruction::JumpIfFalse { test, target: 0 }, position);
            let arm = u16::try_from(index + 1).map_err(|_| OVER_LIMIT)?;
            self.emit(Instruction::CoverBranch { arm }, position);
            let elsif_line = as_expression.then_some(elsif.position.line);
            self.statements_in(&elsif.body, to, false, elsif_line)?;
            done.push(self.emit(Instruction::Jump { target: 0 }, position));
        }
        self.jump_here(not_taken)?;
        let arm = u16::try_from(elsif_branches.len() + 1).map_err(|_| OVER_LIMIT)?;
        if negated {
            self.emit(Instruction::CoverBranch { arm: 1 }, position);
        } else {
            self.emit(Instruction::CoverBranch { arm }, position);
        }
        match else_branch {
            Some(else_branch) => self.statements_in(else_branch, to, false, condition_line)?,
            None => {
                self.emit(Instruction::LoadNil { to: test }, position);
            }
        }
        for jump in done {
            self.jump_here(jump)?;
        }
        Ok(())
    }

    /// Compile `while condition; body; end`, answering nil in register `to`,
    /// or the value a `break` carries.
    fn while_loop(
        &mut self,
        condition: &Expression,
        body: &[Statement],
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let test = to as u8;
        self.emit(Instruction::EnterLoop, position);
        let condition_at = self.unit.instructions.len();
        // Each pass is a place the turn can be handed over, so a loop with
        // nothing in its body still lets the other threads run.
        self.emit(Instruction::Checkpoint, condition.position());
        self.condition(condition, to)?;
        let finished = self.emit(Instruction::JumpIfFalse { test, target: 0 }, position);
        self.emit(Instruction::CoverBranch { arm: 0 }, position);
        let body_at = self.unit.instructions.len();
        self.loops.push(Loop {
            to,
            redo_target: body_at,
            breaks: Vec::new(),
            nexts: Vec::new(),
            pending: self.pending,
            protections: self.protections.len(),
        });
        let compiled = self.statements(body, to, false);
        let inner = self.loops.pop().expect("the loop pushed above");
        compiled?;
        let back = wide(Some(condition_at))?;
        self.emit(Instruction::Jump { target: back }, position);
        for jump in inner.nexts {
            self.jump_to(jump, condition_at)?;
        }
        self.jump_here(finished)?;
        self.emit(Instruction::LoadNil { to: test }, position);
        for jump in inner.breaks {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::LeaveLoop, position);
        Ok(())
    }

    /// Compile `begin; body; end while condition`, which runs its body
    /// before it first reads the condition.
    fn do_while_loop(
        &mut self,
        condition: &Expression,
        body: &[Statement],
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let test = to as u8;
        self.emit(Instruction::EnterLoop, position);
        let top = self.unit.instructions.len();
        self.emit(Instruction::CoverBranch { arm: 0 }, position);
        let body_at = self.unit.instructions.len();
        self.loops.push(Loop {
            to,
            redo_target: body_at,
            breaks: Vec::new(),
            nexts: Vec::new(),
            pending: self.pending,
            protections: self.protections.len(),
        });
        let compiled = self.statements(body, to, false);
        let inner = self.loops.pop().expect("the loop pushed above");
        compiled?;
        for jump in inner.nexts {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::Checkpoint, condition.position());
        self.condition(condition, to)?;
        let again = wide(Some(top))?;
        self.emit(
            Instruction::JumpIfTrue {
                test,
                target: again,
            },
            position,
        );
        self.emit(Instruction::LoadNil { to: test }, position);
        for jump in inner.breaks {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::LeaveLoop, position);
        Ok(())
    }

    /// Compile an `if` or loop condition into register `to`, which the jump
    /// after it reads. `a and b` reads each side as a condition of its own,
    /// a Regexp on its own matches the last line read, and a Range written
    /// as a condition is a flip-flop.
    fn condition(&mut self, condition: &Expression, to: usize) -> Result<(), &'static str> {
        let test = to as u8;
        match condition {
            Expression::BinaryOp {
                op: op @ (BinaryOp::And | BinaryOp::Or),
                left,
                right,
                position,
            } => {
                self.condition(left, to)?;
                let decided = match op {
                    BinaryOp::Or => Instruction::JumpIfTrue { test, target: 0 },
                    _ => Instruction::JumpIfFalse { test, target: 0 },
                };
                let decided = self.emit(decided, *position);
                self.condition(right, to)?;
                self.jump_here(decided)
            }
            Expression::RegexLiteral { position, .. } => {
                let base = self.temporary()?;
                self.expression(condition, base)?;
                self.emit(Instruction::LastLine { to: base as u8 + 1 }, *position);
                self.temporary()?;
                let site = CallSite::new("=~", 1, Receiver::Sent);
                let site = wide(Some(self.unit.call_site(site)))?;
                self.emit(
                    Instruction::Call {
                        base: base as u8,
                        site,
                    },
                    *position,
                );
                self.emit(
                    Instruction::Move {
                        to: test,
                        from: base as u8,
                    },
                    *position,
                );
                self.next_register = base;
                Ok(())
            }
            Expression::Range {
                start,
                end,
                exclusive,
                position,
            } => self.flip_flop(start, end, *exclusive, to, *position),
            other => self.expression(other, to),
        }
    }

    /// Compile a flip-flop, `start..end` written as a condition: on from
    /// when `start` holds until when `end` holds.
    fn flip_flop(
        &mut self,
        start: &Expression,
        end: &Expression,
        exclusive: bool,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let test = to as u8;
        let integer_sides = [start, end]
            .into_iter()
            .filter(|side| matches!(side, Expression::IntLiteral { .. }))
            .count();
        let flip = wide(Some(self.unit.flip_flop(position, integer_sides)))?;
        self.emit(Instruction::FlipFlopState { to: test, flip }, position);
        let running = self.emit(Instruction::JumpIfTrue { test, target: 0 }, position);
        self.flip_flop_side(start, to, position)?;
        let off = self.emit(Instruction::JumpIfFalse { test, target: 0 }, position);
        self.emit(Instruction::FlipFlopSet { on: 1, flip }, position);
        let mut on = Vec::new();
        // `..` reads its second side the moment the first one holds, so a
        // range whose sides hold together is on for that one turn alone.
        // `...` waits for the next turn.
        if !exclusive {
            self.flip_flop_side(end, to, position)?;
            on.push(self.emit(Instruction::JumpIfFalse { test, target: 0 }, position));
            self.emit(Instruction::FlipFlopSet { on: 0, flip }, position);
        }
        on.push(self.emit(Instruction::Jump { target: 0 }, position));
        self.jump_here(running)?;
        self.flip_flop_side(end, to, position)?;
        on.push(self.emit(Instruction::JumpIfFalse { test, target: 0 }, position));
        self.emit(Instruction::FlipFlopSet { on: 0, flip }, position);
        on.push(self.emit(Instruction::Jump { target: 0 }, position));
        self.jump_here(off)?;
        self.emit(Instruction::LoadFalse { to: test }, position);
        let done = self.emit(Instruction::Jump { target: 0 }, position);
        for jump in on {
            self.jump_here(jump)?;
        }
        self.emit(Instruction::LoadTrue { to: test }, position);
        self.jump_here(done)
    }

    /// One side of a flip-flop. A number stands for the line the last read
    /// left off at, which is what `$.` reports.
    fn flip_flop_side(
        &mut self,
        side: &Expression,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let Expression::IntLiteral { value, .. } = side else {
            return self.expression(side, to);
        };
        let constant = self.constant(Object::Int(*value))?;
        self.emit(
            Instruction::LineIs {
                to: to as u8,
                constant,
            },
            position,
        );
        Ok(())
    }

    /// Compile `case subject; when ...; else ...; end` written as an
    /// expression, leaving the value of the arm taken in register `to`.
    fn case_expression(
        &mut self,
        subject: &Expression,
        cases: &[crate::ast::node::ExprMatchCase],
        else_case: Option<&Expression>,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.temporary()?;
        let matched = self.temporary()?;
        self.expression(subject, held)?;
        let mut done = Vec::new();
        for (index, case) in cases.iter().enumerate() {
            let arm = wide(Some(
                self.unit.case_arm(CaseArm {
                    pattern: case.pattern.clone(),
                    guard: case.guard.clone(),
                    position: case.position,
                    case_position: position,
                    index,
                    counted_line: (!matches!(case.body, Expression::BeginRescue { .. }))
                        .then(|| case.body.position().line),
                }),
            ))?;
            self.emit(
                Instruction::CaseArm {
                    subject: held as u8,
                    arm,
                },
                case.position,
            );
            let next_arm = self.emit(
                Instruction::JumpIfFalse {
                    test: matched as u8,
                    target: 0,
                },
                case.position,
            );
            self.pending.scopes += 1;
            self.expression(&case.body, to)?;
            self.pending.scopes -= 1;
            self.emit(Instruction::PopScope, case.position);
            done.push(self.emit(Instruction::Jump { target: 0 }, case.position));
            self.jump_here(next_arm)?;
        }
        let arm = u16::try_from(cases.len()).map_err(|_| OVER_LIMIT)?;
        self.emit(Instruction::CoverBranch { arm }, position);
        match else_case {
            Some(else_case) => {
                if !matches!(else_case, Expression::BeginRescue { .. }) {
                    self.emit(Instruction::CoverLine, else_case.position());
                }
                self.expression(else_case, to)?;
            }
            None => {
                self.emit(Instruction::LoadNil { to: to as u8 }, position);
            }
        }
        for jump in done {
            self.jump_here(jump)?;
        }
        Ok(())
    }

    /// Compile `target = value`, leaving what the assignment answers in
    /// register `to`.
    fn assignment(
        &mut self,
        target: &Expression,
        value: &Expression,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.next_register;
        let compiled = self.assignment_in_place(target, value, to, position);
        self.next_register = held;
        compiled
    }

    fn assignment_in_place(
        &mut self,
        target: &Expression,
        value: &Expression,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let from = to as u8;
        let named = match target {
            Expression::Identifier { name, .. } if self.assigned.contains(name) => {
                Some(Instruction::SetLocal {
                    from,
                    slot: wide(self.unit.locals.iter().position(|held| &held.text == name))?,
                })
            }
            Expression::Identifier { name, .. } if starts_uppercase(name) => {
                Some(Instruction::SetConstant {
                    from,
                    name: self.name(name)?,
                })
            }
            Expression::InstanceVariable { name, .. } => Some(Instruction::SetInstanceVariable {
                from,
                name: self.name(name)?,
            }),
            Expression::GlobalVariable { name, .. } => Some(Instruction::SetGlobal {
                from,
                name: self.name(name)?,
            }),
            Expression::ClassVariable { name, .. } => Some(Instruction::SetClassVariable {
                from,
                name: self.name(name)?,
            }),
            _ => None,
        };
        let shape = match named {
            Some(_) => None,
            None => Some(self.prepare(target)?),
        };
        if let Some(
            receiver_shape @ (Target::Setter { .. }
            | Target::Subscript { .. }
            | Target::ScopedConstant { .. }),
        ) = &shape
        {
            return self.assignment_on_receiver(
                receiver_shape.clone(),
                target,
                value,
                to,
                position,
            );
        }
        let decided = self.assigned_value(target, value, to)?;
        let instruction = match (named, shape) {
            (Some(instruction), _) => instruction,
            (None, Some(shape)) => Instruction::StoreTarget {
                from,
                target: wide(Some(self.unit.target(shape)))?,
            },
            (None, None) => unreachable!("a target is named or prepared"),
        };
        self.emit(instruction, position);
        if let Some(decided) = decided {
            self.jump_here(decided)?;
        }
        Ok(())
    }

    /// Compile the value of an assignment to a variable or bare constant,
    /// answering the jump that skips the write when there is one.
    /// `CONST ||= value` and `$global ||= value` read a target not defined
    /// yet as nil without the notice reading it would give, so only the
    /// right-hand side is evaluated then. `target ||= value` is `target ||
    /// target = value`, and `&&=` the same with `&&`, so the target is not
    /// written when what it reads decides the answer.
    fn assigned_value(
        &mut self,
        target: &Expression,
        value: &Expression,
        to: usize,
    ) -> Result<Option<usize>, &'static str> {
        let to_register = to as u8;
        if let Expression::BinaryOp {
            op: BinaryOp::Or,
            left,
            right,
            position,
        } = value
        {
            let same_global = matches!(
                (target, left.as_ref()),
                (
                    Expression::GlobalVariable { name, .. },
                    Expression::GlobalVariable { name: read, .. }
                ) if name == read
            );
            if crate::vm::statement::constant_target_matches(target, left) || same_global {
                let expression = wide(Some(self.unit.defined_expression(target.clone())))?;
                self.emit(
                    Instruction::IsDefined {
                        to: to_register,
                        expression,
                    },
                    *position,
                );
                let undefined = self.emit(
                    Instruction::JumpIfNil {
                        test: to_register,
                        target: 0,
                    },
                    *position,
                );
                let decided = self.logical_value(target, value, to)?;
                let done = self.emit(Instruction::Jump { target: 0 }, *position);
                self.jump_here(undefined)?;
                self.expression(right, to)?;
                self.jump_here(done)?;
                return Ok(decided);
            }
        }
        self.logical_value(target, value, to)
    }

    /// Compile `value`, and when it is the `target || value` or `target &&
    /// value` an operator assignment is written as, answer the jump taken
    /// when what the target reads decides the answer.
    fn logical_value(
        &mut self,
        target: &Expression,
        value: &Expression,
        to: usize,
    ) -> Result<Option<usize>, &'static str> {
        let Some((or_assigning, reading, right)) =
            crate::vm::statement::logical_assignment(target, value)
        else {
            self.expression(value, to)?;
            return Ok(None);
        };
        self.expression(reading, to)?;
        let test = to as u8;
        let jump = if or_assigning {
            Instruction::JumpIfTrue { test, target: 0 }
        } else {
            Instruction::JumpIfFalse { test, target: 0 }
        };
        let decided = self.emit(jump, value.position());
        self.expression(right, to)?;
        Ok(Some(decided))
    }

    /// Compile an assignment through a setter, `[]=`, or `Ns::Name =`, whose
    /// receiver `shape` was evaluated into registers already.
    fn assignment_on_receiver(
        &mut self,
        mut shape: Target,
        target: &Expression,
        value: &Expression,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let from = to as u8;
        // `held&.name = value` evaluates nothing more when there is no
        // receiver, and answers nil.
        let missing_receiver = match &shape {
            Target::Setter {
                receiver,
                safe: true,
                ..
            } => Some(self.emit(
                Instruction::JumpIfNil {
                    test: *receiver as u8,
                    target: 0,
                },
                position,
            )),
            _ => None,
        };
        let mut decided = None;
        match value {
            // The parser writes `target op= operand` as `target = target op
            // operand`, and the target is read and written through the one
            // evaluation of its receiver.
            Expression::BinaryOp {
                op,
                left,
                right,
                position: operator_at,
            } if left.as_ref() == target => {
                if let Target::ScopedConstant { or_assigning, .. } = &mut shape {
                    *or_assigning = matches!(op, BinaryOp::Or);
                }
                let read = wide(Some(self.unit.target(shape.clone())))?;
                match op {
                    // `||=` and `&&=` write nothing when the value they read
                    // already decides the answer.
                    BinaryOp::Or | BinaryOp::And => {
                        self.emit(
                            Instruction::ReadTarget {
                                to: from,
                                target: read,
                            },
                            *operator_at,
                        );
                        let jump = if matches!(op, BinaryOp::Or) {
                            Instruction::JumpIfTrue {
                                test: from,
                                target: 0,
                            }
                        } else {
                            Instruction::JumpIfFalse {
                                test: from,
                                target: 0,
                            }
                        };
                        decided = Some(self.emit(jump, *operator_at));
                        self.expression(right, to)?;
                    }
                    _ => {
                        let current = self.temporary()?;
                        let operand = self.temporary()?;
                        self.emit(
                            Instruction::ReadTarget {
                                to: current as u8,
                                target: read,
                            },
                            *operator_at,
                        );
                        self.expression(right, operand)?;
                        self.emit(
                            Instruction::BinaryOperator {
                                to: from,
                                first: current as u8,
                                op: operator_index(op),
                            },
                            *operator_at,
                        );
                    }
                }
            }
            // `held <<= 2` calls the shift the value defines.
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                position: call_at,
                ..
            } if receiver.as_ref() == target => {
                let read = wide(Some(self.unit.target(shape.clone())))?;
                let base = self.temporary()?;
                self.emit(
                    Instruction::ReadTarget {
                        to: base as u8,
                        target: read,
                    },
                    *call_at,
                );
                self.plain_arguments(arguments)?;
                let site = CallSite::new(method, arguments.len(), Receiver::Sent);
                let site = wide(Some(self.unit.call_site(site)))?;
                self.emit(
                    Instruction::Call {
                        base: base as u8,
                        site,
                    },
                    *call_at,
                );
                self.emit(
                    Instruction::Move {
                        to: from,
                        from: base as u8,
                    },
                    *call_at,
                );
            }
            _ => self.expression(value, to)?,
        }
        let store = wide(Some(self.unit.target(shape)))?;
        self.emit(
            Instruction::StoreTarget {
                from,
                target: store,
            },
            position,
        );
        if let Some(decided) = decided {
            self.jump_here(decided)?;
        }
        if let Some(missing_receiver) = missing_receiver {
            let done = self.emit(Instruction::Jump { target: 0 }, position);
            self.jump_here(missing_receiver)?;
            self.emit(Instruction::LoadNil { to: from }, position);
            self.jump_here(done)?;
        }
        Ok(())
    }

    /// Evaluate what `target` needs ahead of its value into registers kept
    /// until the assignment is done: the receiver of a setter, the receiver
    /// and subscripts of `[]=`, and the namespace of `Ns::Name`. Ruby
    /// evaluates these left to right before the right-hand side.
    fn prepare(&mut self, target: &Expression) -> Result<Target, &'static str> {
        Ok(match target {
            // `(a, b)` names a group of targets, which takes its value
            // apart the way the whole list does.
            Expression::Array { elements, position } => {
                let mut parts = Vec::with_capacity(elements.len());
                for element in elements {
                    parts.push(self.prepare(element)?);
                }
                Target::Group(parts, *position)
            }
            Expression::Splat { expression, .. } => {
                Target::Splat(Box::new(self.prepare(expression)?))
            }
            Expression::Index {
                array,
                index,
                position,
            } => {
                let receiver = self.temporary()?;
                self.expression(array, receiver)?;
                // `held[*subscripts] = value` spreads the values across the
                // subscript, the same way `held.[]=(*subscripts, value)` does.
                if let Expression::Splat {
                    expression: spread,
                    position: splat_at,
                } = index.as_ref()
                {
                    let first = self.spread_subscripts(spread, *splat_at)?;
                    return Ok(Target::Subscript {
                        receiver,
                        first,
                        count: 1,
                        spread: true,
                        sends: true,
                        position: *position,
                    });
                }
                let first = self.temporary()?;
                self.expression(index, first)?;
                Target::Subscript {
                    receiver,
                    first,
                    count: 1,
                    spread: false,
                    sends: false,
                    position: *position,
                }
            }
            Expression::MethodCall {
                receiver: receiver_expression,
                method,
                arguments,
                position,
                ..
            } => {
                let receiver = self.temporary()?;
                self.expression(receiver_expression, receiver)?;
                match (method.as_str(), arguments.as_slice()) {
                    // `values[start, length] = other` parses as a call to
                    // `[]`, and assigning to it sends `[]=` with the same
                    // subscripts.
                    ("[]", _) if arguments.iter().any(gathered_in_a_list) => {
                        let first = self.temporary()?;
                        let mut site = self.call_arguments("[]", arguments, None)?;
                        site.arguments_in_list = true;
                        self.emit(Instruction::ArgumentsToArray { to: first as u8 }, *position);
                        self.next_register = first + 1;
                        Target::Subscript {
                            receiver,
                            first,
                            count: 1,
                            spread: true,
                            sends: true,
                            position: *position,
                        }
                    }
                    ("[]", _) => {
                        let first = self.next_register;
                        self.plain_arguments(arguments)?;
                        Target::Subscript {
                            receiver,
                            first,
                            count: arguments.len(),
                            spread: false,
                            sends: true,
                            position: *position,
                        }
                    }
                    (crate::parser::SAFE_CALL, [Expression::Symbol { value: named, .. }]) => {
                        Target::Setter {
                            receiver,
                            reader: named.clone(),
                            safe: true,
                            names_self: receiver_written(receiver_expression) != Receiver::Other,
                            position: *position,
                        }
                    }
                    _ => Target::Setter {
                        receiver,
                        reader: method.clone(),
                        safe: false,
                        names_self: receiver_written(receiver_expression) != Receiver::Other,
                        position: *position,
                    },
                }
            }
            Expression::ScopeResolution {
                namespace,
                name,
                position,
            } => {
                let register = self.temporary()?;
                self.expression(namespace, register)?;
                Target::ScopedConstant {
                    namespace: register,
                    name: name.clone(),
                    or_assigning: false,
                    position: *position,
                }
            }
            named => Target::Named(named.clone()),
        })
    }

    /// Compile `targets = values`, which spreads the values across the
    /// targets and answers the right-hand side as written.
    fn multiple_assignment(
        &mut self,
        targets: &[Expression],
        values: &[Expression],
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let held = self.next_register;
        let mut prepared = Vec::with_capacity(targets.len());
        for target in targets {
            prepared.push(self.prepare(target)?);
        }
        let lone_value = match values {
            [single] if !matches!(single, Expression::Splat { .. }) => {
                self.expression(single, to)?;
                true
            }
            _ => {
                let gathered = Expression::Array {
                    elements: values.to_vec(),
                    position,
                };
                self.expression(&gathered, to)?;
                false
            }
        };
        let shape = wide(Some(self.unit.multiple_assignment(MultipleAssignment {
            targets: prepared,
            lone_value,
            position,
        })))?;
        self.emit(
            Instruction::AssignMultiple {
                from: to as u8,
                shape,
            },
            position,
        );
        self.next_register = held;
        Ok(())
    }

    /// Compile a call's arguments to consecutive registers from the next
    /// free one, refusing the forms the compiler does not handle yet.
    fn plain_arguments(&mut self, arguments: &[Expression]) -> Result<(), &'static str> {
        for argument in arguments {
            if matches!(
                argument,
                Expression::Splat { .. }
                    | Expression::KeywordSplat { .. }
                    | Expression::BlockArg { .. }
            ) {
                return Err(argument.kind_name());
            }
            let register = self.temporary()?;
            self.expression(argument, register)?;
        }
        Ok(())
    }

    /// Point the jump at `at` to the next instruction.
    fn jump_here(&mut self, at: usize) -> Result<(), &'static str> {
        let here = self.unit.instructions.len();
        self.jump_to(at, here)
    }

    /// Point the jump at `at` to the instruction at `target`.
    fn jump_to(&mut self, at: usize, target: usize) -> Result<(), &'static str> {
        let here = wide(Some(target))?;
        self.unit.instructions[at] = match self.unit.instructions[at] {
            Instruction::Jump { .. } => Instruction::Jump { target: here },
            Instruction::JumpIfTrue { test, .. } => Instruction::JumpIfTrue { test, target: here },
            Instruction::JumpIfFalse { test, .. } => {
                Instruction::JumpIfFalse { test, target: here }
            }
            Instruction::JumpIfNil { test, .. } => Instruction::JumpIfNil { test, target: here },
            Instruction::SafeNavigation { test, .. } => {
                Instruction::SafeNavigation { test, target: here }
            }
            other => unreachable!("{} is not a jump", other.name()),
        };
        Ok(())
    }

    /// Compile `expression` to leave its value in register `to`.
    fn expression(&mut self, expression: &Expression, to: usize) -> Result<(), &'static str> {
        self.spots.push(
            crate::vm::error_spots::spot_of(expression)
                .map(|(start, name)| ErrorSpot { start, name }),
        );
        let held = self.next_register;
        let compiled = self
            .expression_in_place(expression, to)
            .map_err(naming(expression.kind_name()));
        self.next_register = held;
        self.spots.pop();
        compiled
    }

    fn expression_in_place(
        &mut self,
        expression: &Expression,
        to: usize,
    ) -> Result<(), &'static str> {
        let position = expression.position();
        let to_register = to as u8;
        let instruction = match expression {
            Expression::IntLiteral { value, .. } => self.integer(*value, to_register)?,
            Expression::MagicLine { position } => {
                self.integer(position.line as i64, to_register)?
            }
            Expression::BigIntLiteral { digits, .. } => {
                let Some(value) = num_bigint::BigInt::parse_bytes(digits.as_bytes(), 10) else {
                    return Err(expression.kind_name());
                };
                Instruction::LoadConstant {
                    to: to_register,
                    constant: self.constant(Object::integer(value))?,
                }
            }
            Expression::RegexLiteral { pattern, flags, .. } => Instruction::LoadRegex {
                to: to_register,
                pattern: wide(Some(self.unit.regex(RegexLiteral {
                    pattern: pattern.clone(),
                    flags: flags.clone(),
                })))?,
            },
            Expression::MagicFile { .. } => Instruction::LoadFile { to: to_register },
            Expression::MagicDir { .. } => Instruction::LoadDirectory { to: to_register },
            Expression::ClassVariable { name, .. } => Instruction::GetClassVariable {
                to: to_register,
                name: self.name(name)?,
            },
            Expression::TopLevelConstant { name, .. } => Instruction::GetTopLevelConstant {
                to: to_register,
                name: self.name(name)?,
            },
            Expression::ScopeResolution {
                namespace, name, ..
            } => {
                self.expression(namespace, to)?;
                Instruction::GetScopedConstant {
                    namespace: to_register,
                    name: self.name(name)?,
                }
            }
            Expression::Range {
                start,
                end,
                exclusive,
                ..
            } => {
                let first = self.temporary()?;
                self.expression(start, first)?;
                let last = self.temporary()?;
                self.expression(end, last)?;
                let mut flags = 0;
                if *exclusive {
                    flags |= RANGE_EXCLUSIVE;
                }
                if crate::vm::eval::literal_range_end(start)
                    && crate::vm::eval::literal_range_end(end)
                {
                    flags |= RANGE_LITERAL_ENDS;
                }
                Instruction::NewRange {
                    to: to_register,
                    first: first as u8,
                    flags,
                }
            }
            Expression::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                position,
            } => {
                return self.branches(
                    condition,
                    false,
                    then_branch,
                    elsif_branches,
                    else_branch.as_deref(),
                    to,
                    *position,
                    true,
                );
            }
            Expression::Unless {
                condition,
                then_branch,
                else_branch,
                position,
            } => {
                return self.branches(
                    condition,
                    true,
                    then_branch,
                    &[],
                    else_branch.as_deref(),
                    to,
                    *position,
                    true,
                );
            }
            Expression::Case {
                expression: subject,
                cases,
                else_case,
                position,
            } => {
                return self.case_expression(subject, cases, else_case.as_deref(), to, *position);
            }
            Expression::Defined { expression, .. } => Instruction::IsDefined {
                to: to_register,
                expression: wide(Some(self.unit.defined_expression(*expression.clone())))?,
            },
            Expression::Index { array, index, .. } => {
                let collection = self.temporary()?;
                self.expression(array, collection)?;
                // `held[*subscripts]` spreads the values across the
                // subscript, the same way `held.[](*subscripts)` does.
                if let Expression::Splat {
                    expression: spread,
                    position: splat_at,
                } = index.as_ref()
                {
                    let subscripts = self.spread_subscripts(spread, *splat_at)?;
                    self.emit(
                        Instruction::IndexSpread {
                            to: to_register,
                            collection: collection as u8,
                            subscripts: subscripts as u8,
                        },
                        position,
                    );
                    return Ok(());
                }
                let key = self.temporary()?;
                self.expression(index, key)?;
                Instruction::Index {
                    to: to_register,
                    collection: collection as u8,
                    key: key as u8,
                }
            }
            Expression::Dictionary { entries, .. } => {
                let entries = crate::vm::expression::flattened_entries(entries);
                let mut chunks = entries.chunks(LITERAL_CHUNK);
                let shape = self.hash_shape(chunks.next().unwrap_or(&[]))?;
                let shape = wide(Some(self.unit.hash(shape)))?;
                self.emit(
                    Instruction::NewHash {
                        to: to_register,
                        shape,
                    },
                    position,
                );
                for chunk in chunks {
                    let shape = self.hash_shape(chunk)?;
                    let shape = wide(Some(self.unit.hash(shape)))?;
                    self.emit(
                        Instruction::AddToHash {
                            hash: to_register,
                            shape,
                        },
                        position,
                    );
                }
                return Ok(());
            }
            Expression::InterpolatedString { parts, .. } => {
                let pieces = self.chunked_pieces(parts)?;
                Instruction::Interpolate {
                    to: to_register,
                    shape: wide(Some(self.unit.interpolation(pieces)))?,
                }
            }
            Expression::FloatLiteral { value, .. } => Instruction::LoadConstant {
                to: to_register,
                constant: self.constant(Object::Float(*value))?,
            },
            Expression::BoolLiteral { value: true, .. } => {
                Instruction::LoadTrue { to: to_register }
            }
            Expression::BoolLiteral { value: false, .. } => {
                Instruction::LoadFalse { to: to_register }
            }
            Expression::NilLiteral { .. } => Instruction::LoadNil { to: to_register },
            Expression::SelfExpr { .. } => Instruction::LoadSelf { to: to_register },
            Expression::StringLiteral { value, .. } => Instruction::NewString {
                to: to_register,
                text: self.name(value)?,
            },
            Expression::Symbol { value, .. } => Instruction::LoadSymbol {
                to: to_register,
                name: self.name(value)?,
            },
            Expression::Identifier { name, .. } => self.identifier(name, to_register)?,
            Expression::InstanceVariable { name, .. } => Instruction::GetInstanceVariable {
                to: to_register,
                name: self.name(name)?,
            },
            Expression::GlobalVariable { name, .. } => Instruction::GetGlobal {
                to: to_register,
                name: self.name(name)?,
            },
            Expression::Grouped { expression, .. } => return self.expression(expression, to),
            Expression::BinaryOp {
                op: BinaryOp::Assign,
                left,
                right,
                ..
            } => return self.assignment(left, right, to, position),
            Expression::BinaryOp {
                op: op @ (BinaryOp::And | BinaryOp::Or),
                left,
                right,
                ..
            } => {
                self.expression(left, to)?;
                let jump = match op {
                    BinaryOp::And => Instruction::JumpIfFalse {
                        test: to_register,
                        target: 0,
                    },
                    _ => Instruction::JumpIfTrue {
                        test: to_register,
                        target: 0,
                    },
                };
                let decided = self.emit(jump, position);
                self.expression(right, to)?;
                return self.jump_here(decided);
            }
            Expression::BinaryOp {
                op, left, right, ..
            } => {
                let Some(make) = binary_instruction(op) else {
                    let first = self.register_after(to)?;
                    self.expression(left, first)?;
                    let second = self.temporary()?;
                    self.expression(right, second)?;
                    self.emit(
                        Instruction::BinaryOperator {
                            to: to_register,
                            first: u8::try_from(first).map_err(|_| OVER_LIMIT)?,
                            op: operator_index(op),
                        },
                        position,
                    );
                    return Ok(());
                };
                // A chain such as `a + b + c` takes no more registers for
                // each operator, its left operands all computed in `to`.
                let left_register = match to < usize::from(FIRST_CONSTANT_OPERAND) {
                    true => to,
                    false => self.operand_temporary(expression)?,
                };
                self.expression(left, left_register)?;
                let right_register = self.operand_temporary(expression)?;
                self.expression(right, right_register)?;
                make(to_register, left_register as u8, right_register as u8)
            }
            Expression::UnaryOp { op, operand, .. } => {
                self.expression(operand, to)?;
                match op {
                    UnaryOp::Minus => Instruction::Negate {
                        to: to_register,
                        from: to_register,
                    },
                    UnaryOp::Not => Instruction::Not {
                        to: to_register,
                        from: to_register,
                    },
                    UnaryOp::Plus => Instruction::Positive {
                        to: to_register,
                        from: to_register,
                    },
                }
            }
            Expression::Array { elements, .. }
                if elements
                    .iter()
                    .any(|element| matches!(element, Expression::Splat { .. })) =>
            {
                // `[first, *rest]` splices what a splat spreads in place.
                self.emit(
                    Instruction::NewArray {
                        to: to_register,
                        first: 0,
                        count: 0,
                    },
                    position,
                );
                for element in elements {
                    let register = self.temporary()?;
                    let instruction = match element {
                        Expression::Splat { expression, .. } => {
                            self.expression(expression, register)?;
                            Instruction::SplatInto {
                                array: to_register,
                                from: register as u8,
                            }
                        }
                        _ => {
                            self.expression(element, register)?;
                            Instruction::AppendArray {
                                array: to_register,
                                first: register as u8,
                                count: 1,
                            }
                        }
                    };
                    self.emit(instruction, element.position());
                    self.next_register = register;
                }
                return Ok(());
            }
            // Outside of an argument list, a splat makes a new Array of what
            // it spreads, the way `[*held]` does.
            Expression::Splat {
                expression: spread, ..
            } => {
                self.emit(
                    Instruction::NewArray {
                        to: to_register,
                        first: 0,
                        count: 0,
                    },
                    position,
                );
                let register = self.temporary()?;
                self.expression(spread, register)?;
                Instruction::SplatInto {
                    array: to_register,
                    from: register as u8,
                }
            }
            // Outside of an argument list, `**expr` and `&expr` are `expr`.
            Expression::KeywordSplat {
                expression: inner, ..
            }
            | Expression::BlockArg {
                expression: inner, ..
            } => return self.expression(inner, to),
            Expression::BeginRescue {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                ..
            } => {
                return self.begin(
                    body,
                    rescue_clauses,
                    else_clause.as_deref(),
                    ensure_block.as_deref(),
                    to,
                    position,
                    true,
                );
            }
            Expression::Yield { arguments, .. } => {
                // The block is looked for before the arguments are evaluated,
                // so a method called without one raises before they are.
                let base = self.temporary()?;
                self.emit(Instruction::FindBlock { to: base as u8 }, position);
                let site = self.call_arguments("yield", arguments, None)?;
                let site = wide(Some(self.unit.call_site(site)))?;
                self.emit(
                    Instruction::CallBlock {
                        base: base as u8,
                        site,
                    },
                    position,
                );
                Instruction::Move {
                    to: to_register,
                    from: base as u8,
                }
            }
            Expression::Super {
                arguments,
                forward_args,
                trailing_block,
                ..
            } => {
                let base = self.temporary()?;
                let mut site = CallSite::new("super", 0, Receiver::SelfKeyword);
                site.forwards_arguments = *forward_args;
                // A block written on the call is evaluated before the
                // arguments are.
                if let Some(block) = trailing_block {
                    let register = self.temporary()?;
                    self.expression(block, register)?;
                    self.emit(
                        Instruction::AttachSuperBlock {
                            from: register as u8,
                        },
                        position,
                    );
                    site.has_block = true;
                }
                if !*forward_args {
                    site.block_argument = arguments
                        .iter()
                        .any(|argument| matches!(argument, Expression::BlockArg { .. }));
                    if site.block_argument {
                        self.emit(
                            Instruction::ClearSuperBlock {
                                written: u8::from(trailing_block.is_some()),
                            },
                            position,
                        );
                    }
                    // The block follows the receiver's register when the
                    // arguments are in a list, and the arguments otherwise,
                    // so they are gathered in a list when a block was
                    // written ahead of them.
                    let listed =
                        self.gathered_arguments("super", arguments, None, site.has_block)?;
                    site.written_with_arguments = !arguments.is_empty();
                    site.keyword_splat = listed.keyword_splat;
                    site.argument_count = listed.argument_count;
                    site.arguments_in_list = listed.arguments_in_list;
                }
                let site = wide(Some(self.unit.call_site(site)))?;
                self.emit(
                    Instruction::CallSuper {
                        base: base as u8,
                        site,
                    },
                    position,
                );
                Instruction::Move {
                    to: to_register,
                    from: base as u8,
                }
            }
            Expression::SingletonClass {
                target,
                assign_to,
                body,
                position: opened_at,
            } => {
                let held = self.temporary()?;
                self.expression(target, held)?;
                // `class << @receiver = Object.new` stores the object first,
                // so the body's `def`s land on the same one the name holds.
                if let Some(destination) = assign_to {
                    self.store_register(destination, held, *opened_at)?;
                }
                return self.define(
                    Definition::SingletonClass {
                        target: held,
                        body: body.clone(),
                        position: *opened_at,
                    },
                    to,
                    *opened_at,
                );
            }
            Expression::Lambda { .. } => Instruction::MakeBlock {
                to: to_register,
                literal: wide(Some(self.unit.block(expression.clone())))?,
            },
            Expression::PatternTest {
                value,
                pattern,
                refuses,
                ..
            } => {
                self.expression(value, to)?;
                Instruction::MatchPattern {
                    register: to_register,
                    pattern: wide(Some(self.unit.pattern((**pattern).clone(), *refuses)))?,
                }
            }
            Expression::Call {
                callee,
                arguments,
                trailing_block,
                ..
            } => return self.bare_call(callee, arguments, trailing_block.as_deref(), to, position),
            Expression::Array { elements, .. } => {
                // A long literal is built a chunk of elements at a time, so
                // its elements need no more registers than a chunk does.
                let mut chunks = elements.chunks(LITERAL_CHUNK);
                let first_chunk = chunks.next().unwrap_or(&[]);
                let first = self.chunk_of_elements(first_chunk)?;
                self.emit(
                    Instruction::NewArray {
                        to: to_register,
                        first,
                        count: first_chunk.len() as u8,
                    },
                    position,
                );
                for chunk in chunks {
                    let first = self.chunk_of_elements(chunk)?;
                    self.emit(
                        Instruction::AppendArray {
                            array: to_register,
                            first,
                            count: chunk.len() as u8,
                        },
                        position,
                    );
                }
                return Ok(());
            }
            Expression::MethodCall {
                receiver,
                method,
                arguments,
                trailing_block,
                ..
            } => {
                return self.method_call(
                    receiver,
                    method,
                    arguments,
                    trailing_block.as_deref(),
                    to,
                    position,
                );
            }
        };
        self.emit(instruction, position);
        Ok(())
    }

    /// Compile `receiver.method(arguments) { block }`, leaving the answer in
    /// register `to`.
    fn method_call(
        &mut self,
        receiver: &Expression,
        method: &str,
        arguments: &[Expression],
        trailing_block: Option<&Expression>,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let to_register = to as u8;
        // `native_fn[args]` calls the Kernel function with the bracketed
        // arguments in an Array, which is `private [:a, :b]`. The name is
        // looked up before the receiver is evaluated, since evaluating it
        // would run the function.
        let mut native_done = None;
        if method == "[]"
            && let Expression::Identifier { name, .. } = receiver
        {
            let base = self.temporary()?;
            let name_index = self.name(name)?;
            self.emit(
                Instruction::IsNativeName {
                    to: base as u8,
                    name: name_index,
                },
                position,
            );
            let not_native = self.emit(
                Instruction::JumpIfFalse {
                    test: base as u8,
                    target: 0,
                },
                position,
            );
            let mut site = self.call_arguments(name, arguments, None)?;
            site.receiver = Receiver::SelfKeyword;
            let site = wide(Some(self.unit.call_site(site)))?;
            self.emit(
                Instruction::CallNativeName {
                    base: base as u8,
                    site,
                },
                position,
            );
            self.emit(
                Instruction::Move {
                    to: to_register,
                    from: base as u8,
                },
                position,
            );
            native_done = Some(self.emit(Instruction::Jump { target: 0 }, position));
            self.jump_here(not_native)?;
            self.next_register = base;
        }
        let base = self.register_after(to)?;
        self.expression(receiver, base)?;
        // `held&.name(args) { block }` runs none of what follows when there
        // is no receiver, so the arguments are never evaluated.
        let missing_receiver = (method == crate::parser::SAFE_CALL).then(|| {
            self.emit(
                Instruction::SafeNavigation {
                    test: base as u8,
                    target: 0,
                },
                position,
            )
        });
        let mut site = self.call_arguments(method, arguments, trailing_block)?;
        site.receiver = receiver_written(receiver);
        let site = wide(Some(self.unit.call_site(site)))?;
        self.emit(
            Instruction::Call {
                base: base as u8,
                site,
            },
            position,
        );
        if base != to {
            self.emit(
                Instruction::Move {
                    to: to_register,
                    from: base as u8,
                },
                position,
            );
        }
        if let Some(missing_receiver) = missing_receiver {
            let done = self.emit(Instruction::Jump { target: 0 }, position);
            self.jump_here(missing_receiver)?;
            self.emit(Instruction::LoadNil { to: to_register }, position);
            self.jump_here(done)?;
        }
        if let Some(native_done) = native_done {
            self.jump_here(native_done)?;
        }
        Ok(())
    }

    /// Compile `name(arguments) { block }`, written without a receiver, or
    /// `Namespace::name(arguments)`, leaving the answer in register `to`.
    fn bare_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        trailing_block: Option<&Expression>,
        to: usize,
        position: Position,
    ) -> Result<(), &'static str> {
        let (name, namespace) = match callee {
            Expression::Identifier { name, .. } => (name, None),
            Expression::ScopeResolution {
                namespace, name, ..
            } => (name, Some(namespace.as_ref())),
            other => return Err(other.kind_name()),
        };
        let resolve_at = match namespace {
            None => self.emit(Instruction::ResolveCall { site: 0 }, position),
            Some(namespace) => {
                let register = self.temporary()?;
                self.expression(namespace, register)?;
                self.emit(
                    Instruction::ResolveScopedCall {
                        namespace: register as u8,
                        site: 0,
                    },
                    position,
                )
            }
        };
        let base = self.temporary()?;
        let mut site = self.call_arguments(name, arguments, trailing_block)?;
        site.receiver = match namespace {
            None => Receiver::SelfKeyword,
            Some(namespace) => receiver_written(namespace),
        };
        let site = wide(Some(self.unit.call_site(site)))?;
        self.unit.instructions[resolve_at] = match self.unit.instructions[resolve_at] {
            Instruction::ResolveScopedCall { namespace, .. } => {
                Instruction::ResolveScopedCall { namespace, site }
            }
            _ => Instruction::ResolveCall { site },
        };
        self.emit(
            Instruction::CallResolved {
                base: base as u8,
                site,
            },
            position,
        );
        self.emit(
            Instruction::Move {
                to: to as u8,
                from: base as u8,
            },
            position,
        );
        Ok(())
    }

    /// Compile a call's arguments and block to the registers after the
    /// receiver's, which is the register before the next free one, and
    /// answer the call site for them. Arguments with `*`, `**`, or `&` among
    /// them are gathered in an argument list instead.
    fn call_arguments(
        &mut self,
        name: &str,
        arguments: &[Expression],
        trailing_block: Option<&Expression>,
    ) -> Result<CallSite, &'static str> {
        self.gathered_arguments(name, arguments, trailing_block, false)
    }

    /// Compile a call's arguments as `call_arguments` does, gathering them in
    /// an argument list when `listed` even with none of `*`, `**`, or `&`.
    fn gathered_arguments(
        &mut self,
        name: &str,
        arguments: &[Expression],
        trailing_block: Option<&Expression>,
        listed: bool,
    ) -> Result<CallSite, &'static str> {
        let in_list =
            listed || arguments.len() > LITERAL_CHUNK || arguments.iter().any(gathered_in_a_list);
        let mut site = CallSite::new(name, 0, Receiver::Other);
        site.written_with_arguments = !arguments.is_empty();
        site.keyword_splat = arguments
            .iter()
            .any(|argument| matches!(argument, Expression::KeywordSplat { .. }));
        site.has_block = trailing_block.is_some();
        if !in_list {
            site.argument_count = arguments.len();
            self.plain_arguments(arguments)?;
            if let Some(block) = trailing_block {
                let register = self.temporary()?;
                self.expression(block, register)?;
            }
            return Ok(site);
        }
        site.arguments_in_list = true;
        let block_register = match trailing_block {
            Some(_) => Some(self.temporary()?),
            None => None,
        };
        let start_at = arguments
            .first()
            .map(Expression::position)
            .unwrap_or_default();
        self.emit(Instruction::StartArguments, start_at);
        for argument in arguments {
            let register = self.temporary()?;
            let (value, add): (&Expression, fn(u8) -> Instruction) = match argument {
                Expression::Splat { expression, .. } => {
                    (expression, |from| Instruction::SplatArgument { from })
                }
                Expression::KeywordSplat { expression, .. } => (expression, |from| {
                    Instruction::KeywordSplatArgument { from }
                }),
                Expression::BlockArg { expression, .. } => {
                    (expression, |from| Instruction::BlockArgument { from })
                }
                plain => (plain, |from| Instruction::PushArgument { from }),
            };
            self.expression(value, register)?;
            self.emit(add(register as u8), argument.position());
            self.next_register = register;
        }
        if let (Some(block), Some(register)) = (trailing_block, block_register) {
            self.expression(block, register)?;
        }
        Ok(site)
    }

    /// Compile `*spread` written as a subscript to an Array of the values it
    /// spreads, in a register kept until the subscript is used, answering
    /// the register.
    fn spread_subscripts(
        &mut self,
        spread: &Expression,
        splat_at: Position,
    ) -> Result<usize, &'static str> {
        let subscripts = self.temporary()?;
        let value = self.temporary()?;
        self.emit(Instruction::StartArguments, splat_at);
        self.expression(spread, value)?;
        self.emit(Instruction::SplatArgument { from: value as u8 }, splat_at);
        self.emit(
            Instruction::ArgumentsToArray {
                to: subscripts as u8,
            },
            splat_at,
        );
        self.next_register = value;
        Ok(subscripts)
    }

    /// Load an Integer, in the instruction when it fits 16 bits.
    fn integer(&mut self, value: i64, to: u8) -> Result<Instruction, &'static str> {
        Ok(match i16::try_from(value) {
            Ok(value) => Instruction::LoadInteger { to, value },
            Err(_) => Instruction::LoadConstant {
                to,
                constant: self.constant(Object::Int(value))?,
            },
        })
    }

    /// Compile a Hash literal's keys and values, in the order written, to
    /// consecutive registers.
    fn hash_shape(
        &mut self,
        entries: &[(&Expression, &Expression)],
    ) -> Result<HashShape, &'static str> {
        let first = self.next_register;
        let mut shape = HashShape {
            first,
            entries: Vec::new(),
        };
        for &(key, value) in entries {
            if let Expression::KeywordSplat {
                expression,
                position,
            } = key
            {
                let register = self.temporary()?;
                self.expression(expression, register)?;
                shape.entries.push(HashEntry::Spread {
                    position: *position,
                });
                continue;
            }
            let key_register = self.temporary()?;
            self.expression(key, key_register)?;
            let value_register = self.temporary()?;
            self.expression(value, value_register)?;
            shape.entries.push(HashEntry::Pair {
                key_position: key.position(),
                value_line: value.position().line,
                literal_key: crate::vm::expression::is_literal_key(key),
            });
        }
        self.next_register = first;
        Ok(shape)
    }

    /// Compile a chunk of an Array literal's elements to consecutive
    /// registers, answering the first.
    fn chunk_of_elements(&mut self, elements: &[Expression]) -> Result<u8, &'static str> {
        let held = self.next_register;
        for element in elements {
            let register = self.temporary()?;
            self.expression(element, register)?;
        }
        self.next_register = held;
        u8::try_from(held).map_err(|_| OVER_LIMIT)
    }

    /// Compile an interpolated String's holes as `pieces` does, a long run
    /// of them first joined into a String a chunk at a time.
    fn chunked_pieces(&mut self, parts: &[InterpolationPart]) -> Result<Vec<Piece>, &'static str> {
        let holes = parts
            .iter()
            .filter(|part| matches!(part, InterpolationPart::Expression(_)))
            .count();
        if holes <= INTERPOLATION_CHUNK {
            return self.pieces(parts);
        }
        let mut pieces = Vec::new();
        let mut chunk_start = 0;
        let mut chunk_holes = 0;
        for (at, part) in parts.iter().enumerate() {
            if let InterpolationPart::Expression(_) = part {
                chunk_holes += 1;
            }
            if chunk_holes == INTERPOLATION_CHUNK || at + 1 == parts.len() {
                pieces.push(self.joined_chunk(&parts[chunk_start..=at])?);
                chunk_start = at + 1;
                chunk_holes = 0;
            }
        }
        Ok(pieces)
    }

    /// Join `parts` into a String in a register followed by one holding the
    /// same String, answering the piece naming them.
    fn joined_chunk(&mut self, parts: &[InterpolationPart]) -> Result<Piece, &'static str> {
        let register = self.temporary()?;
        let text = self.temporary()?;
        let position = parts
            .iter()
            .find_map(|part| match part {
                InterpolationPart::Expression(hole) => Some(hole.position()),
                InterpolationPart::Text(_) => None,
            })
            .unwrap_or_default();
        let held = self.next_register;
        let pieces = self.pieces(parts)?;
        self.next_register = held;
        let shape = wide(Some(self.unit.interpolation(pieces)))?;
        self.emit(
            Instruction::Interpolate {
                to: register as u8,
                shape,
            },
            position,
        );
        self.emit(
            Instruction::Move {
                to: text as u8,
                from: register as u8,
            },
            position,
        );
        Ok(Piece::Value { register, position })
    }

    /// Compile an interpolated String's holes, each to a register followed
    /// by one holding its text.
    fn pieces(&mut self, parts: &[InterpolationPart]) -> Result<Vec<Piece>, &'static str> {
        let mut pieces = Vec::with_capacity(parts.len());
        for part in parts {
            match part {
                InterpolationPart::Text(text) => pieces.push(Piece::Text(text.clone())),
                InterpolationPart::Expression(hole) => {
                    let register = self.temporary()?;
                    let text = self.temporary()?;
                    self.expression(hole, register)?;
                    self.emit(
                        Instruction::ToText {
                            to: text as u8,
                            from: register as u8,
                        },
                        hole.position(),
                    );
                    pieces.push(Piece::Value {
                        register,
                        position: hole.position(),
                    });
                }
            }
        }
        Ok(pieces)
    }

    /// A register for an operand of an arithmetic or comparison
    /// instruction, which names registers below 128 only.
    fn operand_temporary(&mut self, expression: &Expression) -> Result<usize, &'static str> {
        let register = self.temporary()?;
        if register >= usize::from(FIRST_CONSTANT_OPERAND) {
            return Err(expression.kind_name());
        }
        Ok(register)
    }

    fn identifier(&mut self, name: &str, to: u8) -> Result<Instruction, &'static str> {
        if let Some(slot) = self
            .assigned
            .iter()
            .position(|held| held == name)
            .and_then(|_| self.unit.locals.iter().position(|held| held.text == name))
        {
            return Ok(Instruction::GetLocal {
                to,
                slot: wide(Some(slot))?,
            });
        }
        let name_index = self.name(name)?;
        Ok(if starts_uppercase(name) {
            Instruction::GetConstant {
                to,
                name: name_index,
            }
        } else {
            Instruction::GetIdentifier {
                to,
                name: name_index,
            }
        })
    }

    fn name(&mut self, text: &str) -> Result<u16, &'static str> {
        let at = self.unit.name(Name::new(text));
        wide(Some(at))
    }

    fn constant(&mut self, value: Object) -> Result<u16, &'static str> {
        let at = self.unit.constant(value);
        wide(Some(at))
    }
}

/// A 16-bit operand for a side table index.
fn wide(index: Option<usize>) -> Result<u16, &'static str> {
    index
        .and_then(|index| u16::try_from(index).ok())
        .ok_or(OVER_LIMIT)
}

/// Whether an argument is `*value`, `**value`, or `&value`, which a call
/// gathers in an argument list.
fn gathered_in_a_list(argument: &Expression) -> bool {
    matches!(
        argument,
        Expression::Splat { .. } | Expression::KeywordSplat { .. } | Expression::BlockArg { .. }
    )
}

/// Whether a method body's last statement answers the method's value
/// itself.
pub(crate) fn answers_its_own_value(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::Expression { .. }
            | Statement::Assignment { .. }
            | Statement::If { .. }
            | Statement::Unless { .. }
            | Statement::Begin { .. }
    )
}

/// What a handler's instructions find open, counted from what the run had
/// open as it began.
fn open_at(pending: Pending) -> OpenAtHandler {
    OpenAtHandler {
        lists: pending.lists,
        routes: pending.routes,
        scopes: pending.scopes,
        loops: pending.loops,
    }
}

fn starts_uppercase(name: &str) -> bool {
    name.starts_with(|first: char| first.is_ascii_uppercase())
}

/// How a call's receiver was written.
fn receiver_written(receiver: &Expression) -> Receiver {
    match receiver {
        Expression::SelfExpr { .. } => Receiver::SelfKeyword,
        Expression::Identifier { name, .. } if name == "self" => Receiver::SelfName,
        _ => Receiver::Other,
    }
}

/// The `op` operand `BinaryOperator` applies `op` with.
fn operator_index(op: &BinaryOp) -> u8 {
    BINARY_OPERATORS
        .iter()
        .position(|held| held == op)
        .expect("an operator BinaryOperator applies") as u8
}

type BinaryInstruction = fn(u8, u8, u8) -> Instruction;

/// The instruction applying `op`, for the operators that have one.
fn binary_instruction(op: &BinaryOp) -> Option<BinaryInstruction> {
    Some(match op {
        BinaryOp::Add => |to, left, right| Instruction::Add { to, left, right },
        BinaryOp::Subtract => |to, left, right| Instruction::Subtract { to, left, right },
        BinaryOp::Multiply => |to, left, right| Instruction::Multiply { to, left, right },
        BinaryOp::Divide => |to, left, right| Instruction::Divide { to, left, right },
        BinaryOp::Modulo => |to, left, right| Instruction::Modulo { to, left, right },
        BinaryOp::Equal => |to, left, right| Instruction::Equal { to, left, right },
        BinaryOp::NotEqual => |to, left, right| Instruction::NotEqual { to, left, right },
        BinaryOp::Less => |to, left, right| Instruction::Less { to, left, right },
        BinaryOp::LessEqual => |to, left, right| Instruction::LessEqual { to, left, right },
        BinaryOp::Greater => |to, left, right| Instruction::Greater { to, left, right },
        BinaryOp::GreaterEqual => |to, left, right| Instruction::GreaterEqual { to, left, right },
        _ => return None,
    })
}
