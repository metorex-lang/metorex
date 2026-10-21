// Whether two values are equal, and whether one covers the other.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Whether two values are equal, which each kind of value answers its
    /// own way.
    pub(crate) fn evaluate_equality(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // by asking that object instead, which is how a class that
        // stands for a number reports equality with one.
        if is_number(&left) && takes_coercion(&right) {
            let answer = self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Two patterns are the same when their source and flags are.
        if let (View::Regex(pattern, flags), View::Regex(other, other_flags)) =
            (left.view(), right.view())
        {
            use crate::vm::native_methods::comparable_flags;
            return Ok(Object::Bool(
                pattern == other && comparable_flags(flags) == comparable_flags(other_flags),
            ));
        }
        // Two hashes compare entry by entry, looking each key up in
        // the other hash the way any key is looked up, which Hash
        // answers from its own method table.
        if matches!((left.view(), right.view()), (View::Dict(_), View::Dict(_)))
            && let Some(answer) =
                self.call_hash_method(&left, "==", std::slice::from_ref(&right), position)?
        {
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Two ranges compare by their ends and their exclusivity,
        // which Range answers from its own method table.
        if matches!((left.view(), &right), (View::Range { .. }, _)) {
            let class = self.builtins().class_of(&left);
            if let Some(answer) = self.call_native_method(
                &class,
                &left,
                "==",
                std::slice::from_ref(&right),
                position,
            )? {
                return Ok(Object::Bool(answer.is_truthy()));
            }
        }
        // For instances, dispatch to user-defined == method if present,
        // or to <=> (Comparable protocol) if the class has <=> defined.
        // A plain String, Array, Hash, Set or Regexp compared with an
        // instance of a subclass of one compares with what that instance
        // holds.
        if !matches!(left.view(), View::Instance(_))
            && let Some(held) = crate::vm::native_methods::subclass_backing(&right)
        {
            return self.evaluate_equality(left, held, position);
        }
        // A module made by `new` on a subclass of Module compares with the
        // `==` that subclass or one of its modules writes.
        if let View::Module(made) = left.view()
            && made.instance_of().is_some()
            && let Some((owner, method)) = self.lookup_method(&left, "==")
            && !method.is_undefined
            && !method.body.is_empty()
        {
            return self.invoke_method(owner, method, left.clone(), vec![right], position);
        }
        // An exception whose class writes `==` answers with it.
        if matches!(left.view(), View::Exception(_))
            && let Some((owner, method)) = self.lookup_method(&left, "==")
            && !method.is_undefined
            && !method.body.is_empty()
        {
            let answer = self.invoke_method(owner, method, left.clone(), vec![right], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        if let View::Instance(inst_rc) = left.view() {
            // A class that undefined `==` answers it through its
            // `method_missing`, an object compared with itself included.
            if self
                .lookup_method(&left, "==")
                .is_some_and(|(_, method)| method.is_undefined)
            {
                if let Some(answer) =
                    self.undefined_hook_via_method_missing(&left, "==", &right, position)
                {
                    return answer;
                }
                let arguments = [right.clone()];
                let wording = self.receiver_wording_for(&left, position);
                return Err(crate::vm::errors::undefined_method_error_worded(
                    "==", &left, &arguments, wording, position,
                ));
            }
            // Identity shortcut: same object is always ==
            if let View::Instance(rhs) = right.view()
                && Rc::ptr_eq(inst_rc, rhs)
            {
                return Ok(Object::Bool(true));
            }
            // An instance of a subclass of String, Array, Hash, Set or
            // Regexp that writes no `==` of its own compares what it holds.
            if let Some(held) = crate::vm::native_methods::subclass_backing(&left) {
                let written = self
                    .lookup_method(&left, "==")
                    .is_some_and(|(_, method)| !method.is_undefined && !method.body.is_empty());
                if !written {
                    let other =
                        crate::vm::native_methods::subclass_backing(&right).unwrap_or(right);
                    return self.evaluate_equality(held, other, position);
                }
            }
            // A Struct that writes no `==` of its own compares its members.
            let instance_class = Rc::clone(&inst_rc.borrow().class);
            if let Some(members) =
                crate::vm::native_methods::struct_methods::struct_members(&instance_class)
            {
                let written = self
                    .lookup_method(&left, "==")
                    .is_some_and(|(_, method)| !method.is_undefined && !method.body.is_empty());
                if !written
                    && let Some(answer) = self.call_struct_instance_method(
                        &instance_class,
                        &members,
                        &left,
                        "==",
                        std::slice::from_ref(&right),
                        position,
                    )?
                {
                    return Ok(Object::Bool(answer.is_truthy()));
                }
            }
            // A stub standing in for a native `==` may come back here with
            // the same pair, and then it is not asked again.
            let pair = (Rc::as_ptr(inst_rc) as usize, identity_of(&right));
            let is_stub = |method: &crate::object::Method| {
                method.body.is_empty() && method.captured_vars.is_none()
            };
            if let Some((class, method)) = self.lookup_method(&left, "==")
                && !method.is_undefined
                && !(is_stub(&method) && STUB_PAIRS.with(|held| held.borrow().contains(&pair)))
            {
                let through_stub = is_stub(&method);
                if through_stub {
                    STUB_PAIRS.with(|held| held.borrow_mut().push(pair));
                }
                let result =
                    self.invoke_method(class, method, left.clone(), vec![right.clone()], position);
                if through_stub {
                    STUB_PAIRS.with(|held| held.borrow_mut().pop());
                }
                return Ok(Object::Bool(result?.is_truthy()));
            }
            // Rational and Complex answer `==` from their native
            // tables rather than a method map, and a Complex with no
            // imaginary part equals the plain number it holds.
            let instance_class = Rc::clone(&inst_rc.borrow().class);
            if matches!(instance_class.name(), "Rational" | "Complex")
                && let Some(result) = self.call_native_method(
                    &instance_class,
                    &left,
                    "==",
                    std::slice::from_ref(&right),
                    position,
                )?
            {
                return Ok(Object::Bool(result.is_truthy()));
            }
            // Comparable's `==` asks `<=>`. Object's `==` is identity, which
            // the check above answered.
            if self.is_comparable(&left)
                && let Some((cmp_class, cmp_method)) = self.lookup_method(&left, "<=>")
                && !cmp_method.is_undefined
            {
                let cmp_obj = self.invoke_method(
                    cmp_class,
                    cmp_method,
                    left.clone(),
                    vec![right.clone()],
                    position,
                )?;
                return match cmp_obj.view() {
                    View::Int(n) => Ok(Object::Bool(n == 0)),
                    View::Float(f) => Ok(Object::Bool(f == 0.0)),
                    View::Nil => Ok(Object::Bool(false)),
                    _ => {
                        // Ruby compares what `<=>` returned with 0, which a
                        // value that is not a number refuses.
                        let msg = format!(
                            "comparison of {} with 0 failed",
                            self.class_name_of(&cmp_obj)
                        );
                        let exc = Object::exception("ArgumentError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                };
            }
        }
        // A Process::Status compares by the number it stands for, so
        // it answers `==` against that number as well as against
        // another status.
        if matches!(left.view(), View::Instance(held) if held.borrow().class.name() == "Process::Status")
            && let Some(answer) = self.call_process_status_method(
                &left,
                "==",
                std::slice::from_ref(&right),
                position,
            )?
        {
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // A Set decides equality itself, since it counts anything
        // that says it is one, however that value is built.
        if matches!(left.view(), View::Set(_))
            && let Some(answer) = self.call_native_method(
                &self.builtins().class_of(&left),
                &left,
                "==",
                std::slice::from_ref(&right),
                position,
            )?
        {
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Two hashes holding objects of the program's own are equal
        // when those objects say so, so each value is asked with `==`
        // rather than compared as data.
        if let (View::Dict(one), View::Dict(other)) = (left.view(), right.view()) {
            if Rc::ptr_eq(one, other) {
                return Ok(Object::Bool(true));
            }
            let held: Vec<(String, Object)> = one
                .borrow()
                .iter()
                .filter(|(key, _)| !key.starts_with("__MX_"))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            let against: Vec<(String, Object)> = other
                .borrow()
                .iter()
                .filter(|(key, _)| !key.starts_with("__MX_"))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            if held.iter().chain(against.iter()).any(|(_, value)| {
                    matches!(value.view(), View::Instance(_))
                        || matches!(value.view(), View::Array(items) if items.borrow().iter().any(|item| matches!(item.view(), View::Instance(_))))
                }) {
                    if held.len() != against.len() {
                        return Ok(Object::Bool(false));
                    }
                    for (key, value) in &held {
                        let Some((_, counterpart)) =
                            against.iter().find(|(other_key, _)| other_key == key)
                        else {
                            return Ok(Object::Bool(false));
                        };
                        let answer = self.evaluate_binary_operation(
                            &BinaryOp::Equal,
                            value.clone(),
                            counterpart.clone(),
                            position,
                        )?;
                        if !answer.is_truthy() {
                            return Ok(Object::Bool(false));
                        }
                    }
                    return Ok(Object::Bool(true));
                }
        }
        // Two arrays are equal when their elements are, which asks
        // each pair rather than comparing the two as data.
        if matches!(
            (left.view(), right.view()),
            (View::Array(_), View::Array(_))
        ) {
            let same = self.arrays_equal(&left, &right, position)?;
            return Ok(Object::Bool(same));
        }
        // An object that spells itself as an array answers for the
        // pair, which is how a wrapper around one compares.
        if matches!(left.view(), View::Array(_))
            && matches!(right.view(), View::Instance(_))
            && crate::vm::native_methods::array_subclass_value(&right).is_none()
            && self
                .send_to_object(
                    right.clone(),
                    "respond_to?",
                    vec![Object::symbol("to_ary".to_string())],
                    position,
                )?
                .is_truthy()
        {
            let answer = self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Two strings are equal when they spell the same thing,
        // whether that is read as bytes or as characters, and their
        // encodings can be compared at all.
        if let (View::String(text), View::String(other)) = (left.view(), right.view()) {
            use crate::vm::native_methods::string_methods::{binary_bytes, strings_comparable};
            return Ok(Object::Bool(
                (binary_bytes(text) == binary_bytes(other) || *text.as_str() == *other.as_str())
                    && strings_comparable(text, other),
            ));
        }
        // Anything else that spells itself as text answers for the
        // pair, which is how a wrapper around a String compares.
        if matches!(left.view(), View::String(_))
            && crate::vm::native_methods::string_subclass_value(&right).is_none()
            && self.responds_to(&right, "to_str")
        {
            let answer = self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        Ok(Object::Bool(left.equals(&right)))
    }

    /// Whether the left operand covers the right, which is what a `when`
    /// clause and a pattern ask.
    pub(crate) fn evaluate_case_equality(
        &mut self,
        left: Object,
        right: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        use BinaryOp::*;
        // A number's `===` is its `==`, so it too answers by asking an
        // operand that is not a number.
        if is_number(&left) && takes_coercion(&right) {
            let answer = self.send_to_object(right.clone(), "==", vec![left.clone()], position)?;
            return Ok(Object::Bool(answer.is_truthy()));
        }
        // Regexp === str: whether the pattern matches anywhere. A
        // Symbol matches on its name, and an object that answers
        // `to_str` on the characters it hands over.
        if let View::Regex(pattern, flags) = left.view() {
            let subject = match right.view() {
                View::String(text) | View::Symbol(text) => Some(text.as_str().to_string()),
                _ => match crate::vm::native_methods::subject_text(&right) {
                    Some(text) => Some(text),
                    None if self.responds_to(&right, "to_str") => {
                        match self
                            .send_to_object(right.clone(), "to_str", vec![], position)?
                            .view()
                        {
                            View::String(text) => Some(text.as_str().to_string()),
                            _ => None,
                        }
                    }
                    None => None,
                },
            };
            let Some(subject) = subject else {
                return Ok(Object::Bool(false));
            };
            let (pattern, flags) = (pattern.as_str().to_string(), flags.as_str().to_string());
            let encoding = crate::vm::native_methods::regexp_methods::subject_encoding(&right);
            let found =
                self.regexp_match_data_in(&pattern, &flags, &subject, 0, encoding, position)?;
            return Ok(Object::Bool(found.is_some()));
        }
        // A callable answers for itself: `===` hands the value to
        // the proc and reports whatever it returns, so a proc can
        // stand in for a pattern in a `case`.
        if let View::Block(block) = left.view() {
            return block.call(self, vec![right], position);
        }
        // A Set holds a value or it does not, which is the branch a
        // `case` over one picks.
        if matches!(left.view(), View::Set(_)) {
            let class = self.builtins().class_of(&left);
            if let Some(answer) = self.call_native_method(
                &class,
                &left,
                "include?",
                std::slice::from_ref(&right),
                position,
            )? {
                return Ok(Object::Bool(answer.is_truthy()));
            }
        }
        // A Range answers `===` the way `cover?` does for a single value,
        // which is what `case` uses to pick a branch.
        if matches!(left.view(), View::Range { .. }) {
            let class = self.builtins().class_of(&left);
            if let Some(answer) = self.call_native_method(
                &class,
                &left,
                "===",
                std::slice::from_ref(&right),
                position,
            )? {
                return Ok(Object::Bool(answer.is_truthy()));
            }
        }
        // Class/Module === obj: check type membership (Ruby's case
        // equality). Modules also count as the "type test" form so
        // `SomeMod === obj` works the same as `obj.is_a?(SomeMod)`.
        let class_rc_opt = match left.view() {
            View::Class(c) | View::Module(c) => Some(c),
            _ => None,
        };
        // An encoding is held as a class below Encoding, and answers
        // Kernel's `===`, which is whether the two are the same encoding.
        if let Some(class_rc) = class_rc_opt
            && class_rc
                .superclass()
                .is_some_and(|parent| parent.name() == "Encoding" && parent.superclass().is_none())
        {
            return Ok(Object::Bool(left.equals(&right)));
        }
        if let Some(class_rc) = class_rc_opt {
            if let View::Exception(exc_ref) = right.view() {
                // Check if exception type matches or is a subclass
                let exc_type = exc_ref.borrow().exception_type.clone();
                if exc_type == class_rc.name() {
                    return Ok(Object::Bool(true));
                }
                // Check the exception class hierarchy. A namespaced
                // type such as `Errno::EINVAL` is a constant on its
                // module rather than a top-level name.
                let carried = exc_ref.borrow().class.clone().map(Object::Class);
                let resolved = match carried {
                    Some(class) => Some(class),
                    None => match self.globals().get(&exc_type).view() {
                        Some(class @ View::Class(_)) => Some(class.to_object()),
                        _ => self.resolve_qualified_constant(&exc_type),
                    },
                };
                if let Some(View::Class(exc_class)) = resolved.view() {
                    return Ok(Object::Bool(
                        self.builtins().is_subclass_of(exc_class, class_rc),
                    ));
                }
                return Ok(Object::Bool(false));
            }
            // A module made by a subclass of Module is an object of that
            // subclass, of what it inherits from, and of what they include.
            if let View::Module(made) = right.view()
                && let Some(made_from) = made.instance_of()
                && crate::vm::eval::super_expr::walk_ancestors(&made_from)
                    .iter()
                    .any(|ancestor| Rc::ptr_eq(ancestor, class_rc))
            {
                return Ok(Object::Bool(true));
            }
            // A class or a module is an object of Class or of
            // Module, which is what makes `Module === String` true.
            if matches!(right.view(), View::Class(_) | View::Module(_)) {
                let meta_name = match right.view() {
                    View::Class(_) => "Class",
                    _ => "Module",
                };
                if let Some(View::Class(meta)) = self.globals().get(meta_name).view()
                    && self.builtins().is_subclass_of(meta, class_rc)
                {
                    return Ok(Object::Bool(true));
                }
            }
            // `Left === right` asks whether the right stands as one
            // of the left, which is `right.is_a?(left)` whatever the
            // right is. A class is an object too, so `Module ===
            // String` is true and `String === String` is false.
            {
                if crate::builtin_classes::value_class_name(&right)
                    .is_some_and(|name| name == class_rc.name())
                {
                    return Ok(Object::Bool(true));
                }
                let right_class = self.builtins().class_of(&right);
                if self.builtins().is_subclass_of(&right_class, class_rc) {
                    return Ok(Object::Bool(true));
                }
                if let View::Instance(inst_rc) = right.view() {
                    let sc_opt = inst_rc.borrow().singleton_class.borrow().clone();
                    if let Some(sc) = sc_opt
                        && self.builtins().is_subclass_of(&sc, class_rc)
                    {
                        return Ok(Object::Bool(true));
                    }
                }
                return Ok(Object::Bool(false));
            }
        }
        // Object#=== is the same object, or whatever #== says. It
        // consults neither #equal? nor #object_id, so a class that
        // overrides those does not change the answer. A class that
        // defines its own `===` wins over that default, and an object
        // rooted at BasicObject, which has no `===`, reaches one only
        // through `method_missing`.
        if let View::Instance(inst_rc) = left.view() {
            if let Some((owner, method)) = self.lookup_method(&left, "===")
                && !method.is_undefined
            {
                return self.invoke_method(owner, method, left, vec![right], position);
            }
            if self.rooted_at_basic_object(&left)
                && let Some((owner, handler)) = self.lookup_method(&left, "method_missing")
                && !handler.is_undefined
            {
                let arguments = vec![Object::symbol("===".to_string()), right];
                return self.invoke_method(owner, handler, left, arguments, position);
            }
            if let View::Instance(rhs) = right.view()
                && Rc::ptr_eq(inst_rc, rhs)
            {
                return Ok(Object::Bool(true));
            }
            return self.evaluate_binary_operation(&Equal, left, right, position);
        }
        // A String's `===` is its `==`, down to the encodings it
        // refuses to compare.
        if matches!(left.view(), View::String(_)) {
            return self.evaluate_binary_operation(&Equal, left, right, position);
        }
        Ok(Object::Bool(left.equals(&right)))
    }
}

thread_local! {
    /// The pairs being compared through a stub standing in for a native
    /// `==`, which that native method may ask about again.
    static STUB_PAIRS: std::cell::RefCell<Vec<(usize, usize)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// A number standing for which object a value is, for the values compared
/// through a stub.
pub(crate) fn identity_of(value: &Object) -> usize {
    match value.view() {
        View::Instance(held) => Rc::as_ptr(held) as usize,
        View::Array(held) => Rc::as_ptr(held) as usize,
        View::Dict(held) => Rc::as_ptr(held) as usize,
        View::String(held) => Rc::as_ptr(held) as usize,
        View::Regex(held, _) => Rc::as_ptr(held) as usize,
        _ => 0,
    }
}
