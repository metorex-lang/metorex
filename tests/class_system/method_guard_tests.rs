// The guard a fast path runs behind: its check runs again only after a method
// table, a mixin chain, or a singleton's attachment changed.

use metorex::class::{MethodGuard, method_state};
use metorex::lexer::Lexer;
use metorex::object::{OptionView, View};
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::cell::Cell;
use std::rc::Rc;

fn run(vm: &mut VirtualMachine, code: &str) {
    let statements = Parser::new(Lexer::new(code).tokenize())
        .parse()
        .expect("parse failed");
    vm.execute_program(&statements).expect("execution failed");
}

/// The method `Ledger#total` the program holds now.
fn ledger_total(vm: &VirtualMachine) -> Rc<metorex::object::Method> {
    match vm.globals().get("Ledger").view() {
        Some(View::Class(class)) => class.find_method("total").expect("Ledger#total"),
        other => panic!("expected the Ledger class, got {other:?}"),
    }
}

const LEDGER: &str = "
class Ledger
  def total = 10
end
";

#[test]
fn a_guard_checks_once_while_no_method_changes() {
    let mut vm = VirtualMachine::new();
    run(&mut vm, LEDGER);
    let guard = MethodGuard::new();
    let checks = Cell::new(0);
    for _ in 0..5 {
        guard.holds(|| {
            checks.set(checks.get() + 1);
            true
        });
    }
    assert_eq!(checks.get(), 1);
}

#[test]
fn defining_a_method_runs_the_check_again() {
    let mut vm = VirtualMachine::new();
    run(&mut vm, LEDGER);
    let guard = MethodGuard::new();
    let checks = Cell::new(0);
    let check = || {
        checks.set(checks.get() + 1);
        true
    };
    guard.holds(check);
    run(&mut vm, "class Receipt\n  def number = 1\nend");
    guard.holds(check);
    assert_eq!(checks.get(), 2);
}

#[test]
fn including_a_module_moves_the_method_state() {
    let mut vm = VirtualMachine::new();
    run(&mut vm, "module Auditing\nend\nclass Ledger\nend");
    let before = method_state();
    run(&mut vm, "Ledger.include(Auditing)");
    assert_ne!(method_state(), before);
}

#[test]
fn a_redefined_method_fails_a_guard_that_holds_the_original() {
    let mut vm = VirtualMachine::new();
    run(&mut vm, LEDGER);
    let original = ledger_total(&vm);
    let guard = MethodGuard::new();
    let unchanged = |vm: &VirtualMachine| Rc::ptr_eq(&ledger_total(vm), &original);
    let held_before = guard.holds(|| unchanged(&vm));
    run(&mut vm, "class Ledger\n  def total = 20\nend");
    let held_after = guard.holds(|| unchanged(&vm));
    assert_eq!((held_before, held_after), (true, false));
}

#[test]
fn an_object_s_new_singleton_class_leaves_the_method_state_alone() {
    let mut vm = VirtualMachine::new();
    run(&mut vm, "class Ledger\nend\n$ledger = Ledger.new");
    let before = method_state();
    run(&mut vm, "$ledger.singleton_class");
    assert_eq!(method_state(), before);
}

#[test]
fn a_method_defined_on_an_object_s_singleton_class_moves_the_method_state() {
    let mut vm = VirtualMachine::new();
    run(
        &mut vm,
        "class Ledger\nend\n$ledger = Ledger.new\n$ledger.singleton_class",
    );
    let before = method_state();
    run(&mut vm, "def $ledger.total = 5");
    assert_ne!(method_state(), before);
}
