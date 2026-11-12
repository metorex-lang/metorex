// Copying one object into another, and the scope a call sits in.

use super::*;
use crate::object::View;

impl VirtualMachine {
    /// Copying one object into another, and the scope a call sits in.
    pub(crate) fn call_object_initializing_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `initialize_copy(source)` is the hook `dup` and `clone` call on
            // the new object. Its default does nothing beyond checking that
            // the copy is allowed.
            "initialize_copy" if arguments.len() == 1 => {
                let source = &arguments[0];
                if same_object(receiver, source) {
                    return Ok(Some(receiver.clone()));
                }
                if self.object_is_frozen(receiver) {
                    let message = format!("can't modify frozen object: {}", receiver);
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("FrozenError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                let receiver_class = self.builtins().class_of(receiver);
                let source_class = self.builtins().class_of(source);
                if !std::rc::Rc::ptr_eq(&receiver_class, &source_class) {
                    let message = "initialize_copy should take same class object".to_string();
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                Ok(Some(receiver.clone()))
            }
            // `clone` and `dup` reach `initialize_copy` through these, so a
            // class can hook either copy on its own.
            "initialize_clone" | "initialize_dup" if !arguments.is_empty() => {
                let source = arguments[0].clone();
                match self.lookup_method(receiver, "initialize_copy") {
                    Some((class, method)) if !method.is_undefined => {
                        self.invoke_method(
                            class,
                            method,
                            receiver.clone(),
                            vec![source],
                            position,
                        )?;
                    }
                    _ => {
                        self.call_object_method(receiver, "initialize_copy", &[source], position)?;
                    }
                }
                Ok(Some(receiver.clone()))
            }
            // `fail` is private on Kernel too, so `send(:fail, ...)` reaches it.
            "fail" if arguments.len() <= 2 => self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some),
            // `binding` is likewise private on Kernel. It captures the frame
            // that called it rather than anything about this receiver, so
            // `obj.send(:binding)` answers the sender's context.
            "binding" if arguments.is_empty() => self
                .call_native_function("binding_kernel", Vec::new(), position)
                .map(Some),
            // Kernel functions that report on the running method, so
            // `send(:__callee__)` reaches them like any other Kernel method.
            "__method__" | "__callee__" => self
                .call_native_function(method_name, arguments.to_vec(), position)
                .map(Some),
            // `hash` — equal values answer equal digests, and reference
            // types fall back to their identity.
            "hash" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(self.hash_digest(receiver, position)?)))
            }
            "object_id" | "__id__" => {
                // An object held by reference takes the next id from a
                // counter the first time it is asked. An immediate gets a
                // value-derived id, so two literals that are the same value
                // share one, as Ruby's do.
                let id = match receiver.view() {
                    View::Instance(inst) => self.counted_object_id(inst) as i64,
                    // Two strings holding the same text are two objects, so
                    // each takes an id of its own the first time one is asked
                    // for.
                    View::String(text) => {
                        let next = self.next_object_id;
                        let given = text.object_id(|| next);
                        if given == next {
                            self.take_object_id();
                        }
                        given as i64
                    }
                    View::Array(arr) => self.counted_object_id(arr) as i64,
                    View::Dict(dict) => self.counted_object_id(dict) as i64,
                    View::Set(set) => self.counted_object_id(set) as i64,
                    View::Class(cls) => self.counted_object_id(cls) as i64,
                    View::Module(m) => self.counted_object_id(m) as i64,
                    View::Block(block) => self.counted_object_id(block) as i64,
                    // A Method and a Binding are objects of their own, so two
                    // that name the same thing have different ids.
                    View::Method(method) => self.counted_object_id(method) as i64,
                    View::Binding(binding) => self.counted_object_id(binding) as i64,
                    View::Exception(exc) => self.counted_object_id(exc) as i64,
                    // An integer past the i64 range is a heap object in Ruby
                    // too, so two of the same value have different ids.
                    View::BigInt(value) => self.counted_object_id(value) as i64,
                    // Ruby's fixnum object_id. Wrapping keeps the far ends of
                    // the range from overflowing.
                    View::Int(n) => n.wrapping_mul(2).wrapping_add(1),
                    View::Bool(true) => 20,
                    View::Bool(false) => 0,
                    View::Nil => 4,
                    View::Symbol(name) => value_object_id("symbol", &name.as_str()),
                    View::Float(value) => value_object_id("float", &value.to_bits().to_string()),
                    other => value_object_id("other", &other.to_string()),
                };
                Ok(Some(Object::Int(id)))
            }
            _ => Ok(None),
        }
    }
}
