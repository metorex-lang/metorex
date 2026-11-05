//! Comparing and coercing numbers from C, as MRI's numeric functions do:
//! `rb_cmpint`, `rb_cmperr`, `rb_num_coerce_bin`, `rb_num_coerce_cmp` and
//! `rb_num_coerce_relop`, with `rb_num_zerodiv`, `rb_Integer`, `NUM2SHORT`,
//! `NUM2USHORT`, `NUM2CHR`, `rb_absint_singlebit_p` and
//! `rb_opts_exception_p`.

use super::arrays::array_of;
use super::calls::{answers, call, class_name_of, top_level_module};
use super::numbers::long_from;
use super::symbols::symbol_name;
use super::values::{QNIL, Value, to_object, to_value};
use super::{called_from, interpreter, or_raise, raise};
use crate::object::Object;
use crate::object::View;

fn error(class: &str, message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        class,
        &message,
        called_from(),
    ))
}

/// How an error names the other side of a comparison or coercion: an
/// immediate value or a Float by its `inspect`, anything else by its class.
fn subject(object: Object) -> String {
    match object.view() {
        View::Nil | View::Bool(_) | View::Int(_) | View::Symbol(_) | View::Float(_) => {
            call(object, "inspect", Vec::new()).to_string()
        }
        _ => class_name_of(object.clone()),
    }
}

fn comparison_error(first: Value, second: Value, reason: Option<&str>) -> ! {
    let mut message = format!(
        "comparison of {} with {} failed",
        class_name_of(to_object(first)),
        subject(to_object(second))
    );
    if let Some(reason) = reason {
        message.push_str(&format!(": {}", reason));
    }
    error("ArgumentError", message)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cmperr(first: Value, second: Value) -> ! {
    comparison_error(first, second, None)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num_zerodiv() -> ! {
    error("ZeroDivisionError", "divided by 0".to_string())
}

/// The sign of what `<=>` answered for `first` and `second`: an Integer's
/// own sign, or what `> 0` and `< 0` say of anything else.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cmpint(answer: Value, first: Value, second: Value) -> i32 {
    match to_object(answer).view() {
        View::Nil => comparison_error(first, second, Some("comparator returned nil")),
        View::Int(number) => number.signum() as i32,
        View::BigInt(number) => match number.sign() {
            num_bigint::Sign::Minus => -1,
            num_bigint::Sign::NoSign => 0,
            num_bigint::Sign::Plus => 1,
        },
        other => {
            if call(other.to_object(), ">", vec![Object::Int(0)]).is_truthy() {
                return 1;
            }
            if call(other.to_object(), "<", vec![Object::Int(0)]).is_truthy() {
                return -1;
            }
            0
        }
    }
}

/// What `second.coerce(first)` turns the pair into, as MRI's `do_coerce`
/// reads it. With `strict`, an object that cannot coerce is a TypeError;
/// without, it and a nil answer give None.
fn coerced(first: Object, second: Object, strict: bool) -> Option<(Object, Object)> {
    if !answers(&second, "coerce") {
        if strict {
            error(
                "TypeError",
                format!(
                    "{} can't be coerced into {}",
                    subject(second),
                    class_name_of(first)
                ),
            );
        }
        return None;
    }
    let answer = call(second, "coerce", vec![first]);
    if !strict && matches!(answer.view(), View::Nil) {
        return None;
    }
    match array_of(answer) {
        Some(pair) if pair.len() == 2 => Some((pair[0].clone(), pair[1].clone())),
        _ => error("TypeError", "coerce must return [x, y]".to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num_coerce_bin(first: Value, second: Value, name: Value) -> Value {
    let (first, second) = coerced(to_object(first), to_object(second), true)
        .expect("a strict coercion answers a pair or raises");
    to_value(&call(first, &symbol_name(name), vec![second]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num_coerce_cmp(first: Value, second: Value, name: Value) -> Value {
    match coerced(to_object(first), to_object(second), false) {
        Some((first, second)) => to_value(&call(first, &symbol_name(name), vec![second])),
        None => QNIL,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num_coerce_relop(first: Value, second: Value, name: Value) -> Value {
    let Some((coerced_first, coerced_second)) = coerced(to_object(first), to_object(second), false)
    else {
        comparison_error(first, second, Some("coercion was not possible"))
    };
    let answer = call(coerced_first, &symbol_name(name), vec![coerced_second]);
    if matches!(answer.view(), View::Nil) {
        comparison_error(first, second, Some("comparator returned nil"));
    }
    to_value(&answer)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_Integer(value: Value) -> Value {
    to_value(&call(
        top_level_module("Kernel"),
        "Integer",
        vec![to_object(value)],
    ))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num2short(value: Value) -> i16 {
    let number = long_from(value);
    i16::try_from(number).unwrap_or_else(|_| {
        let direction = if number < 0 { "small" } else { "big" };
        error(
            "RangeError",
            format!("integer {} too {} to convert to 'short'", number, direction),
        )
    })
}

/// A number from `SHRT_MIN` to `USHRT_MAX`, as an unsigned short: a
/// negative one wraps.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num2ushort(value: Value) -> u16 {
    let number = long_from(value);
    if number < i64::from(i16::MIN) {
        error(
            "RangeError",
            format!(
                "integer {} too small to convert to 'unsigned short'",
                number
            ),
        )
    }
    if number > i64::from(u16::MAX) {
        error(
            "RangeError",
            format!("integer {} too big to convert to 'unsigned short'", number),
        )
    }
    number as u16
}

/// The first byte of a non-empty String, or the low byte of the number
/// `NUM2INT` reads from anything else.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_num2char(value: Value) -> i8 {
    if let View::String(text) = to_object(value).view() {
        let bytes = crate::vm::native_methods::string_methods::binary_bytes(text);
        if let Some(first) = bytes.first() {
            return *first as i8;
        }
    }
    super::numbers::rb_num2int(value) as i8
}

/// 1 when the magnitude of the Integer `to_int` makes of the value has one
/// bit set, and 0 otherwise.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_absint_singlebit_p(value: Value) -> i32 {
    let number = or_raise(interpreter().coerce_integer_argument(&to_object(value), called_from()));
    let magnitude = number.magnitude();
    (magnitude.count_ones() == 1) as i32
}

/// What the `exception:` option in `options` says: 1 for true, 0 for false,
/// and `default_value` when the Hash has no such key or holds nil there.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_opts_exception_p(options: Value, default_value: i32) -> i32 {
    let options = to_object(options);
    if !matches!(options.view(), View::Dict(_)) {
        return default_value;
    }
    let key = Object::symbol("exception".to_string());
    if !call(options.clone(), "key?", vec![key.clone()]).is_truthy() {
        return default_value;
    }
    match call(options, "[]", vec![key]).view() {
        View::Bool(true) => 1,
        View::Bool(false) => 0,
        View::Nil => default_value,
        other => {
            let shown = call(other.to_object(), "inspect", Vec::new()).to_string();
            error(
                "ArgumentError",
                format!("expected true or false as exception: {}", shown),
            )
        }
    }
}
