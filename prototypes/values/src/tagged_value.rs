//! An 8-byte value laid out the way CRuby lays out a VALUE: an Integer that
//! fits in 63 bits with its low bit set, a Float whose exponent is in the
//! common range rotated into a "flonum" tagged 0b10, nil, true, and false as
//! fixed constants, and anything else a pointer to a cell on the heap.
//!
//! The cells are managed one of three ways, chosen by `MODE`:
//!
//! - `COUNTED`: each cell counts its references and is freed when the count
//!   reaches zero. A cycle of cells is never freed.
//! - `CYCLES`: counted the same way, and every cell is listed so a cycle
//!   collector can find cycles nothing outside them refers to and free them.
//!   It runs CPython's trial deletion: each cell's count, less the
//!   references other listed cells hold to it, is what refers to it from
//!   outside the heap, so a Rust local or a Rust `Vec` holding a value keeps
//!   it alive without the collector knowing where it is.
//! - `TRACED`: nothing is counted. Every cell is listed, and a mark and
//!   sweep from the interpreter's stack frees what it cannot reach. This only
//!   works where every root is known, as it is in the prototype's
//!   interpreter.

use crate::machine::Representation;
use std::cell::{Cell, RefCell};

pub const COUNTED: u8 = 0;
pub const CYCLES: u8 = 1;
pub const TRACED: u8 = 2;

const FALSE: u64 = 0x00;
const NIL: u64 = 0x08;
const TRUE: u64 = 0x14;
const FIXNUM_MAX: i64 = i64::MAX >> 1;
const FIXNUM_MIN: i64 = i64::MIN >> 1;
/// The flonum CRuby uses for 0.0, whose bits would otherwise read as false.
const FLONUM_ZERO: u64 = 0x8000_0000_0000_0002;
/// The fewest listed cells a collection waits for, CPython's first
/// generation threshold.
const FIRST_COLLECTION_AT: usize = 700;

pub struct TaggedOf<const MODE: u8>(u64);

/// The value the comparisons other than memory management use.
pub type Tagged = TaggedOf<COUNTED>;

enum Body<const MODE: u8> {
    Slots(RefCell<Vec<TaggedOf<MODE>>>),
    Float(f64),
    Integer(i128),
}

/// What a pointer value points at. The alignment keeps a pointer's low three
/// bits clear, which is what tells it apart from every immediate value.
#[repr(C, align(8))]
struct HeapCell<const MODE: u8> {
    references: Cell<usize>,
    /// Where the cell sits in the list of cells, for the modes that keep one.
    listed_at: Cell<usize>,
    /// The cycle collector's count of references from outside the heap, or
    /// the mark of a mark and sweep.
    scratch: Cell<isize>,
    body: Body<MODE>,
}

/// The cells of one mode, and when the next collection runs.
struct Listing {
    cells: Vec<usize>,
    collect_at: usize,
    collections: usize,
}

thread_local! {
    static LISTINGS: [RefCell<Listing>; 3] = const {
        [
            RefCell::new(Listing { cells: Vec::new(), collect_at: FIRST_COLLECTION_AT, collections: 0 }),
            RefCell::new(Listing { cells: Vec::new(), collect_at: FIRST_COLLECTION_AT, collections: 0 }),
            RefCell::new(Listing { cells: Vec::new(), collect_at: FIRST_COLLECTION_AT, collections: 0 }),
        ]
    };
    /// Cells alive now and at most, for each mode.
    static LIVE: [Cell<usize>; 3] = const { [Cell::new(0), Cell::new(0), Cell::new(0)] };
    static PEAK: [Cell<usize>; 3] = const { [Cell::new(0), Cell::new(0), Cell::new(0)] };
    /// Cells alive when the counts were last reset, which a run that leaks
    /// leaves behind for the next.
    static BASE: [Cell<usize>; 3] = const { [Cell::new(0), Cell::new(0), Cell::new(0)] };
    /// How many Floats outside the flonum range went to the heap.
    pub static HEAP_FLOATS: Cell<usize> = const { Cell::new(0) };
}

/// The most cells mode `mode` held alive at once since the last reset, past
/// those alive at the reset, and how many collections it ran.
pub fn peak_cells_and_collections(mode: u8) -> (usize, usize) {
    let peak =
        PEAK.with(|peak| peak[mode as usize].get()) - BASE.with(|base| base[mode as usize].get());
    let collections = LISTINGS.with(|listings| listings[mode as usize].borrow().collections);
    (peak, collections)
}

pub fn reset_counts(mode: u8) {
    let now = LIVE.with(|live| live[mode as usize].get());
    PEAK.with(|peak| peak[mode as usize].set(now));
    BASE.with(|base| base[mode as usize].set(now));
    LISTINGS.with(|listings| listings[mode as usize].borrow_mut().collections = 0);
}

fn listed(mode: u8) -> bool {
    mode != COUNTED
}

impl<const MODE: u8> TaggedOf<MODE> {
    /// The bits a C extension is handed as the VALUE.
    pub fn bits(&self) -> u64 {
        self.0
    }

    /// The value a VALUE C handed back stands for, holding a reference of
    /// its own when it is a pointer.
    pub fn from_bits_taking_reference(bits: u64) -> Self {
        let borrowed = std::mem::ManuallyDrop::new(TaggedOf::<MODE>(bits));
        (*borrowed).clone()
    }

    fn is_fixnum(&self) -> bool {
        self.0 & 1 == 1
    }

    fn is_flonum(&self) -> bool {
        self.0 & 3 == 2
    }

    fn is_pointer(&self) -> bool {
        self.0 & 7 == 0 && self.0 > TRUE
    }

    fn cell(&self) -> &HeapCell<MODE> {
        debug_assert!(self.is_pointer());
        // SAFETY: a pointer value names a cell made by `on_heap` that has
        // not been freed: counted modes free a cell only when its last
        // reference goes, and a sweep frees only what no root reaches.
        unsafe { &*(self.0 as *const HeapCell<MODE>) }
    }

    fn on_heap(body: Body<MODE>) -> Self {
        let cell = Box::into_raw(Box::new(HeapCell {
            references: Cell::new(1),
            listed_at: Cell::new(0),
            scratch: Cell::new(0),
            body,
        }));
        if listed(MODE) {
            LISTINGS.with(|listings| {
                let mut listing = listings[MODE as usize].borrow_mut();
                // SAFETY: the cell was made just above.
                unsafe { &*cell }.listed_at.set(listing.cells.len());
                listing.cells.push(cell as usize);
            });
        }
        LIVE.with(|live| {
            let now = live[MODE as usize].get() + 1;
            live[MODE as usize].set(now);
            PEAK.with(|peak| peak[MODE as usize].set(peak[MODE as usize].get().max(now)));
        });
        TaggedOf(cell as u64)
    }

    /// Free the cell at `address`, taking it off the list first.
    fn free(address: u64) {
        if listed(MODE) {
            LISTINGS.with(|listings| {
                let mut listing = listings[MODE as usize].borrow_mut();
                // SAFETY: the cell is alive until the drop below.
                let at = unsafe { &*(address as *const HeapCell<MODE>) }
                    .listed_at
                    .get();
                listing.cells.swap_remove(at);
                if let Some(moved) = listing.cells.get(at) {
                    // SAFETY: every listed address is a live cell.
                    unsafe { &*(*moved as *const HeapCell<MODE>) }
                        .listed_at
                        .set(at);
                }
            });
        }
        LIVE.with(|live| live[MODE as usize].set(live[MODE as usize].get() - 1));
        // SAFETY: nothing refers to the cell any more.
        drop(unsafe { Box::from_raw(address as *mut HeapCell<MODE>) });
    }

    fn fixnum(number: i64) -> Self {
        TaggedOf(((number as u64) << 1) | 1)
    }

    pub fn fixnum_value(&self) -> i64 {
        (self.0 as i64) >> 1
    }

    fn wide_integer(&self) -> Option<i128> {
        if self.is_fixnum() {
            return Some(self.fixnum_value() as i128);
        }
        if self.is_pointer()
            && let Body::Integer(number) = self.cell().body
        {
            return Some(number);
        }
        None
    }

    fn float_value(&self) -> Option<f64> {
        if self.is_flonum() {
            if self.0 == FLONUM_ZERO {
                return Some(0.0);
            }
            let sign = self.0 >> 63;
            let restored = ((2 - sign) | (self.0 & !3)).rotate_right(3);
            return Some(f64::from_bits(restored));
        }
        if self.is_pointer()
            && let Body::Float(number) = self.cell().body
        {
            return Some(number);
        }
        None
    }

    fn slots(&self) -> &RefCell<Vec<TaggedOf<MODE>>> {
        match &self.cell().body {
            Body::Slots(held) => held,
            _ => panic!("the value holds no slots"),
        }
    }
}

impl<const MODE: u8> Clone for TaggedOf<MODE> {
    fn clone(&self) -> Self {
        if MODE != TRACED && self.is_pointer() {
            let references = &self.cell().references;
            references.set(references.get() + 1);
        }
        TaggedOf(self.0)
    }
}

impl<const MODE: u8> Drop for TaggedOf<MODE> {
    fn drop(&mut self) {
        if MODE == TRACED || !self.is_pointer() {
            return;
        }
        let references = &self.cell().references;
        let left = references.get() - 1;
        references.set(left);
        if left == 0 {
            Self::free(self.0);
        }
    }
}

/// The listed cells a cell's slots point at.
fn children<const MODE: u8>(address: usize) -> Vec<usize> {
    // SAFETY: every listed address is a live cell.
    let cell = unsafe { &*(address as *const HeapCell<MODE>) };
    match &cell.body {
        Body::Slots(held) => held
            .borrow()
            .iter()
            .filter(|value| value.is_pointer())
            .map(|value| value.0 as usize)
            .collect(),
        _ => Vec::new(),
    }
}

fn scratch<const MODE: u8>(address: usize) -> &'static Cell<isize> {
    // SAFETY: every listed address is a live cell, and the collector reads
    // this only while it is.
    &unsafe { &*(address as *const HeapCell<MODE>) }.scratch
}

/// CPython's trial deletion over every listed cell. Cells whose count is
/// more than the references other cells hold to them are referred to from
/// outside, and they and what they reach stay. The rest are garbage cycles:
/// each is held for the moment, its slots are emptied, and letting go of
/// them frees them.
fn collect_cycles() {
    let cells = LISTINGS.with(|listings| listings[CYCLES as usize].borrow().cells.clone());
    for address in &cells {
        // SAFETY: every listed address is a live cell.
        let references = unsafe { &*(*address as *const HeapCell<CYCLES>) }
            .references
            .get();
        scratch::<CYCLES>(*address).set(references as isize);
    }
    for address in &cells {
        for child in children::<CYCLES>(*address) {
            let held = scratch::<CYCLES>(child);
            held.set(held.get() - 1);
        }
    }
    let mut reached: Vec<usize> = cells
        .iter()
        .copied()
        .filter(|address| scratch::<CYCLES>(*address).get() > 0)
        .collect();
    for address in &reached {
        scratch::<CYCLES>(*address).set(1);
    }
    while let Some(address) = reached.pop() {
        for child in children::<CYCLES>(address) {
            if scratch::<CYCLES>(child).get() <= 0 {
                scratch::<CYCLES>(child).set(1);
                reached.push(child);
            }
        }
    }
    let garbage: Vec<TaggedOf<CYCLES>> = cells
        .iter()
        .filter(|address| scratch::<CYCLES>(**address).get() <= 0)
        .map(|address| TaggedOf::<CYCLES>::from_bits_taking_reference(*address as u64))
        .collect();
    for value in &garbage {
        if let Body::Slots(held) = &value.cell().body {
            let emptied = std::mem::take(&mut *held.borrow_mut());
            drop(emptied);
        }
    }
    drop(garbage);
}

/// Mark what the stack reaches and free every listed cell it does not.
fn mark_and_sweep(roots: &[TaggedOf<TRACED>]) {
    let cells = LISTINGS.with(|listings| listings[TRACED as usize].borrow().cells.clone());
    for address in &cells {
        scratch::<TRACED>(*address).set(0);
    }
    let mut reached: Vec<usize> = roots
        .iter()
        .filter(|value| value.is_pointer())
        .map(|value| value.0 as usize)
        .collect();
    while let Some(address) = reached.pop() {
        if scratch::<TRACED>(address).get() == 1 {
            continue;
        }
        scratch::<TRACED>(address).set(1);
        reached.extend(children::<TRACED>(address));
    }
    for address in cells {
        if scratch::<TRACED>(address).get() == 0 {
            TaggedOf::<TRACED>::free(address as u64);
        }
    }
}

/// Run a collection once the list has doubled since the last one left it,
/// so the work a collection does is paid for by the cells made before it.
fn collect_when_due<const MODE: u8>(roots: &[TaggedOf<MODE>]) {
    if !listed(MODE) {
        return;
    }
    let due = LISTINGS.with(|listings| {
        let listing = listings[MODE as usize].borrow();
        listing.cells.len() >= listing.collect_at
    });
    if !due {
        return;
    }
    if MODE == CYCLES {
        collect_cycles();
    } else {
        // SAFETY: MODE is TRACED here, so the two types are the same.
        let roots: &[TaggedOf<TRACED>] = unsafe { std::mem::transmute(roots) };
        mark_and_sweep(roots);
    }
    LISTINGS.with(|listings| {
        let mut listing = listings[MODE as usize].borrow_mut();
        listing.collect_at = (listing.cells.len() * 2).max(FIRST_COLLECTION_AT);
        listing.collections += 1;
    });
}

fn integer_result<const MODE: u8>(wide: i128) -> TaggedOf<MODE> {
    match i64::try_from(wide) {
        Ok(number) if (FIXNUM_MIN..=FIXNUM_MAX).contains(&number) => TaggedOf::fixnum(number),
        _ => TaggedOf::on_heap(Body::Integer(wide)),
    }
}

fn arithmetic<const MODE: u8>(
    left: TaggedOf<MODE>,
    right: TaggedOf<MODE>,
    checked: fn(i64, i64) -> Option<i64>,
    wide: fn(i128, i128) -> i128,
    float: fn(f64, f64) -> f64,
) -> TaggedOf<MODE> {
    if left.is_fixnum() && right.is_fixnum() {
        // Two fixnums' result fits in an i64 unless the operation overflows,
        // and a result outside the fixnum range goes to the heap.
        return match checked(left.fixnum_value(), right.fixnum_value()) {
            Some(number) if (FIXNUM_MIN..=FIXNUM_MAX).contains(&number) => TaggedOf::fixnum(number),
            _ => TaggedOf::on_heap(Body::Integer(wide(
                left.fixnum_value() as i128,
                right.fixnum_value() as i128,
            ))),
        };
    }
    if let (Some(left), Some(right)) = (left.float_value(), right.float_value()) {
        return Tagging::<MODE>::float(float(left, right));
    }
    match (left.wide_integer(), right.wide_integer()) {
        (Some(left), Some(right)) => integer_result(wide(left, right)),
        _ => {
            let as_float = |value: &TaggedOf<MODE>| {
                value
                    .float_value()
                    .or_else(|| value.wide_integer().map(|number| number as f64))
            };
            match (as_float(&left), as_float(&right)) {
                (Some(left), Some(right)) => Tagging::<MODE>::float(float(left, right)),
                _ => panic!("no arithmetic for these values"),
            }
        }
    }
}

pub struct Tagging<const MODE: u8>;

/// Counted values, which the comparisons other than memory management use.
pub type TaggedValue = Tagging<COUNTED>;

impl<const MODE: u8> Representation for Tagging<MODE> {
    type Value = TaggedOf<MODE>;

    fn nil() -> Self::Value {
        TaggedOf(NIL)
    }

    fn integer(number: i64) -> Self::Value {
        integer_result(number as i128)
    }

    fn float(number: f64) -> Self::Value {
        let bits = number.to_bits();
        let top = (bits >> 60) & 7;
        if bits != 0x3000_0000_0000_0000 && (top.wrapping_sub(3) & !1) == 0 {
            return TaggedOf((bits.rotate_left(3) & !1) | 2);
        }
        if bits == 0 {
            return TaggedOf(FLONUM_ZERO);
        }
        HEAP_FLOATS.set(HEAP_FLOATS.get() + 1);
        TaggedOf::on_heap(Body::Float(number))
    }

    fn truthy(value: &Self::Value) -> bool {
        value.0 != NIL && value.0 != FALSE
    }

    fn add(left: Self::Value, right: Self::Value) -> Self::Value {
        arithmetic(left, right, i64::checked_add, |a, b| a + b, |a, b| a + b)
    }

    fn subtract(left: Self::Value, right: Self::Value) -> Self::Value {
        arithmetic(left, right, i64::checked_sub, |a, b| a - b, |a, b| a - b)
    }

    fn multiply(left: Self::Value, right: Self::Value) -> Self::Value {
        arithmetic(left, right, i64::checked_mul, |a, b| a * b, |a, b| a * b)
    }

    fn less_than(left: Self::Value, right: Self::Value) -> Self::Value {
        let answer = if left.is_fixnum() && right.is_fixnum() {
            left.fixnum_value() < right.fixnum_value()
        } else {
            match (left.float_value(), right.float_value()) {
                (Some(left), Some(right)) => left < right,
                _ => panic!("no comparison for these values"),
            }
        };
        TaggedOf(if answer { TRUE } else { FALSE })
    }

    fn new_object(count: usize) -> Self::Value {
        let slots = (0..count).map(|_| TaggedOf(NIL)).collect();
        TaggedOf::on_heap(Body::Slots(RefCell::new(slots)))
    }

    fn get_slot(object: &Self::Value, slot: usize) -> Self::Value {
        object.slots().borrow()[slot].clone()
    }

    fn set_slot(object: &Self::Value, slot: usize, value: Self::Value) {
        let replaced = std::mem::replace(&mut object.slots().borrow_mut()[slot], value);
        // Dropped after the borrow ends, since freeing it may reach this
        // object again.
        drop(replaced);
    }

    fn new_array() -> Self::Value {
        TaggedOf::on_heap(Body::Slots(RefCell::new(Vec::new())))
    }

    fn array_push(array: &Self::Value, value: Self::Value) {
        array.slots().borrow_mut().push(value);
    }

    fn array_get(array: &Self::Value, index: &Self::Value) -> Self::Value {
        let index = index.fixnum_value() as usize;
        array
            .slots()
            .borrow()
            .get(index)
            .cloned()
            .unwrap_or(TaggedOf(NIL))
    }

    fn safepoint(stack: &[Self::Value]) {
        collect_when_due::<MODE>(stack);
    }

    fn describe(value: &Self::Value) -> String {
        if let Some(number) = value.wide_integer() {
            return number.to_string();
        }
        if let Some(number) = value.float_value() {
            return format!("{number:?}");
        }
        match value.0 {
            NIL => "nil".to_string(),
            TRUE => "true".to_string(),
            FALSE => "false".to_string(),
            _ => "an object".to_string(),
        }
    }
}
