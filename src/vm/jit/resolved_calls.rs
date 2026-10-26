//! A `CallResolved` whose route is a call on `self`, made on an instance
//! with a singleton class or singleton methods, as `main` has both, calls
//! the method a lookup on that instance reached the last time, without the
//! lookup, while none of the singleton methods has the call's name and the
//! method state is as it was. The call site's own cache refuses such an
//! instance and leaves every call to the lookup.

use crate::class::{Class, method_state};
use crate::lexer::Position;
use crate::object::{Method, View};
use crate::vm::VirtualMachine;
use crate::vm::bytecode_runner::KnownCall;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// The method one `CallResolved` reached last on `self`.
#[derive(Default)]
pub(super) struct ResolvedCallSite {
    found: RefCell<Option<SelfCall>>,
}

/// The method a call reached through `class`, an instance's singleton
/// class or its class when it has none, under `method_state`. The weak references keep neither alive, and while they
/// are held no other class or method can take their addresses.
struct SelfCall {
    class: Weak<Class>,
    method: Weak<Method>,
    method_state: u64,
}

impl VirtualMachine {
    /// The call of `name` on `self` that `site` keeps, or that a lookup
    /// finds and `site` then keeps, when `self` is an instance with a
    /// singleton class or singleton methods, none of them `name`, while no
    /// refinement is active.
    pub(super) fn self_call_reaching(
        &self,
        site: &ResolvedCallSite,
        name: &str,
        position: Position,
    ) -> Option<KnownCall> {
        let receiver = self.eval_self(position).ok()?;
        let View::Instance(instance) = receiver.view() else {
            return None;
        };
        if self.any_refinement_active() {
            return None;
        }
        let class = {
            let held = instance.borrow();
            if held.find_singleton_method(name).is_some() {
                return None;
            }
            match held.singleton_class.borrow().as_ref() {
                Some(singleton) => Rc::clone(singleton),
                None if held.has_singleton_methods() => Rc::clone(&held.class),
                None => return None,
            }
        };
        let state = method_state();
        let kept = site
            .found
            .borrow()
            .as_ref()
            .filter(|call| {
                call.method_state == state && std::ptr::eq(call.class.as_ptr(), Rc::as_ptr(&class))
            })
            .and_then(|call| call.method.upgrade());
        let method = match kept {
            Some(method) => method,
            None => {
                let (_, method) = self
                    .lookup_method(&receiver, name)
                    .filter(|(_, method)| !method.is_undefined)?;
                if self.enumerable_stands_in(&receiver, name) {
                    return None;
                }
                *site.found.borrow_mut() = Some(SelfCall {
                    class: Rc::downgrade(&class),
                    method: Rc::downgrade(&method),
                    method_state: state,
                });
                method
            }
        };
        debug_assert!(
            self.lookup_method(&receiver, name)
                .is_some_and(|(found_on, found)| Rc::ptr_eq(&found, &method)
                    && Rc::ptr_eq(&found_on, &class)),
            "the method a call on self reached last is what a lookup finds"
        );
        Some(KnownCall {
            receiver,
            class,
            method,
        })
    }
}
