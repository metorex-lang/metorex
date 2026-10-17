//! The value a constant reference read, kept with the lexical scope it was
//! read in so a later read in the same scope skips the lookup.

use crate::class::{Class, constant_state, method_state};
use crate::object::Object;
use std::cell::RefCell;
use std::rc::Weak;
use std::sync::atomic::{AtomicU64, Ordering};

static MISSES: AtomicU64 = AtomicU64::new(0);

/// The constant reads that looked the constant up, so far.
pub fn misses() -> u64 {
    MISSES.load(Ordering::Relaxed)
}

/// The lexical scope a constant was read in: the classes and modules a
/// lookup reads constants from, held weakly so that while the key is kept
/// no other class can take one's address.
#[derive(Debug, PartialEq)]
pub struct ConstantScope {
    /// The class a block run as a class body reads constants where it was
    /// written in, when one applies.
    pub home: Option<Option<WeakClass>>,
    /// The class and module bodies open, outermost first.
    pub bodies: Vec<WeakClass>,
    /// The nesting the running method or block was written in.
    pub nesting: Option<Vec<WeakClass>>,
    pub written_at_top_level: bool,
}

/// A class compared by its address.
#[derive(Debug)]
pub struct WeakClass(pub Weak<Class>);

impl PartialEq for WeakClass {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

#[derive(Debug, Default)]
pub struct ConstantCache {
    entry: RefCell<Option<Box<Entry>>>,
}

#[derive(Debug)]
struct Entry {
    kept: Kept,
    scope: ConstantScope,
    constant_state: u64,
    method_state: u64,
}

/// What a lookup found, and what a later read has to check again.
#[derive(Debug)]
pub enum Kept {
    /// A value read ahead of the locals.
    AheadOfLocals(Object),
    /// A value read once no local of the name answered, so a local of the
    /// same name defined since answers instead.
    PastLocals(Object),
    /// The value a local of the name holds, read again each time.
    InLocals,
}

/// A local of a constant's name, as `ConstantCache::find` is told of it.
pub enum Local {
    Absent,
    /// A local holding a value the name answers as itself.
    Plain(Object),
    /// A local holding a method or a function, which the name calls.
    Callable,
}

impl ConstantCache {
    /// The value kept for `scope`, while no constant, autoload, or ancestry
    /// has changed since it was read. `local` finds the local of the
    /// constant's name.
    pub fn find(
        &self,
        matches_scope: impl FnOnce(&ConstantScope) -> bool,
        local: impl FnOnce() -> Local,
    ) -> Option<Object> {
        let held = self.entry.borrow();
        let found = held
            .as_ref()
            .filter(|entry| {
                entry.constant_state == constant_state()
                    && entry.method_state == method_state()
                    && matches_scope(&entry.scope)
            })
            .and_then(|entry| match &entry.kept {
                Kept::AheadOfLocals(value) => Some(value.clone()),
                Kept::PastLocals(value) => matches!(local(), Local::Absent).then(|| value.clone()),
                Kept::InLocals => match local() {
                    Local::Plain(value) => Some(value),
                    _ => None,
                },
            });
        if found.is_none() {
            MISSES.fetch_add(1, Ordering::Relaxed);
        }
        found
    }

    pub fn store(&self, kept: Kept, scope: ConstantScope) {
        *self.entry.borrow_mut() = Some(Box::new(Entry {
            kept,
            scope,
            constant_state: constant_state(),
            method_state: method_state(),
        }));
    }
}
