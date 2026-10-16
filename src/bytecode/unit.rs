//! What a body compiles to: its instructions and the side tables they name.

use crate::bytecode::instruction::Instruction;
use crate::lexer::Position;
use crate::object::Object;

/// A compiled body.
#[derive(Debug, Default)]
pub struct Unit {
    pub instructions: Vec<Instruction>,
    /// The source position of each instruction, which backtraces,
    /// `error_highlight`, and line events read.
    pub positions: Vec<Position>,
    /// Whether each instruction starts a statement, where a line event fires
    /// and the interpreter checks for signals, thread switches, and
    /// collections, as it does before each statement it walks.
    pub statement_starts: Vec<bool>,
    pub constants: Vec<Object>,
    pub names: Vec<Name>,
    pub call_sites: Vec<CallSite>,
    pub handlers: Vec<Handler>,
    /// How many registers the unit's instructions name.
    pub register_count: usize,
}

/// An instance variable, constant, or global an instruction names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    pub text: String,
}

impl Name {
    pub fn new(text: &str) -> Name {
        Name {
            text: text.to_string(),
        }
    }
}

/// How a call's receiver was written, which decides whether a private
/// method can be called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Receiver {
    /// The keyword `self`, or no receiver at all.
    SelfKeyword,
    /// Any other expression.
    Other,
}

/// What a `Call` instruction calls.
#[derive(Debug, Clone)]
pub struct CallSite {
    pub name: String,
    pub argument_count: usize,
    pub receiver: Receiver,
}

impl CallSite {
    pub fn new(name: &str, argument_count: usize, receiver: Receiver) -> CallSite {
        CallSite {
            name: name.to_string(),
            argument_count,
            receiver,
        }
    }
}

/// Instructions from `start` up to `end` that raise an exception jump to
/// `target`, with the exception in `register`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handler {
    pub start: usize,
    pub end: usize,
    pub target: usize,
    pub register: usize,
}

/// Each table answers the index of the entry it adds. An index past 65,535
/// is named by an `Extend` before the instruction.
impl Unit {
    pub fn new() -> Unit {
        Unit::default()
    }

    /// Add an instruction, answering where it is.
    pub fn emit(&mut self, instruction: Instruction, position: Position) -> usize {
        self.instructions.push(instruction);
        self.positions.push(position);
        self.statement_starts.push(false);
        self.instructions.len() - 1
    }

    /// Add an instruction that starts a statement, answering where it is.
    pub fn emit_statement(&mut self, instruction: Instruction, position: Position) -> usize {
        let at = self.emit(instruction, position);
        self.statement_starts[at] = true;
        at
    }

    /// Give a register count of at least `count`.
    pub fn use_registers(&mut self, count: usize) {
        self.register_count = self.register_count.max(count);
    }

    pub fn constant(&mut self, value: Object) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn name(&mut self, name: Name) -> usize {
        self.names.push(name);
        self.names.len() - 1
    }

    pub fn call_site(&mut self, site: CallSite) -> usize {
        self.call_sites.push(site);
        self.call_sites.len() - 1
    }

    pub fn handler(&mut self, handler: Handler) {
        self.handlers.push(handler);
    }

    /// The handler covering the instruction at `at`, the innermost first.
    pub fn handler_for(&self, at: usize) -> Option<&Handler> {
        self.handlers
            .iter()
            .filter(|handler| handler.start <= at && at < handler.end)
            .min_by_key(|handler| handler.end - handler.start)
    }
}
