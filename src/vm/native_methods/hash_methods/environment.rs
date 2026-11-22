// ENV, the hash the process shares with the operating system.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// The text an argument to an environment method names, refusing anything
    /// that names none.
    pub(crate) fn environment_text(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let View::String(text) = value.view() {
            return Ok(text.to_text());
        }
        if self.responds_to(value, "to_str")
            && let View::String(text) = self
                .send_to_object(value.clone(), "to_str", vec![], position)?
                .view()
        {
            return Ok(text.to_text());
        }
        let message = format!(
            "no implicit conversion of {} into String",
            crate::vm::errors::conversion_subject(value)
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    /// The slot a variable is kept under, read from the bytes its name is
    /// spelled with, whatever encoding the name is tagged with.
    pub(crate) fn environment_name(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let View::String(text) = value.view() {
            return Ok(environment_slot(&text.bytes()));
        }
        self.environment_text(value, position)
    }

    /// The encoding the environment is read as: the one a program named, or
    /// the one the system is set to.
    pub(crate) fn environment_reading_encoding(&mut self) -> Option<String> {
        if let Some(View::Class(held)) = self.globals().get("__Encoding_default_internal").view() {
            return Some(held.name().to_string());
        }
        // With nothing named, the environment reads as the locale does,
        // which is what the C library hands the program its variables in.
        match crate::vm::locale_charmap_name().as_str() {
            "" => None,
            "ANSI_X3.4-1968" => Some("US-ASCII".to_string()),
            other => Some(other.to_string()),
        }
    }

    /// Refuse a name the environment could never hold.
    pub(crate) fn refuse_bad_variable_name(
        &self,
        key: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if key.is_empty() || key.contains('=') {
            let message = format!("Invalid argument - setenv({})", key);
            return Err(crate::vm::errors::simple_exception(
                "Errno::EINVAL",
                &message,
                position,
            ));
        }
        Ok(())
    }

    /// Put one variable into the environment, where the C library sees it too.
    pub(crate) fn store_variable(
        &mut self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
        key: String,
        value: String,
    ) {
        let named = environment_key_object(&key);
        // What the environment holds is read back in the encoding the locale
        // names, so it is kept in that encoding from the moment it is set.
        let reading = self.environment_reading_encoding();
        let held = retagged(Object::string(value), &reading);
        // A name that reads back as some other kind of value is kept beside
        // the entry, so `ENV["1"]` stays a name rather than a number.
        if key.starts_with(UNREADABLE_NAME_PREFIX) || !crate::vm::utils::is_primitive_key(&named) {
            let slot = "__MX_KEY_OBJECTS__".to_string();
            let mut objects = match dict_rc.borrow().get(slot.as_str()).view() {
                Some(View::Dict(held)) => held.borrow().clone(),
                _ => Default::default(),
            };
            objects.insert(key.clone().into(), named.clone());
            dict_rc
                .borrow_mut()
                .insert(slot.into(), Object::Dict(Rc::new(RefCell::new(objects))));
        }
        dict_rc.borrow_mut().insert(key.into(), held.clone());
        self.record_environment_change(dict_rc, &named, &held);
    }

    /// Whether a hash is the one ENV names, which a few methods answer
    /// differently on.
    pub(crate) fn dict_is_environment(
        &self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
    ) -> bool {
        matches!(
            self.globals().get(crate::vm::init::PROCESS_ENVIRONMENT).view(),
            Some(View::Dict(held)) if Rc::ptr_eq(held, dict_rc)
        )
    }

    /// A write to ENV reaches the process environment too, so the C library
    /// sees it. Anything the C library reads, such as the time zone, follows
    /// what the program set.
    pub(crate) fn record_environment_change(
        &mut self,
        dict_rc: &Rc<RefCell<crate::object::HashEntries>>,
        key: &Object,
        value: &Object,
    ) {
        let viewed = self.globals().get(crate::vm::init::PROCESS_ENVIRONMENT);
        let Some(View::Dict(environment)) = viewed.view() else {
            return;
        };
        if !Rc::ptr_eq(environment, dict_rc) {
            return;
        }
        let View::String(name) = key.view() else {
            return;
        };
        let slot = environment_slot(&name.bytes());
        let Ok(name) = std::ffi::CString::new(environment_bytes(&slot)) else {
            return;
        };
        // Ruby removes the entry outright when a name is set to nil, so the
        // name stops being one the environment holds at all.
        if matches!(value.view(), View::Nil) {
            let mut held = dict_rc.borrow_mut();
            held.shift_remove(slot.as_str());
        }
        unsafe {
            match value.view() {
                View::Nil => {
                    libc::unsetenv(name.as_ptr());
                }
                View::String(text) => {
                    let Ok(setting) = std::ffi::CString::new(text.as_str().as_bytes().to_vec())
                    else {
                        return;
                    };
                    libc::setenv(name.as_ptr(), setting.as_ptr(), 1);
                }
                _ => {}
            }
        }
    }
}

/// The environment methods whose one argument names a variable or a value,
/// which is written as text however it arrives.
pub(crate) const ENVIRONMENT_TEXT_ARGUMENT: &[&str] = &[
    "[]",
    "delete",
    "has_key?",
    "key?",
    "include?",
    "member?",
    "key",
    "assoc",
    "has_value?",
    "value?",
    "rassoc",
];

/// The subset of those whose argument is a name.
pub(crate) const ENVIRONMENT_NAME_ARGUMENT: &[&str] =
    &["delete", "has_key?", "key?", "include?", "member?", "assoc"];

/// The subset of those whose argument is a value rather than a name.
pub(crate) const ENVIRONMENT_VALUE_ARGUMENT: &[&str] = &["has_value?", "value?", "rassoc"];

/// What starts the slot of a variable whose name is not UTF-8. No variable
/// name holds a NUL, so no readable name starts this way.
const UNREADABLE_NAME_PREFIX: &str = "\u{0}bytes:";

/// The slot a variable name spelled with `bytes` is kept under.
pub(crate) fn environment_slot(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => {
            let spelled: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
            format!("{UNREADABLE_NAME_PREFIX}{spelled}")
        }
    }
}

/// The bytes the variable kept under `slot` is named with.
fn environment_bytes(slot: &str) -> Vec<u8> {
    let Some(spelled) = slot.strip_prefix(UNREADABLE_NAME_PREFIX) else {
        return slot.as_bytes().to_vec();
    };
    (0..spelled.len())
        .step_by(2)
        .filter_map(|start| u8::from_str_radix(&spelled[start..start + 2], 16).ok())
        .collect()
}

/// The name ENV answers for the variable kept under `slot`.
fn environment_key_object(slot: &str) -> Object {
    if slot.starts_with(UNREADABLE_NAME_PREFIX) {
        let bytes = environment_bytes(slot);
        return Object::String(Rc::new(crate::object::StringValue::of_bytes(
            bytes, "UTF-8",
        )));
    }
    Object::string(slot)
}
