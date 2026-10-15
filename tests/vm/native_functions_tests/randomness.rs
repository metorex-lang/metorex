// Drawing a number, waiting, and seeding the generator.

use super::*;
use metorex::object::{OptionView, View};

#[test]
fn rand_no_args_returns_float() {
    let result = run("rand()");
    assert!(matches!(result.view(), Some(View::Float(_))));
}

#[test]
fn rand_with_int_arg() {
    let result = run("rand(100)");
    assert!(matches!(result.view(), Some(View::Int(_))));
}

#[test]
fn rand_with_zero_arg_gives_a_float() {
    // Ruby treats a bound of zero as no bound, so it draws a Float.
    let result = run("rand(0).is_a?(Float)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── sleep ────────────────────────────────────────────────────────────────────

#[test]
fn sleep_with_zero() {
    let result = run("sleep(0)");
    assert_eq!(result, Some(Object::Int(0)));
}

// ── srand ────────────────────────────────────────────────────────────────────

#[test]
fn srand_returns_int() {
    let result = run("srand()");
    assert!(matches!(result.view(), Some(View::Int(_))));
}

// ── load function ────────────────────────────────────────────────────────────

#[test]
fn load_nonexistent_file_error() {
    let err = run_err("load 'nonexistent_file_xyz.rb'");
    assert!(err.contains("cannot load"));
}
