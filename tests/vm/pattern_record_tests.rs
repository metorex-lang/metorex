// The entries the VM keeps against the address of each pattern a program
// builds.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run_in(vm: &mut VirtualMachine, code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    vm.execute_program(&statements).expect("execution failed")
}

#[test]
fn patterns_built_and_freed_in_a_loop_leave_few_entries() {
    let mut vm = VirtualMachine::new();
    run_in(
        &mut vm,
        "10_000.times { |step| Regexp.new(\"a#{step}\".encode(\"US-ASCII\")) }",
    );
    assert!(vm.pattern_record_count() < 64);
}

#[test]
fn patterns_still_alive_keep_their_entries_past_a_loop_of_freed_ones() {
    let mut vm = VirtualMachine::new();
    let answer = run_in(
        &mut vm,
        "kept = (1..100).map { |step| Regexp.new(\"k#{step}\".encode(\"US-ASCII\")) }\n\
         10_000.times { |step| Regexp.new(\"a#{step}\") }\n\
         [kept.none?(&:frozen?), kept.map { |pattern| pattern.encoding.to_s }.uniq]",
    );
    assert_eq!(
        answer,
        Some(Object::array(vec![
            Object::Bool(true),
            Object::array(vec![Object::string("US-ASCII")]),
        ]))
    );
}
