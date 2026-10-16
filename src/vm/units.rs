//! A method body, block body, class or module body, or file top level,
//! compiled the first time it runs under `METOREX_ENGINE=bytecode`. A body
//! holding a node the compiler does not handle yet runs on the tree walker,
//! and the node is recorded for the engine report.

use std::cell::OnceCell;
use std::rc::Rc;

use crate::ast::Statement;

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
pub fn compile(body: &[Statement]) -> Compiled {
    let compiled = match crate::bytecode::compiler::compile(body) {
        Ok(unit) => Compiled::Unit(Box::new(unit)),
        Err(node) => Compiled::Fallback(node),
    };
    crate::vm::engine_report::record(&compiled);
    compiled
}

impl crate::vm::VirtualMachine {
    /// Record for the engine report a class or module body, which runs on
    /// the tree walker under either engine: its statements follow rules of
    /// their own, such as a bare `private` and `@name = value` declaring an
    /// instance variable of the class. `node` names the definition.
    pub(crate) fn body_on_the_tree_walker(&self, node: &'static str) {
        if crate::vm::engine::engine() == crate::vm::engine::Engine::Bytecode {
            crate::vm::engine_report::record(&Compiled::Fallback(node));
        }
    }

    /// Run `body` through what it compiled to, when the bytecode engine was
    /// chosen and the body compiled, answering its value. None means the
    /// body runs on the tree walker. A body with no slot of its own to keep
    /// the result in is compiled each time it runs.
    pub(crate) fn run_compiled_unit(
        &mut self,
        slot: Option<&UnitSlot>,
        body: &[Statement],
    ) -> Option<Result<crate::object::Object, crate::error::MetorexError>> {
        if crate::vm::engine::engine() != crate::vm::engine::Engine::Bytecode {
            return None;
        }
        let compiled = match slot {
            Some(slot) => slot.compiled(|| self.compile_body(body)),
            None => Rc::new(self.compile_body(body)),
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
                Some(self.run_unit(unit))
            }
            Compiled::Fallback(_) => None,
        }
    }

    /// Compile `body`, writing the unit to standard error when
    /// `METOREX_DISASM=1` asked for it. A unit of the libraries Metorex
    /// carries, whose file is named `<metorex>` or `<internal:...>`, is left
    /// out.
    fn compile_body(&self, body: &[Statement]) -> Compiled {
        let compiled = compile(body);
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
