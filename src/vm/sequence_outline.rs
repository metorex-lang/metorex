//! The outline the heap walk builds a file's instruction sequences from:
//! each method, class, module and block written in the file, with the lines
//! its statements start on. Metorex's own parser reads the file, where
//! `RubyVM::AbstractSyntaxTree` reads it with a parser written in Ruby.
//!
//! Each scope is the Array `[type, label, first line, last line, statement
//! lines, parameters, children]`, with the parameters as `Method#parameters`
//! lists them.

use std::cell::RefCell;

use crate::ast::{Expression, Parameter, Statement};
use crate::bytecode::compiler::UnitKind;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::coverage::{nested_bodies, own_expressions};
use crate::vm::native_methods::method_object_methods::block_parameter_pairs;

/// The outline of a whole program, or None where it does not parse.
pub(crate) fn outline_of_source(source: &str) -> Option<Object> {
    let (statements, closing) = parsed(source)?;
    let context = Context {
        closing,
        wanted: None,
        found: RefCell::new(None),
    };
    Some(outline(
        top_scope(source),
        &statements,
        "<main>",
        0,
        &context,
    ))
}

/// The body of the scope of `kind` that opens on `line` of a program, and
/// what kind of body it is, or None where the program does not parse or
/// holds no such scope. A `top` or `eval` scope is the whole program.
pub(crate) fn body_of_scope(
    source: &str,
    kind: &str,
    line: usize,
) -> Option<(Vec<Statement>, UnitKind)> {
    let (statements, closing) = parsed(source)?;
    if matches!(kind, "top" | "eval") {
        return Some((statements, UnitKind::Plain));
    }
    let context = Context {
        closing,
        wanted: Some((kind.to_string(), line)),
        found: RefCell::new(None),
    };
    outline(top_scope(source), &statements, "<main>", 0, &context);
    context.found.into_inner()
}

/// The instructions the scope `body_of_scope` finds compiles to, as
/// `__unit_listing__` answers them.
pub(crate) fn listing_of_scope(source: &str, kind: &str, line: usize) -> Option<Object> {
    let (body, unit_kind) = body_of_scope(source, kind, line)?;
    let unit = crate::bytecode::compiler::compile_as(&body, unit_kind).ok()?;
    let listed = crate::bytecode::disassembly::listing(&unit)
        .into_iter()
        .map(|listed| {
            Object::array(vec![
                Object::string(listed.name),
                Object::array(
                    listed
                        .operands
                        .into_iter()
                        .map(|(field, value)| {
                            Object::array(vec![Object::symbol(field), Object::Int(value)])
                        })
                        .collect(),
                ),
                Object::Int(listed.line as i64),
                Object::Bool(listed.line_event),
                listed.note.map_or(Object::Nil, Object::string),
            ])
        })
        .collect();
    Some(Object::array(listed))
}

/// A program's statements and the lines its scopes close on.
fn parsed(source: &str) -> Option<(Vec<Statement>, ClosingLines)> {
    let tokens = crate::lexer::Lexer::new(source).tokenize();
    let mut parser = crate::parser::Parser::new(tokens);
    let statements = parser.parse().ok()?;
    let closing = parser.closing_lines().clone();
    Some((statements, closing))
}

fn top_scope(source: &str) -> Scope {
    Scope {
        kind: "top",
        label: "<main>".to_string(),
        first: 1,
        last: source.lines().count().max(1),
        parameters: Vec::new(),
    }
}

/// The lines each class, module and block closes on, by where it opens.
type ClosingLines = std::collections::HashMap<(usize, usize), usize>;

/// What a walk over a program reads by and looks for.
struct Context {
    closing: ClosingLines,
    /// The kind of scope and the line it opens on whose body is wanted.
    wanted: Option<(String, usize)>,
    found: RefCell<Option<(Vec<Statement>, UnitKind)>>,
}

/// What an outline says of one scope before its body is read.
struct Scope {
    kind: &'static str,
    label: String,
    first: usize,
    last: usize,
    parameters: Vec<Object>,
}

/// The outline of a scope running `body`, where `base` is the label of the
/// method, class or program its blocks are counted within and `levels` how
/// many blocks deep the scope sits in it.
fn outline(
    scope: Scope,
    body: &[Statement],
    base: &str,
    levels: usize,
    context: &Context,
) -> Object {
    if let Some((kind, line)) = &context.wanted
        && kind == scope.kind
        && *line == scope.first
        && context.found.borrow().is_none()
    {
        let unit_kind = match scope.kind {
            "method" => UnitKind::Method,
            "class" if scope.label.starts_with("<module:") => UnitKind::ModuleBody,
            "class" => UnitKind::ClassBody,
            _ => UnitKind::Plain,
        };
        *context.found.borrow_mut() = Some((body.to_vec(), unit_kind));
    }
    let mut walk = Walk {
        lines: Vec::new(),
        children: Vec::new(),
        base,
        levels,
        context,
    };
    walk.statements(body);
    walk.lines.dedup();
    Object::array(vec![
        Object::symbol(scope.kind.to_string()),
        Object::string(scope.label),
        Object::Int(scope.first as i64),
        Object::Int(scope.last.max(scope.first) as i64),
        Object::array(
            walk.lines
                .into_iter()
                .map(|line| Object::Int(line as i64))
                .collect(),
        ),
        Object::array(scope.parameters),
        Object::array(walk.children),
    ])
}

/// What reading one scope's body gathers.
struct Walk<'a> {
    lines: Vec<usize>,
    children: Vec<Object>,
    base: &'a str,
    levels: usize,
    context: &'a Context,
}

impl Walk<'_> {
    fn statements(&mut self, body: &[Statement]) {
        for statement in body {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &Statement) {
        match statement {
            Statement::ClassDef {
                name,
                body,
                position,
                ..
            } => self.definition("<class:", name, *position, body),
            Statement::ModuleDef {
                name,
                body,
                position,
                ..
            } => self.definition("<module:", name, *position, body),
            Statement::MethodDef {
                name,
                parameters,
                body,
                position,
                end_position,
                ..
            }
            | Statement::FunctionDef {
                name,
                parameters,
                body,
                position,
                end_position,
                ..
            } => {
                self.lines.push(position.line);
                let scope = Scope {
                    kind: "method",
                    label: name.clone(),
                    first: position.line,
                    last: end_position.line,
                    parameters: parameters.iter().map(parameter_pair).collect(),
                };
                self.children
                    .push(outline(scope, body, name, 0, self.context));
            }
            _ => {
                if !matches!(statement, Statement::Begin { .. } | Statement::Block { .. }) {
                    self.lines.push(statement.position().line);
                }
                for inner in nested_bodies(statement) {
                    self.statements(inner);
                }
                for expression in own_expressions(statement) {
                    self.blocks_in(expression);
                }
            }
        }
    }

    /// A class or module body, which is a scope of its own named after the
    /// constant it opens.
    fn definition(&mut self, opening: &str, name: &str, opened: Position, body: &[Statement]) {
        let line = opened.line;
        self.lines.push(line);
        let constant = name.rsplit("::").next().unwrap_or(name);
        let label = format!("{opening}{constant}>");
        let scope = Scope {
            kind: "class",
            label: label.clone(),
            first: line,
            last: self.closing_line(opened),
            parameters: Vec::new(),
        };
        self.children
            .push(outline(scope, body, &label, 0, self.context));
    }

    /// The blocks written in an expression, each a scope of its own.
    fn blocks_in(&mut self, expression: &Expression) {
        match expression {
            Expression::Lambda {
                parameters,
                parameter_defaults,
                body,
                position,
                ..
            } => {
                let levels = self.levels + 1;
                let label = if levels > 1 {
                    format!("block ({levels} levels) in {}", self.base)
                } else {
                    format!("block in {}", self.base)
                };
                let scope = Scope {
                    kind: "block",
                    label,
                    first: position.line,
                    last: self.closing_line(*position),
                    parameters: block_parameter_pairs(
                        parameters,
                        &parameter_defaults
                            .iter()
                            .map(|(index, _)| *index)
                            .collect::<Vec<_>>(),
                        "req",
                    ),
                };
                self.children
                    .push(outline(scope, body, self.base, levels, self.context));
            }
            Expression::SingletonClass { body, position, .. } => {
                let scope = Scope {
                    kind: "class",
                    label: "singleton class".to_string(),
                    first: position.line,
                    last: self.closing_line(*position),
                    parameters: Vec::new(),
                };
                self.children
                    .push(outline(scope, body, "singleton class", 0, self.context));
            }
            Expression::MethodCall {
                receiver: callee,
                arguments,
                trailing_block,
                ..
            }
            | Expression::Call {
                callee,
                arguments,
                trailing_block,
                ..
            } => {
                self.blocks_in(callee);
                for argument in arguments {
                    self.blocks_in(argument);
                }
                if let Some(block) = trailing_block {
                    self.blocks_in(block);
                }
            }
            Expression::BinaryOp { left, right, .. }
            | Expression::Index {
                array: left,
                index: right,
                ..
            }
            | Expression::Range {
                start: left,
                end: right,
                ..
            } => {
                self.blocks_in(left);
                self.blocks_in(right);
            }
            Expression::UnaryOp { operand: inner, .. }
            | Expression::Grouped {
                expression: inner, ..
            }
            | Expression::Splat {
                expression: inner, ..
            }
            | Expression::KeywordSplat {
                expression: inner, ..
            }
            | Expression::BlockArg {
                expression: inner, ..
            }
            | Expression::Defined {
                expression: inner, ..
            } => self.blocks_in(inner),
            Expression::Array {
                elements: items, ..
            }
            | Expression::Yield {
                arguments: items, ..
            } => {
                for item in items {
                    self.blocks_in(item);
                }
            }
            Expression::Dictionary { entries, .. } => {
                for (key, value) in entries {
                    self.blocks_in(key);
                    self.blocks_in(value);
                }
            }
            Expression::If {
                condition,
                then_branch,
                elsif_branches,
                else_branch,
                ..
            } => {
                self.blocks_in(condition);
                self.statements(then_branch);
                for branch in elsif_branches {
                    self.blocks_in(&branch.condition);
                    self.statements(&branch.body);
                }
                if let Some(otherwise) = else_branch {
                    self.statements(otherwise);
                }
            }
            Expression::Unless {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.blocks_in(condition);
                self.statements(then_branch);
                if let Some(otherwise) = else_branch {
                    self.statements(otherwise);
                }
            }
            Expression::Case {
                expression,
                cases,
                else_case,
                ..
            } => {
                self.blocks_in(expression);
                for case in cases {
                    self.blocks_in(&case.body);
                }
                if let Some(otherwise) = else_case {
                    self.blocks_in(otherwise);
                }
            }
            Expression::BeginRescue {
                body,
                rescue_clauses,
                else_clause,
                ensure_block,
                ..
            } => {
                self.statements(body);
                for clause in rescue_clauses {
                    self.statements(&clause.body);
                }
                for held in [else_clause, ensure_block].into_iter().flatten() {
                    self.statements(held);
                }
            }
            _ => {}
        }
    }
}

impl Walk<'_> {
    /// The line the scope opened at `opened` closes on, which the parser
    /// noted as it read the scope.
    fn closing_line(&self, opened: Position) -> usize {
        self.context
            .closing
            .get(&(opened.line, opened.column))
            .copied()
            .unwrap_or(opened.line)
    }
}

/// One `[kind, name]` pair of a method's parameter list.
fn parameter_pair(parameter: &Parameter) -> Object {
    let kind = if parameter.is_block {
        "block"
    } else if parameter.is_keyword {
        "keyrest"
    } else if parameter.is_variadic {
        "rest"
    } else if parameter.is_named_keyword {
        if parameter.default_value.is_some() {
            "key"
        } else {
            "keyreq"
        }
    } else if parameter.default_value.is_some() {
        "opt"
    } else {
        "req"
    };
    Object::array(vec![
        Object::symbol(kind.to_string()),
        Object::symbol(parameter.name.clone()),
    ])
}
