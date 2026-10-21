// The instance methods every generated struct class inherits.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// The instance methods every generated struct class inherits from Struct.
    pub(crate) fn call_struct_instance_method(
        &mut self,
        class_rc: &Rc<Class>,
        members: &[String],
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // A method the struct class or one of its mixins defines wins over
        // Struct's own, which is how `include`ing a module that defines
        // `hash` replaces it.
        // A member's own reader or writer is the one Struct.new wrote, and
        // one the program wrote over it answers in its place.
        if let Some((_, method)) = self.lookup_method(receiver, method_name)
            && !method.body.is_empty()
            && !is_generated_member_accessor(&method, method_name, members)
            && !self.enumerable_stands_in(receiver, method_name)
        {
            return Ok(None);
        }

        // A member accessor wins over Struct's own method of the same name,
        // so `Struct.new(:length).new(42).length` answers 42, not 1.
        if arguments.is_empty() && members.iter().any(|member| member == method_name) {
            return Ok(Some(member_value(receiver, method_name)));
        }
        if arguments.len() == 1
            && let Some(target) = method_name.strip_suffix('=')
            && members.iter().any(|member| member == target)
            && let View::Instance(instance) = receiver.view()
        {
            if self.object_is_frozen(receiver) {
                return Err(self.frozen_modification_error(receiver, position));
            }
            instance
                .borrow_mut()
                .instance_vars
                .insert(member_slot(target), arguments[0].clone());
            return Ok(Some(arguments[0].clone()));
        }

        match method_name {
            "members" => Ok(Some(symbols(members))),
            "size" | "length" => Ok(Some(Object::Int(members.len() as i64))),
            "to_a" | "values" | "deconstruct" => Ok(Some(Object::Array(Rc::new(RefCell::new(
                member_values(receiver, members),
            ))))),
            "to_h" => {
                let viewed = self.pending_block.take();
                let block = match viewed.view() {
                    Some(View::Block(block)) => Some(block),
                    _ => None,
                };
                let mut pairs = IndexMap::new();
                for member in members {
                    let value = member_value(receiver, member);
                    let (key, value) = match &block {
                        None => (Object::symbol(member.clone()), value),
                        Some(block) => {
                            let produced = self.execute_block_callable(
                                block,
                                vec![Object::symbol(member.clone()), value],
                                position,
                            )?;
                            self.pair_from_block_result(produced, position)?
                        }
                    };
                    let rendered = crate::vm::utils::object_to_dict_key(&key).unwrap_or_default();
                    if !crate::vm::utils::is_primitive_key(&key) {
                        remember_key_object(&mut pairs, &rendered, &key);
                    }
                    pairs.insert(rendered, value);
                }
                Ok(Some(Object::Dict(Rc::new(RefCell::new(pairs)))))
            }
            "deconstruct_keys" => {
                if arguments.len() != 1 {
                    return Err(argument_error(
                        format!(
                            "wrong number of arguments (given {}, expected 1)",
                            arguments.len()
                        ),
                        position,
                    ));
                }
                self.deconstruct_struct_keys(members, receiver, &arguments[0], position)
                    .map(Some)
            }
            "[]" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let member = resolve_member(members, &arguments[0], position)?;
                Ok(Some(member_value(receiver, &member)))
            }
            "[]=" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let member = resolve_member(members, &arguments[0], position)?;
                if self.object_is_frozen(receiver) {
                    return Err(self.frozen_modification_error(receiver, position));
                }
                if let View::Instance(instance) = receiver.view() {
                    instance
                        .borrow_mut()
                        .instance_vars
                        .insert(member_slot(&member), arguments[1].clone());
                }
                Ok(Some(arguments[1].clone()))
            }
            "dig" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(method_name, 1, 0, position));
                }
                let member = match resolve_member(members, &arguments[0], position) {
                    Ok(member) => member,
                    Err(_) => return Ok(Some(Object::Nil)),
                };
                let value = member_value(receiver, &member);
                if arguments.len() == 1 || matches!(value.view(), View::Nil) {
                    return Ok(Some(if arguments.len() == 1 {
                        value
                    } else {
                        Object::Nil
                    }));
                }
                self.dig_into(&value, &arguments[1..], position).map(Some)
            }
            "values_at" => {
                let values = member_values(receiver, members);
                let mut picked = Vec::with_capacity(arguments.len());
                for key in arguments {
                    match key.view() {
                        View::Int(index) => {
                            picked.push(indexed_value(&values, index, position)?);
                        }
                        View::Range { .. } => {
                            picked.extend(range_values(&values, key, position)?);
                        }
                        _ => {
                            return Err(MetorexError::type_error(
                                format!(
                                    "no implicit conversion of {} into Integer",
                                    crate::vm::errors::conversion_subject(key)
                                ),
                                position_to_location(position),
                            ));
                        }
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(picked)))))
            }
            "select" | "filter" => {
                let viewed = self.pending_block.take();
                let Some(View::Block(block)) = viewed.view() else {
                    if !arguments.is_empty() {
                        return Err(argument_error(
                            format!(
                                "wrong number of arguments (given {}, expected 0)",
                                arguments.len()
                            ),
                            position,
                        ));
                    }
                    return self
                        .build_enumerator(
                            receiver.clone(),
                            method_name,
                            arguments.to_vec(),
                            Some(members.len() as i64),
                            position,
                        )
                        .map(Some);
                };
                if !arguments.is_empty() {
                    return Err(argument_error(
                        format!(
                            "wrong number of arguments (given {}, expected 0)",
                            arguments.len()
                        ),
                        position,
                    ));
                }
                let mut kept = Vec::new();
                for value in member_values(receiver, members) {
                    let verdict =
                        self.execute_block_callable(block, vec![value.clone()], position)?;
                    if verdict.is_truthy() {
                        kept.push(value);
                    }
                }
                Ok(Some(Object::Array(Rc::new(RefCell::new(kept)))))
            }
            "each" | "each_pair" => {
                let viewed = self.pending_block.take();
                let Some(View::Block(block)) = viewed.view() else {
                    return self
                        .build_enumerator(
                            receiver.clone(),
                            method_name,
                            arguments.to_vec(),
                            Some(members.len() as i64),
                            position,
                        )
                        .map(Some);
                };
                for member in members {
                    // `each_pair` yields each member and its value as one
                    // pair, which a block taking two parameters spreads.
                    let args = if method_name == "each" {
                        vec![member_value(receiver, member)]
                    } else {
                        vec![Object::array(vec![
                            Object::symbol(member.clone()),
                            member_value(receiver, member),
                        ])]
                    };
                    self.execute_block_with_control_flow(block, args, position)?;
                }
                Ok(Some(receiver.clone()))
            }
            "==" | "eql?" => {
                let Some(other) = arguments.first() else {
                    return Err(method_argument_error(method_name, 1, 0, position));
                };
                Ok(Some(Object::Bool(self.struct_equals(
                    class_rc,
                    members,
                    receiver,
                    other,
                    method_name == "eql?",
                ))))
            }
            "hash" => {
                let rendered = format!(
                    "{}[{}]({})",
                    class_rc.ruby_name(),
                    members.join(","),
                    member_values(receiver, members)
                        .iter()
                        .map(crate::vm::native_methods::array_methods::inspect_element)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let mut digest: i64 = 0;
                for byte in rendered.bytes() {
                    digest = digest.wrapping_mul(31).wrapping_add(byte as i64);
                }
                Ok(Some(Object::Int(digest)))
            }
            "inspect" | "to_s" => {
                // A struct reached again while it is being inspected shows
                // as its class alone.
                let address = match receiver.view() {
                    View::Instance(instance) => std::rc::Rc::as_ptr(instance) as usize,
                    _ => 0,
                };
                if STRUCTS_BEING_INSPECTED.with(|held| held.borrow().contains(&address)) {
                    return Ok(Some(Object::string(format!(
                        "#<struct {}:...>",
                        class_rc.inspect_name()
                    ))));
                }
                STRUCTS_BEING_INSPECTED.with(|held| held.borrow_mut().push(address));
                // An object of the program's own, another struct among them,
                // answers `inspect` itself.
                let mut shown = Vec::with_capacity(members.len());
                let mut failed = None;
                for member in members {
                    let value = member_value(receiver, member);
                    let text = if matches!(value.view(), View::Instance(_)) {
                        match self.inspect_value(&value, position) {
                            Ok(text) => text,
                            Err(error) => {
                                failed = Some(error);
                                break;
                            }
                        }
                    } else {
                        crate::vm::native_methods::array_methods::inspect_element(&value)
                    };
                    shown.push(format!("{member}={text}"));
                }
                STRUCTS_BEING_INSPECTED
                    .with(|held| held.borrow_mut().retain(|seen| *seen != address));
                if let Some(error) = failed {
                    return Err(error);
                }
                let body = shown.join(", ");
                // A struct nested in an anonymous class or module has no
                // Ruby name, which the synthesized `#<Class:0x..>::Foo` label
                // stands in for. Print it the way an anonymous struct prints.
                let name = class_rc.ruby_name();
                let anonymous = name.is_empty() || name.contains("#<");
                Ok(Some(Object::string(if anonymous {
                    format!("#<struct {}>", body)
                } else {
                    format!("#<struct {} {}>", name, body)
                })))
            }
            _ => Ok(None),
        }
    }

    /// The rest of a `dig` sequence, handed to the intermediate value's own
    /// `dig` the way Ruby does, so it decides how far the walk goes.
    pub(crate) fn dig_into(
        &mut self,
        value: &Object,
        keys: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some((class, method)) = self.lookup_method(value, "dig") {
            return self.invoke_method(class, method, value.clone(), keys.to_vec(), position);
        }
        let class = self.builtins().class_of(value);
        if let Some(result) = self.call_native_method(&class, value, "dig", keys, position)? {
            return Ok(result);
        }
        Err(MetorexError::type_error(
            format!("{} does not have #dig method", class.ruby_name()),
            position_to_location(position),
        ))
    }

    /// `deconstruct_keys(keys)` for pattern matching. It answers the members
    /// the keys name, stopping at the first key the struct has no value for,
    /// and an empty hash when more keys are asked for than there are members.
    pub(crate) fn deconstruct_struct_keys(
        &mut self,
        members: &[String],
        receiver: &Object,
        keys: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut pairs = IndexMap::new();
        let requested = match keys.view() {
            View::Nil => {
                for member in members {
                    pairs.insert(format!(":{}", member), member_value(receiver, member));
                }
                return Ok(Object::Dict(Rc::new(RefCell::new(pairs))));
            }
            View::Array(elements) => elements.borrow().clone(),
            _ => {
                let class_name = crate::vm::native_methods::define_method::ruby_class_name(keys);
                return Err(MetorexError::type_error(
                    format!("wrong argument type {} (expected Array or nil)", class_name),
                    position_to_location(position),
                ));
            }
        };
        if requested.len() > members.len() {
            return Ok(Object::Dict(Rc::new(RefCell::new(pairs))));
        }
        for key in &requested {
            let found = match key.view() {
                View::Symbol(name) | View::String(name) => members
                    .iter()
                    .find(|member| *member == &**name)
                    .map(|member| member_value(receiver, member)),
                _ => {
                    let index = self.key_as_index(key, position)?;
                    let length = members.len() as i64;
                    let resolved = if index < 0 { index + length } else { index };
                    if resolved < 0 || resolved >= length {
                        None
                    } else {
                        Some(member_value(receiver, &members[resolved as usize]))
                    }
                }
            };
            let Some(value) = found else {
                break;
            };
            let rendered = crate::vm::utils::object_to_dict_key(key).unwrap_or_default();
            if !crate::vm::utils::is_primitive_key(key) {
                remember_key_object(&mut pairs, &rendered, key);
            }
            pairs.insert(rendered, value);
        }
        Ok(Object::Dict(Rc::new(RefCell::new(pairs))))
    }

    /// A `deconstruct_keys` key that names a position rather than a member,
    /// coerced through `to_int` when it is not already an Integer.
    pub(crate) fn key_as_index(
        &mut self,
        key: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        if let View::Int(index) = key.view() {
            return Ok(index);
        }
        let class_name = match key.view() {
            View::Instance(instance) => Rc::clone(&instance.borrow().class).ruby_name(),
            _ => crate::vm::native_methods::define_method::ruby_class_name(key).to_string(),
        };
        let Some((class, method)) = self.lookup_method(key, "to_int") else {
            return Err(MetorexError::type_error(
                format!("no implicit conversion of {} into Integer", class_name),
                position_to_location(position),
            ));
        };
        match self
            .invoke_method(class, method, key.clone(), vec![], position)?
            .view()
        {
            View::Int(index) => Ok(index),
            _ => Err(MetorexError::type_error(
                format!("can't convert {} into Integer", class_name),
                position_to_location(position),
            )),
        }
    }

    /// The `[key, value]` pair a `to_h` block answers, coerced with `to_ary`
    /// when it is not already an Array.
    pub(crate) fn pair_from_block_result(
        &mut self,
        produced: Object,
        position: Position,
    ) -> Result<(Object, Object), MetorexError> {
        let pair = match produced.view() {
            View::Array(_) => produced.clone(),
            other => {
                let coerced = match self.lookup_method(&produced, "to_ary") {
                    Some((class, method)) => {
                        self.invoke_method(class, method, produced.clone(), vec![], position)?
                    }
                    None => Object::Nil,
                };
                if !matches!(coerced.view(), View::Array(_)) {
                    let class_name = match other {
                        View::Instance(instance) => Rc::clone(&instance.borrow().class).ruby_name(),
                        _ => crate::vm::native_methods::define_method::ruby_class_name(&produced)
                            .to_string(),
                    };
                    return Err(MetorexError::type_error(
                        format!("wrong element type {} (expected array)", class_name),
                        position_to_location(position),
                    ));
                }
                coerced
            }
        };
        let View::Array(elements) = pair.view() else {
            unreachable!("pair is an Array by construction");
        };
        let elements = elements.borrow();
        if elements.len() != 2 {
            return Err(argument_error(
                format!(
                    "element has wrong array length (expected 2, was {})",
                    elements.len()
                ),
                position,
            ));
        }
        Ok((elements[0].clone(), elements[1].clone()))
    }

    /// Two structs are equal when they share a class and every member value
    /// compares equal. `strict` selects `eql?` semantics, under which 1998
    /// and 1998.0 differ.
    pub(crate) fn struct_equals(
        &mut self,
        class_rc: &Rc<Class>,
        members: &[String],
        receiver: &Object,
        other: &Object,
        strict: bool,
    ) -> bool {
        let mut in_flight = Vec::new();
        struct_values_equal(class_rc, members, receiver, other, strict, &mut in_flight)
    }
}

/// A cyclic struct compares equal to another cycle of the same shape, so a
/// pair already being compared higher up the stack is taken as equal.
pub(crate) fn struct_values_equal(
    class_rc: &Rc<Class>,
    members: &[String],
    receiver: &Object,
    other: &Object,
    strict: bool,
    in_flight: &mut Vec<(usize, usize)>,
) -> bool {
    let View::Instance(other_instance) = other.view() else {
        return false;
    };
    if !Rc::ptr_eq(&other_instance.borrow().class, class_rc) {
        return false;
    }
    let pair = match (receiver.view(), other.view()) {
        (View::Instance(left), View::Instance(right)) => {
            (Rc::as_ptr(left) as usize, Rc::as_ptr(right) as usize)
        }
        _ => return false,
    };
    if pair.0 == pair.1 || in_flight.contains(&pair) {
        return true;
    }
    in_flight.push(pair);
    let equal = members.iter().all(|member| {
        member_values_equal(
            &member_value(receiver, member),
            &member_value(other, member),
            strict,
            in_flight,
        )
    });
    in_flight.pop();
    equal
}

pub(crate) fn member_values_equal(
    left: &Object,
    right: &Object,
    strict: bool,
    in_flight: &mut Vec<(usize, usize)>,
) -> bool {
    if let View::Instance(instance) = left.view()
        && let Some(members) = struct_members(&Rc::clone(&instance.borrow().class))
    {
        let class_rc = Rc::clone(&instance.borrow().class);
        return struct_values_equal(&class_rc, &members, left, right, strict, in_flight);
    }
    if strict && std::mem::discriminant(&left.view()) != std::mem::discriminant(&right.view()) {
        return false;
    }
    left.equals(right)
}

/// Whether `method` is the reader or writer Struct.new wrote for one of
/// `members`, or a copy of one, rather than a method the program wrote.
fn is_generated_member_accessor(
    method: &crate::object::Method,
    method_name: &str,
    members: &[String],
) -> bool {
    use crate::ast::{Expression, Statement};
    let member = method_name.strip_suffix('=').unwrap_or(method_name);
    if !members.iter().any(|held| held == member) {
        return false;
    }
    let slot = member_slot(member);
    match &method.body[..] {
        [
            Statement::Return {
                value: Some(Expression::InstanceVariable { name, .. }),
                ..
            },
        ] => *name == slot,
        [
            Statement::Assignment {
                target: Expression::InstanceVariable { name, .. },
                ..
            },
        ] => *name == slot,
        _ => false,
    }
}

thread_local! {
    /// The structs whose `inspect` is being rendered now.
    static STRUCTS_BEING_INSPECTED: std::cell::RefCell<Vec<usize>> =
        const { std::cell::RefCell::new(Vec::new()) };
}
