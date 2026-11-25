//! The `struct RBasic` C reads an object's flags and class through with
//! `RBASIC`. Metorex keeps neither in a struct of that shape, so each object
//! C asks about is given one, filled from the object when C asks for it.
//! What C writes into its flags is kept, and the FREEZE bit of it is carried
//! back into the object whenever control leaves C, as is a Hash's
//! PROC_DEFAULT bit, which says its default is a Proc to call.

use super::calls::call;
use super::exports::rb_class_of;
use super::values::{Value, to_object};
use super::{interpreter, running_interpreter};
use crate::object::{Object, View};
use std::cell::RefCell;
use std::collections::HashMap;

/// `RUBY_FL_FREEZE` in the headers.
const FREEZE_FLAG: Value = 1 << 11;
/// `RHASH_PROC_DEFAULT`, `RUBY_FL_USER2`, on a Hash.
const PROC_DEFAULT_FLAG: Value = 1 << 14;

#[repr(C)]
pub(super) struct RBasic {
    flags: Value,
    klass: Value,
}

/// The struct handed to C for one object, and the flags it held when C was
/// last handed it, which tell a write of C's apart.
struct Handed {
    basic: Box<RBasic>,
    flags: Value,
}

thread_local! {
    static HANDED: RefCell<HashMap<Value, Handed>> = RefCell::new(HashMap::new());
}

/// Freezes or thaws each object whose FREEZE flag C changed.
pub(super) fn carry_flag_writes_back() {
    let changed: Vec<(Value, Value, Value)> = HANDED.with(|handed| {
        handed
            .borrow_mut()
            .iter_mut()
            .filter(|(_, held)| held.basic.flags != held.flags)
            .map(|(value, held)| {
                let before = held.flags;
                held.flags = held.basic.flags;
                (*value, before, held.flags)
            })
            .collect()
    });
    for (value, before, after) in changed {
        let object = to_object(value);
        if (before ^ after) & PROC_DEFAULT_FLAG != 0 {
            carry_proc_default(&object, after & PROC_DEFAULT_FLAG != 0);
        }
        let frozen = after & FREEZE_FLAG != 0;
        let machine = running_interpreter();
        if frozen == machine.object_is_frozen(&object) {
            continue;
        }
        if frozen {
            machine.freeze_object(&object);
        } else {
            machine.thaw_object(&object);
        }
    }
}

/// Makes a Hash's default its default Proc when C set PROC_DEFAULT, or its
/// default Proc its plain default when C cleared it.
fn carry_proc_default(object: &Object, proc_default: bool) {
    if !matches!(object.view(), View::Dict(_)) {
        return;
    }
    if proc_default {
        let default = call(object.clone(), "default", Vec::new());
        call(object.clone(), "default_proc=", vec![default]);
    } else {
        let default = call(object.clone(), "default_proc", Vec::new());
        call(object.clone(), "default=", vec![default]);
    }
}

/// The flags a Hash with a default Proc is handed to C with.
fn hash_flags(object: Value) -> Value {
    let hash = to_object(object);
    if matches!(hash.view(), View::Dict(_)) && !call(hash, "default_proc", Vec::new()).is_nil() {
        PROC_DEFAULT_FLAG
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_rbasic(object: Value) -> *mut RBasic {
    let machine = interpreter();
    let frozen = machine.object_is_frozen(&to_object(object));
    let klass = rb_class_of(object);
    let is_hash = matches!(to_object(object).view(), View::Dict(_));
    let proc_default = hash_flags(object);
    HANDED.with(|handed| {
        let mut handed = handed.borrow_mut();
        let held = handed.entry(object).or_insert_with(|| Handed {
            basic: Box::new(RBasic {
                flags: super::sequences::initial_flags(object),
                klass,
            }),
            flags: 0,
        });
        let refreshed = FREEZE_FLAG | if is_hash { PROC_DEFAULT_FLAG } else { 0 };
        let kept = held.basic.flags & !refreshed;
        held.basic.flags = kept | proc_default | if frozen { FREEZE_FLAG } else { 0 };
        held.basic.klass = klass;
        held.flags = held.basic.flags;
        &mut *held.basic as *mut RBasic
    })
}

/// The VALUEs this table names, which stay pinned while it does.
pub(super) fn named_values(found: &mut std::collections::HashSet<Value>) {
    HANDED.with(|held| found.extend(held.borrow().keys().copied()));
}
