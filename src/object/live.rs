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

/// The fewest entries made between two collections.
const FIRST_COLLECTION_AT: usize = 2048;

/// The share of the objects that survived earlier collections that a
/// collection waits for the program to make before it runs: a fourth.
const OLD_ENTRIES_PER_YOUNG_ONE: usize = 4;

/// How much the objects that survived collections grow before a collection
/// covers them too, rather than only the objects made since the last one.
const OLD_GROWTH_BEFORE_A_WHOLE_COLLECTION: usize = 8;

/// The share of the live entries that dead ones reach before they are
/// cleared out between collections: a sixteenth. A dead entry's weak
/// reference keeps the object's allocation.
const LIVE_ENTRIES_PER_DEAD_ONE: usize = 16;

/// The fewest new entries between two clearings.
const FEWEST_ENTRIES_BETWEEN_CLEARINGS: usize = 1024;

/// Weak references to what was made, in the order it was made. A reference
/// keeps only its allocation, not the object, so the dead ones are cleared
/// out at each collection, and between collections whenever enough have been
/// made. The entries before `young_from` survived a collection.
pub(crate) struct Registry<T> {
    held: Vec<Weak<T>>,
    young_from: usize,
}

impl<T> Registry<T> {
    const fn new() -> Self {
        Self {
            held: Vec::new(),
            young_from: 0,
        }
    }

    fn record(&mut self, made: &Rc<T>) {
        self.held.push(Rc::downgrade(made));
    }

    pub(crate) fn alive(&self) -> Vec<Rc<T>> {
        self.held.iter().filter_map(Weak::upgrade).collect()
    }

    /// The objects alive that were made since the last collection.
    pub(crate) fn alive_young(&self) -> Vec<Rc<T>> {
        self.held[self.young_from..]
            .iter()
            .filter_map(Weak::upgrade)
            .collect()
    }

    fn alive_count(&self) -> usize {
        self.held
            .iter()
            .filter(|one| one.strong_count() > 0)
            .count()
    }

    /// Clear out the dead entries, answering how many are left and how many
    /// of those survived a collection.
    fn clear_dead(&mut self) -> (usize, usize) {
        let mut at = 0;
        let mut old = 0;
        let young_from = self.young_from;
        self.held.retain(|one| {
            let alive = one.strong_count() > 0;
            if alive && at < young_from {
                old += 1;
            }
            at += 1;
            alive
        });
        self.young_from = old;
        (self.held.len(), old)
    }

    fn promote(&mut self) {
        self.young_from = self.held.len();
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
    /// The entries that survived a collection.
    old: usize,
    /// The entries that survived the last collection that covered them all.
    old_after_whole: usize,
    /// The entry count that makes the next collection due.
    due_at: usize,
    /// The entry count at which the dead entries are next cleared out.
    clear_at: usize,
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
            old: 0,
            old_after_whole: FIRST_COLLECTION_AT,
            due_at: FIRST_COLLECTION_AT,
            clear_at: FEWEST_ENTRIES_BETWEEN_CLEARINGS,
            allocated: 0,
        })
    };
    static DUE: Cell<bool> = const { Cell::new(false) };
    static ENABLED: Cell<bool> = const { Cell::new(true) };
}

fn count_entry() {
    let clearing_due = ACCOUNT.with(|account| {
        let mut account = account.borrow_mut();
        account.entries += 1;
        account.allocated += 1;
        if account.entries >= account.due_at {
            DUE.with(|due| due.set(true));
        }
        account.entries >= account.clear_at
    });
    if clearing_due {
        let (alive, old) = clear_dead_entries();
        ACCOUNT.with(|account| {
            let mut account = account.borrow_mut();
            account.entries = alive;
            account.old = old;
            account.due_at = old + next_collection_after(old);
            account.clear_at = alive + next_clearing_after(alive);
        });
    }
    if crate::vm::memory_report::walk_wanted() {
        walk_for_memory_report();
    }
}

/// The counts an `Rc` keeps before what it holds.
const RC_COUNTS_BYTES: usize = 2 * std::mem::size_of::<usize>();

/// The bytes an `Rc` of `T` takes with its counts, plus the cell of the
/// value that points at it.
fn listed_bytes<T>() -> usize {
    RC_COUNTS_BYTES + std::mem::size_of::<T>() + super::types::BODY_CELL_BYTES
}

/// Add up what each listed object holds by its class, from the sizes of its
/// parts and the capacities of what it keeps on the heap, for the memory
/// report.
fn walk_for_memory_report() {
    use crate::vm::memory_report::ClassRow;

    crate::vm::memory_report::walk_starting();
    let mut by_class: std::collections::HashMap<String, (u64, u64)> =
        std::collections::HashMap::new();
    let mut count = |class: &str, bytes: usize| {
        let row = by_class.entry(class.to_string()).or_default();
        row.0 += 1;
        row.1 += bytes as u64;
    };
    let Ok(lists) = LISTS.try_with(|lists| {
        let Ok(lists) = lists.try_borrow() else {
            return;
        };
        for held in lists.instances.alive() {
            let Ok(instance) = held.try_borrow() else {
                continue;
            };
            let bytes = listed_bytes::<RefCell<Instance>>()
                + RC_COUNTS_BYTES * 2
                + std::mem::size_of::<RefCell<std::collections::HashMap<String, Rc<Method>>>>()
                + std::mem::size_of::<RefCell<Option<Rc<Class>>>>()
                + instance.instance_vars.held_bytes();
            let class = instance.class.name();
            count(
                if class.is_empty() {
                    "(anonymous class)"
                } else {
                    class
                },
                bytes,
            );
        }
        for held in lists.strings.alive() {
            count("String", listed_bytes::<StringValue>() + held.held_bytes());
        }
        for held in lists.arrays.alive() {
            let capacity = held.try_borrow().map_or(0, |items| items.capacity());
            count(
                "Array",
                listed_bytes::<RefCell<Vec<Object>>>() + capacity * std::mem::size_of::<Object>(),
            );
        }
        for held in lists.hashes.alive() {
            let held_bytes = held.try_borrow().map_or(0, |entries| {
                entries.capacity()
                    * (std::mem::size_of::<(u64, String, Object)>() + std::mem::size_of::<usize>())
                    + entries.keys().map(String::capacity).sum::<usize>()
            });
            count(
                "Hash",
                listed_bytes::<RefCell<IndexMap<String, Object>>>() + held_bytes,
            );
        }
        for held in lists.sets.alive() {
            let held_bytes = held.try_borrow().map_or(0, |entries| {
                entries.capacity()
                    * (std::mem::size_of::<(u64, ObjectHash)>() + std::mem::size_of::<usize>())
            });
            count(
                "Set",
                listed_bytes::<RefCell<IndexSet<ObjectHash>>>() + held_bytes,
            );
        }
        for held in lists.exceptions.alive() {
            let class = held.try_borrow().map_or_else(
                |_| "Exception".to_string(),
                |exception| exception.exception_type.clone(),
            );
            count(&class, listed_bytes::<RefCell<Exception>>());
        }
        for _ in lists.blocks.alive() {
            count("Proc", listed_bytes::<BlockStatement>());
        }
        for _ in lists.methods.alive() {
            count("(method definitions)", listed_bytes::<Method>());
        }
        for _ in lists.bindings.alive() {
            count("Binding", listed_bytes::<Binding>());
        }
        for _ in lists.classes.alive() {
            count("(classes and modules)", listed_bytes::<Class>());
        }
    }) else {
        return;
    };
    let () = lists;
    crate::vm::memory_report::record_walk(
        by_class
            .into_iter()
            .map(|(class, (count, bytes))| ClassRow {
                class,
                count,
                bytes,
            })
            .collect(),
    );
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

/// How many entries are made before the dead ones among them are cleared
/// out, with `alive` entries left by the last clearing.
fn next_clearing_after(alive: usize) -> usize {
    (alive / LIVE_ENTRIES_PER_DEAD_ONE).max(FEWEST_ENTRIES_BETWEEN_CLEARINGS)
}

/// How many entries are made before the next collection, with `old`
/// entries surviving earlier ones.
fn next_collection_after(old: usize) -> usize {
    (old / OLD_ENTRIES_PER_YOUNG_ONE).max(FIRST_COLLECTION_AT)
}

/// Clear the dead entries out of the list, answering how many are left and
/// how many of those survived a collection.
fn clear_dead_entries() -> (usize, usize) {
    LISTS.with(|lists| {
        let mut lists = lists.borrow_mut();
        [
            lists.instances.clear_dead(),
            lists.classes.clear_dead(),
            lists.strings.clear_dead(),
            lists.arrays.clear_dead(),
            lists.hashes.clear_dead(),
            lists.sets.clear_dead(),
            lists.exceptions.clear_dead(),
            lists.blocks.clear_dead(),
            lists.methods.clear_dead(),
            lists.bindings.clear_dead(),
        ]
        .into_iter()
        .fold((0, 0), |(alive, old), (more_alive, more_old)| {
            (alive + more_alive, old + more_old)
        })
    })
}

/// Whether the next collection covers every object, since those that
/// survived collections have grown enough since one last did.
pub(crate) fn whole_collection_due() -> bool {
    ACCOUNT.with(|account| {
        let account = account.borrow();
        account.old >= account.old_after_whole * OLD_GROWTH_BEFORE_A_WHOLE_COLLECTION
    })
}

/// Clear the dead entries out of the list after a collection, which covered
/// every object when `whole`, count every entry left as having survived
/// one, and wait for the next.
pub(crate) fn settle(whole: bool) {
    let (alive, _) = clear_dead_entries();
    LISTS.with(|lists| {
        let mut lists = lists.borrow_mut();
        lists.instances.promote();
        lists.classes.promote();
        lists.strings.promote();
        lists.arrays.promote();
        lists.hashes.promote();
        lists.sets.promote();
        lists.exceptions.promote();
        lists.blocks.promote();
        lists.methods.promote();
        lists.bindings.promote();
    });
    ACCOUNT.with(|account| {
        let mut account = account.borrow_mut();
        account.entries = alive;
        account.old = alive;
        if whole {
            account.old_after_whole = alive.max(FIRST_COLLECTION_AT);
        }
        account.due_at = alive + next_collection_after(alive);
        account.clear_at = alive + next_clearing_after(alive);
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

/// How many entries the list holds, dead or alive.
pub fn entry_count() -> usize {
    ACCOUNT.with(|account| account.borrow().entries)
}

/// How many objects have ever been listed.
pub fn allocated_count() -> u64 {
    ACCOUNT.with(|account| account.borrow().allocated)
}
