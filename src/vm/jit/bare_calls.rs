//! A bare name machine code reads with `GetIdentifier` that called a method
//! of the program's own on `self` the last time is called the same way
//! again, without the scope and method lookups the interpreter makes, while
//! nothing those lookups read has changed. A name bound to the method a
//! `def` made, as a `def` at the top level binds its name, is called on
//! `self` while the binding holds that method and the class it was defined
//! in still holds it too. A name the block running closed over as a local
//! is read from where the site found it among the names the block closed
//! over, while the block's scope holds those same names and binds no such
//! name of its own.

use crate::bytecode::instruction::Instruction;
use crate::class::{Class, method_state};
use crate::error::MetorexError;
use crate::object::{Method, Object, OptionView, View};
use crate::scope::{CapturedNames, Scope};
use crate::vm::VirtualMachine;
use crate::vm::bytecode_runner::{Next, UnitRun};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

/// What one `GetIdentifier` called last, the definition it found its name
/// bound to, the state a lookup there last found no such call under, and
/// how many times it found the name read some other way, and where among
/// the names a block closed over it found the name last.
#[derive(Default)]
pub(super) struct BareCallSite {
    found: RefCell<Option<BareCall>>,
    closed_over: RefCell<Option<(Weak<ClosedOver>, usize)>>,
    defined: RefCell<Option<(Weak<Method>, u64)>>,
    refused_under: Cell<Option<(u64, u64)>>,
    read_otherwise: Cell<u8>,
}

type ClosedOver = <CapturedNames as std::ops::Deref>::Target;

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
    /// Whether the method answers an instance variable of `self` as a
    /// reader does, and no native method answers in place of it.
    reads: bool,
}

impl BareCallSite {
    /// Whether the site has found the name read otherwise often enough to
    /// be left to the interpreter.
    fn gave_up(&self) -> bool {
        self.read_otherwise.get() >= READS_OTHERWISE_BEFORE_GIVING_UP
    }

    fn kept(
        &self,
        class: &Rc<Class>,
        state: u64,
        root: &Rc<RefCell<Scope>>,
        root_version: u64,
    ) -> Option<(Rc<Class>, Rc<Method>, bool)> {
        let found = self.found.borrow();
        let call = found.as_ref()?;
        let same = call.method_state == state
            && call.root_version == root_version
            && std::ptr::eq(call.class.as_ptr(), Rc::as_ptr(class))
            && std::ptr::eq(call.root.as_ptr(), Rc::as_ptr(root));
        if !same {
            return None;
        }
        Some((
            call.called_on.upgrade()?,
            call.method.upgrade()?,
            call.reads,
        ))
    }

    fn note_read_otherwise(&self) {
        self.read_otherwise.set(self.read_otherwise.get() + 1);
    }
}

impl VirtualMachine {
    /// Run the `GetIdentifier` at `at` as a call of the method its name is
    /// bound to by a `def`, or of the method `site` holds or a lookup finds
    /// and `site` then holds, when the name is bound nowhere and is read in
    /// a scope that reads through the root scope alone, and `self` is an
    /// object its class alone decides the calls of. Any other read is left
    /// to the interpreter's, which None stands for.
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
        if site.gave_up() {
            return None;
        }
        let position = unit.positions[at];
        let (called_on, method, receiver) = match self.environment().get(name) {
            Some(held) => {
                let Some(method) = self.definition_bound(site, name, &held) else {
                    site.note_read_otherwise();
                    return None;
                };
                let receiver = self.eval_self(position).unwrap_or(Object::Nil);
                (self.builtins().class_of(&receiver), method, receiver)
            }
            None => {
                let (called_on, method, receiver, reads) = self.method_on_self(site, name)?;
                if reads
                    && self.tracepoints.is_empty()
                    && let Some(variable) = super::attributes::plain_reader(&method)
                    && let View::Instance(instance) = receiver.view()
                {
                    let value = instance
                        .borrow()
                        .instance_vars
                        .get(variable)
                        .cloned()
                        .unwrap_or(Object::Nil);
                    run.set_register(usize::from(to), value);
                    return Some(Ok(Next::Step));
                }
                (called_on, method, receiver)
            }
        };
        let ran = self
            .invoke_method(called_on, method, receiver, Vec::new(), position)
            .map(|value| {
                run.set_register(usize::from(to), value);
                Next::Step
            });
        Some(self.settle_instruction(run, at, ran))
    }

    /// Read the `GetIdentifier` at `at` from the cell the block running
    /// closed over for its name, when the block's scope binds no such name
    /// itself and the value is neither a method nor a native function,
    /// which a read calls. Whether it was read so.
    pub(super) fn read_closed_over(
        &self,
        site: &BareCallSite,
        run: &mut UnitRun,
        at: usize,
    ) -> bool {
        let unit = run.unit();
        let Instruction::GetIdentifier { to, name } = unit.instructions[at] else {
            return false;
        };
        let name = unit.names[usize::from(name)].text.as_str();
        let scope = self.environment().current_scope_ref().borrow();
        if scope.binds(name) {
            return false;
        }
        let Some(captured) = scope.captured_names() else {
            return false;
        };
        let kept = site
            .closed_over
            .borrow()
            .as_ref()
            .filter(|(names, _)| std::ptr::eq(names.as_ptr(), Rc::as_ptr(captured)))
            .map(|(_, index)| *index);
        let index = match kept {
            Some(index) => index,
            None => {
                let Some(index) = captured.get_index_of(name) else {
                    return false;
                };
                *site.closed_over.borrow_mut() = Some((Rc::downgrade(captured), index));
                index
            }
        };
        let Some((held_name, cell)) = captured.get_index(index) else {
            return false;
        };
        debug_assert_eq!(held_name, name, "a closed-over name keeps its place");
        let value = cell.borrow().clone();
        if matches!(value.view(), View::Method(_) | View::NativeFunction(_)) {
            return false;
        }
        drop(scope);
        run.set_register(usize::from(to), value);
        true
    }

    /// The method `held`, the value `name` is bound to, when a `def` bound
    /// it there, as `name_is_a_definition` decides: the class it was
    /// defined in, or Object, still holds it under that name, which the
    /// method state alone decides, so `site` keeps the answer under it.
    fn definition_bound(
        &self,
        site: &BareCallSite,
        name: &str,
        held: &Object,
    ) -> Option<Rc<Method>> {
        let View::Method(method) = held.view() else {
            return None;
        };
        let state = method_state();
        let kept = site.defined.borrow().as_ref().is_some_and(|(kept, under)| {
            *under == state && std::ptr::eq(kept.as_ptr(), Rc::as_ptr(method))
        });
        if !kept {
            if method.name != name || method.receiver.is_some() {
                return None;
            }
            let installed = |owner: &Rc<Class>| {
                owner
                    .find_own_method(name)
                    .is_some_and(|found| Rc::ptr_eq(&found, method))
            };
            let object_holds = || match self.globals().get("Object").view() {
                Some(View::Class(object)) => installed(object),
                _ => false,
            };
            if !method.definee.as_ref().is_some_and(installed) && !object_holds() {
                return None;
            }
            *site.defined.borrow_mut() = Some((Rc::downgrade(method), state));
        }
        debug_assert!(
            self.name_is_a_definition(name, held),
            "a name bound to the method a def made is called"
        );
        Some(Rc::clone(method))
    }

    /// The method a call of `name` on `self` reaches, the class it was
    /// found on, and `self`, when no scope binds `name` and the current
    /// scope reads through the root scope alone.
    fn method_on_self(
        &mut self,
        site: &BareCallSite,
        name: &str,
    ) -> Option<(Rc<Class>, Rc<Method>, Object, bool)> {
        let Some((receiver, root)) = self.scope_reading_root() else {
            site.note_read_otherwise();
            return None;
        };
        let Some(class) = self.class_deciding_dispatch(&receiver) else {
            site.note_read_otherwise();
            return None;
        };
        let state = method_state();
        let root_version = root.borrow().version();
        let (called_on, method, reads) = match site.kept(&class, state, &root, root_version) {
            Some(kept) => kept,
            None => {
                if site.refused_under.get() == Some((state, root_version)) {
                    return None;
                }
                let found = self
                    .lookup_method(&receiver, name)
                    .filter(|(_, method)| !method.is_undefined);
                let Some((called_on, method)) = found else {
                    site.refused_under.set(Some((state, root_version)));
                    return None;
                };
                let reads = super::attributes::plain_reader(&method).is_some()
                    && !crate::vm::native_methods::class_answers_natively(&class, name, true);
                *site.found.borrow_mut() = Some(BareCall {
                    class: Rc::downgrade(&class),
                    called_on: Rc::downgrade(&called_on),
                    method: Rc::downgrade(&method),
                    method_state: state,
                    root: Rc::downgrade(&root),
                    root_version,
                    reads,
                });
                (called_on, method, reads)
            }
        };
        debug_assert!(
            self.lookup_method(&receiver, name)
                .is_some_and(|(_, found)| Rc::ptr_eq(&found, &method)),
            "the method a bare name called last is what a lookup finds"
        );
        Some((called_on, method, receiver, reads))
    }

    /// `self` and the root scope, when the current scope reads through the
    /// root scope alone. `self` is the current scope's, or the root scope's
    /// when the current scope binds none.
    fn scope_reading_root(&self) -> Option<(Object, Rc<RefCell<Scope>>)> {
        let environment = self.environment();
        let root = environment.root_scope();
        let scope = environment.current_scope_ref().borrow();
        let reads_root_alone = scope
            .parent()
            .is_some_and(|parent| Rc::ptr_eq(parent, root));
        if !reads_root_alone {
            return None;
        }
        let receiver = match scope.own_value("self") {
            Some(receiver) => receiver,
            None if !scope.holds_own("self") => root.borrow().own_value("self")?,
            None => return None,
        };
        Some((receiver, Rc::clone(root)))
    }
}
