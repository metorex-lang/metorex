// The class an object alone has, and the modules mixed into it.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// The class an object alone has, and the modules mixed into it.
    pub(crate) fn call_object_singleton_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "singleton_class" => {
                self.warn_chilled_string(receiver, position)?;
                if let Some(sole) = match receiver.view() {
                    View::Nil => Some("NilClass"),
                    View::Bool(true) => Some("TrueClass"),
                    View::Bool(false) => Some("FalseClass"),
                    _ => None,
                } && let Some(class @ View::Class(_)) = self.globals().get(sole).view()
                {
                    return Ok(Some(class.to_object()));
                }
                // One of the interned strings `String#-@` hands back stands
                // for every use of that text, so it has no singleton class.
                if refuses_a_singleton(receiver) {
                    let msg = "can't define singleton".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", msg.clone()),
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let singleton = self.singleton_class_of(receiver);
                // A frozen object's singleton class is frozen too, so no
                // method can be added to it afterwards. Only an instance
                // carries per-object frozen state; the immediates share one
                // singleton class per type, which must stay writable.
                if matches!(receiver.view(), View::Instance(_)) && self.object_is_frozen(receiver) {
                    singleton.freeze();
                }
                Ok(Some(Object::Class(singleton)))
            }
            // Object#extend(mod) — add `mod` as a mixin on the receiver's
            // singleton class so the module's instance methods become callable
            // on this specific object. (Class/Module already have their own
            // dedicated `extend` in class_methods.rs that hits earlier.)
            "extend" => {
                self.extend_with_modules(receiver, arguments, position)?;
                Ok(Some(receiver.clone()))
            }
            "singleton_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method = match arguments[0].view() {
                    View::String(s) => s.as_str().to_string(),
                    View::Symbol(s) => s.as_str().to_string(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "no implicit conversion into String".to_string(),
                            position_to_location(position),
                        ));
                    }
                };
                // Only the singleton layer counts: a method the object's
                // class defines is not a singleton method of the object.
                if let Some((owner, found)) = self.singleton_layer_method(receiver, &method) {
                    let mut bound = (*found).clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    if bound.owner_class.is_none() {
                        bound.owner_class = Some(owner);
                    }
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
                let msg = format!("undefined singleton method '{}' for {}", method, receiver);
                let exc = Object::exception("NameError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            _ => Ok(None),
        }
    }
}

impl VirtualMachine {
    /// `extend` with the modules given, which must all be modules. Ruby
    /// extends with the last one first, so the first one named ends up
    /// nearest the object.
    pub(crate) fn extend_with_modules(
        &mut self,
        receiver: &Object,
        arguments: &[Object],
        position: Position,
    ) -> Result<(), MetorexError> {
        if arguments.is_empty() {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "wrong number of arguments (given 0, expected 1+)",
                position,
            ));
        }
        let mut modules = Vec::with_capacity(arguments.len());
        for argument in arguments {
            match argument.view() {
                View::Module(module) => modules.push(std::rc::Rc::clone(module)),
                View::Instance(instance) => {
                    let named = instance.borrow().class.name().to_string();
                    return Err(not_a_module(&named, position));
                }
                _ => {
                    let named = crate::vm::native_methods::define_method::ruby_class_name(argument);
                    return Err(not_a_module(named, position));
                }
            }
        }
        for module in modules.iter().rev() {
            self.apply_module_extend(receiver, module, position)?;
        }
        Ok(())
    }
}

/// The TypeError `extend` raises for something that is not a module.
fn not_a_module(named: &str, position: Position) -> MetorexError {
    crate::vm::errors::simple_exception(
        "TypeError",
        &format!("wrong argument type {named} (expected Module)"),
        position,
    )
}
