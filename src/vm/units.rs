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
    Unit(Unit),
    /// The body runs on the tree walker, held back by the node named.
    Fallback(&'static str),
}

pub use crate::bytecode::unit::Unit;

/// Where a `Method` or `BlockStatement` keeps what its body compiled to.
/// Copies share it, since they share the body.
#[derive(Debug, Clone, Default)]
pub struct UnitSlot(OnceCell<Rc<Compiled>>);

impl UnitSlot {
    /// What `body` compiles to, compiling it the first time.
    pub fn compiled(&self, body: &[Statement]) -> Rc<Compiled> {
        self.0.get_or_init(|| Rc::new(compile(body))).clone()
    }
}

/// Compile `body`, recording the result for the engine report.
pub fn compile(body: &[Statement]) -> Compiled {
    let compiled = match body.first() {
        None => Compiled::Unit(answering_nil()),
        Some(statement) => Compiled::Fallback(unhandled_node(statement)),
    };
    crate::vm::engine_report::record(&compiled);
    compiled
}

/// The unit of a body with no statements, which answers nil. The compiler
/// handles no node yet, so this is the only body it compiles.
fn answering_nil() -> Unit {
    use crate::bytecode::instruction::Instruction;
    let mut unit = Unit::new();
    unit.use_registers(1);
    let position = crate::lexer::Position::default();
    unit.emit(Instruction::LoadNil { to: 0 }, position);
    unit.emit(Instruction::Return { from: 0 }, position);
    unit
}

/// The first node in `statement` the compiler cannot handle.
fn unhandled_node(statement: &Statement) -> &'static str {
    match statement {
        Statement::Expression { expression, .. } => expression.kind_name(),
        other => other.kind_name(),
    }
}

impl crate::vm::VirtualMachine {
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
            Some(slot) => slot.compiled(body),
            None => Rc::new(compile(body)),
        };
        match &*compiled {
            Compiled::Unit(unit) => Some(self.run_unit(unit)),
            Compiled::Fallback(_) => None,
        }
    }
}
