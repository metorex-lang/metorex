//! The methods a call site has reached, kept by the receiver's class so a
//! later call on an object of the same class skips the lookup.

use crate::class::{Class, method_state};
use crate::object::Method;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

/// How many receiver classes one call site keeps a method for. A site that
/// sees more replaces the class it stored first.
pub const CLASSES_PER_SITE: usize = 4;

static HITS: AtomicU64 = AtomicU64::new(0);
static MISSES: AtomicU64 = AtomicU64::new(0);

/// The calls answered from a call site's cache and those that looked the
/// method up, so far.
pub fn hits_and_misses() -> (u64, u64) {
    (HITS.load(Ordering::Relaxed), MISSES.load(Ordering::Relaxed))
}

#[derive(Debug, Default)]
pub struct CallCache {
    entries: RefCell<Vec<Entry>>,
}

/// One class and the method a call on its instances reached. The weak
/// references keep neither alive, and while the class's allocation is held
/// no other class can take its address.
#[derive(Debug)]
struct Entry {
    class: Weak<Class>,
    method: Weak<Method>,
    /// The method state the lookup ran under. Defining or removing a method,
    /// a mixin, or a visibility marking moves the state on.
    state: u64,
}

impl CallCache {
    /// The method kept for `class` under the current method state.
    pub fn find(&self, class: &Rc<Class>) -> Option<Rc<Method>> {
        let state = method_state();
        let found = self
            .entries
            .borrow()
            .iter()
            .find(|entry| {
                entry.state == state && std::ptr::eq(entry.class.as_ptr(), Rc::as_ptr(class))
            })
            .and_then(|entry| entry.method.upgrade());
        match found {
            Some(_) => HITS.fetch_add(1, Ordering::Relaxed),
            None => MISSES.fetch_add(1, Ordering::Relaxed),
        };
        found
    }

    /// Keep `method` as what a call on an instance of `class` reaches.
    pub fn store(&self, class: &Rc<Class>, method: &Rc<Method>) {
        let state = method_state();
        let mut entries = self.entries.borrow_mut();
        entries.retain(|entry| {
            entry.state == state && !std::ptr::eq(entry.class.as_ptr(), Rc::as_ptr(class))
        });
        if entries.len() == CLASSES_PER_SITE {
            entries.remove(0);
        }
        entries.push(Entry {
            class: Rc::downgrade(class),
            method: Rc::downgrade(method),
            state,
        });
    }
}
