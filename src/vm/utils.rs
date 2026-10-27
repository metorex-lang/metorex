//! Utility functions for the Metorex virtual machine.
//!
//! This module provides common helper functions used throughout the VM implementation.

use crate::error::SourceLocation;
use crate::lexer::Position;
use crate::object::View;
use crate::object::{Object, Slot};

/// Convert a lexer position into a runtime source location.
pub(super) fn position_to_location(position: Position) -> SourceLocation {
    SourceLocation::from(position)
}

/// Format an exception object for display.
pub(super) fn format_exception(exception: &Object) -> String {
    match exception.view() {
        View::Exception(ex) => {
            let exc = ex.borrow();
            // Ruby reports a RuntimeError with an empty message, which a bare
            // `raise` produces, as "unhandled exception".
            if exc.message.is_empty() && exc.exception_type == "RuntimeError" {
                return format!("{}: unhandled exception", exc.exception_type);
            }
            format!("{}: {}", exc.exception_type, exc.message)
        }
        _ => format!("{:?}", exception),
    }
}

/// Convert an object into a dictionary key string representation.
pub(super) fn object_to_dict_key(value: &Object) -> Option<Slot> {
    Some(match value.view() {
        // A String that reads as a number, a boolean or nil takes a slot of
        // its own, so `"1"` and `1` are different keys, as `eql?` makes
        // them. The key object kept beside the entry reads it back.
        View::String(s) if reads_as_another_kind(&s.as_str()) => {
            format!("{AMBIGUOUS_STRING_PREFIX}{}", s.as_str()).into()
        }
        View::String(s) => (&*s.as_str()).into(),
        View::Symbol(s) => symbol_slot(&s.as_str()),
        View::Int(i) => i.to_string().into(),
        // A Float key keeps its fraction, so 4.0 and 4 are different keys the
        // way Ruby's `eql?` makes them.
        View::Float(f) if f.fract() == 0.0 && f.is_finite() => format!("{:.1}", f).into(),
        View::Float(f) => f.to_string().into(),
        View::Bool(b) => b.to_string().into(),
        View::Nil => "nil".into(),
        other => format!("{}", other).into(),
    })
}

thread_local! {
    /// The slot each Symbol named so far takes as a key, which every Hash
    /// keyed by it shares.
    static SYMBOL_SLOTS: std::cell::RefCell<crate::fast_hash::NameMap<Box<str>, Slot>> =
        std::cell::RefCell::new(crate::fast_hash::NameMap::default());
}

/// The slot the Symbol `name` takes as a key.
fn symbol_slot(name: &str) -> Slot {
    SYMBOL_SLOTS.with(|slots| {
        if let Some(held) = slots.borrow().get(name) {
            return held.clone();
        }
        let made: Slot = format!(":{name}").into();
        slots.borrow_mut().insert(name.into(), made.clone());
        made
    })
}

/// What starts the slot of a String key that reads as another kind.
const AMBIGUOUS_STRING_PREFIX: char = '\u{2}';

/// Whether a string reads back as a number, a boolean, or nil, which is what
/// makes it ambiguous as a hash key.
fn reads_as_another_kind(text: &str) -> bool {
    text == "nil"
        || text == "true"
        || text == "false"
        || text.parse::<i64>().is_ok()
        || text.parse::<f64>().is_ok()
}

/// Check if a key value is primitive (reconstructible from string alone).
pub(super) fn is_primitive_key(value: &Object) -> bool {
    // A String that reads back as some other kind is not reconstructible from
    // its text, so `{"1" => x}` keeps the key object beside the entry.
    if let View::String(text) = value.view() {
        return !reads_as_another_kind(&text.as_str());
    }
    // A name standing for the bytes an encoding spells it with is not
    // reconstructible from its text either, so the object is kept beside the
    // entry and answers with the encoding it was made under.
    if let View::Symbol(name) = value.view() {
        return !name.holds_bytes()
            && name.encoding_name() == crate::object::string_value::DEFAULT_ENCODING;
    }
    matches!(
        value.view(),
        View::Int(_) | View::Float(_) | View::Bool(_) | View::Nil
    )
}

/// Reconstruct a primitive key Object from its string representation.
pub(super) fn dict_key_to_object(key_str: &str) -> Object {
    if key_str == "nil" {
        return Object::Nil;
    }
    if key_str == "true" {
        return Object::Bool(true);
    }
    if key_str == "false" {
        return Object::Bool(false);
    }
    if let Some(sym) = key_str.strip_prefix(':') {
        return Object::symbol(sym.to_string());
    }
    if let Ok(n) = key_str.parse::<i64>() {
        return Object::Int(n);
    }
    if let Ok(f) = key_str.parse::<f64>() {
        return Object::Float(f);
    }
    Object::string(key_str.to_string())
}

/// Determine if a value is truthy for conditional statements.
/// In Metorex, only `false` and `nil` are falsy; everything else is truthy.
pub(super) fn is_truthy(value: &Object) -> bool {
    !matches!(value.view(), View::Bool(false) | View::Nil)
}
