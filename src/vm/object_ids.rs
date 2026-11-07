//! The ids `object_id` answers for objects held by reference. Ruby hands
//! them out from a counter the first time each object is asked, a step of
//! eight apart.

use std::rc::{Rc, Weak};

use super::VirtualMachine;
use crate::object::{Object, View};

pub(crate) const FIRST_OBJECT_ID: u64 = 8;
const OBJECT_ID_STEP: u64 = 8;
pub(crate) const FIRST_PRUNE_AT: usize = 4096;

/// A weak hold on an object that was given an id. While it is held the
/// object's allocation stays, so its address cannot go to another object.
pub(crate) struct HeldWeakly(Box<dyn Fn() -> bool>);

impl HeldWeakly {
    fn of<T: 'static>(held: &Rc<T>) -> Self {
        let weak: Weak<T> = Rc::downgrade(held);
        Self(Box::new(move || weak.strong_count() > 0))
    }

    fn is_alive(&self) -> bool {
        (self.0)()
    }
}

impl VirtualMachine {
    /// The next id from the counter.
    pub(crate) fn take_object_id(&mut self) -> u64 {
        let id = self.next_object_id;
        self.next_object_id += OBJECT_ID_STEP;
        id
    }

    /// The id of an object held by reference, given the first time it is
    /// asked for and the same every time after.
    pub(crate) fn counted_object_id<T: 'static>(&mut self, held: &Rc<T>) -> u64 {
        let address = Rc::as_ptr(held) as *const () as usize;
        if let Some((_, id)) = self.object_ids.get(&address) {
            return *id;
        }
        if self.object_ids.len() >= self.object_ids_prune_at {
            self.object_ids.retain(|_, (kept, _)| kept.is_alive());
            self.object_ids_prune_at = FIRST_PRUNE_AT.max(self.object_ids.len() * 2);
        }
        let id = self.take_object_id();
        self.object_ids.insert(address, (HeldWeakly::of(held), id));
        id
    }
}

/// The address of an object held by reference, which tells it apart from
/// every other live object without giving it an id. None for a value that
/// stands for itself.
pub(crate) fn reference_address(held: &Object) -> Option<usize> {
    let address = match held.view() {
        View::Instance(instance) => Rc::as_ptr(&instance) as *const () as usize,
        View::String(text) => Rc::as_ptr(&text) as *const () as usize,
        View::Array(elements) => Rc::as_ptr(&elements) as *const () as usize,
        View::Dict(entries) => Rc::as_ptr(&entries) as *const () as usize,
        View::Set(members) => Rc::as_ptr(&members) as *const () as usize,
        View::Class(class) | View::Module(class) => Rc::as_ptr(&class) as *const () as usize,
        View::Block(block) => Rc::as_ptr(&block) as *const () as usize,
        View::Method(method) => Rc::as_ptr(&method) as *const () as usize,
        View::Binding(binding) => Rc::as_ptr(&binding) as *const () as usize,
        View::Exception(details) => Rc::as_ptr(&details) as *const () as usize,
        View::BigInt(value) => Rc::as_ptr(&value) as *const () as usize,
        _ => return None,
    };
    Some(address)
}
