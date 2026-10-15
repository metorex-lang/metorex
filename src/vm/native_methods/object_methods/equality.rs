// Whether two objects count as equal, the copies of one, and matching
// against a pattern.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Whether two objects count as equal, the copies of one, and matching
    /// against a pattern.
    pub(crate) fn call_object_equality_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `eql?` is equality without conversion, so a number equals only
            // one of its own kind: `1.0.eql?(1)` is false where `1.0 == 1` is
            // true.
            "eql?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Object's own `eql?` is identity. Asking `values_eql` would
                // send `eql?` back to this very method.
                if let View::Instance(instance) = receiver.view()
                    && crate::vm::native_methods::array_subclass_value(receiver).is_none()
                    && crate::vm::native_methods::string_subclass_value(receiver).is_none()
                {
                    let same = matches!(arguments[0].view(), View::Instance(other) if std::rc::Rc::ptr_eq(instance, other));
                    return Ok(Some(Object::Bool(same)));
                }
                Ok(Some(Object::Bool(self.values_eql(
                    receiver,
                    &arguments[0],
                    position,
                )?)))
            }
            "equal?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = &arguments[0];
                let identity = match (receiver.view(), other.view()) {
                    (View::Instance(a), View::Instance(b)) => std::rc::Rc::ptr_eq(a, b),
                    // Two strings holding the same text are still two
                    // strings, which is what `equal?` tells apart.
                    (View::String(a), View::String(b)) => std::rc::Rc::ptr_eq(a, b),
                    (View::Array(a), View::Array(b)) => std::rc::Rc::ptr_eq(a, b),
                    (View::Dict(a), View::Dict(b)) => std::rc::Rc::ptr_eq(a, b),
                    (View::Class(a), View::Class(b)) => std::rc::Rc::ptr_eq(a, b),
                    (View::Module(a), View::Module(b)) => std::rc::Rc::ptr_eq(a, b),
                    (View::Set(a), View::Set(b)) => std::rc::Rc::ptr_eq(a, b),
                    (View::Exception(a), View::Exception(b)) => std::rc::Rc::ptr_eq(a, b),
                    // Two ranges over the same values are still two ranges,
                    // which the mark each carries tells apart.
                    (View::Range { mark: a, .. }, View::Range { mark: b, .. }) => {
                        std::rc::Rc::ptr_eq(a, b)
                    }
                    // An integer past the i64 range is its own object, so two
                    // of the same value are not identical.
                    (View::BigInt(a), View::BigInt(b)) => std::rc::Rc::ptr_eq(a, b),
                    // A block can close over itself, so comparing two of them
                    // structurally would never terminate. Ruby's identity
                    // check is the address anyway.
                    (View::Block(a), View::Block(b)) => std::rc::Rc::ptr_eq(a, b),
                    // A Method or an UnboundMethod compares `==` by what it
                    // names, so two copies of one are equal without being
                    // the same object.
                    (View::Method(a), View::Method(b)) => std::rc::Rc::ptr_eq(a, b),
                    (View::Binding(a), View::Binding(b)) => std::rc::Rc::ptr_eq(a, b),
                    // A Float is a value, so two of the same bits are the same
                    // object. That covers NaN, which is never `==` itself but
                    // is identical to itself.
                    (View::Float(a), View::Float(b)) => a.to_bits() == b.to_bits(),
                    // For value types, equal? is the same as ==
                    (a, b) => a == b,
                };
                Ok(Some(Object::Bool(identity)))
            }
            "dup" | "clone" => {
                // `clone` takes a `freeze:` keyword naming what the copy's
                // frozen state should be; `dup` takes nothing at all.
                let mut freeze = None;
                if method_name == "clone" {
                    freeze = self.clone_freeze_argument(arguments, position)?;
                } else if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let copy = self.copy_for(receiver, position)?;
                // `dup` answers a range that is not frozen, while `clone`
                // keeps the frozen state the original carried.
                if let Some(View::Range { mark, .. }) = copy.view()
                    && (method_name == "dup" || matches!(freeze, Some(false)))
                {
                    self.thawed_ranges
                        .insert(std::rc::Rc::as_ptr(mark) as usize, std::rc::Rc::clone(mark));
                }
                if let Some(copy) = &copy {
                    if method_name == "clone" {
                        self.finish_clone(receiver, copy, freeze, arguments, position)?;
                    } else if let Some((class, method)) = self.lookup_method(copy, "initialize_dup")
                        && !method.is_undefined
                        && !method.body.is_empty()
                    {
                        // A class of the program's own may shape what `dup`
                        // hands back, which it does through this hook.
                        self.invoke_method(
                            class,
                            method,
                            copy.clone(),
                            vec![receiver.clone()],
                            position,
                        )?;
                    }
                }
                Ok(copy)
            }
            // BasicObject's negation, which every object answers.
            "!" if arguments.is_empty() => Ok(Some(Object::Bool(!receiver.is_truthy()))),
            // Only a String, a Regexp, a Symbol and nil answer `=~`; Ruby no
            // longer gives every object one.
            "=~" if !matches!(
                receiver.view(),
                View::String(_) | View::Regex(_, _) | View::Symbol(_) | View::Nil
            ) =>
            {
                Ok(None)
            }
            "=~" => {
                // Regex match: string =~ regex or regex =~ string
                if arguments.len() != 1 {
                    return Err(method_argument_error("=~", 1, arguments.len(), position));
                }
                // A Symbol matches as its name, and text against text is no
                // match at all.
                let reads_as_text = |value: &Object| {
                    matches!(value.view(), View::String(_) | View::Symbol(_))
                        || crate::vm::native_methods::string_subclass_value(value).is_some()
                };
                if reads_as_text(receiver) && reads_as_text(&arguments[0]) {
                    return Err(crate::vm::errors::simple_exception(
                        "TypeError",
                        "type mismatch: String given",
                        position,
                    ));
                }
                // Text matched against anything but a pattern asks that
                // object, with the text, the way `String#=~` does.
                if let View::String(_) | View::Symbol(_) = receiver.view()
                    && !matches!(arguments[0].view(), View::Regex(_, _))
                {
                    let text = match receiver.view() {
                        View::Symbol(name) => Object::string(name.as_str().to_string()),
                        _ => receiver.clone(),
                    };
                    return self
                        .send_to_object(arguments[0].clone(), "=~", vec![text], position)
                        .map(Some);
                }
                // A Symbol matches on the characters it is named with, so
                // `/_pri\z/ =~ :ds_pri` finds a match the way Ruby's does.
                match (matchable_text(receiver), matchable_text(&arguments[0])) {
                    (Some(MatchSide::Pattern(pattern, flags)), Some(MatchSide::Text(text)))
                    | (Some(MatchSide::Text(text)), Some(MatchSide::Pattern(pattern, flags))) => {
                        let (pattern_side, subject_side) = match receiver.view() {
                            View::Regex(_, _) => (receiver, &arguments[0]),
                            _ => (&arguments[0], receiver),
                        };
                        if let View::Regex(written, _) = pattern_side.view() {
                            self.prepare_match_subject(written, &flags, subject_side, position)?;
                        }
                        let encoding =
                            crate::vm::native_methods::regexp_methods::subject_encoding(receiver)
                                .or_else(|| {
                                    crate::vm::native_methods::regexp_methods::subject_encoding(
                                        &arguments[0],
                                    )
                                });
                        match self
                            .regexp_match_data_in(&pattern, &flags, &text, 0, encoding, position)?
                        {
                            Some(data) => self
                                .send_to_object(data, "begin", vec![Object::Int(0)], position)
                                .map(Some),
                            None => Ok(Some(Object::Nil)),
                        }
                    }
                    // A string matched against anything but a pattern hands
                    // the match to that other object, and refuses outright
                    // when it is another string.
                    (Some(MatchSide::Text(_)), _) if matches!(receiver.view(), View::String(_)) => {
                        let other = &arguments[0];
                        if matches!(other.view(), View::String(_))
                            || crate::vm::native_methods::string_subclass_value(other).is_some()
                        {
                            let named = self.builtins().class_of(other).name().to_string();
                            return Err(crate::vm::errors::simple_exception(
                                "TypeError",
                                &format!("type mismatch: {} given", named),
                                position,
                            ));
                        }
                        self.send_to_object(other.clone(), "=~", vec![receiver.clone()], position)
                            .map(Some)
                    }
                    _ => Ok(Some(Object::Nil)),
                }
            }
            _ => Ok(None),
        }
    }
}
