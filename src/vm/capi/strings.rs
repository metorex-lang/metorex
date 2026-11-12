//! The bytes of a String as C reads and writes them through `RSTRING_PTR`
//! and `RSTRING_LEN`. Metorex keeps a string's characters as Rust text, so C
//! is handed a buffer holding a copy of its bytes, ended with as many NUL
//! bytes as one code unit of its encoding takes, and with room beyond that
//! C can grow into. What C writes there is carried back into the string
//! whenever control leaves C, what Ruby code changes is carried into the
//! buffer when control comes back to C, and the buffer stays where it is
//! until the string outgrows it.

use super::calls::{answers, call, class_name_of};
use super::values::{Value, to_object, to_value};
use super::{interpreter, raise};
use crate::object::{Object, StringValue};
use crate::object::{OptionView, View};
use crate::vm::native_methods::string_methods::{
    binary_bytes, bytes_are_characters, wide_encoding,
};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ffi::c_char;
use std::rc::Rc;

/// The memory handed to C for one string: its bytes, the terminator, and
/// the room beyond, which keeps what C wrote there until the string grows
/// over it. `handed` holds the string's bytes as they stood when C was last
/// handed the memory or last left, which tells a write of C's apart. A
/// string made over static C memory hands that memory back instead while
/// its bytes are unchanged.
struct Buffer {
    area: Vec<u8>,
    handed: Vec<u8>,
    external: Option<*mut c_char>,
}

/// How many NUL bytes end a string in `encoding`: one code unit of it.
fn terminator_length(string: &StringValue) -> usize {
    wide_encoding(string.encoding_name()).map_or(1, |shape| shape.unit())
}

impl Buffer {
    fn holding(current: Vec<u8>, terminator: usize) -> Buffer {
        let mut area = current.clone();
        area.resize(current.len() + terminator, 0);
        Buffer {
            area,
            handed: current,
            external: None,
        }
    }

    /// Puts `current`, the bytes the string holds now, in the memory,
    /// growing it when it is too small and ending it with the terminator.
    fn take(&mut self, current: Vec<u8>, terminator: usize) {
        let needed = current.len() + terminator;
        if self.area.len() < needed {
            self.area.resize(needed, 0);
        }
        self.area[..current.len()].copy_from_slice(&current);
        self.area[current.len()..needed].fill(0);
        self.handed = current;
        self.external = None;
    }

    /// How many bytes the memory holds beyond the terminator's room.
    fn capacity(&self, terminator: usize) -> usize {
        self.area.len() - terminator
    }
}

thread_local! {
    static BUFFERS: RefCell<crate::fast_hash::AddressMap<Value, Buffer>> = RefCell::new(Default::default());
    /// The strings whose buffers C was handed since control last crossed
    /// from C into Ruby, which are the ones C has written into since then
    /// unless it kept a pointer from before.
    static HANDED_SINCE_CROSSING: RefCell<HashSet<Value>> = RefCell::new(HashSet::new());
    /// For each call into C running, the strings whose buffers it was
    /// handed, which are compared with their strings when it returns.
    static HANDED_IN_FRAME: RefCell<Vec<HashSet<Value>>> = const { RefCell::new(Vec::new()) };
    /// How many changes to strings the buffers had seen when they were last
    /// brought up to date with their strings.
    static CHANGES_SEEN: Cell<u64> = const { Cell::new(0) };
}

/// Let go of the buffer of a string C no longer reaches.
pub(super) fn forget(string: Value) {
    BUFFERS.with(|held| held.borrow_mut().remove(&string));
    HANDED_SINCE_CROSSING.with(|held| held.borrow_mut().remove(&string));
    HANDED_IN_FRAME.with(|held| {
        for frame in held.borrow_mut().iter_mut() {
            frame.remove(&string);
        }
    });
}

/// Start counting the buffers a call into C is handed.
pub(super) fn enter_frame() {
    HANDED_IN_FRAME.with(|held| held.borrow_mut().push(HashSet::new()));
}

/// Carry back what C wrote into the buffers the call into C returning now
/// was handed, and into those handed since the last crossing into Ruby.
pub(super) fn leave_frame() {
    carry_writes_back();
    let handed = HANDED_IN_FRAME.with(|held| held.borrow_mut().pop().unwrap_or_default());
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        for value in handed {
            if let Some(buffer) = buffers.get_mut(&value) {
                carry_write_back(value, buffer);
            }
        }
    });
}

/// The String a VALUE stands for, or the one an instance of a String
/// subclass holds, or None for any other object.
fn held_string(object: &Object) -> Option<Rc<StringValue>> {
    if let View::String(held) = object.view() {
        return Some(held.clone());
    }
    match crate::vm::native_methods::string_subclass_value(object)
        .as_ref()
        .view()
    {
        Some(View::String(held)) => Some(held.clone()),
        _ => None,
    }
}

/// The String a VALUE C code handed over stands for.
pub(super) fn string_from(value: Value) -> Rc<StringValue> {
    let object = to_object(value);
    if let Some(held) = held_string(&object) {
        return held;
    }
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        &format!(
            "wrong argument type {} (expected String)",
            class_name_of(object)
        ),
        super::called_from(),
    ))
}

/// The String `object` converts to through `to_str`, as `StringValue` does.
pub(super) fn string_value(object: Object) -> Object {
    if held_string(&object).is_some() {
        return object;
    }
    if !answers(&object, "to_str") {
        raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!(
                "no implicit conversion of {} into String",
                interpreter().conversion_name(&object)
            ),
            super::called_from(),
        ));
    }
    let converted = call(object.clone(), "to_str", Vec::new());
    if let View::String(_) = converted.view() {
        return converted;
    }
    let class_name = class_name_of(object);
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        &format!(
            "can't convert {} to String ({}#to_str gives {})",
            class_name,
            class_name,
            class_name_of(converted)
        ),
        super::called_from(),
    ))
}

/// `object` as a String when it is one or converts through `to_str`, as
/// `rb_check_string_type` finds it, or None when it has no `to_str`.
pub(super) fn check_string(object: Object) -> Option<Object> {
    if held_string(&object).is_some() || answers(&object, "to_str") {
        return Some(string_value(object));
    }
    None
}

/// Puts `bytes` in `string`, read the way the string's encoding reads them.
/// Bytes that spell no text are held one character per byte.
fn store_bytes(string: &StringValue, bytes: &[u8]) {
    let as_characters = || bytes.iter().map(|byte| *byte as char).collect::<String>();
    if bytes_are_characters(string) {
        string.replace_text(as_characters());
        return;
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => string.replace_text(text),
        Err(_) => {
            string.replace_text(as_characters());
            string.mark_bytes();
        }
    }
}

/// Carries what C wrote into the buffers it was handed since control last
/// crossed into Ruby back into their strings.
pub(super) fn carry_writes_back() {
    let handed: Vec<Value> = HANDED_SINCE_CROSSING.with(|held| held.borrow_mut().drain().collect());
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        for value in handed {
            if let Some(buffer) = buffers.get_mut(&value) {
                carry_write_back(value, buffer);
            }
        }
    });
}

/// Carries what C wrote into the buffer of `string` back into it, leaving
/// the other buffers for the next crossing into Ruby.
fn carry_own_writes_back(string: Value) {
    BUFFERS.with(|buffers| {
        if let Some(buffer) = buffers.borrow_mut().get_mut(&string) {
            carry_write_back(string, buffer);
        }
    });
}

fn carry_write_back(value: Value, buffer: &mut Buffer) {
    let length = buffer.handed.len();
    if buffer.external.is_some() || buffer.area[..length] == buffer.handed[..] {
        return;
    }
    if let Some(string) = held_string(&to_object(value)) {
        store_bytes_in_sync(&string, &buffer.area[..length]);
    }
    buffer.handed.copy_from_slice(&buffer.area[..length]);
}

/// Puts what C wrote in `string`, a change its buffer already holds.
fn store_bytes_in_sync(string: &StringValue, bytes: &[u8]) {
    let before = crate::object::string_value::string_changes_made();
    store_bytes(string, bytes);
    if CHANGES_SEEN.with(Cell::get) == before {
        CHANGES_SEEN.with(|seen| seen.set(crate::object::string_value::string_changes_made()));
    }
}

/// Carries what Ruby code changed in each string into its buffer, once it
/// has run and control is back with C. When no string changed while it ran
/// there is nothing to carry.
pub(super) fn carry_changes_in() {
    let changes = crate::object::string_value::string_changes_made();
    if CHANGES_SEEN.with(Cell::get) == changes {
        return;
    }
    CHANGES_SEEN.with(|seen| seen.set(changes));
    BUFFERS.with(|buffers| {
        for (value, buffer) in buffers.borrow_mut().iter_mut() {
            if let Some(string) = held_string(&to_object(*value))
                && *string.bytes() != buffer.handed[..]
            {
                buffer.take(binary_bytes(&string), terminator_length(&string));
            }
        }
    });
}

/// Runs `with` on the buffer of `string`, making one holding its bytes as
/// they stand when it has none.
fn with_buffer<T>(string: Value, with: impl FnOnce(&mut Buffer, &StringValue) -> T) -> T {
    carry_own_writes_back(string);
    let held = string_from(string);
    let current = binary_bytes(&held);
    let terminator = terminator_length(&held);
    HANDED_SINCE_CROSSING.with(|held| held.borrow_mut().insert(string));
    HANDED_IN_FRAME.with(|held| {
        if let Some(frame) = held.borrow_mut().last_mut() {
            frame.insert(string);
        }
    });
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers
            .entry(string)
            .or_insert_with(|| Buffer::holding(current.clone(), terminator));
        if buffer.handed != current {
            buffer.take(current, terminator);
        }
        with(buffer, &held)
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rstring_ptr(string: Value) -> *mut c_char {
    with_buffer(string, |buffer, _| {
        buffer
            .external
            .unwrap_or(buffer.area.as_mut_ptr().cast::<c_char>())
    })
}

/// How many bytes the string can hold before its buffer has to grow.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_capacity(string: Value) -> usize {
    with_buffer(string, |buffer, held| {
        buffer.capacity(terminator_length(held))
    })
}

/// Makes room for `expand` more bytes after the string's own, leaving its
/// bytes as they are.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_modify_expand(string: Value, expand: i64) {
    rb_str_modify(string);
    if expand < 0 {
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            "negative expanding string size",
            super::called_from(),
        ));
    }
    with_buffer(string, |buffer, held| {
        let needed = buffer.handed.len() + expand as usize + terminator_length(held);
        if buffer.area.len() < needed {
            buffer.area.resize(needed, 0);
        }
        buffer.external = None;
    });
}

/// Refuses a change to a frozen string or one locked for C.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_modify(string: Value) {
    super::objects::rb_check_frozen(string);
    if string_from(string).is_borrowed() {
        raise(crate::vm::errors::simple_exception(
            "RuntimeError",
            "can't modify string; temporarily locked",
            super::called_from(),
        ));
    }
}

/// Makes the string the first `length` bytes of its buffer, which may hold
/// bytes C wrote past the end it had, and ends it with the terminator.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_set_len(string: Value, length: i64) {
    rb_str_modify(string);
    with_buffer(string, |buffer, held| {
        let terminator = terminator_length(held);
        let capacity = buffer.capacity(terminator) as i64;
        if length > capacity || length < 0 {
            raise(crate::vm::errors::simple_exception(
                "ArgumentError",
                &format!("probable buffer overflow: {} for {}", length, capacity),
                super::called_from(),
            ));
        }
        let length = length as usize;
        buffer.area[length..length + terminator].fill(0);
        let kept = buffer.area[..length].to_vec();
        store_bytes_in_sync(held, &kept);
        buffer.handed = kept;
        buffer.external = None;
    });
}

/// Makes the string `length` bytes long, growing its buffer first when it
/// is too small, and keeping the bytes the buffer holds.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_resize(string: Value, length: i64) -> Value {
    if length < 0 {
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            "negative string size (or size too big)",
            super::called_from(),
        ));
    }
    let capacity = rb_str_capacity(string) as i64;
    if length > capacity {
        rb_str_modify_expand(
            string,
            length - binary_bytes(&string_from(string)).len() as i64,
        );
    }
    rb_str_set_len(string, length);
    string
}

/// A new String over the `length` bytes of static C memory at `text`,
/// whose `RSTRING_PTR` is that memory while its bytes are unchanged.
pub(super) fn static_string(text: *const c_char, length: i64, encoding: &str) -> Value {
    let made = rb_str_new(text.cast::<u8>(), length);
    let held = string_from(made);
    held.set_encoding(encoding.to_string());
    let bytes = binary_bytes(&held);
    held.clear_bytes();
    store_bytes_in_sync(&held, &bytes);
    with_buffer(made, |buffer, _| {
        buffer.external = Some(text as *mut c_char)
    });
    made
}

/// A new String of `capacity` bytes' room and none of its own, tagged
/// ASCII-8BIT.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_buf_new(capacity: i64) -> Value {
    let made = rb_str_new(std::ptr::null(), 0);
    with_buffer(made, |buffer, _| {
        buffer.area.resize(capacity.max(0) as usize + 1, 0);
    });
    made
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_rstring_len(string: Value) -> i64 {
    carry_own_writes_back(string);
    binary_bytes(&string_from(string)).len() as i64
}

/// Converts what `pointer` holds to a String through `to_str` and stores it
/// back, as `StringValue` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_string_value(pointer: *mut Value) -> Value {
    // SAFETY: `StringValue` hands over the address of a VALUE it can write.
    let held = unsafe { *pointer };
    let converted = to_value(&string_value(to_object(held)));
    // SAFETY: as above.
    unsafe { *pointer = converted };
    converted
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_string_value_ptr(pointer: *mut Value) -> *mut c_char {
    rb_rstring_ptr(rb_string_value(pointer))
}

/// The string's bytes for C to read as a NUL-terminated string, refusing one
/// that holds a NUL of its own. An encoding that spells a character in more
/// than one byte looks for a whole code unit of zeros.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_string_value_cstr(pointer: *mut Value) -> *mut c_char {
    let string = rb_string_value(pointer);
    let held = string_from(string);
    let bytes = binary_bytes(&held);
    let unit = wide_encoding(held.encoding_name()).map_or(1, |shape| shape.unit());
    if bytes
        .chunks(unit)
        .any(|chunk| chunk.iter().all(|byte| *byte == 0))
    {
        let message = if unit > 1 {
            "string contains null char"
        } else {
            "string contains null byte"
        };
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            message,
            super::called_from(),
        ));
    }
    rb_rstring_ptr(string)
}

/// A String holding the `length` bytes at `text`, tagged ASCII-8BIT as
/// MRI's `rb_str_new` makes it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_new(text: *const u8, length: i64) -> Value {
    let length = usize::try_from(length).unwrap_or(0);
    // With no text, the String holds `length` zero bytes for C to write
    // through `RSTRING_PTR`.
    let characters = if text.is_null() {
        "\0".repeat(length)
    } else {
        // SAFETY: C hands over `length` bytes at `text`.
        let bytes = unsafe { std::slice::from_raw_parts(text, length) };
        bytes.iter().map(|byte| *byte as char).collect::<String>()
    };
    to_value(&Object::String(Rc::new(StringValue::from_bytes(
        characters,
    ))))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_new_cstr(text: *const c_char) -> Value {
    // SAFETY: C hands over a NUL-terminated string.
    let length = unsafe { std::ffi::CStr::from_ptr(text) }.to_bytes().len();
    rb_str_new(text.cast::<u8>(), length as i64)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_append(string: Value, added: Value) -> Value {
    let added = string_value(to_object(added));
    call(to_object(string), "<<", vec![added]);
    string
}

/// Appends `added` to `string` as `String#<<` does, where an Integer
/// stands for the codepoint it names.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_concat(string: Value, added: Value) -> Value {
    call(to_object(string), "<<", vec![to_object(added)]);
    string
}

/// Gives `destination` the bytes and encoding of `source`, as
/// `String#replace` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_str_replace(destination: Value, source: Value) -> Value {
    let source = string_value(to_object(source));
    call(to_object(destination), "replace", vec![source]);
    destination
}

/// Retags a formatted string with the encoding of a string written into it,
/// when that one is ASCII-compatible and not binary, as MRI's formatting
/// does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_adopt_encoding(string: Value, source: Value) {
    let made = string_from(string);
    let named = string_from(source).encoding_name();
    if matches!(named, "ASCII-8BIT" | "BINARY") || wide_encoding(named).is_some() {
        return;
    }
    let bytes = binary_bytes(&made);
    made.set_encoding(named);
    made.clear_bytes();
    store_bytes(&made, &bytes);
}
