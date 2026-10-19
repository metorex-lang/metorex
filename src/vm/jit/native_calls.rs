//! A call machine code makes with `Call` on a built-in value, such as an
//! Array or a String, that the interpreter's lookup would answer with the
//! value's native method goes straight to that method, while nothing the
//! lookup read has changed.

use crate::bytecode::instruction::Instruction;
use crate::bytecode::unit::Receiver;
use crate::class::{Class, method_state};
use crate::object::{Object, View};
use crate::vm::VirtualMachine;
use crate::vm::bytecode_runner::UnitRun;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

/// The class of the values one `Call` was found to call natively, and the
/// one it was found not to, each with the method state it was found under,
/// and how many times the call was made on a value that is no built-in one.
#[derive(Default)]
pub(super) struct NativeCallSite {
    answered: RefCell<Option<(Weak<Class>, u64)>>,
    refused: RefCell<Option<(Weak<Class>, u64)>>,
    made_otherwise: Cell<u8>,
}

/// How many calls a site makes on values that are no built-in ones before
/// the unit is compiled again with the site left to the interpreter.
const CALLS_OTHERWISE_BEFORE_GIVING_UP: u8 = 4;

impl NativeCallSite {
    /// Whether the site has made enough calls on other values to be left to
    /// the interpreter.
    pub(super) fn gave_up(&self) -> bool {
        self.made_otherwise.get() >= CALLS_OTHERWISE_BEFORE_GIVING_UP
    }
}

fn holds(kept: &RefCell<Option<(Weak<Class>, u64)>>, class: &Rc<Class>, state: u64) -> bool {
    kept.borrow().as_ref().is_some_and(|(held, under)| {
        *under == state && std::ptr::eq(held.as_ptr(), Rc::as_ptr(class))
    })
}

/// Whether the calls of `receiver` are decided by its class alone among
/// the values with no slot for methods of their own beyond what a
/// singleton class gives them.
fn built_in_value(receiver: &Object) -> bool {
    matches!(
        receiver.view(),
        View::Nil
            | View::Bool(_)
            | View::Int(_)
            | View::Float(_)
            | View::Symbol(_)
            | View::String(_)
            | View::Array(_)
            | View::Dict(_)
    )
}

impl VirtualMachine {
    /// The class whose native method the `Call` at `at` reaches, when the
    /// receiver is a built-in value without a singleton class, no
    /// refinement is active, and the lookup the interpreter makes, which
    /// `site` keeps the answer of for the receiver's class under the
    /// method state, passes over every step before the native methods:
    /// the name is no private method of every object, Enumerable does not
    /// stand in for it, and no method of the program's answers it. None
    /// leaves the call to the interpreter.
    pub(super) fn native_call_class(
        &self,
        site: &NativeCallSite,
        run: &UnitRun,
        at: usize,
    ) -> Option<Rc<Class>> {
        let unit = run.unit();
        let Instruction::Call { base, site: called } = unit.instructions[at] else {
            return None;
        };
        let called = &unit.call_sites[usize::from(called)];
        if called.receiver == Receiver::Sent || called.sends_literal_name {
            return None;
        }
        let receiver = run.register(usize::from(base));
        if !built_in_value(&receiver)
            || self.any_refinement_active()
            || self.existing_singleton_class(&receiver).is_some()
        {
            site.made_otherwise
                .set(site.made_otherwise.get().saturating_add(1));
            return None;
        }
        let class = self.builtins().class_of(&receiver);
        let state = method_state();
        if holds(&site.answered, &class, state) {
            return Some(class);
        }
        if holds(&site.refused, &class, state) {
            return None;
        }
        let name = called.name.as_str();
        let passes_over = !crate::vm::native_methods::is_kernel_private_function(name)
            && !crate::vm::native_methods::class_methods::BASIC_OBJECT_PRIVATE_METHODS
                .contains(&name)
            && !self.enumerable_stands_in(&receiver, name)
            && self
                .lookup_method(&receiver, name)
                .is_none_or(|(_, method)| method.is_undefined);
        let kept = if passes_over {
            &site.answered
        } else {
            &site.refused
        };
        *kept.borrow_mut() = Some((Rc::downgrade(&class), state));
        passes_over.then_some(class)
    }
}
