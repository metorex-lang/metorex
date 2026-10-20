// The copy `dup` and `clone` hand back, and what it carries over.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// The copy `dup` and `clone` hand back, before any frozen state or
    /// singleton class is carried over.
    /// Whether the receiver is an instance whose class chain reaches
    /// BasicObject without passing through Object. Only such an object is
    /// limited to BasicObject's own methods.
    pub(crate) fn rooted_at_basic_object(&self, receiver: &Object) -> bool {
        let View::Instance(instance) = receiver.view() else {
            return false;
        };
        let mut walked = Some(std::rc::Rc::clone(&instance.borrow().class));
        let mut reaches_basic_object = false;
        while let Some(class) = walked {
            if class.name() == "Object" {
                return false;
            }
            // A class rooted at BasicObject may still take Kernel on, and
            // then it answers everything Kernel defines.
            if class
                .mixin_chain()
                .iter()
                .any(|mixed| mixed.name() == "Kernel")
            {
                return false;
            }
            reaches_basic_object |= class.name() == "BasicObject";
            walked = class.superclass();
        }
        reaches_basic_object
    }

    pub(crate) fn copy_for(
        &mut self,
        receiver: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match receiver.view() {
            // Complex and Rational are value objects: there is nothing
            // to copy, so Ruby answers the receiver itself.
            View::Instance(inst_rc)
                if matches!(inst_rc.borrow().class.name(), "Complex" | "Rational") =>
            {
                Ok(Some(receiver.clone()))
            }
            View::Instance(inst_rc) => {
                let copy = {
                    let inst = inst_rc.borrow();
                    let new_inst = crate::object::Instance::new(std::rc::Rc::clone(&inst.class));
                    for (k, v) in &inst.instance_vars {
                        // The characters or elements behind a subclass of a
                        // primitive belong to the instance, so the copy gets
                        // its own rather than sharing them.
                        let held = match (k.as_str(), v.view()) {
                            (
                                crate::vm::native_methods::STRING_SUBCLASS_VAR,
                                View::String(text),
                            ) => {
                                let made = crate::object::StringValue::with_encoding(
                                    text.to_text(),
                                    text.encoding_name(),
                                );
                                if text.holds_bytes() {
                                    made.mark_bytes();
                                }
                                Object::String(std::rc::Rc::new(made))
                            }
                            (
                                crate::vm::native_methods::ARRAY_SUBCLASS_VAR,
                                View::Array(elements),
                            ) => Object::array(elements.borrow().clone()),
                            (crate::vm::native_methods::HASH_SUBCLASS_VAR, View::Dict(pairs)) => {
                                Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                                    pairs.borrow().clone(),
                                )))
                            }
                            (crate::vm::native_methods::SET_SUBCLASS_VAR, View::Set(held)) => {
                                Object::Set(std::rc::Rc::new(std::cell::RefCell::new(
                                    held.borrow().clone(),
                                )))
                            }
                            _ => v.clone(),
                        };
                        new_inst.borrow_mut().set_var(k.clone(), held);
                    }
                    new_inst.borrow_mut().program_dunder_names = inst.program_dunder_names.clone();
                    Object::Instance(new_inst)
                };
                // The copy gets `initialize_copy` with the original, so
                // a class can deep-copy what the shallow copy shared. A class
                // that undefined it has a copy that cannot be finished.
                self.refuse_undefined_copy_hook(&copy, receiver, position)?;
                if let Some((class, method)) = self.lookup_method(&copy, "initialize_copy")
                    && !method.is_undefined
                {
                    self.invoke_method(
                        class,
                        method,
                        copy.clone(),
                        vec![receiver.clone()],
                        position,
                    )?;
                }
                Ok(Some(copy))
            }
            // An exception copies its message, backtrace, cause, and
            // instance variables. `dup` leaves the singleton class
            // behind, so a method defined on the original is not on
            // the copy.
            View::Exception(details) => {
                let copy = {
                    // The backtrace Array is shared with the original,
                    // the way Ruby's copy shares it.
                    let copied = details.borrow().clone();
                    Object::Exception(std::rc::Rc::new(std::cell::RefCell::new(copied)))
                };
                self.refuse_undefined_copy_hook(&copy, receiver, position)?;
                if let Some((class, method)) = self.lookup_method(&copy, "initialize_copy")
                    && !method.is_undefined
                    && !method.body.is_empty()
                {
                    self.invoke_method(
                        class,
                        method,
                        copy.clone(),
                        vec![receiver.clone()],
                        position,
                    )?;
                }
                Ok(Some(copy))
            }
            // A copy of a string holds its own characters, so changing one
            // of them leaves the other alone.
            View::String(text) => {
                let made =
                    crate::object::StringValue::with_encoding(text.to_text(), text.encoding_name());
                if text.holds_bytes() {
                    made.mark_bytes();
                }
                Ok(Some(Object::String(std::rc::Rc::new(made))))
            }
            View::Array(arr_rc) => {
                let arr = arr_rc.borrow().clone();
                let copy = Object::Array(std::rc::Rc::new(std::cell::RefCell::new(arr)));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            View::Dict(dict_rc) => {
                let dict = dict_rc.borrow().clone();
                let copy = Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(dict)));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            View::Set(set_rc) => {
                let elements = set_rc.borrow().clone();
                let copy = Object::Set(std::rc::Rc::new(std::cell::RefCell::new(elements)));
                self.carry_collection_variables(receiver, &copy);
                // A set that places its elements by identity hands that on to
                // the copy, which holds the same elements.
                if self.set_by_identity(set_rc)
                    && let View::Set(made) = copy.view()
                {
                    self.identity_sets
                        .insert(std::rc::Rc::as_ptr(made) as usize, copy.clone());
                }
                Ok(Some(copy))
            }
            View::Class(class_rc) => {
                if class_rc.name() == "BasicObject" {
                    let msg = "can't copy the root class".to_string();
                    let exc = Object::exception("TypeError", msg.clone());
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: position_to_location(position),
                        message: msg,
                    });
                }
                let copy = std::rc::Rc::new(crate::class::Class::duplicate(class_rc));
                copy.copy_singleton_class_from(class_rc, Object::Class(std::rc::Rc::clone(&copy)));
                copy.take_ownership_of_methods_from(class_rc);
                Ok(Some(Object::Class(copy)))
            }
            View::Module(mod_rc) => {
                let copy = std::rc::Rc::new(crate::class::Class::duplicate(mod_rc));
                copy.copy_singleton_class_from(mod_rc, Object::Module(std::rc::Rc::clone(&copy)));
                copy.take_ownership_of_methods_from(mod_rc);
                Ok(Some(Object::Module(copy)))
            }
            // A callable and a Binding copy as a new reference to the same
            // code and scope, so the copy is a separate object with the
            // instance variables the original carried.
            // A copy of a range is a range of its own, which `equal?` tells
            // apart from the one it was cut from.
            View::Range {
                start,
                end,
                exclusive,
                ..
            } => Ok(Some(Object::Range(
                Box::new(start.clone()),
                Box::new(end.clone()),
                exclusive,
                std::rc::Rc::new(()),
            ))),
            View::Method(method) => {
                let copy = Object::Method(std::rc::Rc::new((**method).clone()));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            View::Block(block) => {
                let copy = Object::Block(std::rc::Rc::new((**block).clone()));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            View::Binding(binding) => {
                let copy = Object::Binding(std::rc::Rc::new((**binding).clone()));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            // Immutable types return themselves
            _ => Ok(Some(receiver.clone())),
        }
    }

    /// Whether any finalizer has been registered at all, which is what
    /// makes a clone worth asking ObjectSpace about.
    fn has_finalizers(&self) -> bool {
        let viewed = self.globals().get("ObjectSpace");
        let Some(View::Module(space) | View::Class(space)) = viewed.view() else {
            return false;
        };
        matches!(space.get_class_var("@finalizers").view(), Some(View::Dict(held)) if !held.borrow().is_empty())
    }

    /// Give the copy the instance variables the original was carrying, for
    /// the objects that keep them against their address.
    fn carry_collection_variables(&mut self, receiver: &Object, copy: &Object) {
        let Some(from) = Self::collection_address(receiver) else {
            return;
        };
        let Some(held) = self.collection_variables.get(&from).cloned() else {
            return;
        };
        if let Some(to) = Self::collection_address(copy) {
            self.collection_variables.insert(to, held);
            self.collection_variable_owners.insert(to, copy.clone());
        }
    }

    /// The `freeze:` keyword `clone` was given: Some(true), Some(false), or
    /// None for `freeze: nil` and for no keyword at all.
    pub(crate) fn clone_freeze_argument(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<bool>, MetorexError> {
        let Some(View::Dict(entries)) = arguments.first().view() else {
            if arguments.is_empty() {
                return Ok(None);
            }
            return Err(method_argument_error("clone", 0, arguments.len(), position));
        };
        let value = entries.borrow().get(":freeze").cloned();
        match value.view() {
            None | Some(View::Nil) => Ok(None),
            Some(View::Bool(freeze)) => Ok(Some(freeze)),
            Some(other) => {
                let message = format!(
                    "unexpected value for freeze: {}",
                    self.builtins().class_of(&other.to_object()).name()
                );
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: position_to_location(position),
                    message,
                })
            }
        }
    }

    /// Carry over what `clone` copies beyond the instance variables: the
    /// singleton class, the frozen state, and the `initialize_clone` call.
    pub(crate) fn finish_clone(
        &mut self,
        receiver: &Object,
        copy: &Object,
        freeze: Option<bool>,
        arguments: &[Object],
        position: Position,
    ) -> Result<(), MetorexError> {
        // A string handed back with notice that it will be frozen carries that
        // notice into a clone, the way Ruby's does.
        if let (View::String(original), View::String(copied)) = (receiver.view(), copy.view())
            && original.is_chilled()
        {
            let notice = original.take_chill();
            if let Some(notice) = notice {
                original.chill(notice.clone());
                copied.chill(notice);
            }
        }
        // A singleton class travels with a clone, so a method defined on the
        // original answers on the copy too.
        if let (View::Instance(original), View::Instance(copied)) = (receiver.view(), copy.view()) {
            let singleton = original.borrow().singleton_class.borrow().clone();
            if let Some(singleton) = singleton {
                let copied_singleton = std::rc::Rc::new(crate::class::Class::duplicate(&singleton));
                copied_singleton.take_ownership_of_methods_from(&singleton);
                *copied.borrow().singleton_class.borrow_mut() = Some(copied_singleton);
            }
            let methods = original.borrow().singleton_methods.borrow().clone();
            for (name, method) in methods {
                copied
                    .borrow_mut()
                    .singleton_methods
                    .borrow_mut()
                    .insert(name, method);
            }
        }
        // A collection keeps its singleton methods against its address, so
        // they are carried across the same way a class's are.
        if !matches!(receiver.view(), View::Instance(_)) {
            self.copy_singleton_methods(receiver, copy);
        }
        if let Some((class, method)) = self.lookup_method(copy, "initialize_clone")
            && !method.is_undefined
            && !method.body.is_empty()
        {
            let mut call_arguments = vec![receiver.clone()];
            call_arguments.extend(arguments.iter().cloned());
            self.invoke_method(class, method, copy.clone(), call_arguments, position)?;
        }
        // Ruby's `clone` copies the finalizers the original was given, so
        // both the original and the copy run theirs.
        if self.has_finalizers()
            && let Some(space) = self.globals().get("ObjectSpace")
        {
            self.send_to_object(
                space,
                "__carry_finalizers__",
                vec![receiver.clone(), copy.clone()],
                position,
            )?;
        }
        let frozen = match freeze {
            Some(freeze) => freeze,
            None => self.object_is_frozen(receiver),
        };
        if frozen {
            match copy.view() {
                View::Class(class) | View::Module(class) => class.freeze(),
                View::Instance(instance) => instance.borrow_mut().frozen = true,
                View::String(text) => text.freeze(),
                View::Array(_)
                | View::Dict(_)
                | View::Set(_)
                | View::Method(_)
                | View::Block(_)
                | View::Binding(_)
                | View::Regex(_, _) => {
                    if let Some(address) = Self::collection_address(copy) {
                        self.frozen_collections.insert(address, copy.clone());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl VirtualMachine {
    /// Refuses a copy whose class undefined `initialize_copy`, as the
    /// NoMethodError Ruby raises when the copy calls it.
    fn refuse_undefined_copy_hook(
        &mut self,
        copy: &Object,
        original: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let undefined = self
            .lookup_method(copy, "initialize_copy")
            .is_some_and(|(_, method)| method.is_undefined);
        if !undefined {
            return Ok(());
        }
        let wording = self.receiver_wording_for(copy, position);
        Err(crate::vm::errors::undefined_method_error_worded(
            "initialize_copy",
            copy,
            std::slice::from_ref(original),
            wording,
            position,
        ))
    }
}
