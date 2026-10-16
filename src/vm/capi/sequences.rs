//! The instruction sequence and heap walking functions C declares for
//! itself, past the public headers, as the debug gem does. Metorex keeps no
//! `rb_iseq_t` apart from the `RubyVM::InstructionSequence` that wraps it,
//! so the pointer C holds for a sequence is the VALUE of its wrapper. The
//! heap walk hands C one `T_IMEMO` object for each sequence of the Ruby
//! files loaded so far.

use super::calls::call;
use super::values::{Value, to_object, to_value};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;

/// `RUBY_T_IMEMO` in the headers.
pub(super) const T_IMEMO: i32 = 0x1a;
/// MRI's `imemo_iseq`, which C reads from the flags past `FL_USHIFT`.
const IMEMO_ISEQ: Value = 7;
const FL_USHIFT: u32 = 12;

thread_local! {
    /// The `T_IMEMO` objects the running heap walk handed C, each with the
    /// sequence it stands for.
    static WALKED: RefCell<HashMap<Value, Object>> = RefCell::new(HashMap::new());
}

/// Whether `object` is a sequence the heap walk handed C.
pub(super) fn is_walked_sequence(object: Value) -> bool {
    WALKED.with(|walked| walked.borrow().contains_key(&object))
}

/// The flags `RBASIC` starts `object` with: the type and kind of a walked
/// sequence, and none for the rest.
pub(super) fn initial_flags(object: Value) -> Value {
    if is_walked_sequence(object) {
        T_IMEMO as Value | (IMEMO_ISEQ << FL_USHIFT)
    } else {
        0
    }
}

/// The sequence a pointer from `rb_iseqw_to_iseq` stands for.
fn sequence(iseq: *const c_void) -> Object {
    to_object(iseq as Value)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iseqw_to_iseq(iseqw: Value) -> *const c_void {
    iseqw as *const c_void
}

/// The wrapper of a sequence the heap walk handed C.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iseqw_new(iseq: Value) -> Value {
    let walked = WALKED.with(|walked| walked.borrow().get(&iseq).cloned());
    walked.map_or(iseq, |sequence| to_value(&sequence))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iseq_realpath(iseq: *const c_void) -> Value {
    to_value(&call(sequence(iseq), "absolute_path", Vec::new()))
}

/// The kind of code the sequence runs, such as `:method` or `:block`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iseq_type(iseq: *const c_void) -> Value {
    let arguments = vec![Object::symbol("@type")];
    to_value(&call(sequence(iseq), "instance_variable_get", arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iseq_parameters(iseq: *const c_void, is_proc: i32) -> Value {
    let arguments = vec![
        Object::symbol("__parameter_list__"),
        Object::Bool(is_proc != 0),
    ];
    to_value(&call(sequence(iseq), "__send__", arguments))
}

/// Writes where the sequence's code starts and ends through each pointer C
/// passes that is not NULL.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iseq_code_location(
    iseq: *const c_void,
    first_lineno: *mut i32,
    first_column: *mut i32,
    last_lineno: *mut i32,
    last_column: *mut i32,
) {
    let arguments = vec![Object::symbol("code_location")];
    let location = to_value(&call(sequence(iseq), "__send__", arguments));
    let targets = [first_lineno, first_column, last_lineno, last_column];
    for (index, target) in targets.into_iter().enumerate() {
        if target.is_null() {
            continue;
        }
        let number =
            super::numbers::rb_num2int(super::arrays::rb_ary_entry(location, index as i64));
        // SAFETY: C passes either NULL or a pointer to an int it owns.
        unsafe { *target = number as i32 };
    }
}

/// Calls `callback` once for each sequence of the Ruby files loaded so far,
/// over a range holding that one object, until it answers nonzero.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_objspace_each_objects(
    callback: extern "C-unwind" fn(*mut c_void, *mut c_void, usize, *mut c_void) -> i32,
    data: *mut c_void,
) {
    let sequences = super::modules::class_at_path("RubyVM::InstructionSequence");
    let arguments = vec![Object::symbol("__heap__")];
    let loaded = to_value(&call(to_object(sequences), "__send__", arguments));
    let walked: Vec<Value> = (0..super::arrays::rb_rarray_len(loaded))
        .map(|index| {
            let sequence = to_object(super::arrays::rb_ary_entry(loaded, index));
            // A one element Array gives each sequence an object of its own
            // to stand for it.
            let standing = to_value(&Object::array(vec![sequence.clone()]));
            WALKED.with(|walked| walked.borrow_mut().insert(standing, sequence));
            standing
        })
        .collect();
    for standing in &walked {
        let start = *standing as *mut c_void;
        let end = (*standing + 1) as *mut c_void;
        if callback(start, end, 1, data) != 0 {
            break;
        }
    }
    WALKED.with(|held| held.borrow_mut().clear());
}

/// The bytes `ObjectSpace.memsize_of` counts for `object`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_memsize_of(object: Value) -> usize {
    super::calls::kernel_function("require", vec![Object::string("objspace".to_string())]);
    let space = super::calls::top_level_module("ObjectSpace");
    let size = call(space, "memsize_of", vec![to_object(object)]);
    super::numbers::rb_num2ulong(to_value(&size)) as usize
}

/// How many frames the call stack holds, counting the C method running
/// now, as `rb_debug_inspector_frame_depth` counts them.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_debug_inspector_current_depth() -> Value {
    let backtrace = to_object(super::calls::rb_make_backtrace());
    to_value(&call(backtrace, "size", Vec::new()))
}

/// The VALUEs this table names, which stay pinned while it does.
pub(super) fn named_values(found: &mut std::collections::HashSet<Value>) {
    WALKED.with(|held| found.extend(held.borrow().keys().copied()));
}
