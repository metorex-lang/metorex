//! The VALUE a C extension holds for each object, which is the object's
//! bits. A value held inline, such as a Fixnum, a flonum, a static Symbol,
//! nil, true, or false, is nothing more. An object on the heap is pinned the
//! first time it is handed over: the pinned objects keep a copy of the value,
//! which keeps the object alive, under the address that tells it apart from
//! every other object. An object handed to C twice gets the same VALUE both
//! times, so C code comparing two VALUEs compares the objects.

use crate::object::Object;
use crate::object::View;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub(crate) type Value = usize;

pub(crate) const QFALSE: Value = 0x00;
pub(crate) const QNIL: Value = 0x08;
pub(crate) const QTRUE: Value = 0x14;
pub(crate) const QUNDEF: Value = 0x24;

#[derive(Default)]
struct Pinned {
    /// Each pinned object's value, by its VALUE.
    by_value: HashMap<Value, Object>,
    /// Each pinned object's VALUE, by the address that tells it apart.
    by_identity: HashMap<usize, Value>,
}

thread_local! {
    static PINNED: RefCell<Pinned> = RefCell::new(Pinned::default());
}

/// The address an object on the heap lives at, which is what tells it apart
/// from every other object of its kind. A value whose cell is all there is
/// to it is told apart by the cell.
fn identity(object: &Object) -> usize {
    match object.view() {
        View::BigInt(held) => Rc::as_ptr(held) as *const () as usize,
        View::String(held) => Rc::as_ptr(held) as *const () as usize,
        View::Array(held) => Rc::as_ptr(held) as *const () as usize,
        View::Dict(held) => Rc::as_ptr(held) as *const () as usize,
        View::Instance(held) => Rc::as_ptr(held) as *const () as usize,
        View::Class(held) | View::Module(held) => Rc::as_ptr(held) as *const () as usize,
        View::Method(held) => Rc::as_ptr(held) as *const () as usize,
        View::Block(held) => Rc::as_ptr(held) as *const () as usize,
        View::Exception(held) => Rc::as_ptr(held) as *const () as usize,
        View::Set(held) => Rc::as_ptr(held) as *const () as usize,
        View::Binding(held) => Rc::as_ptr(held) as *const () as usize,
        View::Regex(pattern, _) => Rc::as_ptr(pattern) as *const () as usize,
        View::Range { mark, .. } => Rc::as_ptr(mark) as usize,
        _ => object.bits() as usize,
    }
}

/// The VALUE C code is handed for `object`, pinning the object when it is
/// on the heap.
pub(crate) fn to_value(object: &Object) -> Value {
    if !object.is_pointer() {
        return object.bits() as Value;
    }
    let key = identity(object);
    PINNED.with(|pinned| {
        let mut pinned = pinned.borrow_mut();
        if let Some(value) = pinned.by_identity.get(&key) {
            return *value;
        }
        let value = object.bits() as Value;
        pinned.by_identity.insert(key, value);
        pinned.by_value.insert(value, object.clone());
        value
    })
}

/// The object a VALUE C code handed back stands for. A VALUE that points at
/// no object pinned now reads as nil.
pub(crate) fn to_object(value: Value) -> Object {
    if value == QUNDEF {
        return Object::Nil;
    }
    if let Some(inline) = Object::inline_from_bits(value as u64) {
        return inline;
    }
    PINNED
        .with(|pinned| pinned.borrow().by_value.get(&value).cloned())
        .unwrap_or(Object::Nil)
}

/// How many places hold the object behind `value`, the pinned copy among
/// them.
pub(crate) fn references_to(value: Value) -> usize {
    PINNED.with(|pinned| {
        pinned
            .borrow()
            .by_value
            .get(&value)
            .map_or(0, Object::references)
    })
}

/// The objects the `count` VALUEs C handed over at `values` stand for.
pub(crate) fn objects_from(count: i64, values: *const Value) -> Vec<Object> {
    if count <= 0 {
        return Vec::new();
    }
    // SAFETY: C hands over `count` VALUEs at `values`.
    unsafe { std::slice::from_raw_parts(values, count as usize) }
        .iter()
        .map(|held| to_object(*held))
        .collect()
}

/// Let go of the object behind `value`, which C no longer reaches.
pub(crate) fn unpin(value: Value) {
    let released = PINNED.with(|pinned| {
        let mut pinned = pinned.borrow_mut();
        let released = pinned.by_value.remove(&value)?;
        let key = identity(&released);
        pinned.by_identity.remove(&key);
        Some(released)
    });
    drop(released);
}

/// Let go of every pinned object `kept` does not name, answering how many
/// there were.
pub(crate) fn unpin_all_but(kept: &HashSet<Value>) -> usize {
    let released: Vec<Object> = PINNED.with(|pinned| {
        let mut pinned = pinned.borrow_mut();
        let unreached: Vec<Value> = pinned
            .by_value
            .keys()
            .filter(|value| !kept.contains(value))
            .copied()
            .collect();
        unreached
            .into_iter()
            .filter_map(|value| {
                let released = pinned.by_value.remove(&value)?;
                let key = identity(&released);
                pinned.by_identity.remove(&key);
                Some(released)
            })
            .collect()
    });
    let count = released.len();
    drop(released);
    count
}
