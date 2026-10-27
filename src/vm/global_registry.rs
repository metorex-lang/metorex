//! Global object registry for the Metorex virtual machine.
//!
//! This module provides a registry that owns global objects accessible throughout the VM,
//! including built-in classes and singleton values.

use crate::class::{constant_state_changed, names_a_constant};
use crate::object::Object;
use std::collections::BTreeSet;

/// Registry that owns global objects accessible throughout the VM.
#[derive(Debug, Default)]
pub struct GlobalRegistry {
    objects: crate::fast_hash::NameMap<String, Object>,
    /// The subset of names that are Ruby global variables rather than
    /// constants, classes, or native functions, which share this registry.
    variable_names: BTreeSet<String>,
    /// Whether `$~` and `$_` hold anything but nil, which a method call
    /// reads to skip putting them aside when neither does.
    last_match_held: bool,
    last_line_held: bool,
}

impl GlobalRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a named global object.
    pub fn set(&mut self, name: impl Into<String>, object: Object) {
        let name = name.into();
        crate::vm::block_reuse::scope_state_changed();
        if names_a_constant(&name) {
            constant_state_changed();
        }
        self.note_last_match_or_line(&name, &object);
        self.objects.insert(name, object);
    }

    /// Insert or replace a global *variable*, recording its name so
    /// `global_variables` can report it.
    pub fn set_variable(&mut self, name: impl Into<String>, object: Object) {
        let name = name.into();
        if names_a_constant(&name) {
            constant_state_changed();
        }
        self.variable_names.insert(name.clone());
        self.note_last_match_or_line(&name, &object);
        self.objects.insert(name, object);
    }

    fn note_last_match_or_line(&mut self, name: &str, object: &Object) {
        let held = !matches!(object.view(), crate::object::View::Nil);
        match name {
            crate::vm::native_methods::regexp_methods::LAST_MATCH => self.last_match_held = held,
            "_" => self.last_line_held = held,
            _ => {}
        }
    }

    /// Whether `$~` or `$_` holds anything but nil.
    pub fn holds_last_match_or_line(&self) -> bool {
        self.last_match_held || self.last_line_held
    }

    /// The object a bare constant name stands for. A global variable is held
    /// here under its name with the `$` dropped, so one is never answered as
    /// a constant: `$DEBUG` and `DEBUG` are two different names in Ruby.
    pub fn constant(&self, name: &str) -> Option<Object> {
        if self.variable_names.contains(name) {
            return None;
        }
        self.objects.get(name).cloned()
    }

    /// The global variable names, without their `$` sigil, in sorted order.
    pub fn variable_names(&self) -> impl Iterator<Item = &String> {
        self.variable_names.iter()
    }

    /// Fetch a named global object if present.
    /// Remove a named global object, returning it when present.
    pub fn remove(&mut self, name: &str) -> Option<Object> {
        if names_a_constant(name) {
            constant_state_changed();
        }
        self.note_last_match_or_line(name, &Object::Nil);
        self.objects.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.objects.get(name).cloned()
    }

    /// Determine whether a name exists in the registry.
    pub fn contains(&self, name: &str) -> bool {
        self.objects.contains_key(name)
    }

    /// Iterator over registered globals (useful for seeding environments).
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Object)> {
        self.objects.iter()
    }
}
