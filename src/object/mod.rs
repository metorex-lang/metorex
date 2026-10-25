// Object module - Runtime object representation for Metorex
// This module defines the core Object type that represents all runtime values

// Declare submodules
mod binding;
mod block;
mod constructors;
mod core_aliases;
mod display;
pub mod string_value;
pub(crate) use display::{
    begin_rendering, end_rendering, inspect_symbol, render_guarded, rendering_in_progress,
};
pub mod collector;
mod exception;
mod hash;
mod instance;
pub mod live;
mod method;
mod operations;
pub mod shape;
mod types;

// Re-export core types and traits
pub use binding::Binding;
pub use block::{
    BLOCK_LOCAL_PREFIX, BlockStatement, CodePlaces, DESTRUCTURED_GROUP_PREFIX, IMPLICIT_IT_PARAM,
    KEYWORD_PARAM_PREFIX, NO_KEYWORDS_PARAM, TRAILING_COMMA_PARAM, UNNAMED_PARAMETER,
};
pub use exception::{Exception, SourceLocation};
pub use hash::ObjectHash;
pub use instance::Instance;
pub use method::{Method, Provenance, Seldom, Signature};

/// The classes and modules lexically open where a method or block was
/// written, innermost first, shared by every frame that runs it.
pub type Nesting = std::rc::Rc<[std::rc::Rc<Class>]>;

/// The nesting of code written outside any class or module, shared so that
/// making one allocates nothing.
pub fn no_nesting() -> Nesting {
    thread_local! {
        static EMPTY: Nesting = std::rc::Rc::from(Vec::new());
    }
    EMPTY.with(std::rc::Rc::clone)
}
pub use types::{Object, OptionView, View};

// Re-export from callable and class modules
pub use crate::callable::Callable;
pub use crate::class::Class;

pub use string_value::StringValue;
