//! The cost of turning a value into the VALUE a C extension holds and back.
//!
//! `HandleTable` is the scheme in `src/vm/capi/handles.rs`: an object held by
//! reference is looked up by its address in a hash map, and given the next
//! slot of a table that keeps it alive when it has none, and a VALUE turns
//! back into an object by indexing that table. With a tagged value, the
//! VALUE is the value's own bits.

use crate::enum_value::Object;
use crate::machine::Representation;
use crate::tagged_value::{Tagged, TaggedValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const FIRST_HANDLE: usize = 0x40;
const HANDLE_STRIDE: usize = 8;

#[derive(Default)]
struct HandleTable {
    objects: Vec<Object>,
    by_identity: HashMap<usize, usize>,
}

thread_local! {
    static HANDLES: RefCell<HandleTable> = RefCell::new(HandleTable::default());
}

fn to_value(object: &Object) -> usize {
    let Object::Array(held) = object else {
        panic!("the comparison hands C arrays");
    };
    let address = Rc::as_ptr(held) as *const () as usize;
    HANDLES.with(|table| {
        let mut table = table.borrow_mut();
        if let Some(known) = table.by_identity.get(&address) {
            return *known;
        }
        let value = FIRST_HANDLE + table.objects.len() * HANDLE_STRIDE;
        table.objects.push(object.clone());
        table.by_identity.insert(address, value);
        value
    })
}

fn to_object(value: usize) -> Object {
    HANDLES.with(|table| {
        let index = (value - FIRST_HANDLE) / HANDLE_STRIDE;
        table.borrow().objects[index].clone()
    })
}

/// Hand each of `objects` arrays to C and back `rounds` times, as a C
/// extension called in a loop sees its arguments, through the handle table.
/// Answers how many came back.
pub fn through_handles(objects: usize, rounds: usize) -> usize {
    let held: Vec<Object> = (0..objects)
        .map(|_| Object::Array(Rc::new(RefCell::new(Vec::new()))))
        .collect();
    let mut returned = 0;
    for _ in 0..rounds {
        for object in &held {
            let value = std::hint::black_box(to_value(object));
            let back = to_object(value);
            returned += matches!(back, Object::Array(_)) as usize;
        }
    }
    returned
}

/// The same with tagged values, where the VALUE is the value's bits and
/// turning it back is taking a reference.
pub fn through_tagged(objects: usize, rounds: usize) -> usize {
    let held: Vec<Tagged> = (0..objects).map(|_| TaggedValue::new_array()).collect();
    let mut returned = 0;
    for _ in 0..rounds {
        for object in &held {
            let value = std::hint::black_box(object.bits());
            let back = Tagged::from_bits_taking_reference(value);
            returned += (back.bits() == value) as usize;
        }
    }
    returned
}
