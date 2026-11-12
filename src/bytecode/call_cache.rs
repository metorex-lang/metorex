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
    /// For a `send` written with the name to send: the class or module
    /// deciding the last receiver's dispatch, the method state then, and
    /// whether the core library's `send` answered it.
    core_send: RefCell<Option<(Weak<Class>, u64, bool)>>,
    /// The class and kind of the last receiver a native method answered
    /// after every check the lookup makes, and the method state then.
    native: RefCell<Option<(Weak<Class>, ReceiverKind, u64)>>,
}

/// What a receiver is, among the kinds a native answer is kept for. Values
/// of different kinds that share a class, nil and true among them, are told
/// apart by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiverKind {
    Instance,
    /// A class itself, kept by the class it is rather than by `Class`.
    ClassObject,
    Nil,
    False,
    True,
    Integer,
    BigInteger,
    Float,
    String,
    Symbol,
    Array,
    Hash,
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
        let found = self.peek(class);
        match found {
            Some(_) => HITS.fetch_add(1, Ordering::Relaxed),
            None => MISSES.fetch_add(1, Ordering::Relaxed),
        };
        found
    }

    /// `find`, without counting the lookup among the hits and misses.
    pub fn peek(&self, class: &Rc<Class>) -> Option<Rc<Method>> {
        let state = method_state();
        self.entries
            .borrow()
            .iter()
            .find(|entry| {
                entry.state == state && std::ptr::eq(entry.class.as_ptr(), Rc::as_ptr(class))
            })
            .and_then(|entry| entry.method.upgrade())
    }

    /// Whether the core library's `send` answered a receiver whose dispatch
    /// `class` decides, under the current method state, when that is known.
    pub fn core_send_for(&self, class: &Rc<Class>) -> Option<bool> {
        let state = method_state();
        match &*self.core_send.borrow() {
            Some((held, at, answer))
                if *at == state && std::ptr::eq(held.as_ptr(), Rc::as_ptr(class)) =>
            {
                Some(*answer)
            }
            _ => None,
        }
    }

    /// Keep whether the core library's `send` answers a receiver whose
    /// dispatch `class` decides.
    pub fn keep_core_send(&self, class: &Rc<Class>, answer: bool) {
        *self.core_send.borrow_mut() = Some((Rc::downgrade(class), method_state(), answer));
    }

    /// Whether a native method answered the last receiver of `kind` whose
    /// dispatch `class` decides, under the current method state.
    pub fn answered_natively(&self, class: &Rc<Class>, kind: ReceiverKind) -> bool {
        matches!(
            &*self.native.borrow(),
            Some((held, held_kind, state))
                if *held_kind == kind
                    && *state == method_state()
                    && std::ptr::eq(held.as_ptr(), Rc::as_ptr(class))
        )
    }

    /// Keep that a native method answered a receiver of `kind` whose
    /// dispatch `class` decides.
    pub fn keep_native_answer(&self, class: &Rc<Class>, kind: ReceiverKind) {
        *self.native.borrow_mut() = Some((Rc::downgrade(class), kind, method_state()));
    }

    /// Count a call answered from the method `peek` found.
    pub fn count_hit(&self) {
        HITS.fetch_add(1, Ordering::Relaxed);
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
