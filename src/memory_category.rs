//! What the memory report counts a thread's heap allocations under, which
//! the lexer and parser set too, so it sits outside the virtual machine.

use std::cell::Cell;

/// What allocated a heap block.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Category {
    /// Anything not named below: objects, scopes, frames, and tables.
    Other = 0,
    /// Lexing, parsing, and reading a library's syntax tree from the binary.
    SyntaxTree = 1,
    /// Compiling a body to bytecode.
    CompiledUnits = 2,
    /// The JIT compiling a unit to machine code.
    JitCompiling = 3,
}

thread_local! {
    static CURRENT: Cell<u8> = const { Cell::new(Category::Other as u8) };
}

/// What this thread's allocations are counted under now.
pub fn current() -> u8 {
    CURRENT.try_with(Cell::get).unwrap_or(Category::Other as u8)
}

/// Counts what this thread allocates under `category` until it is dropped.
pub struct Attributed {
    before: u8,
}

/// Count this thread's allocations under `category` until the answer is
/// dropped.
#[inline]
pub fn attribute(category: Category) -> Attributed {
    let before = CURRENT
        .try_with(|current| current.replace(category as u8))
        .unwrap_or(Category::Other as u8);
    Attributed { before }
}

impl Drop for Attributed {
    fn drop(&mut self) {
        let _ = CURRENT.try_with(|current| current.set(self.before));
    }
}
