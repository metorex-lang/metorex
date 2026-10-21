//! A method body, block body, class or module body, or file top level,
//! compiled the first time it runs. A body nested deeper than an
//! instruction has registers for raises SystemStackError, and the node that
//! ran out is recorded for the engine report.

use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ast::{Expression, Statement};
pub use crate::bytecode::compiler::UnitKind;
use crate::bytecode::compiler::{compile_expression, compile_statement};
use crate::error::MetorexError;
use crate::object::Object;
use crate::vm::ControlFlow;
use crate::vm::errors::loop_control_error;
use crate::vm::utils::position_to_location;

/// A class or module body running as a compiled unit.
#[derive(Debug, Clone)]
pub struct BodyContext {
    pub class: Rc<crate::class::Class>,
    /// Whether the body is a `module` body, whose statements follow the
    /// module body's rules rather than the class body's.
    pub module: bool,
    /// Where the body was opened.
    pub position: crate::lexer::Position,
    /// The method frame the body runs in, which code written in the body,
    /// blocks included, runs in too.
    pub frame: Option<u64>,
}

/// What compiling a body gave.
#[derive(Debug)]
pub enum Compiled {
    Unit(Box<Unit>),
    /// The body needs more registers than an instruction names, which the
    /// node named ran out of.
    Refused(&'static str),
}

pub use crate::bytecode::unit::Unit;

/// Where a `Method` or `BlockStatement` keeps what its body compiled to.
/// Copies share it, since they share the body.
#[derive(Debug, Clone, Default)]
pub struct UnitSlot(Rc<OnceCell<Rc<Compiled>>>);

impl UnitSlot {
    /// What the body compiles to, compiling it with `compile` the first
    /// time.
    pub fn compiled(&self, compile: impl FnOnce() -> Compiled) -> Rc<Compiled> {
        self.0.get_or_init(|| Rc::new(compile())).clone()
    }
}

/// Whether `METOREX_DISASM=1` asked for each unit to be written to standard
/// error as it is compiled.
fn disassembly_wanted() -> bool {
    static WANTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *WANTED.get_or_init(|| std::env::var("METOREX_DISASM").is_ok_and(|value| value == "1"))
}

/// How many units have been compiled, and how many instructions they hold,
/// which `RubyVM.stat` answers.
static UNITS_COMPILED: AtomicU64 = AtomicU64::new(0);
static INSTRUCTIONS_COMPILED: AtomicU64 = AtomicU64::new(0);

/// The units compiled so far and the instructions they hold.
pub fn compiled_so_far() -> (u64, u64) {
    (
        UNITS_COMPILED.load(Ordering::Relaxed),
        INSTRUCTIONS_COMPILED.load(Ordering::Relaxed),
    )
}

/// Count `unit` among those compiled.
fn count_compiled(unit: &Unit) {
    UNITS_COMPILED.fetch_add(1, Ordering::Relaxed);
    INSTRUCTIONS_COMPILED.fetch_add(unit.instructions.len() as u64, Ordering::Relaxed);
}

/// Compile `body`, recording the result for the engine report.
pub fn compile(body: &[Statement], kind: UnitKind) -> Compiled {
    let compiled = match crate::bytecode::compiler::compile_as(body, kind) {
        Ok(unit) => {
            count_compiled(&unit);
            Compiled::Unit(Box::new(unit))
        }
        Err(node) => Compiled::Refused(node),
    };
    crate::vm::engine_report::record(&compiled);
    compiled
}

impl crate::vm::VirtualMachine {
    /// Run a class body, or a module body when `module`, through what it
    /// compiled to, answering the value it answers, and taking a `return`
    /// that ends it as the body's own rules take one.
    pub(crate) fn run_compiled_body(
        &mut self,
        class: &Rc<crate::class::Class>,
        body: &[Statement],
        position: crate::lexer::Position,
        module: bool,
    ) -> Result<Object, MetorexError> {
        let kind = if module {
            UnitKind::ModuleBody
        } else {
            UnitKind::ClassBody
        };
        self.body_contexts.push(BodyContext {
            class: Rc::clone(class),
            module,
            position,
            frame: self.current_method_frame,
        });
        let ran = self.run_compiled_unit(None, body, kind);
        self.body_contexts.pop();
        ran.and_then(|flow| match flow {
            ControlFlow::Value(value) => Ok(value),
            ControlFlow::Return { value, position } if !module => {
                self.class_body_return(class, value, position)
            }
            ControlFlow::Return { position, .. } => Err(self.module_body_return(position)),
            // A `break`, `next`, or `redo` written in a body ends nothing.
            _ => Ok(Object::Nil),
        })
    }

    /// Run `body` through what it compiled to, answering how it ended. A
    /// body with no slot of its own to keep the result in is compiled each
    /// time it runs.
    pub(crate) fn run_compiled_unit(
        &mut self,
        slot: Option<&UnitSlot>,
        body: &[Statement],
        kind: UnitKind,
    ) -> Result<ControlFlow, MetorexError> {
        let compiled = match slot {
            Some(slot) => slot.compiled(|| self.compile_body(body, kind)),
            None => Rc::new(self.compile_body(body, kind)),
        };
        match &*compiled {
            Compiled::Unit(unit) => {
                self.hoist_unit_locals(unit);
                self.run_unit_flow(unit)
            }
            Compiled::Refused(_) => Err(too_deep(
                body.first().map(Statement::position).unwrap_or_default(),
            )),
        }
    }

    /// Reserve in the scope each local `unit` assigns, before any of it
    /// runs.
    fn hoist_unit_locals(&mut self, unit: &Unit) {
        for local in &unit.locals {
            if self
                .environment()
                .assignment_introduces_a_local(&local.text)
            {
                self.environment_mut().hoist(local.text.clone());
            }
        }
    }

    /// Run `expression` alone, compiled the first time it runs here,
    /// answering its value. A jump it ends with unwinds as one written
    /// where a value goes does.
    pub(crate) fn run_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<Object, MetorexError> {
        let unit = EXPRESSION_UNITS
            .with(|units| node_unit(units, expression, compile_expression))
            .map_err(|_| too_deep(expression.position()))?;
        self.hoist_unit_locals(&unit);
        let flow = self.run_unit_flow(&unit)?;
        self.flow_as_value(flow)
    }

    /// Run `statement` alone, compiled the first time it runs here,
    /// answering how it ended.
    pub(crate) fn run_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, MetorexError> {
        let unit = STATEMENT_UNITS
            .with(|units| node_unit(units, statement, compile_statement))
            .map_err(|_| too_deep(statement.position()))?;
        self.hoist_unit_locals(&unit);
        self.run_unit_flow(&unit)
    }

    /// The value code written where a value goes answers when it ends with
    /// `flow`: a jump unwinds to what it leaves.
    fn flow_as_value(&self, flow: ControlFlow) -> Result<Object, MetorexError> {
        match flow {
            ControlFlow::Value(value) => Ok(value),
            ControlFlow::Next => Ok(Object::Nil),
            ControlFlow::Return { value, position } => Err(MetorexError::NonLocalReturn {
                value,
                location: position_to_location(position),
                home_frame: self.current_method_frame,
            }),
            ControlFlow::Break { value, position } => {
                Err(self.break_signal(value, position_to_location(position)))
            }
            ControlFlow::Retry { position } => Err(MetorexError::BlockRetry {
                location: position_to_location(position),
            }),
            ControlFlow::Redo { position } => Err(MetorexError::BlockRedo {
                location: position_to_location(position),
            }),
            ControlFlow::Continue { value, position } => Err(MetorexError::BlockNext {
                value,
                location: position_to_location(position),
            }),
        }
    }

    /// Run a method body through what it compiled to, taking a `break`,
    /// `next`, or `redo` that ends it: a body with lambda semantics, from `define_method` with a
    /// block, answers the value `break` or `next` carries and runs again on
    /// `redo`.
    pub(crate) fn run_compiled_method_body(
        &mut self,
        slot: &UnitSlot,
        body: &[Statement],
        lambda_semantics: bool,
    ) -> Result<Object, MetorexError> {
        loop {
            let ran = self.run_compiled_unit(Some(slot), body, UnitKind::Method);
            if let Ok(ControlFlow::Return { .. }) = &ran
                && let Some(running) = self.running_code.last_mut()
            {
                running.returned_explicitly = true;
            }
            let answer = ran.and_then(|flow| method_body_answer(flow, lambda_semantics));
            if !lambda_semantics {
                return answer;
            }
            match answer {
                Err(MetorexError::BlockRedo { .. }) => continue,
                Err(MetorexError::BlockNext { value, .. })
                | Err(MetorexError::BlockBreak { value, .. }) => return Ok(value),
                other => return other,
            }
        }
    }

    /// Run a file's top level through what it compiled to. A `break`,
    /// `next`, or `redo` that ends it has no loop to leave.
    pub(crate) fn run_compiled_program(
        &mut self,
        body: &[Statement],
        a_return_unwinds: bool,
    ) -> Result<Option<Object>, MetorexError> {
        let ran = self.run_compiled_unit(None, body, UnitKind::Plain);
        ran.and_then(|flow| match flow {
            ControlFlow::Value(value) => Ok(Some(value)),
            ControlFlow::Next => Ok(None),
            ControlFlow::Break { position, .. } => Err(loop_control_error("break", position)),
            ControlFlow::Continue { position, .. } => Err(loop_control_error("next", position)),
            ControlFlow::Redo { position } => Err(loop_control_error("redo", position)),
            ControlFlow::Retry { position } => Err(MetorexError::BlockRetry {
                location: position_to_location(position),
            }),
            ControlFlow::Return { value, position } if a_return_unwinds => {
                Err(self.unwinding_return(value, position))
            }
            ControlFlow::Return { value, .. } => Ok(Some(value)),
        })
    }

    /// Compile `body`, writing the unit to standard error when
    /// `METOREX_DISASM=1` asked for it. A unit of the libraries Metorex
    /// carries, whose file is named `<metorex>` or `<internal:...>`, is left
    /// out, and so is one Metorex made with no file, such as the body of a
    /// reader `attr_reader` defines.
    fn compile_body(&self, body: &[Statement], kind: UnitKind) -> Compiled {
        let compiled = compile(body, kind);
        let file = self
            .current_source_file
            .clone()
            .or_else(|| {
                self.reported_current_file()
                    .map(|path| path.display().to_string())
            })
            .unwrap_or_default();
        if disassembly_wanted()
            && let Compiled::Unit(unit) = &compiled
            && !file.is_empty()
            && !file.starts_with('<')
        {
            let source = source_text(&file);
            eprint!(
                "{}",
                crate::bytecode::disassembly::disassemble(unit, &file, source.as_deref())
            );
        }
        compiled
    }
}

/// The units compiled from nodes run alone, by the node's address. Each is
/// kept with a copy of its node, since a node freed and another made at the
/// same address needs a unit of its own.
type NodeUnits<Node> = RefCell<HashMap<usize, (Node, Rc<Unit>)>>;

/// How many units of nodes run alone are kept before they are compiled
/// afresh.
const MOST_NODE_UNITS: usize = 65_536;

thread_local! {
    static EXPRESSION_UNITS: NodeUnits<Expression> = RefCell::default();
    static STATEMENT_UNITS: NodeUnits<Statement> = RefCell::default();
}

/// The unit `node` compiles to, compiled with `compile` unless `units`
/// holds it.
fn node_unit<Node: Clone + PartialEq>(
    units: &NodeUnits<Node>,
    node: &Node,
    compile: fn(&Node) -> Result<Unit, &'static str>,
) -> Result<Rc<Unit>, &'static str> {
    let address = node as *const Node as usize;
    if let Some((held, unit)) = units.borrow().get(&address)
        && held == node
    {
        return Ok(Rc::clone(unit));
    }
    let unit = Rc::new(compile(node)?);
    count_compiled(&unit);
    let mut held = units.borrow_mut();
    if held.len() >= MOST_NODE_UNITS {
        held.clear();
    }
    held.insert(address, (node.clone(), Rc::clone(&unit)));
    Ok(unit)
}

/// What code nested deeper than an instruction has registers for raises.
fn too_deep(position: crate::lexer::Position) -> MetorexError {
    crate::vm::errors::simple_exception("SystemStackError", "stack level too deep", position)
}

/// The text of the file at `path`, read once, for the lines a disassembly
/// shows.
fn source_text(path: &str) -> Option<Rc<str>> {
    thread_local! {
        static READ: std::cell::RefCell<std::collections::HashMap<String, Option<Rc<str>>>> =
            Default::default();
    }
    READ.with(|read| {
        read.borrow_mut()
            .entry(path.to_string())
            .or_insert_with(|| std::fs::read_to_string(path).ok().map(Rc::from))
            .clone()
    })
}

/// What a method body answers when it ends with `flow`.
fn method_body_answer(flow: ControlFlow, lambda_semantics: bool) -> Result<Object, MetorexError> {
    match flow {
        ControlFlow::Value(value) | ControlFlow::Return { value, .. } => Ok(value),
        ControlFlow::Next => Ok(Object::Nil),
        ControlFlow::Break { value, .. } | ControlFlow::Continue { value, .. }
            if lambda_semantics =>
        {
            Ok(value)
        }
        ControlFlow::Break { position, .. } => Err(loop_control_error("break", position)),
        ControlFlow::Continue { position, .. } => Err(loop_control_error("next", position)),
        ControlFlow::Redo { position } if lambda_semantics => Err(MetorexError::BlockRedo {
            location: position_to_location(position),
        }),
        ControlFlow::Redo { position } => Err(loop_control_error("redo", position)),
        ControlFlow::Retry { position } => Err(MetorexError::BlockRetry {
            location: position_to_location(position),
        }),
    }
}
