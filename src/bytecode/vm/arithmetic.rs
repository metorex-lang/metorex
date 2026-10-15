// Arithmetic helper functions for bytecode VM binary operations

use crate::object::Object;
use crate::object::View;

pub fn binary_add(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(a), View::Int(b)) => Ok(Object::Int(a + b)),
        (View::Float(a), View::Float(b)) => Ok(Object::Float(a + b)),
        (View::Int(a), View::Float(b)) => Ok(Object::Float(a as f64 + b)),
        (View::Float(a), View::Int(b)) => Ok(Object::Float(a + b as f64)),
        (View::String(a), View::String(b)) => Ok(Object::string(format!("{}{}", a, b))),
        _ => Err(format!(
            "Cannot add {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn binary_sub(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(a), View::Int(b)) => Ok(Object::Int(a - b)),
        (View::Float(a), View::Float(b)) => Ok(Object::Float(a - b)),
        (View::Int(a), View::Float(b)) => Ok(Object::Float(a as f64 - b)),
        (View::Float(a), View::Int(b)) => Ok(Object::Float(a - b as f64)),
        _ => Err(format!(
            "Cannot subtract {} from {}",
            b.type_name(),
            a.type_name()
        )),
    }
}

pub fn binary_mul(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(a), View::Int(b)) => Ok(Object::Int(a * b)),
        (View::Float(a), View::Float(b)) => Ok(Object::Float(a * b)),
        (View::Int(a), View::Float(b)) => Ok(Object::Float(a as f64 * b)),
        (View::Float(a), View::Int(b)) => Ok(Object::Float(a * b as f64)),
        _ => Err(format!(
            "Cannot multiply {} and {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn binary_div(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(_), View::Int(0)) => Err("Division by zero".to_string()),
        (View::Int(a), View::Int(b)) => Ok(Object::Int(a / b)),
        (View::Float(a), View::Float(b)) => Ok(Object::Float(a / b)),
        (View::Int(a), View::Float(b)) => Ok(Object::Float(a as f64 / b)),
        (View::Float(a), View::Int(b)) => Ok(Object::Float(a / b as f64)),
        _ => Err(format!(
            "Cannot divide {} by {}",
            a.type_name(),
            b.type_name()
        )),
    }
}

pub fn binary_mod(a: &Object, b: &Object) -> Result<Object, String> {
    match (a.view(), b.view()) {
        (View::Int(_), View::Int(0)) => Err("Modulo by zero".to_string()),
        (View::Int(a), View::Int(b)) => Ok(Object::Int(a % b)),
        _ => Err(format!(
            "Cannot modulo {} by {}",
            a.type_name(),
            b.type_name()
        )),
    }
}
