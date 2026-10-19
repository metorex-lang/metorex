//! A bare name machine code reads with `GetIdentifier` that called a method
//! of the program's own on `self` the last time is called the same way
//! again, without the scope and method lookups the interpreter makes, while
//! nothing those lookups read has changed.

use crate::bytecode::instruction::Instruction;
use crate::class::{Class, method_state};
use crate::error::MetorexError;
use crate::object::{Method, Object};
use crate::scope::Scope;
use crate::vm::VirtualMachine;
use crate::vm::bytecode_runner::{Next, UnitRun};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

/// What one `GetIdentifier` called last, the state a lookup there last
/// found no such call under, and how many times it found the name read
/// somewhere other than a method's own scope or `self` not an object its
/// class alone decides the calls of.
#[derive(Default)]
pub(super) struct BareCallSite {
    found: RefCell<Option<BareCall>>,
    refused_under: Cell<Option<(u64, u64)>>,
    read_otherwise: Cell<u8>,
}

/// How many times a site finds a name read otherwise before it leaves every
/// later read to the interpreter's, as a site reading a local always does.
const READS_OTHERWISE_BEFORE_GIVING_UP: u8 = 4;

/// The method a bare name called on an instance of `class`, found under
/// `method_state` while the root scope, which a name is read through last,
/// was at `root_version`. The weak references keep nothing alive, and while
/// they are held no other class, method, or scope can take their addresses.
struct BareCall {
    class: Weak<Class>,
    called_on: Weak<Class>,
    method: Weak<Method>,
    method_state: u64,
    root: Weak<RefCell<Scope>>,
    root_version: u64,
}

impl BareCallSite {
    fn kept(
        &self,
        class: &Rc<Class>,
        state: u64,
        root: &Rc<RefCell<Scope>>,
        root_version: u64,
    ) -> Option<(Rc<Class>, Rc<Method>)> {
        let found = self.found.borrow();
        let call = found.as_ref()?;
        let same = call.method_state == state
            && call.root_version == root_version
            && std::ptr::eq(call.class.as_ptr(), Rc::as_ptr(class))
            && std::ptr::eq(call.root.as_ptr(), Rc::as_ptr(root));
        if !same {
            return None;
        }
        Some((call.called_on.upgrade()?, call.method.upgrade()?))
    }
}

impl VirtualMachine {
    /// Run the `GetIdentifier` at `at` as a call of the method `site` holds,
    /// or of the one a lookup finds and `site` then holds, when the name is
    /// read in a method's own scope, which neither binds it nor reads
    /// through anything but the root scope, which does not bind it either,
    /// and `self` is an object its class alone decides the calls of. Any
    /// other read is left to the interpreter's, which None stands for.
    pub(super) fn step_bare_call(
        &mut self,
        site: &BareCallSite,
        run: &mut UnitRun,
        at: usize,
    ) -> Option<Result<Next, MetorexError>> {
        let unit = run.unit();
        let Instruction::GetIdentifier { to, name } = unit.instructions[at] else {
            return None;
        };
        let name = &unit.names[usize::from(name)].text;
        if site.read_otherwise.get() >= READS_OTHERWISE_BEFORE_GIVING_UP {
            return None;
        }
        let read_otherwise = || {
            site.read_otherwise.set(site.read_otherwise.get() + 1);
            None
        };
        let Some((receiver, root)) = self.method_scope_without(name) else {
            return read_otherwise();
        };
        let Some(class) = self.class_deciding_dispatch(&receiver) else {
            return read_otherwise();
        };
        let state = method_state();
        let root_version = root.borrow().version();
        let (called_on, method) = match site.kept(&class, state, &root, root_version) {
            Some(kept) => kept,
            None => {
                if site.refused_under.get() == Some((state, root_version)) {
                    return None;
                }
                let found = self
                    .lookup_method(&receiver, name)
                    .filter(|(_, method)| !method.is_undefined)
                    .filter(|_| !root.borrow().holds_own(name));
                let Some((called_on, method)) = found else {
                    site.refused_under.set(Some((state, root_version)));
                    return None;
                };
                *site.found.borrow_mut() = Some(BareCall {
                    class: Rc::downgrade(&class),
                    called_on: Rc::downgrade(&called_on),
                    method: Rc::downgrade(&method),
                    method_state: state,
                    root: Rc::downgrade(&root),
                    root_version,
                });
                (called_on, method)
            }
        };
        debug_assert!(
            !root.borrow().holds_own(name)
                && self
                    .lookup_method(&receiver, name)
                    .is_some_and(|(_, found)| Rc::ptr_eq(&found, &method)),
            "the method a bare name called last is what a lookup finds"
        );
        let position = unit.positions[at];
        let ran = self
            .invoke_method(called_on, method, receiver, Vec::new(), position)
            .map(|value| {
                run.set_register(usize::from(to), value);
                Next::Step
            });
        Some(self.settle_instruction(run, at, ran))
    }

    /// `self` and the root scope, when the current scope is a method's own,
    /// read through the root scope alone, and does not bind or capture
    /// `name`.
    fn method_scope_without(&self, name: &str) -> Option<(Object, Rc<RefCell<Scope>>)> {
        let environment = self.environment();
        let root = environment.root_scope();
        let scope = environment.current_scope_ref().borrow();
        let reads_root_alone = scope
            .parent()
            .is_some_and(|parent| Rc::ptr_eq(parent, root));
        if !reads_root_alone || scope.holds_own(name) {
            return None;
        }
        Some((scope.own_value("self")?, Rc::clone(root)))
    }
}
