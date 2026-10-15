//! Metorex's value as it is now: the `Object` enum. An object's slots are an
//! `Object::Array`, a reference-counted vector, which is the same body the
//! tagged representation gives an object, so the two differ only in the
//! value.

use crate::machine::Representation;
use metorex::object::Object;
use std::cell::RefCell;
use std::rc::Rc;

pub struct EnumValue;

/// The operation's result on two Integers, checked for overflow the way
/// Metorex checks it, with a Bignum when it does not fit.
fn integer_result(
    left: i64,
    right: i64,
    checked: fn(i64, i64) -> Option<i64>,
    wide: fn(i128, i128) -> i128,
) -> Object {
    match checked(left, right) {
        Some(fits) => Object::Int(fits),
        None => Object::BigInt(Rc::new(num_bigint::BigInt::from(wide(
            left as i128,
            right as i128,
        )))),
    }
}

fn arithmetic(
    left: Object,
    right: Object,
    checked: fn(i64, i64) -> Option<i64>,
    wide: fn(i128, i128) -> i128,
    float: fn(f64, f64) -> f64,
) -> Object {
    match (left, right) {
        (Object::Int(left), Object::Int(right)) => integer_result(left, right, checked, wide),
        (Object::Float(left), Object::Float(right)) => Object::Float(float(left, right)),
        (Object::Float(left), Object::Int(right)) => Object::Float(float(left, right as f64)),
        (Object::Int(left), Object::Float(right)) => Object::Float(float(left as f64, right)),
        (left, right) => panic!("no arithmetic for {left:?} and {right:?}"),
    }
}

fn slots(object: &Object) -> &Rc<RefCell<Vec<Object>>> {
    match object {
        Object::Array(held) => held,
        other => panic!("{other:?} holds no slots"),
    }
}

impl Representation for EnumValue {
    type Value = Object;

    fn nil() -> Object {
        Object::Nil
    }

    fn integer(number: i64) -> Object {
        Object::Int(number)
    }

    fn float(number: f64) -> Object {
        Object::Float(number)
    }

    fn truthy(value: &Object) -> bool {
        !matches!(value, Object::Nil | Object::Bool(false))
    }

    fn add(left: Object, right: Object) -> Object {
        arithmetic(left, right, i64::checked_add, |a, b| a + b, |a, b| a + b)
    }

    fn subtract(left: Object, right: Object) -> Object {
        arithmetic(left, right, i64::checked_sub, |a, b| a - b, |a, b| a - b)
    }

    fn multiply(left: Object, right: Object) -> Object {
        arithmetic(left, right, i64::checked_mul, |a, b| a * b, |a, b| a * b)
    }

    fn less_than(left: Object, right: Object) -> Object {
        Object::Bool(match (left, right) {
            (Object::Int(left), Object::Int(right)) => left < right,
            (Object::Float(left), Object::Float(right)) => left < right,
            (left, right) => panic!("no comparison for {left:?} and {right:?}"),
        })
    }

    fn new_object(count: usize) -> Object {
        Object::Array(Rc::new(RefCell::new(vec![Object::Nil; count])))
    }

    fn get_slot(object: &Object, slot: usize) -> Object {
        slots(object).borrow()[slot].clone()
    }

    fn set_slot(object: &Object, slot: usize, value: Object) {
        slots(object).borrow_mut()[slot] = value;
    }

    fn new_array() -> Object {
        Object::Array(Rc::new(RefCell::new(Vec::new())))
    }

    fn array_push(array: &Object, value: Object) {
        slots(array).borrow_mut().push(value);
    }

    fn array_get(array: &Object, index: &Object) -> Object {
        let Object::Int(index) = index else {
            panic!("{index:?} is no index");
        };
        slots(array)
            .borrow()
            .get(*index as usize)
            .cloned()
            .unwrap_or(Object::Nil)
    }

    fn describe(value: &Object) -> String {
        match value {
            Object::Int(number) => number.to_string(),
            Object::BigInt(number) => number.to_string(),
            Object::Float(number) => format!("{number:?}"),
            other => format!("{other:?}"),
        }
    }
}
