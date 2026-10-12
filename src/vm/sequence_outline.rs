//! The outline the heap walk builds a file's instruction sequences from:
//! each method, class, module and block written in the file, with the lines
//! its statements start on. Metorex's own parser reads the file, where
//! `RubyVM::AbstractSyntaxTree` reads it with a parser written in Ruby.
//!
//! Each scope is the Array `[type, label, first line, last line, statement
//! lines, parameters, children]`, with the parameters as `Method#parameters`
//! lists them.

use crate::ast::{Expression, Parameter, Statement};
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::coverage::{nested_bodies, own_expressions};
use crate::vm::native_methods::method_object_methods::block_parameter_pairs;

/// The outline of a whole program, or None where it does not parse.
pub(crate) fn outline_of_source(source: &str) -> Option<Object> {
    let tokens = crate::lexer::Lexer::new(source).tokenize();
    let mut parser = crate::parser::Parser::new(tokens);
    let statements = parser.parse().ok()?;
    let closing = parser.closing_lines();
    let last = source.lines().count().max(1);
    let top = Scope {
        kind: "top",
        label: "<main>".to_string(),
        first: 1,
        last,
        parameters: Vec::new(),
    };
    Some(outline(top, &statements, "<main>", 0, closing))
}

/// The lines each class, module and block closes on, by where it opens.
type ClosingLines = std::collections::HashMap<(usize, usize), usize>;

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
    closing: &ClosingLines,
) -> Object {
    let mut walk = Walk {
        lines: Vec::new(),
        children: Vec::new(),
        base,
        levels,
        closing,
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
    closing: &'a ClosingLines,
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
                    .push(outline(scope, body, name, 0, self.closing));
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
            .push(outline(scope, body, &label, 0, self.closing));
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
                    .push(outline(scope, body, self.base, levels, self.closing));
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
                    .push(outline(scope, body, "singleton class", 0, self.closing));
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
        self.closing
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
