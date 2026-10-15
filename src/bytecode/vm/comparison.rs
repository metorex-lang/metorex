// Comparison helper functions for bytecode VM binary operations

use crate::object::Object;
use crate::object::View;

pub fn compare_less(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(a), View::Int(b)) => Ok(Object::Bool(a < b)),
        (View::Float(a), View::Float(b)) => Ok(Object::Bool(a < b)),
        (View::Int(a), View::Float(b)) => Ok(Object::Bool((a as f64) < b)),
        (View::Float(a), View::Int(b)) => Ok(Object::Bool(a < b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn compare_greater(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(a), View::Int(b)) => Ok(Object::Bool(a > b)),
        (View::Float(a), View::Float(b)) => Ok(Object::Bool(a > b)),
        (View::Int(a), View::Float(b)) => Ok(Object::Bool(a as f64 > b)),
        (View::Float(a), View::Int(b)) => Ok(Object::Bool(a > b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn compare_less_equal(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(a), View::Int(b)) => Ok(Object::Bool(a <= b)),
        (View::Float(a), View::Float(b)) => Ok(Object::Bool(a <= b)),
        (View::Int(a), View::Float(b)) => Ok(Object::Bool(a as f64 <= b)),
        (View::Float(a), View::Int(b)) => Ok(Object::Bool(a <= b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn compare_greater_equal(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(a), View::Int(b)) => Ok(Object::Bool(a >= b)),
        (View::Float(a), View::Float(b)) => Ok(Object::Bool(a >= b)),
        (View::Int(a), View::Float(b)) => Ok(Object::Bool(a as f64 >= b)),
        (View::Float(a), View::Int(b)) => Ok(Object::Bool(a >= b as f64)),
        _ => Err(format!(
            "Cannot compare {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}
