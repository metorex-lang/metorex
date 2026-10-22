// Whether an object belongs to a class or one of its ancestors.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Whether an object belongs to a class or one of its ancestors.
    pub(crate) fn call_object_type_check_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "is_a?" | "kind_of?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let target_class = match arguments[0].view() {
                    View::Class(c) => c,
                    View::Module(m) => m,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Class",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                // A Class object is itself an instance of Class (and its
                // ancestors: Module, Object, BasicObject). Walk the global
                // Class's chain so anonymous Class.new instances answer
                // correctly without needing their own class pointer.
                // Metorex holds an encoding as a class of its own under
                // Encoding, so one is an instance of Encoding rather than of
                // Class, which is what `class` already reports.
                if self.names_an_encoding(receiver) {
                    return Ok(Some(Object::Bool(matches!(
                        target_class.name(),
                        "Encoding" | "Object" | "BasicObject"
                    ))));
                }
                // A module made by `new` on a subclass of Module is a kind of
                // that subclass, each class it inherits from, and each module
                // they include.
                if let View::Module(module) = receiver.view()
                    && let Some(made_from) = module.instance_of()
                    && (crate::vm::method_invocation::descends_from_class(&made_from, target_class)
                        || crate::vm::eval::super_expr::walk_ancestors(&made_from)
                            .iter()
                            .any(|ancestor| std::rc::Rc::ptr_eq(ancestor, target_class)))
                {
                    return Ok(Some(Object::Bool(true)));
                }
                // A class is a kind of the singleton class of itself and of
                // each class it inherits from, since those are the classes
                // its own singleton class inherits from.
                if let (View::Class(named) | View::Module(named), true) =
                    (receiver.view(), target_class.is_singleton_class())
                    && let Some(View::Class(attached) | View::Module(attached)) =
                        target_class.get_class_var("__attached__").view()
                {
                    let mut current = Some(std::rc::Rc::clone(named));
                    while let Some(held) = current {
                        if std::rc::Rc::ptr_eq(&held, attached) {
                            return Ok(Some(Object::Bool(true)));
                        }
                        current = held.superclass();
                    }
                    return Ok(Some(Object::Bool(false)));
                }
                if matches!(receiver.view(), View::Class(_) | View::Module(_)) {
                    let target_name = target_class.name();
                    let meta_name = match receiver.view() {
                        View::Class(_) => "Class",
                        View::Module(_) => "Module",
                        _ => unreachable!(),
                    };
                    if let Some(View::Class(meta)) = self.globals().get(meta_name).view() {
                        if self.builtins().is_subclass_of(meta, target_class) {
                            return Ok(Some(Object::Bool(true)));
                        }
                        let mut current = meta.superclass();
                        while let Some(anc) = current {
                            if anc.name() == target_name {
                                return Ok(Some(Object::Bool(true)));
                            }
                            current = anc.superclass();
                        }
                    }
                }
                // An exception is tagged with its type name rather than
                // carrying a class pointer, so its ancestry is walked from the
                // class that name resolves to.
                if let View::Exception(details) = receiver.view() {
                    let carried = details.borrow().class.clone().map(Object::Class);
                    let exception_type = details.borrow().exception_type.clone();
                    let resolved = match carried {
                        Some(class) => Some(class),
                        None => match self.globals().get(&exception_type).view() {
                            Some(class @ View::Class(_)) => Some(class.to_object()),
                            _ => self.resolve_qualified_constant(&exception_type),
                        },
                    };
                    if let Some(View::Class(exception_class)) = resolved.view()
                        && self
                            .builtins()
                            .is_subclass_of(exception_class, target_class)
                    {
                        return Ok(Some(Object::Bool(true)));
                    }
                }
                if self.builtins().is_instance_of(receiver, target_class) {
                    return Ok(Some(Object::Bool(true)));
                }
                // A value such as an Array or a String keeps the modules it
                // was extended with on a singleton class held apart from it.
                let extended = self
                    .existing_singleton_class(receiver)
                    .is_some_and(|singleton| {
                        self.builtins().is_subclass_of(&singleton, target_class)
                    });
                Ok(Some(Object::Bool(extended)))
            }
            _ => Ok(None),
        }
    }
}
