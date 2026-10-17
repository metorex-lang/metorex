// Each instruction, run through VirtualMachine.

use metorex::bytecode::instruction::Instruction::{self, *};
use metorex::bytecode::unit::{CallSite, Handler, Name, Receiver, Unit};
use metorex::error::MetorexError;
use metorex::lexer::{Lexer, Position};
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn at(line: usize) -> Position {
    Position::new(line, 1, 0)
}

/// A unit of `instructions`, each on its own line, with `registers`
/// registers and the constants given.
fn unit_of(registers: usize, constants: Vec<Object>, instructions: &[Instruction]) -> Unit {
    let mut unit = Unit::new();
    unit.use_registers(registers);
    for constant in constants {
        unit.constant(constant);
    }
    for (line, instruction) in instructions.iter().enumerate() {
        unit.emit(*instruction, at(line + 1));
    }
    unit
}

/// A machine that has run `program` first.
fn machine_after(program: &str) -> VirtualMachine {
    let mut machine = VirtualMachine::new();
    let statements = Parser::new(Lexer::new(program).tokenize())
        .parse()
        .expect("parse failed");
    machine
        .execute_program(&statements)
        .expect("program failed");
    machine
}

fn run(unit: &Unit) -> Object {
    VirtualMachine::new().run_unit(unit).expect("unit failed")
}

/// The message of the error running `unit` on `machine` ended with.
fn error_of(machine: &mut VirtualMachine, unit: &Unit) -> String {
    machine
        .run_unit(unit)
        .expect_err("the unit ran to its end")
        .to_string()
}

#[test]
fn an_instruction_is_four_bytes() {
    assert_eq!(std::mem::size_of::<Instruction>(), 4);
}

#[test]
fn loads_put_each_kind_of_value_in_a_register() {
    let loaded = |instruction| {
        run(&unit_of(
            1,
            vec![Object::string("text")],
            &[instruction, Return { from: 0 }],
        ))
    };
    assert_eq!(
        loaded(LoadConstant { to: 0, constant: 0 }),
        Object::string("text")
    );
    assert_eq!(loaded(LoadInteger { to: 0, value: -7 }), Object::Int(-7));
    assert_eq!(loaded(LoadNil { to: 0 }), Object::Nil);
    assert_eq!(loaded(LoadTrue { to: 0 }), Object::Bool(true));
    assert_eq!(loaded(LoadFalse { to: 0 }), Object::Bool(false));
}

#[test]
fn move_copies_one_register_to_another() {
    let unit = unit_of(
        2,
        vec![],
        &[
            LoadInteger { to: 1, value: 5 },
            Move { to: 0, from: 1 },
            Return { from: 0 },
        ],
    );
    assert_eq!(run(&unit), Object::Int(5));
}

#[test]
fn load_self_reads_the_running_self() {
    let mut machine = machine_after("class Holder\n  def itself_named\n    :holder\n  end\nend\n");
    let mut unit = unit_of(
        1,
        vec![],
        &[
            LoadSelf { to: 0 },
            Call { base: 0, site: 0 },
            Return { from: 0 },
        ],
    );
    unit.call_site(CallSite::new("to_s", 0, Receiver::Other));
    assert_eq!(machine.run_unit(&unit).unwrap(), Object::string("main"));
}

#[test]
fn arithmetic_reads_registers_and_constants() {
    let operate = |instruction| {
        run(&unit_of(
            2,
            vec![Object::Int(4)],
            &[
                LoadInteger { to: 1, value: 10 },
                instruction,
                Return { from: 0 },
            ],
        ))
    };
    assert_eq!(
        operate(Add {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Int(14)
    );
    assert_eq!(
        operate(Subtract {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Int(6)
    );
    assert_eq!(
        operate(Multiply {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Int(40)
    );
    assert_eq!(
        operate(Divide {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Int(2)
    );
    assert_eq!(
        operate(Modulo {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Int(2)
    );
    assert_eq!(
        operate(Subtract {
            to: 0,
            left: 128,
            right: 1
        }),
        Object::Int(-6)
    );
}

#[test]
fn comparisons_answer_true_or_false() {
    let compare = |instruction| {
        run(&unit_of(
            2,
            vec![Object::Int(4)],
            &[
                LoadInteger { to: 1, value: 10 },
                instruction,
                Return { from: 0 },
            ],
        ))
    };
    assert_eq!(
        compare(Equal {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Bool(false)
    );
    assert_eq!(
        compare(NotEqual {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Bool(true)
    );
    assert_eq!(
        compare(Less {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Bool(false)
    );
    assert_eq!(
        compare(LessEqual {
            to: 0,
            left: 128,
            right: 1
        }),
        Object::Bool(true)
    );
    assert_eq!(
        compare(Greater {
            to: 0,
            left: 1,
            right: 128
        }),
        Object::Bool(true)
    );
    assert_eq!(
        compare(GreaterEqual {
            to: 0,
            left: 128,
            right: 1
        }),
        Object::Bool(false)
    );
}

#[test]
fn an_operator_a_class_defines_is_called() {
    let mut machine = machine_after(
        "class Money\n  def initialize(cents)\n    @cents = cents\n  end\n  def +(other)\n    @cents + other\n  end\nend\n$wallet = Money.new(250)\n",
    );
    let mut unit = unit_of(
        2,
        vec![Object::Int(50)],
        &[
            GetGlobal { to: 1, name: 0 },
            Add {
                to: 0,
                left: 1,
                right: 128,
            },
            Return { from: 0 },
        ],
    );
    unit.name(Name::new("wallet"));
    assert_eq!(machine.run_unit(&unit).unwrap(), Object::Int(300));
}

#[test]
fn negate_and_not_apply_the_unary_operators() {
    let unit = unit_of(
        2,
        vec![],
        &[
            LoadInteger { to: 1, value: 3 },
            Negate { to: 0, from: 1 },
            Return { from: 0 },
        ],
    );
    assert_eq!(run(&unit), Object::Int(-3));
    let unit = unit_of(
        2,
        vec![],
        &[
            LoadNil { to: 1 },
            Not { to: 0, from: 1 },
            Return { from: 0 },
        ],
    );
    assert_eq!(run(&unit), Object::Bool(true));
}

#[test]
fn jumps_run_a_loop_until_its_test_is_false() {
    // total = 0; count = 5; while count > 0: total += count; count -= 1
    let unit = unit_of(
        3,
        vec![Object::Int(0), Object::Int(1)],
        &[
            LoadInteger { to: 0, value: 0 },
            LoadInteger { to: 1, value: 5 },
            Greater {
                to: 2,
                left: 1,
                right: 128,
            },
            JumpIfFalse { test: 2, target: 7 },
            Add {
                to: 0,
                left: 0,
                right: 1,
            },
            Subtract {
                to: 1,
                left: 1,
                right: 129,
            },
            Jump { target: 2 },
            Return { from: 0 },
        ],
    );
    assert_eq!(run(&unit), Object::Int(15));
}

#[test]
fn jump_if_true_jumps_on_a_truthy_value_only() {
    let jumped = |test| {
        run(&unit_of(
            2,
            vec![],
            &[
                test,
                JumpIfTrue { test: 1, target: 4 },
                LoadInteger { to: 0, value: 1 },
                Return { from: 0 },
                LoadInteger { to: 0, value: 2 },
                Return { from: 0 },
            ],
        ))
    };
    assert_eq!(jumped(LoadTrue { to: 1 }), Object::Int(2));
    assert_eq!(jumped(LoadFalse { to: 1 }), Object::Int(1));
}

#[test]
fn call_sends_the_arguments_after_the_receiver() {
    let mut machine = machine_after(
        "class Adder\n  def sum(left, right)\n    left + right\n  end\nend\n$adder = Adder.new\n",
    );
    let mut unit = unit_of(
        3,
        vec![],
        &[
            GetGlobal { to: 0, name: 0 },
            LoadInteger { to: 1, value: 2 },
            LoadInteger { to: 2, value: 3 },
            Call { base: 0, site: 0 },
            Return { from: 0 },
        ],
    );
    unit.name(Name::new("adder"));
    unit.call_site(CallSite::new("sum", 2, Receiver::Other));
    assert_eq!(machine.run_unit(&unit).unwrap(), Object::Int(5));
}

#[test]
fn a_private_method_is_called_only_through_self() {
    let program =
        "class Vault\n  def open\n    :opened\n  end\n  private :open\nend\n$vault = Vault.new\n";
    let unit_with = |receiver| {
        let mut unit = unit_of(
            1,
            vec![],
            &[
                GetGlobal { to: 0, name: 0 },
                Call { base: 0, site: 0 },
                Return { from: 0 },
            ],
        );
        unit.name(Name::new("vault"));
        unit.call_site(CallSite::new("open", 0, receiver));
        unit
    };
    let mut machine = machine_after(program);
    assert_eq!(
        machine.run_unit(&unit_with(Receiver::SelfKeyword)).unwrap(),
        Object::symbol("opened")
    );
    let refused = error_of(&mut machine_after(program), &unit_with(Receiver::Other));
    assert!(
        refused.contains("private method 'open' called"),
        "{refused}"
    );
}

#[test]
fn instance_variables_are_written_and_read_on_self() {
    let mut unit = unit_of(
        2,
        vec![],
        &[
            LoadInteger { to: 1, value: 9 },
            SetInstanceVariable { from: 1, name: 0 },
            GetInstanceVariable { to: 0, name: 0 },
            Return { from: 0 },
        ],
    );
    unit.name(Name::new("@count"));
    assert_eq!(run(&unit), Object::Int(9));
}

#[test]
fn constants_are_written_and_read() {
    let mut unit = unit_of(
        2,
        vec![],
        &[
            LoadInteger { to: 1, value: 42 },
            SetConstant { from: 1, name: 0 },
            GetConstant { to: 0, name: 0 },
            Return { from: 0 },
        ],
    );
    unit.name(Name::new("ANSWER"));
    assert_eq!(run(&unit), Object::Int(42));
}

#[test]
fn an_undefined_constant_raises_name_error() {
    let mut unit = unit_of(
        1,
        vec![],
        &[GetConstant { to: 0, name: 0 }, Return { from: 0 }],
    );
    unit.name(Name::new("Missing"));
    let refused = error_of(&mut VirtualMachine::new(), &unit);
    assert!(
        refused.contains("uninitialized constant Missing"),
        "{refused}"
    );
}

#[test]
fn globals_are_written_and_read() {
    let mut unit = unit_of(
        2,
        vec![Object::string("set")],
        &[
            LoadConstant { to: 1, constant: 0 },
            SetGlobal { from: 1, name: 0 },
            GetGlobal { to: 0, name: 0 },
            Return { from: 0 },
        ],
    );
    unit.name(Name::new("state"));
    assert_eq!(run(&unit), Object::string("set"));
}

#[test]
fn new_array_gathers_consecutive_registers() {
    let unit = unit_of(
        4,
        vec![],
        &[
            LoadInteger { to: 1, value: 1 },
            LoadInteger { to: 2, value: 2 },
            LoadInteger { to: 3, value: 3 },
            NewArray {
                to: 0,
                first: 1,
                count: 3,
            },
            Return { from: 0 },
        ],
    );
    assert_eq!(
        run(&unit),
        Object::array(vec![Object::Int(1), Object::Int(2), Object::Int(3)])
    );
}

#[test]
fn raise_without_a_handler_ends_the_unit_with_the_error() {
    let unit = unit_of(
        1,
        vec![Object::string("boom")],
        &[
            LoadConstant { to: 0, constant: 0 },
            Raise { from: 0 },
            Return { from: 0 },
        ],
    );
    let refused = error_of(&mut VirtualMachine::new(), &unit);
    assert!(refused.contains("boom"), "{refused}");
}

#[test]
fn a_handler_catches_what_its_instructions_raise() {
    let mut unit = unit_of(
        2,
        vec![Object::string("boom")],
        &[
            LoadConstant { to: 1, constant: 0 },
            Raise { from: 1 },
            Return { from: 1 },
            Call { base: 0, site: 0 },
            Return { from: 0 },
        ],
    );
    unit.handler(Handler::rescue(0, 3, 3, 0));
    unit.call_site(CallSite::new("message", 0, Receiver::Other));
    assert_eq!(run(&unit), Object::string("boom"));
}

#[test]
fn a_handler_catches_an_error_the_interpreter_raises() {
    let mut unit = unit_of(
        2,
        vec![],
        &[
            LoadNil { to: 1 },
            Call { base: 1, site: 0 },
            Return { from: 1 },
            Call { base: 0, site: 1 },
            Return { from: 0 },
        ],
    );
    unit.handler(Handler::rescue(0, 3, 3, 0));
    unit.call_site(CallSite::new("no_such_method", 0, Receiver::Other));
    unit.call_site(CallSite::new("class", 0, Receiver::Other));
    let class = run(&unit);
    assert_eq!(format!("{class}"), "NoMethodError");
}

#[test]
fn extend_supplies_the_high_bits_of_a_register() {
    let mut unit = Unit::new();
    unit.use_registers(300);
    for instruction in [
        Extend { high: 1, wide: 0 },
        LoadInteger { to: 34, value: 100 },
        Extend { high: 1, wide: 0 },
        Return { from: 34 },
    ] {
        unit.emit(instruction, at(1));
    }
    assert_eq!(run(&unit), Object::Int(100));
}

#[test]
fn extend_supplies_the_high_bits_of_arithmetic_operands() {
    // With `high` 1, the left operand 129 names constant 129, the right
    // operand 0 names register 128, and the destination 0 names register
    // 256.
    let mut unit = Unit::new();
    unit.use_registers(257);
    for value in 0..130 {
        unit.constant(Object::Int(value));
    }
    for instruction in [
        LoadInteger { to: 128, value: 1 },
        Extend { high: 1, wide: 0 },
        Add {
            to: 0,
            left: 129,
            right: 0,
        },
        Extend { high: 1, wide: 0 },
        Return { from: 0 },
    ] {
        unit.emit(instruction, at(1));
    }
    assert_eq!(run(&unit), Object::Int(130));
}

#[test]
fn extend_supplies_the_high_bits_of_a_wide_operand() {
    let mut unit = Unit::new();
    unit.use_registers(1);
    for value in 0..65_538 {
        unit.constant(Object::Int(value));
    }
    for instruction in [
        Extend { high: 0, wide: 1 },
        LoadConstant { to: 0, constant: 1 },
        Return { from: 0 },
    ] {
        unit.emit(instruction, at(1));
    }
    assert_eq!(run(&unit), Object::Int(65_537));
}

#[test]
fn an_error_carries_the_line_of_the_instruction_that_raised_it() {
    let mut unit = unit_of(
        1,
        vec![],
        &[
            LoadNil { to: 0 },
            LoadNil { to: 0 },
            Call { base: 0, site: 0 },
            Return { from: 0 },
        ],
    );
    unit.call_site(CallSite::new("no_such_method", 0, Receiver::Other));
    let error = VirtualMachine::new().run_unit(&unit).unwrap_err();
    let MetorexError::UncaughtException { location, .. } = error else {
        panic!("not an exception: {error:?}");
    };
    assert_eq!(location.line, 3);
}

#[test]
fn an_instruction_starting_a_statement_fires_a_line_event() {
    let mut machine = machine_after(
        "$lines = []\n$trace = TracePoint.new(:line) { |point| $lines << point.lineno if point.lineno > 100 }\n$trace.enable\n",
    );
    let mut unit = Unit::new();
    unit.use_registers(1);
    unit.emit_statement(LoadInteger { to: 0, value: 1 }, at(101));
    unit.emit(LoadInteger { to: 0, value: 2 }, at(101));
    unit.emit_statement(Return { from: 0 }, at(102));
    machine.run_unit(&unit).unwrap();
    let statements = Parser::new(Lexer::new("$trace.disable\n$lines").tokenize())
        .parse()
        .unwrap();
    let lines = machine.execute_program(&statements).unwrap();
    assert_eq!(
        lines,
        Some(Object::array(vec![Object::Int(101), Object::Int(102)]))
    );
}

#[test]
fn load_symbol_names_the_symbol() {
    let mut unit = unit_of(
        1,
        vec![],
        &[LoadSymbol { to: 0, name: 0 }, Return { from: 0 }],
    );
    unit.name(Name::new("held"));
    assert_eq!(run(&unit), Object::symbol("held"));
}

#[test]
fn new_string_makes_a_new_string_each_time() {
    let mut unit = unit_of(
        2,
        vec![],
        &[
            NewString { to: 0, text: 0 },
            NewString { to: 1, text: 0 },
            Call { base: 0, site: 0 },
            Return { from: 0 },
        ],
    );
    unit.name(Name::new("text"));
    unit.call_site(CallSite::new("equal?", 1, Receiver::Other));
    assert_eq!(run(&unit), Object::Bool(false));
}

#[test]
fn set_local_writes_the_local_the_scope_holds() {
    let mut machine = machine_after("count = 1\n");
    let mut unit = unit_of(
        1,
        vec![],
        &[
            LoadInteger { to: 0, value: 5 },
            SetLocal { from: 0, slot: 0 },
            Return { from: 0 },
        ],
    );
    unit.local("count");
    machine.run_unit(&unit).unwrap();
    let statements = Parser::new(Lexer::new("count").tokenize()).parse().unwrap();
    assert_eq!(
        machine.execute_program(&statements).unwrap(),
        Some(Object::Int(5))
    );
}

#[test]
fn get_local_reads_what_a_binding_wrote_to_the_local() {
    let mut machine = machine_after("count = 1\n");
    let mut unit = unit_of(
        3,
        vec![],
        &[
            LoadInteger { to: 0, value: 2 },
            SetLocal { from: 0, slot: 0 },
            GetIdentifier { to: 0, name: 0 },
            LoadSymbol { to: 1, name: 1 },
            LoadInteger { to: 2, value: 9 },
            Call { base: 0, site: 0 },
            GetLocal { to: 0, slot: 0 },
            Return { from: 0 },
        ],
    );
    unit.local("count");
    unit.name(Name::new("binding"));
    unit.name(Name::new("count"));
    unit.call_site(CallSite::new("local_variable_set", 2, Receiver::Other));
    assert_eq!(machine.run_unit(&unit).unwrap(), Object::Int(9));
}

#[test]
fn get_local_reads_the_scope_before_the_unit_writes_the_local() {
    let mut machine = machine_after("count = 4\n");
    let mut unit = unit_of(
        1,
        vec![],
        &[GetLocal { to: 0, slot: 0 }, Return { from: 0 }],
    );
    unit.local("count");
    assert_eq!(machine.run_unit(&unit).unwrap(), Object::Int(4));
}

#[test]
fn get_identifier_calls_a_method_of_that_name() {
    let mut machine = machine_after("def answer\n  42\nend\n");
    let mut unit = unit_of(
        1,
        vec![],
        &[GetIdentifier { to: 0, name: 0 }, Return { from: 0 }],
    );
    unit.name(Name::new("answer"));
    assert_eq!(machine.run_unit(&unit).unwrap(), Object::Int(42));
}
