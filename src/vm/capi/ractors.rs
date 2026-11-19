//! Ractors as C sees them: storage each Ractor holds apart under a key, and
//! the shareability Ruby's `Ractor` methods answer.

use super::calls::{call, top_level_module};
use super::values::{QNIL, Value, to_object, to_value};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The number the next Ractor-local key takes. A key is never 0, which C
/// could take for a key not made.
static NEXT_KEY: AtomicUsize = AtomicUsize::new(1);

/// What a Ractor holds under a key.
enum Stored {
    Value(Object),
    Pointer(usize),
}

thread_local! {
    /// What each Ractor holds under each key, by the Ractor and the key.
    static STORED: RefCell<HashMap<(Value, usize), Stored>> = RefCell::new(HashMap::new());
}

fn current_ractor() -> Value {
    to_value(&call(top_level_module("Ractor"), "current", Vec::new()))
}

fn new_key() -> *mut c_void {
    NEXT_KEY.fetch_add(1, Ordering::SeqCst) as *mut c_void
}

fn store(key: *mut c_void, held: Stored) {
    let ractor = current_ractor();
    STORED.with(|stored| stored.borrow_mut().insert((ractor, key as usize), held));
}

/// Marks the extension as safe to call from any Ractor. Metorex runs the
/// Ractors' threads one at a time, so every extension is.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ext_ractor_safe(_flag: bool) {}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_local_storage_value_newkey() -> *mut c_void {
    new_key()
}

/// A key for a pointer each Ractor holds. The type's functions are not
/// called, since a Ractor's storage lasts as long as the program.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_local_storage_ptr_newkey(_type: *const c_void) -> *mut c_void {
    new_key()
}

/// Whether the current Ractor holds a value under the key, written to
/// `found` when it does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_local_storage_value_lookup(
    key: *mut c_void,
    found: *mut Value,
) -> bool {
    let ractor = current_ractor();
    let held = STORED.with(
        |stored| match stored.borrow().get(&(ractor, key as usize)) {
            Some(Stored::Value(object)) => Some(to_value(object)),
            _ => None,
        },
    );
    match held {
        Some(value) => {
            if !found.is_null() {
                // SAFETY: C hands a place for one VALUE.
                unsafe { *found = value };
            }
            true
        }
        None => false,
    }
}

/// The value the current Ractor holds under the key, or nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_local_storage_value(key: *mut c_void) -> Value {
    let mut found = QNIL;
    rb_ractor_local_storage_value_lookup(key, &mut found);
    found
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_local_storage_value_set(key: *mut c_void, value: Value) {
    store(key, Stored::Value(to_object(value)));
}

/// The pointer the current Ractor holds under the key, or NULL.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_local_storage_ptr(key: *mut c_void) -> *mut c_void {
    let ractor = current_ractor();
    STORED.with(
        |stored| match stored.borrow().get(&(ractor, key as usize)) {
            Some(Stored::Pointer(address)) => *address as *mut c_void,
            _ => std::ptr::null_mut(),
        },
    )
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_local_storage_ptr_set(key: *mut c_void, pointer: *mut c_void) {
    store(key, Stored::Pointer(pointer as usize));
}

/// Whether the main Ractor is the one running.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_main_p() -> bool {
    call(top_level_module("Ractor"), "main?", Vec::new()).is_truthy()
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_shareable_p(object: Value) -> bool {
    call(
        top_level_module("Ractor"),
        "shareable?",
        vec![to_object(object)],
    )
    .is_truthy()
}

/// Freezes the object and everything it refers to, as
/// `Ractor.make_shareable` does, and answers it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ractor_make_shareable(object: Value) -> Value {
    to_value(&call(
        top_level_module("Ractor"),
        "make_shareable",
        vec![to_object(object)],
    ))
}
