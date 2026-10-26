// Instance struct - represents an instance of a class

use crate::class::Class;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::{Method, Object};

/// Instance of a class with instance variables
#[derive(Debug, PartialEq)]
pub struct Instance {
    /// Reference to the class this is an instance of
    pub class: Rc<Class>,
    /// Instance variables (@variable), in the order they were first assigned
    /// so `instance_variables` reports Ruby's declaration order.
    pub instance_vars: super::shape::InstanceVariables,
    /// Lazily-allocated singleton class for this instance (Ruby semantics:
    /// every object conceptually has one, but we only materialize on demand).
    pub singleton_class: RefCell<Option<Rc<Class>>>,
    /// Whether `freeze` has been called. Subsequent ivar assignments / setter
    /// invocations must raise FrozenError.
    pub frozen: bool,
    /// What few instances carry, made the first time one is given any of it.
    extras: RefCell<Option<Box<Extras>>>,
}

#[derive(Debug, Default, PartialEq)]
struct Extras {
    /// Singleton methods defined directly on this instance (def obj.method).
    singleton_methods: HashMap<String, Rc<Method>>,
    /// The variables starting with `__` the program assigned, which it
    /// lists, unlike the interpreter's own slots of that shape.
    program_dunder_names: HashSet<String>,
    /// The variables the core library written in Ruby gave the object,
    /// which stand for state a class MRI writes in C keeps out of sight.
    interpreter_names: HashSet<String>,
}

impl Instance {
    /// Make an instance of a class, recorded among the objects still alive.
    pub fn new(class: Rc<Class>) -> Rc<RefCell<Self>> {
        let made = Rc::new(RefCell::new(Self {
            class,
            instance_vars: Default::default(),
            singleton_class: RefCell::new(None),
            frozen: false,
            extras: RefCell::new(None),
        }));
        super::live::record_instance(&made);
        made
    }

    fn read_extras<R>(&self, read: impl FnOnce(&Extras) -> R) -> Option<R> {
        self.extras.borrow().as_deref().map(read)
    }

    fn change_extras<R>(&self, change: impl FnOnce(&mut Extras) -> R) -> R {
        change(self.extras.borrow_mut().get_or_insert_with(Box::default))
    }

    /// Attach a singleton method to this instance.
    pub fn define_singleton_method(&self, name: String, method: Rc<Method>) {
        self.change_extras(|extras| extras.singleton_methods.insert(name, method));
    }

    /// Look up a singleton method on this instance.
    pub fn find_singleton_method(&self, name: &str) -> Option<Rc<Method>> {
        self.read_extras(|extras| extras.singleton_methods.get(name).cloned())
            .flatten()
    }

    /// Whether the instance has singleton methods of its own.
    pub fn has_singleton_methods(&self) -> bool {
        self.read_extras(|extras| !extras.singleton_methods.is_empty())
            .unwrap_or(false)
    }

    /// The names and methods of the instance's own singleton methods.
    pub fn singleton_methods(&self) -> Vec<(String, Rc<Method>)> {
        self.read_extras(|extras| {
            extras
                .singleton_methods
                .iter()
                .map(|(name, method)| (name.clone(), Rc::clone(method)))
                .collect()
        })
        .unwrap_or_default()
    }

    /// Drop the instance's own singleton methods.
    pub fn clear_singleton_methods(&self) {
        if let Some(extras) = self.extras.borrow_mut().as_deref_mut() {
            extras.singleton_methods.clear();
        }
    }

    /// Give this instance the variable names `other` lists and leaves out,
    /// as a copy of it keeps them.
    pub fn take_names_listed_by(&self, other: &Instance) {
        let names = other.read_extras(|extras| {
            (
                extras.program_dunder_names.clone(),
                extras.interpreter_names.clone(),
            )
        });
        if let Some((program_dunder_names, interpreter_names)) = names {
            self.change_extras(|extras| {
                extras.program_dunder_names = program_dunder_names;
                extras.interpreter_names = interpreter_names;
            });
        }
    }

    /// Whether the core library written in Ruby gave the object any
    /// variable of its own.
    pub fn has_interpreter_vars(&self) -> bool {
        self.read_extras(|extras| !extras.interpreter_names.is_empty())
            .unwrap_or(false)
    }

    /// Whether the program assigned the variable `name`, one starting with
    /// `__`.
    pub fn program_assigned_dunder(&self, name: &str) -> bool {
        self.read_extras(|extras| extras.program_dunder_names.contains(name))
            .unwrap_or(false)
    }

    /// Get an instance variable
    pub fn get_var(&self, name: &str) -> Option<&Object> {
        self.instance_vars.get(name)
    }

    /// Set an instance variable
    pub fn set_var(&mut self, name: String, value: Object) {
        self.instance_vars.insert(name, value);
    }

    /// Set an instance variable the program assigned, which `instance_variables`
    /// lists whatever its name.
    pub fn set_program_var(&mut self, name: String, value: Object) {
        if name.starts_with("__") {
            let named = name.clone();
            self.change_extras(|extras| extras.program_dunder_names.insert(named));
        }
        if let Some(extras) = self.extras.get_mut().as_deref_mut() {
            extras.interpreter_names.remove(&name);
        }
        self.instance_vars.insert(name, value);
    }

    /// Set a variable the core library written in Ruby assigned, which
    /// `instance_variables` leaves out unless the program assigned it first.
    pub fn set_interpreter_var(&mut self, name: String, value: Object) {
        if self.instance_vars.get(&name).is_none() {
            let named = name.clone();
            self.change_extras(|extras| extras.interpreter_names.insert(named));
        }
        self.instance_vars.insert(name, value);
    }

    /// Whether `instance_variables` and `inspect` list the variable `name`:
    /// every one the program assigned, and none of the interpreter's own
    /// slots, whose names start with `__`.
    pub fn lists_var(&self, name: &str) -> bool {
        self.read_extras(|extras| {
            (!name.starts_with("__") && !extras.interpreter_names.contains(name))
                || extras.program_dunder_names.contains(name)
        })
        .unwrap_or(!name.starts_with("__"))
    }

    /// Check if this instance's class (or a superclass) knows about the variable.
    pub fn is_var_declared(&self, name: &str) -> bool {
        self.class.has_instance_var(name)
    }

    /// Find a method on this instance's class (walks the inheritance chain)
    pub fn find_method(&self, name: &str) -> Option<Rc<Method>> {
        self.class.find_method(name)
    }

    /// Get the class name of this instance
    pub fn class_name(&self) -> &str {
        self.class.name()
    }
}
