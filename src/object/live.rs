// Every heap object a program has made that is still alive: the list the
// cycle collector runs over, and the one `ObjectSpace.each_object` and
// `count_objects` read.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use indexmap::{IndexMap, IndexSet};

use crate::class::Class;

use super::{
    Binding, BlockStatement, Exception, Instance, Method, Object, ObjectHash, StringValue,
};

/// How many entries the list holds before the first collection. Each
/// collection doubles what it leaves for the next one to wait for.
const FIRST_COLLECTION_AT: usize = 2048;

/// Weak references to what was made, in the order it was made. A reference
/// keeps only its allocation, not the object, so the dead ones are cleared
/// out at each collection.
pub(crate) struct Registry<T> {
    held: Vec<Weak<T>>,
}

impl<T> Registry<T> {
    const fn new() -> Self {
        Self { held: Vec::new() }
    }

    fn record(&mut self, made: &Rc<T>) {
        self.held.push(Rc::downgrade(made));
    }

    pub(crate) fn alive(&self) -> Vec<Rc<T>> {
        self.held.iter().filter_map(Weak::upgrade).collect()
    }

    fn alive_count(&self) -> usize {
        self.held
            .iter()
            .filter(|one| one.strong_count() > 0)
            .count()
    }

    fn clear_dead(&mut self) -> usize {
        self.held.retain(|one| one.strong_count() > 0);
        self.held.len()
    }
}

/// The list, one registry for each kind of object.
pub(crate) struct Lists {
    pub(crate) instances: Registry<RefCell<Instance>>,
    pub(crate) classes: Registry<Class>,
    pub(crate) strings: Registry<StringValue>,
    pub(crate) arrays: Registry<RefCell<Vec<Object>>>,
    pub(crate) hashes: Registry<RefCell<IndexMap<String, Object>>>,
    pub(crate) sets: Registry<RefCell<IndexSet<ObjectHash>>>,
    pub(crate) exceptions: Registry<RefCell<Exception>>,
    pub(crate) blocks: Registry<BlockStatement>,
    pub(crate) methods: Registry<Method>,
    pub(crate) bindings: Registry<Binding>,
}

/// What the list has counted.
struct Account {
    /// Entries in the list, dead or alive.
    entries: usize,
    /// The entry count that makes the next collection due.
    due_at: usize,
    /// Every object ever listed.
    allocated: u64,
}

thread_local! {
    static LISTS: RefCell<Lists> = const {
        RefCell::new(Lists {
            instances: Registry::new(),
            classes: Registry::new(),
            strings: Registry::new(),
            arrays: Registry::new(),
            hashes: Registry::new(),
            sets: Registry::new(),
            exceptions: Registry::new(),
            blocks: Registry::new(),
            methods: Registry::new(),
            bindings: Registry::new(),
        })
    };
    static ACCOUNT: RefCell<Account> = const {
        RefCell::new(Account {
            entries: 0,
            due_at: FIRST_COLLECTION_AT,
            allocated: 0,
        })
    };
    static DUE: Cell<bool> = const { Cell::new(false) };
    static ENABLED: Cell<bool> = const { Cell::new(true) };
}

fn count_entry() {
    ACCOUNT.with(|account| {
        let mut account = account.borrow_mut();
        account.entries += 1;
        account.allocated += 1;
        if account.entries >= account.due_at {
            DUE.with(|due| due.set(true));
        }
    });
}

macro_rules! recorder {
    ($name:ident, $field:ident, $kind:ty) => {
        pub fn $name(made: &Rc<$kind>) {
            LISTS.with(|lists| lists.borrow_mut().$field.record(made));
            count_entry();
        }
    };
}

recorder!(record_instance, instances, RefCell<Instance>);
recorder!(record_class, classes, Class);
recorder!(record_string, strings, StringValue);
recorder!(record_array, arrays, RefCell<Vec<Object>>);
recorder!(record_hash, hashes, RefCell<IndexMap<String, Object>>);
recorder!(record_set, sets, RefCell<IndexSet<ObjectHash>>);
recorder!(record_exception, exceptions, RefCell<Exception>);
recorder!(record_block, blocks, BlockStatement);
recorder!(record_method, methods, Method);
recorder!(record_binding, bindings, Binding);

/// Read the list.
pub(crate) fn with_lists<R>(read: impl FnOnce(&Lists) -> R) -> R {
    LISTS.with(|lists| read(&lists.borrow()))
}

/// Every instance still alive, oldest first.
pub fn live_instances() -> Vec<Rc<RefCell<Instance>>> {
    with_lists(|lists| lists.instances.alive())
}

/// Every class and module still alive, oldest first.
pub fn live_classes() -> Vec<Rc<Class>> {
    with_lists(|lists| lists.classes.alive())
}

/// Where this thread's flag for a due collection is, which machine code
/// reads at each statement start.
pub(crate) fn due_flag() -> *const u8 {
    DUE.with(|due| due.as_ptr().cast_const().cast())
}

/// Whether the list has doubled since the last collection left it.
#[inline]
pub fn collection_due() -> bool {
    DUE.with(Cell::get)
}

/// Whether collections run, which `GC.disable` and `GC.enable` switch.
pub fn collecting() -> bool {
    ENABLED.with(Cell::get)
}

/// Switch collections on or off, answering whether they were off.
pub fn set_collecting(wanted: bool) -> bool {
    ENABLED.with(|enabled| !enabled.replace(wanted))
}

/// Clear the dead entries out of the list, and wait for it to double again.
pub(crate) fn settle() {
    let alive = LISTS.with(|lists| {
        let mut lists = lists.borrow_mut();
        lists.instances.clear_dead()
            + lists.classes.clear_dead()
            + lists.strings.clear_dead()
            + lists.arrays.clear_dead()
            + lists.hashes.clear_dead()
            + lists.sets.clear_dead()
            + lists.exceptions.clear_dead()
            + lists.blocks.clear_dead()
            + lists.methods.clear_dead()
            + lists.bindings.clear_dead()
    });
    ACCOUNT.with(|account| {
        let mut account = account.borrow_mut();
        account.entries = alive;
        account.due_at = (alive * 2).max(FIRST_COLLECTION_AT);
    });
    DUE.with(|due| due.set(false));
}

/// How many listed objects are alive.
pub fn alive_count() -> usize {
    with_lists(|lists| {
        lists.instances.alive_count()
            + lists.classes.alive_count()
            + lists.strings.alive_count()
            + lists.arrays.alive_count()
            + lists.hashes.alive_count()
            + lists.sets.alive_count()
            + lists.exceptions.alive_count()
            + lists.blocks.alive_count()
            + lists.methods.alive_count()
            + lists.bindings.alive_count()
    })
}

/// How many objects have ever been listed.
pub fn allocated_count() -> u64 {
    ACCOUNT.with(|account| account.borrow().allocated)
}
