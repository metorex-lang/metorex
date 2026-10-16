//! A method body, block body, class or module body, or file top level,
//! compiled the first time it runs under `METOREX_ENGINE=bytecode`. A body
//! holding a node the compiler does not handle yet runs on the tree walker,
//! and the node is recorded for the engine report.

use std::cell::OnceCell;
use std::rc::Rc;

use crate::ast::Statement;
pub use crate::bytecode::compiler::UnitKind;
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
}

/// What compiling a body gave.
#[derive(Debug)]
pub enum Compiled {
    Unit(Box<Unit>),
    /// The body runs on the tree walker, held back by the node named.
    Fallback(&'static str),
}

pub use crate::bytecode::unit::Unit;

/// Where a `Method` or `BlockStatement` keeps what its body compiled to.
/// Copies share it, since they share the body.
#[derive(Debug, Clone, Default)]
pub struct UnitSlot(OnceCell<Rc<Compiled>>);

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

/// Compile `body`, recording the result for the engine report.
pub fn compile(body: &[Statement], kind: UnitKind) -> Compiled {
    let compiled = match crate::bytecode::compiler::compile_as(body, kind) {
        Ok(unit) => Compiled::Unit(Box::new(unit)),
        Err(node) => Compiled::Fallback(node),
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
    ) -> Option<Result<Object, MetorexError>> {
        let kind = if module {
            UnitKind::ModuleBody
        } else {
            UnitKind::ClassBody
        };
        self.body_contexts.push(BodyContext {
            class: Rc::clone(class),
            module,
            position,
        });
        let ran = self.run_compiled_unit(None, body, kind);
        self.body_contexts.pop();
        Some(ran?.and_then(|flow| match flow {
            ControlFlow::Value(value) => Ok(value),
            ControlFlow::Return { value, position } if !module => {
                self.class_body_return(class, value, position)
            }
            ControlFlow::Return { position, .. } => Err(self.module_body_return(position)),
            ControlFlow::Exception {
                exception,
                position,
            } => Err(MetorexError::UncaughtException {
                message: crate::vm::utils::format_exception(&exception),
                exception,
                location: position_to_location(position),
            }),
            // A `break`, `next`, or `redo` written in a body ends nothing.
            _ => Ok(Object::Nil),
        }))
    }

    /// Run `body` through what it compiled to, when the bytecode engine was
    /// chosen and the body compiled, answering its value. None means the
    /// body runs on the tree walker. A body with no slot of its own to keep
    /// the result in is compiled each time it runs.
    pub(crate) fn run_compiled_unit(
        &mut self,
        slot: Option<&UnitSlot>,
        body: &[Statement],
        kind: UnitKind,
    ) -> Option<Result<ControlFlow, MetorexError>> {
        if crate::vm::engine::engine() != crate::vm::engine::Engine::Bytecode {
            return None;
        }
        let compiled = match slot {
            Some(slot) => slot.compiled(|| self.compile_body(body, kind)),
            None => Rc::new(self.compile_body(body, kind)),
        };
        match &*compiled {
            Compiled::Unit(unit) => {
                // The scope reserves each local the body assigns before any
                // of it runs, as the tree walker's runners do.
                for local in &unit.locals {
                    if self
                        .environment()
                        .assignment_introduces_a_local(&local.text)
                    {
                        self.environment_mut().hoist(local.text.clone());
                    }
                }
                Some(self.run_unit_flow(unit))
            }
            Compiled::Fallback(_) => None,
        }
    }

    /// Run a method body through what it compiled to, taking a `break`,
    /// `next`, or `redo` that ends it as the tree walker takes one ending
    /// the body: a body with lambda semantics, from `define_method` with a
    /// block, answers the value `break` or `next` carries and runs again on
    /// `redo`.
    pub(crate) fn run_compiled_method_body(
        &mut self,
        slot: &UnitSlot,
        body: &[Statement],
        lambda_semantics: bool,
    ) -> Option<Result<Object, MetorexError>> {
        loop {
            let ran = self.run_compiled_unit(Some(slot), body, UnitKind::Method)?;
            if let Ok(ControlFlow::Return { .. }) = &ran
                && let Some(running) = self.running_code.last_mut()
            {
                running.returned_explicitly = true;
            }
            let answer = ran.and_then(|flow| method_body_answer(flow, lambda_semantics));
            if !lambda_semantics {
                return Some(answer);
            }
            match answer {
                Err(MetorexError::BlockRedo { .. }) => continue,
                Err(MetorexError::BlockNext { value, .. })
                | Err(MetorexError::BlockBreak { value, .. }) => return Some(Ok(value)),
                other => return Some(other),
            }
        }
    }

    /// Run a file's top level through what it compiled to. A `break`,
    /// `next`, or `redo` that ends it has no loop to leave.
    pub(crate) fn run_compiled_program(
        &mut self,
        body: &[Statement],
    ) -> Option<Result<Option<Object>, MetorexError>> {
        let ran = self.run_compiled_unit(None, body, UnitKind::Plain)?;
        Some(ran.and_then(|flow| match flow {
            ControlFlow::Value(value) => Ok(Some(value)),
            ControlFlow::Next => Ok(None),
            ControlFlow::Break { position, .. } => Err(loop_control_error("break", position)),
            ControlFlow::Continue { position, .. } => Err(loop_control_error("next", position)),
            ControlFlow::Redo { position } => Err(loop_control_error("redo", position)),
            ControlFlow::Retry { position } => Err(MetorexError::BlockRetry {
                location: position_to_location(position),
            }),
            ControlFlow::Return { value, .. } => Ok(Some(value)),
            ControlFlow::Exception {
                exception,
                position,
            } => Err(MetorexError::UncaughtException {
                message: crate::vm::utils::format_exception(&exception),
                exception,
                location: position_to_location(position),
            }),
        }))
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
        ControlFlow::Exception {
            exception,
            position,
        } => Err(MetorexError::UncaughtException {
            message: crate::vm::utils::format_exception(&exception),
            exception,
            location: position_to_location(position),
        }),
    }
}
