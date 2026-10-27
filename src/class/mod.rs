//! Runtime class definition with method table and inheritance.

pub(crate) use crate::object::{Method, Object};

pub use constants::constant_state;
pub(crate) use constants::{constant_state_changed, names_a_constant};
pub use methods::{MethodGuard, method_state};
pub use shape::Class;
pub(crate) use shape::{MethodCache, STALE_LOOKUP};
pub(crate) use std::cell::RefCell;
pub(crate) use std::collections::HashSet;
pub(crate) use std::rc::{Rc, Weak};

mod class_variables;
mod constants;
mod methods;
mod names;
mod shape;
mod visibility;
