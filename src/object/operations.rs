// Object operations - comparison and boolean logic

use super::{BlockStatement, Method, Object};
use crate::object::View;

impl Object {
    /// Check if this object is truthy (for conditional evaluation)
    pub fn is_truthy(&self) -> bool {
        match self.view() {
            View::Nil => false,
            View::Bool(b) => b,
            // All other values are truthy
            _ => true,
        }
    }

    /// Check if this object is falsy
    pub fn is_falsy(&self) -> bool {
        !self.is_truthy()
    }

    /// Deep equality comparison between objects
    pub fn equals(&self, other: &Object) -> bool {
        self.equals_within(other, &mut Vec::new())
    }

    /// Compare two objects, remembering the pairs of collections already being
    /// compared. An array that holds itself is equal to another built the same
    /// way, and the record of what is in flight is what stops the comparison
    /// from following the cycle forever.
    fn equals_within(&self, other: &Object, in_flight: &mut Vec<(usize, usize)>) -> bool {
        let pair = match (self.view(), other.view()) {
            (View::Array(a), View::Array(b)) => Some((
                std::rc::Rc::as_ptr(a) as usize,
                std::rc::Rc::as_ptr(b) as usize,
            )),
            (View::Dict(a), View::Dict(b)) => Some((
                std::rc::Rc::as_ptr(a) as usize,
                std::rc::Rc::as_ptr(b) as usize,
            )),
            _ => None,
        };
        if let Some(pair) = pair {
            if in_flight.contains(&pair) {
                return true;
            }
            in_flight.push(pair);
            let answer = self.equals_inner(other, in_flight);
            in_flight.pop();
            return answer;
        }
        self.equals_inner(other, in_flight)
    }

    fn equals_inner(&self, other: &Object, in_flight: &mut Vec<(usize, usize)>) -> bool {
        // An instance of an Array subclass compares by the elements it holds,
        // so it equals a plain Array with the same contents.
        // An instance of a Hash subclass compares by the entries it holds,
        // so it equals a plain Hash with the same contents.
        match (backing_hash(self), backing_hash(other)) {
            (Some(left), None) => return left.equals_within(other, in_flight),
            (None, Some(right)) => return self.equals_within(&right, in_flight),
            (Some(left), Some(right)) => return left.equals_within(&right, in_flight),
            (None, None) => {}
        }
        match (backing_array(self), backing_array(other)) {
            (Some(left), None) => return left.equals_within(other, in_flight),
            (None, Some(right)) => return self.equals_within(&right, in_flight),
            (Some(left), Some(right)) => return left.equals_within(&right, in_flight),
            (None, None) => {}
        }
        // A Rational equals a number of any other kind when the two stand for
        // the same value, which is what a Hash or an Array holding one has to
        // compare by.
        if let Some(answer) = rational_equals(self, other) {
            return answer;
        }
        match (self.view(), other.view()) {
            (View::Nil, View::Nil) => true,
            (View::Bool(a), View::Bool(b)) => a == b,
            (View::Int(a), View::Int(b)) => a == b,
            (View::BigInt(a), View::BigInt(b)) => a == b,
            // A normalized BigInt never holds a value an Int could, so the
            // two variants can only be equal through a Float.
            (View::Int(_), View::BigInt(_)) | (View::BigInt(_), View::Int(_)) => false,
            // Two Floats are equal when they are the same number, which is
            // what Ruby compares: `0.1 + 0.2 == 0.3` is false, and a number as
            // small as 1e-16 is not zero. NaN equals nothing, itself included.
            (View::Float(a), View::Float(b)) => a == b,
            // A Float equals the Integer it holds exactly, which is how Ruby
            // compares numbers of different kinds.
            (View::Int(a), View::Float(b)) | (View::Float(b), View::Int(a)) => a as f64 == b,
            // A large Integer equals a Float only when the Float holds that
            // exact whole number.
            (View::BigInt(a), View::Float(b)) | (View::Float(b), View::BigInt(a)) => {
                use num_bigint::ToBigInt;
                b.fract() == 0.0 && b.to_bigint().is_some_and(|whole| whole == **a)
            }
            (View::String(a), View::String(b)) => a == b,
            (View::Symbol(a), View::Symbol(b)) => a == b,
            (View::Array(a), View::Array(b)) => {
                let arr_a = a.borrow();
                let arr_b = b.borrow();
                if arr_a.len() != arr_b.len() {
                    return false;
                }
                arr_a
                    .iter()
                    .zip(arr_b.iter())
                    .all(|(left, right)| left.equals_within(right, in_flight))
            }
            (View::Dict(a), View::Dict(b)) => {
                // The sentinels a hash keeps for its default, its key objects,
                // and its identity setting are bookkeeping, not contents.
                let dict_a = a.borrow();
                let dict_b = b.borrow();
                let entries = |dict: &indexmap::IndexMap<String, Object>| {
                    dict.iter()
                        .filter(|(key, _)| !key.starts_with("__MX_"))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect::<Vec<_>>()
                };
                let left_entries = entries(&dict_a);
                let right_entries = entries(&dict_b);
                if left_entries.len() != right_entries.len() {
                    return false;
                }
                left_entries.iter().all(|(key, value)| {
                    right_entries.iter().any(|(other_key, other_value)| {
                        key == other_key && value.equals_within(other_value, in_flight)
                    })
                })
            }
            (View::Set(a), View::Set(b)) => {
                let set_a = a.borrow();
                let set_b = b.borrow();
                if set_a.len() != set_b.len() {
                    return false;
                }
                set_a.iter().all(|item| set_b.contains(item))
            }
            (
                View::Range {
                    start: left_start,
                    end: left_end,
                    exclusive: left_exclusive,
                    ..
                },
                View::Range {
                    start: right_start,
                    end: right_end,
                    exclusive: right_exclusive,
                    ..
                },
            ) => {
                left_exclusive == right_exclusive
                    && left_start.equals_within(right_start, in_flight)
                    && left_end.equals_within(right_end, in_flight)
            }
            (View::Result(a), View::Result(b)) => match (a, b) {
                (Ok(a_val), Ok(b_val)) => a_val.equals(b_val),
                (Err(a_err), Err(b_err)) => a_err.equals(b_err),
                _ => false,
            },
            // An instance of a String subclass compares by its characters,
            // in either order.
            (View::Instance(_), View::String(_)) | (View::String(_), View::Instance(_)) => {
                match (subclass_string(self), subclass_string(other)) {
                    (Some(text), None) => text.equals(other),
                    (None, Some(text)) => self.equals(&text),
                    _ => false,
                }
            }
            // Instance comparisons: value equality for Rational/Complex, reference for others
            (View::Instance(a), View::Instance(b)) => {
                let inst_a = a.borrow();
                let inst_b = b.borrow();
                let class_name = inst_a.class.name();
                if class_name == inst_b.class.name() && matches!(class_name, "Rational" | "Complex")
                {
                    // Compare by stored instance variables
                    let vars_a = &inst_a.instance_vars;
                    let vars_b = &inst_b.instance_vars;
                    vars_a.len() == vars_b.len()
                        && vars_a
                            .iter()
                            .all(|(k, v)| vars_b.get(k).is_some_and(|v2| v.equals(v2)))
                } else {
                    drop(inst_a);
                    drop(inst_b);
                    Rc::ptr_eq(a, b)
                }
            }
            // Classes and modules share one representation, so the same
            // object compares equal whichever variant is holding it.
            (View::Class(a) | View::Module(a), View::Class(b) | View::Module(b)) => {
                Rc::ptr_eq(a, b)
            }
            (View::Method(a), View::Method(b)) => {
                // Two Method objects are equal when they wrap the same
                // definition bound to the same receiver, as two calls of
                // `obj.method(:x)` do.
                let defined_name = |method: &Method| {
                    method
                        .provenance
                        .original_name
                        .clone()
                        .unwrap_or_else(|| method.name.clone())
                };
                let same_receiver = match (&a.receiver, &b.receiver) {
                    (Some(ra), Some(rb)) => ra.equals(rb),
                    (None, None) => true,
                    _ => false,
                };
                // A core method Ruby defines under two names is the same
                // method under either.
                // A method written in Ruby is the one written at its source
                // location, and one written in C the function it names.
                let same_written_method = defined_name(a) == defined_name(b)
                    && a.body == b.body
                    && a.source_location.is_some()
                    && a.source_location == b.source_location;
                let same_c_function = a.provenance.c_function.is_some()
                    && a.provenance.c_function == b.provenance.c_function;
                Rc::ptr_eq(a, b)
                    || (same_receiver
                        && (same_written_method
                            || same_c_function
                            || super::core_aliases::same_core_method(a, b)))
            }
            // Two blocks written apart are never equal, which the positions
            // their bodies carry settle. A copy of one is equal to it: it was
            // written in the same place and closes over the same cells.
            (View::Block(a), View::Block(b)) => {
                Rc::ptr_eq(a, b) || (**a == **b && closes_over_the_same(a, b))
            }
            (View::Binding(a), View::Binding(b)) => Rc::ptr_eq(a, b),
            // Ruby compares two exceptions by class, message, and backtrace,
            // so a dup equals its original.
            (View::Exception(a), View::Exception(b)) => {
                if Rc::ptr_eq(a, b) {
                    return true;
                }
                let (left, right) = (a.borrow(), b.borrow());
                let same_class = match (&left.class, &right.class) {
                    (Some(one), Some(two)) => Rc::ptr_eq(one, two),
                    _ => left.exception_type == right.exception_type,
                };
                same_class && left.message == right.message && left.backtrace == right.backtrace
            }
            // Different types are not equal
            _ => false,
        }
    }

    /// Compute hash for hashable types (for use in dictionaries)
    pub fn hash(&self) -> Option<u64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        match self.view() {
            View::Nil => {
                let mut hasher = DefaultHasher::new();
                "nil".hash(&mut hasher);
                Some(hasher.finish())
            }
            View::Bool(b) => {
                let mut hasher = DefaultHasher::new();
                b.hash(&mut hasher);
                Some(hasher.finish())
            }
            View::Int(i) => {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                Some(hasher.finish())
            }
            View::Float(f) => {
                let mut hasher = DefaultHasher::new();
                // Convert float to bits for consistent hashing
                f.to_bits().hash(&mut hasher);
                Some(hasher.finish())
            }
            View::String(s) => {
                let mut hasher = DefaultHasher::new();
                s.as_str().hash(&mut hasher);
                Some(hasher.finish())
            }
            // Complex types are not hashable
            _ => None,
        }
    }
}

use std::rc::Rc;

/// The characters behind an instance of a String subclass.
fn subclass_string(value: &Object) -> Option<Object> {
    let View::Instance(instance) = value.view() else {
        return None;
    };
    instance.borrow().instance_vars.get("__string__").cloned()
}

/// The elements an instance of an Array subclass holds, or None for anything
/// else. Kept here so equality can compare such an instance to a plain Array
/// without reaching into the VM.
fn backing_array(value: &Object) -> Option<Object> {
    let View::Instance(instance) = value.view() else {
        return None;
    };
    instance.borrow().instance_vars.get("__array__").cloned()
}

/// The entries an instance of a Hash subclass holds, or None for anything
/// else.
fn backing_hash(value: &Object) -> Option<Object> {
    let View::Instance(instance) = value.view() else {
        return None;
    };
    instance.borrow().instance_vars.get("__hash__").cloned()
}

/// The numerator and denominator a Rational instance holds.
fn fraction_parts(value: &Object) -> Option<(num_bigint::BigInt, num_bigint::BigInt)> {
    let View::Instance(instance) = value.view() else {
        return None;
    };
    let instance = instance.borrow();
    if instance.class.name() != "Rational" {
        return None;
    }
    let numerator = instance.instance_vars.get("numerator")?.as_big_integer()?;
    let denominator = instance
        .instance_vars
        .get("denominator")?
        .as_big_integer()?;
    Some((numerator, denominator))
}

/// A fraction rounded to the nearest Float, which is how Ruby compares a
/// Rational against one.
fn fraction_as_float(
    numerator: &num_bigint::BigInt,
    denominator: &num_bigint::BigInt,
) -> Option<f64> {
    let numerator = numerator.to_string().parse::<f64>().ok()?;
    let denominator = denominator.to_string().parse::<f64>().ok()?;
    Some(numerator / denominator)
}

/// Whether a pair holds a Rational and something it can be compared against
/// exactly. Answers None when neither side is a Rational, which leaves the
/// pair to the comparisons above.
fn rational_equals(left: &Object, right: &Object) -> Option<bool> {
    let left_parts = fraction_parts(left);
    let right_parts = fraction_parts(right);
    let whole = |value: &Object| match value.view() {
        View::Int(_) | View::BigInt(_) => value
            .as_big_integer()
            .map(|held| (held, num_bigint::BigInt::from(1))),
        _ => None,
    };
    match (left_parts, right_parts) {
        (Some(one), Some(other)) => Some(one.0 * other.1 == other.0 * one.1),
        (Some(one), None) => match right.view() {
            View::Float(number) => Some(fraction_as_float(&one.0, &one.1) == Some(number)),
            _ => whole(right).map(|other| one.0 * other.1 == other.0 * one.1),
        },
        (None, Some(other)) => match left.view() {
            View::Float(number) => Some(fraction_as_float(&other.0, &other.1) == Some(number)),
            _ => whole(left).map(|one| one.0 * other.1 == other.0 * one.1),
        },
        (None, None) => None,
    }
}

/// Whether two blocks hold the same closed-over cells, which is what tells a
/// copy of a block from another block written the same way.
fn closes_over_the_same(one: &Rc<BlockStatement>, other: &Rc<BlockStatement>) -> bool {
    if one.captured_vars.len() != other.captured_vars.len() {
        return false;
    }
    one.captured_vars.iter().all(|(name, cell)| {
        other
            .captured_vars
            .get(name)
            .is_some_and(|theirs| Rc::ptr_eq(cell, theirs))
    })
}
