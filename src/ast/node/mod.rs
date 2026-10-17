// AST node definitions for Metorex.

pub(crate) use crate::lexer::Position;
pub(crate) use std::fmt;

mod display;
mod expressions;
mod parameters;
mod patterns;
mod statements;

/// The statements of a method, block, class, or module body, which every
/// copy of the definition and the unit it compiles to share.
pub type Body = std::rc::Rc<[Statement]>;

pub use expressions::*;
pub use patterns::*;
pub use statements::*;
