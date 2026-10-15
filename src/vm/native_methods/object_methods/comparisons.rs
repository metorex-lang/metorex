// Ordering one object against another.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Whether `receiver` is an `Instance` whose class descends from `Module`
    /// (e.g. `class Sub < Module; end; Sub.new`). Such instances are modules
    /// and answer the module-body methods (`class_eval`/`class_exec`/...).
    pub(crate) fn instance_acts_as_module(&self, receiver: &Object) -> bool {
        let View::Instance(inst) = receiver.view() else {
            return false;
        };
        let class = std::rc::Rc::clone(&inst.borrow().class);
        match self.globals().get("Module").view() {
            Some(View::Class(module_class)) => self.builtins().is_subclass_of(&class, module_class),
            _ => false,
        }
    }

    /// Dispatch <=> on receiver with other, returning Some(i64) or None.
    pub(crate) fn dispatch_spaceship(
        &mut self,
        receiver: &Object,
        other: &Object,
        position: Position,
    ) -> Result<Option<i64>, MetorexError> {
        if let Some((class, method)) = self.lookup_method(receiver, "<=>") {
            let result = self.invoke_method(
                class,
                method,
                receiver.clone(),
                vec![other.clone()],
                position,
            )?;
            match result.view() {
                View::Int(n) => Ok(Some(n)),
                View::Nil => Ok(None),
                _ => Ok(None),
            }
        } else {
            // Fallback for built-in types
            match (receiver.view(), other.view()) {
                (View::Int(_) | View::BigInt(_), View::Int(_) | View::BigInt(_)) => {
                    let a = receiver.as_big_integer().expect("integer-kinded");
                    let b = other.as_big_integer().expect("integer-kinded");
                    Ok(Some(a.cmp(&b) as i64))
                }
                (View::String(a), View::String(b)) => Ok(Some((**a).cmp(b) as i64)),
                (View::Symbol(a), View::Symbol(b)) => Ok(Some(a.as_str().cmp(&b.as_str()) as i64)),
                // Integers and Floats compare against each other, so a Range
                // with one of each answers `include?` for either.
                (View::Float(_) | View::Int(_), View::Float(_) | View::Int(_)) => {
                    let to_f = |value: &Object| match value.view() {
                        View::Int(n) => n as f64,
                        View::Float(n) => n,
                        _ => unreachable!(),
                    };
                    Ok(to_f(receiver).partial_cmp(&to_f(other)).map(|o| o as i64))
                }
                _ => Ok(None),
            }
        }
    }

    /// The receiver's singleton method names as an array of symbols. A class
    /// or module keeps `def self.name` in its own table under the
    /// `__class__` convention; other objects keep them on a singleton class.
    pub(crate) fn singleton_method_names(&mut self, receiver: &Object) -> Object {
        self.singleton_method_names_with_ancestors(receiver, false)
    }

    /// The receiver's singleton method names. `include_ancestors` adds the
    /// class methods and singleton-class methods its superclasses supply,
    /// which is what `singleton_methods` reports by default.
    pub(crate) fn singleton_method_names_with_ancestors(
        &mut self,
        receiver: &Object,
        include_ancestors: bool,
    ) -> Object {
        let mut names: Vec<String> = Vec::new();
        if let View::Class(class_rc) | View::Module(class_rc) = receiver.view() {
            let mut current = Some(std::rc::Rc::clone(class_rc));
            while let Some(class) = current {
                // `def self.name` is stored under the `__class__` convention.
                // `private_class_method` marks the name on the singleton
                // class, which is where the visibility of a class method
                // lives, so that is checked alongside the class's own mark.
                let singleton = class.singleton_class_slot().clone();
                for name in class.method_names() {
                    // An undefined name stands in the table only to hide the
                    // method, so it is not listed.
                    if let Some(bare) = name.strip_prefix("__class__")
                        && !class
                            .find_method(&name)
                            .is_some_and(|method| method.is_undefined)
                        && !class.is_method_private(&name)
                        && !singleton
                            .as_ref()
                            .is_some_and(|held| held.is_method_private(bare))
                        && !names.contains(&bare.to_string())
                    {
                        names.push(bare.to_string());
                    }
                }
                if include_ancestors {
                    // `class << self` puts a method on the singleton class,
                    // and a subclass inherits its superclass's.
                    if let Some(singleton) = class.singleton_class_slot().clone() {
                        for name in singleton.method_names() {
                            if !name.starts_with("__class__")
                                && !singleton
                                    .find_method(&name)
                                    .is_some_and(|method| method.is_undefined)
                                && !class
                                    .find_own_method(&format!("__class__{name}"))
                                    .is_some_and(|method| method.is_undefined)
                                && !singleton.is_method_private(&name)
                                && !names.contains(&name)
                            {
                                names.push(name);
                            }
                        }
                    }
                }
                current = if include_ancestors {
                    class.superclass()
                } else {
                    None
                };
            }
        }
        for name in self.singleton_layer_names_with_mixins(receiver, include_ancestors) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names.sort();
        names.dedup();
        let symbols: Vec<Object> = names.into_iter().map(Object::symbol).collect();
        Object::Array(std::rc::Rc::new(std::cell::RefCell::new(symbols)))
    }
}
