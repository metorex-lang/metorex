//! A block literal run again where it ran last, with nothing holding the
//! block it made then, hands out that block again rather than building one.
//! Everything building a block reads is recorded beside it, and the block is
//! used again only while all of it is unchanged.

use crate::class::Class;
use crate::object::{BlockStatement, CodePlaces};
use crate::scope::Scope;
use crate::vm::VirtualMachine;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

thread_local! {
    /// Every literal's kept block, which a collection lets go of so that
    /// what the blocks captured can be freed.
    static KEPT: RefCell<Vec<Weak<KeptBlock>>> = const { RefCell::new(Vec::new()) };
    /// A number every change to a top-level name, or to a local holding a
    /// method or a function, moves on. Either changes which names a block
    /// lists as locals.
    static SCOPE_STATE: Cell<u64> = const { Cell::new(0) };
}

/// Mark every kept block out of date.
pub(crate) fn scope_state_changed() {
    SCOPE_STATE.with(|state| state.set(state.get() + 1));
}

fn scope_state() -> u64 {
    SCOPE_STATE.with(Cell::get)
}

/// Let go of every kept block.
pub(crate) fn release_kept_blocks() {
    KEPT.with(|kept| {
        for held in kept.borrow_mut().drain(..) {
            if let Some(held) = held.upgrade() {
                held.kept.borrow_mut().take();
            }
        }
    });
}

/// The block a literal made last, with what it was built from.
#[derive(Debug, Default)]
pub struct KeptBlock {
    kept: RefCell<Option<(Rc<BlockStatement>, Built)>>,
}

/// What building a block read. The classes and the scope are held weakly,
/// which keeps their addresses from being taken by another while they are
/// compared by address.
#[derive(Debug)]
struct Built {
    /// The scope the block was built in and each it reads names through,
    /// with the version each had.
    scopes: Vec<(Weak<RefCell<Scope>>, u64)>,
    scope_state: u64,
    method_frame: Option<u64>,
    lexical_home_frame: Option<Option<u64>>,
    frames: usize,
    bodies: Vec<Weak<Class>>,
    nesting: Option<Vec<Weak<Class>>>,
    method_owner: Option<Option<Weak<Class>>>,
    source_file: Option<String>,
    current_file: Option<std::path::PathBuf>,
    within: Option<CodePlaces>,
}

fn weak_list(classes: &[Rc<Class>]) -> Vec<Weak<Class>> {
    classes.iter().map(Rc::downgrade).collect()
}

fn same_classes(kept: &[Weak<Class>], open: &[Rc<Class>]) -> bool {
    kept.len() == open.len()
        && kept
            .iter()
            .zip(open)
            .all(|(kept, open)| std::ptr::eq(kept.as_ptr(), Rc::as_ptr(open)))
}

impl VirtualMachine {
    fn block_built_here(&self) -> Built {
        let mut scopes = Vec::new();
        let mut cursor = Some(self.environment().current_scope());
        while let Some(scope) = cursor {
            let held = scope.borrow();
            scopes.push((Rc::downgrade(&scope), held.version()));
            cursor = held.parent().cloned();
        }
        Built {
            scopes,
            scope_state: scope_state(),
            method_frame: self.current_method_frame,
            lexical_home_frame: self.lexical_home_frame,
            frames: self.call_stack.len(),
            bodies: weak_list(&self.def_scope_stack),
            nesting: self
                .method_nesting_stack
                .last()
                .map(|nesting| weak_list(nesting)),
            method_owner: self
                .method_owner_stack
                .last()
                .map(|owner| owner.as_ref().map(Rc::downgrade)),
            source_file: self.current_source_file.clone(),
            current_file: self.current_file.clone(),
            within: self
                .running_code
                .last()
                .map(|running| Rc::clone(&running.within)),
        }
    }

    fn built_here(&self, built: &Built) -> bool {
        let same_owner = match (&built.method_owner, self.method_owner_stack.last()) {
            (None, None) | (Some(None), Some(None)) => true,
            (Some(Some(kept)), Some(Some(open))) => std::ptr::eq(kept.as_ptr(), Rc::as_ptr(open)),
            _ => false,
        };
        let same_nesting = match (&built.nesting, self.method_nesting_stack.last()) {
            (None, None) => true,
            (Some(kept), Some(open)) => same_classes(kept, open),
            _ => false,
        };
        let same_within = match (&built.within, self.running_code.last()) {
            (None, None) => true,
            (Some(kept), Some(running)) => Rc::ptr_eq(kept, &running.within),
            _ => false,
        };
        built.scope_state == scope_state()
            && self.same_scopes(&built.scopes)
            && built.method_frame == self.current_method_frame
            && built.lexical_home_frame == self.lexical_home_frame
            && built.frames == self.call_stack.len()
            && same_classes(&built.bodies, &self.def_scope_stack)
            && same_nesting
            && same_owner
            && same_within
            && built.source_file == self.current_source_file
            && built.current_file == self.current_file
    }

    /// Whether the scope running now and those it reads through are the ones
    /// `scopes` lists, each at the version listed.
    fn same_scopes(&self, scopes: &[(Weak<RefCell<Scope>>, u64)]) -> bool {
        let mut cursor = Some(self.environment().current_scope());
        for (kept, version) in scopes {
            let Some(scope) = cursor else {
                return false;
            };
            let held = scope.borrow();
            if !std::ptr::eq(kept.as_ptr(), Rc::as_ptr(&scope)) || held.version() != *version {
                return false;
            }
            cursor = held.parent().cloned();
        }
        cursor.is_none()
    }

    /// The block `kept` made last, when nothing else holds it or the flags
    /// it shares and it would be built the same way here.
    pub(crate) fn reusable_block(&self, kept: &KeptBlock) -> Option<Rc<BlockStatement>> {
        let held = kept.kept.borrow();
        let (block, built) = held.as_ref()?;
        // The list of live objects holds every block weakly, and nothing
        // else may.
        let unshared = Rc::strong_count(block) == 1
            && Rc::weak_count(block) <= 1
            && Rc::strong_count(&block.attached_call_running) == 1
            && Rc::strong_count(&block.ruby2_keywords) == 1;
        if !unshared || !self.built_here(built) {
            return None;
        }
        // A block starts out attached to the call it is written on.
        block.attached_call_running.set(true);
        Some(Rc::clone(block))
    }

    /// Keep `block` as the one its literal made here.
    pub(crate) fn keep_block(&self, kept: &Rc<KeptBlock>, block: &Rc<BlockStatement>) {
        let first = kept
            .kept
            .borrow_mut()
            .replace((Rc::clone(block), self.block_built_here()))
            .is_none();
        if first {
            KEPT.with(|registry| registry.borrow_mut().push(Rc::downgrade(kept)));
        }
    }
}
