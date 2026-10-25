// The list of every object made, which keeps a weak reference to each.

use metorex::lexer::Lexer;
use metorex::object::live;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    VirtualMachine::new()
        .execute_program(&statements)
        .expect("execution failed");
}

#[test]
fn dead_entries_are_cleared_with_collections_switched_off() {
    run("GC.disable\n50_000.times { |step| \"made #{step}\" }");
    assert!(live::entry_count() < 10_000);
}

#[test]
fn live_entries_stay_with_collections_switched_off() {
    run(
        "GC.disable\n$kept = Array.new(20_000) { |step| \"kept #{step}\" }\n50_000.times { |step| \"made #{step}\" }",
    );
    assert!(live::entry_count() >= 20_000);
}
