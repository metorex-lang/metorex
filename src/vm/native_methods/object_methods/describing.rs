// What an object reads back as, and the class it belongs to.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// What an object reads back as, and the class it belongs to.
    pub(crate) fn call_object_describing_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "to_s" | "inspect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // A String already is one, so `to_s` answers the receiver
                // itself and the encoding it is tagged with travels with it.
                if method_name == "to_s" && matches!(receiver.view(), View::String(_)) {
                    return Ok(Some(receiver.clone()));
                }
                // `true` and `false` each write themselves as one string
                // that never changes, so asking twice answers the same
                // object.
                if method_name == "to_s"
                    && let Some((slot, text)) = match receiver.view() {
                        View::Bool(true) => Some(("__true_to_s", "true")),
                        View::Bool(false) => Some(("__false_to_s", "false")),
                        _ => None,
                    }
                {
                    return Ok(Some(self.memoized_text(slot, text)));
                }
                if let View::Symbol(s) = receiver.view() {
                    // A name past ASCII in an encoding other than the one the
                    // answer is written in cannot be shown as it is, so it is
                    // quoted and escaped the way the String would be.
                    let named = s.encoding_name();
                    let writes_otherwise =
                        named == "UTF-8" && self.inspect_result_encoding() != "UTF-8";
                    if method_name != "to_s"
                        && (!matches!(named.as_str(), "UTF-8" | "US-ASCII" | "ASCII-8BIT")
                            || writes_otherwise)
                        && crate::vm::native_methods::string_methods::encoding_is_ascii_compatible(
                            &named,
                        )
                        && !crate::vm::native_methods::string_methods::binary_bytes(s).is_ascii()
                    {
                        let made = crate::object::StringValue::with_encoding(s.to_text(), named);
                        if s.holds_bytes() {
                            made.mark_bytes();
                        }
                        let text = Object::String(std::rc::Rc::new(made));
                        let shown = self.send_to_object(text, "inspect", Vec::new(), position)?;
                        let (shown, written_in) = match shown.view() {
                            View::String(held) => (held.as_str().to_string(), held.encoding_name()),
                            _ => (String::new(), self.inspect_result_encoding()),
                        };
                        return Ok(Some(Object::String(std::rc::Rc::new(
                            crate::object::StringValue::with_encoding(
                                format!(":{shown}"),
                                written_in,
                            ),
                        ))));
                    }
                    if method_name != "to_s" {
                        return Ok(Some(Object::string(
                            crate::vm::native_methods::string_methods::symbol_inspect_text(s),
                        )));
                    }
                    // The name keeps the encoding it was interned in, bytes
                    // and all, which is what `Symbol#encoding` reports.
                    let made =
                        crate::object::StringValue::with_encoding(s.to_text(), s.encoding_name());
                    if s.holds_bytes() {
                        made.mark_bytes();
                    }
                    return Ok(Some(Object::String(std::rc::Rc::new(made))));
                }
                if method_name == "inspect"
                    && let View::Instance(_) = receiver.view()
                {
                    return self.default_instance_inspect(receiver, position).map(Some);
                }
                // A callable describes itself in bytes rather than in text,
                // which is what Ruby tags the description with.
                if matches!(receiver.view(), View::Block(_)) {
                    return Ok(Some(Object::String(std::rc::Rc::new(
                        crate::object::StringValue::with_encoding(
                            receiver.to_string(),
                            "ASCII-8BIT",
                        ),
                    ))));
                }
                Ok(Some(Object::string(receiver.to_string())))
            }
            "class" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // For exceptions, return the specific exception class, not generic Exception
                if let View::Exception(exc_ref) = receiver.view() {
                    if let Some(class) = exc_ref.borrow().class.clone() {
                        return Ok(Some(Object::Class(class)));
                    }
                    let exc_type = exc_ref.borrow().exception_type.clone();
                    if let Some(View::Class(cls)) = self.globals().get(&exc_type).view() {
                        return Ok(Some(Object::Class(cls.clone())));
                    }
                    // A namespaced type such as `Errno::EINVAL` lives as a
                    // constant on its module rather than at the top level.
                    if let Some(class @ View::Class(_)) =
                        self.resolve_qualified_constant(&exc_type).view()
                    {
                        return Ok(Some(class.to_object()));
                    }
                }
                // A module made by `new` on a subclass of Module is an
                // instance of that subclass.
                if let View::Module(module) = receiver.view()
                    && let Some(made_from) = module.instance_of()
                {
                    return Ok(Some(Object::Class(made_from)));
                }
                // A class is an instance of Class, a module an instance of
                // Module. `class_of` answers Object for both because it drives
                // `is_a?` over the inheritance chain.
                // An encoding is held as a class whose parent is Encoding,
                // and what it stands for is an Encoding rather than a class.
                if self.names_an_encoding(receiver)
                    && let Some(held @ View::Class(_)) = self.globals().get("Encoding").view()
                {
                    return Ok(Some(held.to_object()));
                }
                match receiver.view() {
                    View::Class(_) => {
                        if let Some(View::Class(cls)) = self.globals().get("Class").view() {
                            return Ok(Some(Object::Class(cls.clone())));
                        }
                    }
                    View::Module(_) => {
                        if let Some(View::Class(cls)) = self.globals().get("Module").view() {
                            return Ok(Some(Object::Class(cls.clone())));
                        }
                    }
                    _ => {}
                }
                // For booleans and nil, return the specific class
                match receiver.view() {
                    View::Bool(true) => {
                        if let Some(View::Class(cls)) = self.globals().get("TrueClass").view() {
                            return Ok(Some(Object::Class(cls.clone())));
                        }
                    }
                    View::Bool(false) => {
                        if let Some(View::Class(cls)) = self.globals().get("FalseClass").view() {
                            return Ok(Some(Object::Class(cls.clone())));
                        }
                    }
                    View::Nil => {
                        if let Some(View::Class(cls)) = self.globals().get("NilClass").view() {
                            return Ok(Some(Object::Class(cls.clone())));
                        }
                    }
                    _ => {}
                }
                Ok(Some(Object::Class(self.builtins().class_of(receiver))))
            }
            _ => Ok(None),
        }
    }
}
