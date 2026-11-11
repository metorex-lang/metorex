//! The elements of an Array as C reads and writes them through `RARRAY_PTR`
//! and `RARRAY_LEN`, and Arrays C builds. C is handed a copy of the elements'
//! VALUEs that metorex holds until the same Array is asked for again with
//! different elements. What C writes there is carried back into the Array
//! whenever control leaves C.

use super::calls::{call, class_name_of};
use super::values::{QNIL, Value, objects_from, to_object, to_value};
use super::{called_from, raise};
use crate::object::Object;
use crate::object::{OptionView, View};
use std::cell::RefCell;
use std::rc::Rc;

/// The VALUEs handed to C for one Array, and the VALUEs they were when C
/// was last handed them or last left, which tell a write of C's apart.
#[derive(Default)]
struct Buffer {
    values: Vec<Value>,
    handed: Vec<Value>,
}

thread_local! {
    static BUFFERS: RefCell<crate::fast_hash::AddressMap<Value, Buffer>> = RefCell::new(Default::default());
}

/// Carries what C wrote into each buffer back into its Array.
pub(super) fn carry_element_writes_back() {
    let written: Vec<(Value, Vec<Value>)> = BUFFERS.with(|buffers| {
        buffers
            .borrow_mut()
            .iter_mut()
            .filter(|(_, buffer)| buffer.values != buffer.handed)
            .map(|(array, buffer)| {
                buffer.handed.clone_from(&buffer.values);
                (*array, buffer.values.clone())
            })
            .collect()
    });
    for (array, values) in written {
        if let Some(elements) = held_elements(&to_object(array)) {
            *elements.borrow_mut() = values.into_iter().map(to_object).collect();
        }
    }
}

/// The elements of an Array, or of the one an instance of an Array
/// subclass holds, or None for any other object.
fn held_elements(object: &Object) -> Option<Rc<RefCell<Vec<Object>>>> {
    if let View::Array(elements) = object.view() {
        return Some(elements.clone());
    }
    match crate::vm::native_methods::array_subclass_value(object)
        .as_ref()
        .view()
    {
        Some(View::Array(elements)) => Some(elements.clone()),
        _ => None,
    }
}

/// The VALUEs of the elements of the Array a VALUE C code handed over
/// stands for.
fn element_values(array: Value) -> Vec<Value> {
    with_elements(array, |elements| elements.iter().map(to_value).collect())
}

/// Runs `with` on the elements of the Array a VALUE C code handed over
/// stands for.
fn with_elements<T>(array: Value, with: impl FnOnce(&[Object]) -> T) -> T {
    let object = to_object(array);
    if let Some(elements) = held_elements(&object) {
        return with(&elements.borrow());
    }
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        &format!(
            "wrong argument type {} (expected Array)",
            class_name_of(object)
        ),
        super::called_from(),
    ))
}

/// The Array `object` converts to through `to_ary`, as `rb_check_array_type`
/// finds it, or None for an object that has no `to_ary`.
pub(super) fn array_of(object: Object) -> Option<Vec<Object>> {
    if let Some(elements) = held_elements(&object) {
        return Some(elements.borrow().clone());
    }
    if !super::calls::answers(&object, "to_ary") {
        return None;
    }
    match super::calls::call(object.clone(), "to_ary", Vec::new()).view() {
        View::Array(elements) => Some(elements.borrow().clone()),
        View::Nil => None,
        converted => {
            let class_name = class_name_of(object);
            raise(crate::vm::errors::simple_exception(
                "TypeError",
                &format!(
                    "can't convert {} to Array ({}#to_ary gives {})",
                    class_name,
                    class_name,
                    class_name_of(converted.to_object())
                ),
                called_from(),
            ))
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rarray_ptr(array: Value) -> *mut Value {
    carry_element_writes_back();
    let values = element_values(array);
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let held = buffers.entry(array).or_default();
        if held.handed != values {
            held.values.clone_from(&values);
            held.handed = values;
        }
        held.values.as_mut_ptr()
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rarray_len(array: Value) -> i64 {
    carry_element_writes_back();
    with_elements(array, |elements| elements.len() as i64)
}

/// The element at `offset`, counted from the end when negative, or nil past
/// either end.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_entry(array: Value, offset: i64) -> Value {
    let element = with_elements(array, |elements| {
        let index = if offset < 0 {
            offset + elements.len() as i64
        } else {
            offset
        };
        usize::try_from(index)
            .ok()
            .and_then(|index| elements.get(index).cloned())
    });
    element.map_or(QNIL, |element| to_value(&element))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_new_from_values(count: i64, values: *const Value) -> Value {
    to_value(&Object::array(objects_from(count, values)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_new() -> Value {
    to_value(&Object::array(Vec::new()))
}

/// The elements of `array` when it is an Array that is not frozen, which C
/// changes in place without a method call, as MRI's functions do. C's own
/// writes to the elements are carried in first.
fn changeable_elements(array: Value) -> Option<Rc<RefCell<Vec<Object>>>> {
    carry_element_writes_back();
    let object = to_object(array);
    let View::Array(elements) = object.view() else {
        return None;
    };
    (!super::running_interpreter().object_is_frozen(&object)).then(|| elements.clone())
}

/// Appends `element`, refusing a frozen Array as `Array#push` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_push(array: Value, element: Value) -> Value {
    match changeable_elements(array) {
        Some(elements) => elements.borrow_mut().push(to_object(element)),
        None => {
            super::calls::call(to_object(array), "push", vec![to_object(element)]);
        }
    }
    array
}

/// Removes and answers the last element, or nil for an empty Array,
/// refusing a frozen Array as `Array#pop` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_pop(array: Value) -> Value {
    match changeable_elements(array) {
        Some(elements) => {
            let popped = elements.borrow_mut().pop();
            popped.map_or(QNIL, |element| to_value(&element))
        }
        None => to_value(&super::calls::call(to_object(array), "pop", Vec::new())),
    }
}

/// Stores `element` at `index`, counted from the end when negative, as
/// `Array#[]=` does with one index, filling with nil up to an index past
/// the end.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_store(array: Value, index: i64, element: Value) {
    if let Some(elements) = changeable_elements(array) {
        let mut elements = elements.borrow_mut();
        let at = if index < 0 {
            index + elements.len() as i64
        } else {
            index
        };
        if let Ok(at) = usize::try_from(at) {
            if at >= elements.len() {
                elements.resize(at + 1, Object::Nil);
            }
            elements[at] = to_object(element);
            return;
        }
    }
    let arguments = vec![Object::Int(index), to_object(element)];
    super::calls::call(to_object(array), "[]=", arguments);
}

/// Makes the Array `length` elements long, filling what it grows by with
/// nil and dropping what it shrinks by, refusing a frozen Array either way.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_resize(array: Value, length: i64) -> Value {
    let held = rb_rarray_len(array);
    if length > held {
        let arguments = vec![Object::Int(length - 1), Object::Nil];
        call(to_object(array), "[]=", arguments);
    } else {
        let arguments = vec![Object::Int(length), Object::Int(held - length)];
        call(to_object(array), "slice!", arguments);
    }
    array
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_dup(array: Value) -> Value {
    to_value(&Object::array(
        element_values(array).into_iter().map(to_object).collect(),
    ))
}

/// An empty Array, refusing a negative capacity. Metorex sizes an Array as
/// it grows, so the capacity is not otherwise used.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_new_capa(capacity: i64) -> Value {
    if capacity < 0 {
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            "negative array size (or size too big)",
            called_from(),
        ));
    }
    rb_ary_new()
}

/// Puts `element` at the front, refusing a frozen Array as `Array#unshift`
/// does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_unshift(array: Value, element: Value) -> Value {
    super::calls::call(to_object(array), "unshift", vec![to_object(element)]);
    array
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_hash_dup(hash: Value) -> Value {
    to_value(&super::calls::call(to_object(hash), "dup", Vec::new()))
}

/// Answers `array` after calling `name` on it, for the C functions that
/// change an Array in place and answer it.
fn changed(array: Value, name: &str, arguments: Vec<Object>) -> Value {
    call(to_object(array), name, arguments);
    array
}

/// Answers what `name` called on `array` answers.
fn answered(array: Value, name: &str, arguments: Vec<Object>) -> Value {
    to_value(&call(to_object(array), name, arguments))
}

/// The Array `Kernel#Array` converts `object` to.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_Array(object: Value) -> Value {
    let kernel = super::calls::top_level_module("Kernel");
    to_value(&call(kernel, "Array", vec![to_object(object)]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_aref(count: i32, values: *const Value, array: Value) -> Value {
    answered(array, "[]", objects_from(i64::from(count), values))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_cat(array: Value, values: *const Value, count: i64) -> Value {
    changed(array, "push", objects_from(count, values))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_clear(array: Value) -> Value {
    changed(array, "clear", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_concat(array: Value, other: Value) -> Value {
    changed(array, "concat", vec![to_object(other)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_delete(array: Value, element: Value) -> Value {
    answered(array, "delete", vec![to_object(element)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_delete_at(array: Value, index: i64) -> Value {
    answered(array, "delete_at", vec![Object::Int(index)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_freeze(array: Value) -> Value {
    answered(array, "freeze", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_includes(array: Value, element: Value) -> Value {
    answered(array, "include?", vec![to_object(element)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_join(array: Value, separator: Value) -> Value {
    answered(array, "join", vec![to_object(separator)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_plus(array: Value, other: Value) -> Value {
    answered(array, "+", vec![to_object(other)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_reverse(array: Value) -> Value {
    changed(array, "reverse!", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_rotate(array: Value, count: i64) -> Value {
    changed(array, "rotate!", vec![Object::Int(count)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_shift(array: Value) -> Value {
    answered(array, "shift", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_sort(array: Value) -> Value {
    answered(array, "sort", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_sort_bang(array: Value) -> Value {
    changed(array, "sort!", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_to_s(array: Value) -> Value {
    answered(array, "to_s", Vec::new())
}

/// The `length` elements from `start`, fewer when the Array ends first, or
/// nil when `start` is past the end or either number is negative.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_subseq(array: Value, start: i64, length: i64) -> Value {
    let values = element_values(array);
    if start < 0 || length < 0 || start as usize > values.len() {
        return QNIL;
    }
    let start = start as usize;
    let end = values.len().min(start.saturating_add(length as usize));
    to_value(&Object::array(
        values[start..end].iter().copied().map(to_object).collect(),
    ))
}

/// `object` itself when it is an Array, the Array its `to_ary` answers, or
/// else a new Array holding it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ary_to_ary(object: Value) -> Value {
    let held = to_object(object);
    if held_elements(&held).is_some() {
        return object;
    }
    let elements = array_of(held.clone()).unwrap_or_else(|| vec![held]);
    to_value(&Object::array(elements))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_assoc_new(first: Value, second: Value) -> Value {
    to_value(&Object::array(vec![to_object(first), to_object(second)]))
}

/// Sets each of the `count` VALUEs at `values` to nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mem_clear(values: *mut Value, count: i64) {
    for index in 0..count.max(0) as usize {
        // SAFETY: C hands a pointer to at least `count` VALUEs.
        unsafe { *values.add(index) = QNIL };
    }
}

/// The VALUEs this table names, which stay pinned while it does.
pub(super) fn named_values(found: &mut std::collections::HashSet<Value>) {
    BUFFERS.with(|held| found.extend(held.borrow().keys().copied()));
}
