// Reading, writing and removing the instance variables an object holds.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// Reading, writing and removing the instance variables an object holds.
    pub(crate) fn call_object_instance_variable_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "instance_variables" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let vars = if let View::Instance(inst_rc) = receiver.view() {
                    let inst = inst_rc.borrow();
                    inst.instance_vars
                        .keys()
                        // The interpreter's own slots start with `__`.
                        .filter(|name| {
                            !crate::vm::native_methods::struct_methods::is_member_slot(name)
                                && inst.lists_var(name)
                        })
                        .map(|k| Object::symbol(format!("@{}", k)))
                        .collect()
                } else if let Some(address) = Self::collection_address(receiver) {
                    // A collection or a String keeps its instance variables
                    // aside, since it has nowhere of its own to put them.
                    self.collection_variables
                        .get(&address)
                        .map(|held| {
                            held.keys()
                                .map(|name| Object::symbol(format!("@{}", name)))
                                .collect()
                        })
                        .unwrap_or_default()
                } else if let View::Exception(details) = receiver.view() {
                    // An exception keeps the program's variables beside the
                    // ones the interpreter uses, which start with `__`.
                    details
                        .borrow()
                        .instance_vars
                        .keys()
                        .filter(|name| !name.starts_with("__"))
                        .map(|name| Object::symbol(format!("@{}", name)))
                        .collect()
                } else if let View::Class(class_rc) | View::Module(class_rc) = receiver.view() {
                    // A class keeps its own instance variables among its
                    // class-level storage, under an `@` prefix.
                    class_rc
                        .class_var_names()
                        .into_iter()
                        .filter(|name| name.starts_with('@') && !name.starts_with("@@"))
                        .map(Object::symbol)
                        .collect()
                } else {
                    vec![]
                };
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(vars),
                ))))
            }
            // `instance_variable_defined?(name)` — whether the receiver holds
            // that variable. Nothing but an instance, class, or module can.
            "instance_variable_defined?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let var_name = match arguments[0].view() {
                    View::String(name) => name.as_str().to_string(),
                    View::Symbol(name) => name.as_str().to_string(),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let bare_name = var_name.strip_prefix('@').unwrap_or(&var_name);
                let defined = match receiver.view() {
                    View::Instance(inst_rc) => {
                        inst_rc.borrow().instance_vars.contains_key(bare_name)
                    }
                    View::Class(class_rc) | View::Module(class_rc) => {
                        class_rc.get_class_var(&format!("@{}", bare_name)).is_some()
                    }
                    View::Exception(details) => {
                        details.borrow().instance_vars.contains_key(bare_name)
                    }
                    // A collection's variables are recorded against it by the
                    // VM, which is where they are looked for.
                    _ => Self::collection_address(receiver).is_some_and(|address| {
                        self.collection_variables
                            .get(&address)
                            .is_some_and(|held| held.contains_key(bare_name))
                    }),
                };
                Ok(Some(Object::Bool(defined)))
            }
            "instance_variable_get" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let clean_name =
                    self.coerce_instance_variable_name(&arguments[0], receiver, position)?;
                let clean_name = clean_name.as_str();
                match receiver.view() {
                    View::Instance(inst_rc) => {
                        let inst = inst_rc.borrow();
                        Ok(Some(
                            inst.instance_vars
                                .get(clean_name)
                                .cloned()
                                .unwrap_or(Object::Nil),
                        ))
                    }
                    View::Class(class_rc) => Ok(Some(
                        class_rc
                            .get_class_var(&format!("@{}", clean_name))
                            .unwrap_or(Object::Nil),
                    )),
                    View::Array(_)
                    | View::Dict(_)
                    | View::Set(_)
                    | View::String(_)
                    | View::Method(_)
                    | View::Block(_)
                    | View::Binding(_)
                    | View::Regex(_, _) => Ok(Some(
                        Self::collection_address(receiver)
                            .and_then(|address| self.collection_variables.get(&address))
                            .and_then(|held| held.get(clean_name))
                            .cloned()
                            .unwrap_or(Object::Nil),
                    )),
                    View::Module(module_rc) => Ok(Some(
                        module_rc
                            .get_class_var(&format!("@{}", clean_name))
                            .unwrap_or(Object::Nil),
                    )),
                    View::Exception(details) => Ok(Some(
                        details
                            .borrow()
                            .instance_vars
                            .get(clean_name)
                            .cloned()
                            .unwrap_or(Object::Nil),
                    )),
                    _ => Ok(Some(Object::Nil)),
                }
            }
            // Kernel#remove_instance_variable — take the variable off the
            // object and answer what it held. Unlike its siblings this one is
            // public. The name is validated before the frozen check, so
            // `o.remove_instance_variable(:foo)` raises NameError even on a
            // frozen receiver.
            "remove_instance_variable" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let var_name =
                    self.coerce_instance_variable_name(&arguments[0], receiver, position)?;
                let holds_variables = matches!(
                    receiver.view(),
                    View::Instance(_) | View::Class(_) | View::Module(_) | View::Exception(_)
                ) || Self::collection_address(receiver).is_some();
                if self.object_is_frozen(receiver) || !holds_variables {
                    let class_name = self.builtins().class_of(receiver).name().to_string();
                    let msg = format!("can't modify frozen {}: {}", class_name, receiver);
                    let exc = Object::exception("FrozenError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                // Each kind keeps its variables where `instance_variable_set`
                // put them.
                let removed = match receiver.view() {
                    View::Instance(instance_rc) => instance_rc
                        .borrow_mut()
                        .instance_vars
                        .shift_remove(&var_name),
                    View::Class(class_rc) | View::Module(class_rc) => {
                        class_rc.remove_class_var(&format!("@{}", var_name))
                    }
                    View::Exception(details) => details
                        .borrow_mut()
                        .instance_vars
                        .shift_remove(var_name.as_str()),
                    _ => Self::collection_address(receiver).and_then(|address| {
                        self.collection_variables
                            .get_mut(&address)
                            .and_then(|held| held.remove(&var_name))
                    }),
                };
                match removed {
                    Some(value) => Ok(Some(value)),
                    None => {
                        let msg = format!("instance variable @{} not defined", var_name);
                        let exc = Object::exception("NameError", msg.clone());
                        if let View::Exception(cell) = exc.view() {
                            cell.borrow_mut().name = Some(format!("@{}", var_name));
                        }
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                }
            }
            "instance_variable_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // Ruby validates the name before it checks whether the
                // receiver is frozen, so `"".instance_variable_set(:c, 1)`
                // raises NameError rather than FrozenError.
                let var_name =
                    self.coerce_instance_variable_name(&arguments[0], receiver, position)?;
                self.warn_chilled_string(receiver, position)?;
                let value = arguments[1].clone();
                match receiver.view() {
                    View::Instance(instance_rc) => {
                        let is_frozen = instance_rc.borrow().frozen;
                        if is_frozen {
                            return Err(self.frozen_modification_error(receiver, position));
                        }
                        // The core library keeps its own state on the object it
                        // is running for. A variable it sets on another object,
                        // as Marshal does restoring one, is that object's own.
                        let running_self = self.environment().get("self");
                        let on_itself = matches!(
                            running_self.as_ref().map(Object::view),
                            Some(View::Instance(running)) if std::rc::Rc::ptr_eq(running, instance_rc)
                        );
                        if position.prelude && on_itself {
                            instance_rc
                                .borrow_mut()
                                .set_interpreter_var(var_name, value.clone());
                        } else {
                            instance_rc
                                .borrow_mut()
                                .set_program_var(var_name, value.clone());
                        }
                        Ok(Some(value))
                    }
                    View::Class(class_rc) => {
                        class_rc.set_class_var(format!("@{}", var_name), value.clone());
                        Ok(Some(value))
                    }
                    View::Module(module_rc) => {
                        module_rc.set_class_var(format!("@{}", var_name), value.clone());
                        Ok(Some(value))
                    }
                    View::Exception(details) => {
                        details
                            .borrow_mut()
                            .instance_vars
                            .insert(var_name.into(), value.clone());
                        Ok(Some(value))
                    }
                    // A collection has nowhere of its own to keep an instance
                    // variable, so the VM records it against the collection.
                    View::Array(_)
                    | View::Dict(_)
                    | View::Set(_)
                    | View::String(_)
                    | View::Method(_)
                    | View::Block(_)
                    | View::Binding(_)
                    | View::Regex(_, _) => {
                        if self.object_is_frozen(receiver) {
                            return Err(self.frozen_modification_error(receiver, position));
                        }
                        if let Some(address) = Self::collection_address(receiver) {
                            self.collection_variables
                                .entry(address)
                                .or_default()
                                .insert(var_name, value.clone());
                            self.collection_variable_owners
                                .insert(address, receiver.clone());
                        }
                        Ok(Some(value))
                    }
                    // Immediates (true/false/nil/integers/symbols/floats) are
                    // always frozen — assigning an ivar raises FrozenError to
                    // match Ruby (FrozenError is a subclass of RuntimeError).
                    other => {
                        let class_name = match other {
                            View::Bool(true) => "TrueClass".to_string(),
                            View::Bool(false) => "FalseClass".to_string(),
                            View::Nil => "NilClass".to_string(),
                            View::Symbol(_) => "Symbol".to_string(),
                            _ => self.builtins().class_of(receiver).name().to_string(),
                        };
                        let msg = format!("can't modify frozen {}: {}", class_name, other);
                        let exc = Object::exception("FrozenError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                }
            }
            _ => Ok(None),
        }
    }
}
