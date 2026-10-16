// Compiling a body's statements to a unit.

use metorex::bytecode::compiler::compile;
use metorex::bytecode::instruction::Instruction::{self, *};
use metorex::bytecode::unit::Unit;
use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn compiled(source: &str) -> Result<Unit, &'static str> {
    let statements = Parser::new(Lexer::new(source).tokenize())
        .parse()
        .expect("parse failed");
    compile(&statements)
}

fn unit_of(source: &str) -> Unit {
    compiled(source).expect("not compiled")
}

fn instructions_of(source: &str) -> Vec<Instruction> {
    unit_of(source).instructions
}

#[test]
fn an_empty_body_answers_nil() {
    assert_eq!(
        instructions_of(""),
        vec![LoadNil { to: 0 }, Return { from: 0 }]
    );
}

#[test]
fn locals_are_numbered_in_the_order_the_body_assigns_them() {
    let unit = unit_of("first = 1\nsecond = first\n");
    let names: Vec<&str> = unit.locals.iter().map(|name| name.text.as_str()).collect();
    assert_eq!(names, vec!["first", "second"]);
    assert_eq!(
        unit.instructions,
        vec![
            LoadInteger { to: 0, value: 1 },
            SetLocal { from: 0, slot: 0 },
            GetLocal { to: 0, slot: 0 },
            SetLocal { from: 0, slot: 1 },
            Return { from: 0 },
        ]
    );
}

#[test]
fn a_name_the_body_does_not_assign_is_read_as_an_identifier() {
    assert_eq!(
        instructions_of("outer"),
        vec![GetIdentifier { to: 0, name: 0 }, Return { from: 0 }]
    );
}

#[test]
fn a_capitalized_name_is_read_as_a_constant() {
    assert_eq!(
        instructions_of("Integer"),
        vec![GetConstant { to: 0, name: 0 }, Return { from: 0 }]
    );
}

#[test]
fn each_statement_starts_at_its_first_instruction() {
    let unit = unit_of("first = 1\nsecond = 2\n");
    let starts: Vec<(usize, usize, usize)> = unit
        .statement_starts
        .iter()
        .enumerate()
        .filter_map(|(at, start)| start.map(|start| (at, start.index, start.position.line)))
        .collect();
    assert_eq!(starts, vec![(0, 0, 1), (2, 1, 2)]);
}

#[test]
fn a_method_call_puts_its_arguments_after_the_receiver() {
    let unit = unit_of("1.between?(0, 2)");
    assert_eq!(
        unit.instructions,
        vec![
            LoadInteger { to: 1, value: 1 },
            LoadInteger { to: 2, value: 0 },
            LoadInteger { to: 3, value: 2 },
            Call { base: 1, site: 0 },
            Move { to: 0, from: 1 },
            Return { from: 0 },
        ]
    );
    assert_eq!(unit.call_sites[0].name, "between?");
    assert_eq!(unit.call_sites[0].argument_count, 2);
}

#[test]
fn an_integer_past_sixteen_bits_is_a_constant() {
    let unit = unit_of("100000");
    assert_eq!(
        unit.instructions,
        vec![LoadConstant { to: 0, constant: 0 }, Return { from: 0 }]
    );
    assert_eq!(unit.constants, vec![Object::Int(100_000)]);
}

#[test]
fn a_call_is_pointed_at_by_its_receiver_and_method_name() {
    let unit = unit_of("\n  held.missing");
    let spot = unit.error_spots[1].clone().expect("no spot");
    assert_eq!((spot.start.line, spot.start.column), (2, 3));
    assert_eq!(spot.name.as_deref(), Some("missing"));
}

#[test]
fn a_body_holding_a_node_the_compiler_does_not_handle_names_it() {
    assert_eq!(compiled("while false\nend\n").err(), Some("While"));
    assert_eq!(compiled("held = -> { 1 }\n").err(), Some("Lambda"));
    assert_eq!(compiled("[1].each { |n| n }\n").err(), Some("MethodCall"));
}

#[test]
fn a_compiled_body_answers_its_last_statement() {
    let unit =
        unit_of("width = 2\nheight = width * 3\n[width, height, -height, !nil, :done, \"text\"]\n");
    assert_eq!(
        VirtualMachine::new().run_unit(&unit).unwrap(),
        Object::array(vec![
            Object::Int(2),
            Object::Int(6),
            Object::Int(-6),
            Object::Bool(true),
            Object::symbol("done"),
            Object::string("text"),
        ])
    );
}

#[test]
fn a_long_array_literal_is_built_a_chunk_at_a_time() {
    let elements: Vec<String> = (0..300).map(|number| number.to_string()).collect();
    let unit = unit_of(&format!("[{}]", elements.join(", ")));
    let appends = unit
        .instructions
        .iter()
        .filter(|instruction| matches!(instruction, AppendArray { .. }))
        .count();
    assert_eq!(appends, 4);
    assert!(unit.register_count < 100);
    let built = VirtualMachine::new().run_unit(&unit).unwrap();
    assert_eq!(built, Object::array((0..300).map(Object::Int).collect()));
}

#[test]
fn a_long_hash_literal_is_built_a_chunk_at_a_time() {
    let pairs: Vec<String> = (0..100)
        .map(|number| format!("{number} => {number}"))
        .collect();
    let unit = unit_of(&format!("{{{}}}.size", pairs.join(", ")));
    assert!(unit.instructions.contains(&AddToHash { hash: 1, shape: 1 }));
    assert_eq!(
        VirtualMachine::new().run_unit(&unit).unwrap(),
        Object::Int(100)
    );
}

#[test]
fn an_assignment_to_a_target_not_compiled_is_refused_as_an_assignment() {
    assert_eq!(compiled("Math::HELD = 1\n").err(), Some("Assignment"));
}

#[test]
fn disassembly_notes_what_each_side_table_operand_names() {
    let source = "held = 100000\n$total = held\n:done\n";
    let text =
        metorex::bytecode::disassembly::disassemble(&unit_of(source), "notes.rb", Some(source));
    assert_eq!(
        text,
        concat!(
            "== notes.rb\n",
            "    1 | held = 100000\n",
            "        0000  LoadConstant { to: 0, constant: 0 }  ; 100000\n",
            "        0001  SetLocal { from: 0, slot: 0 }  ; held\n",
            "    2 | $total = held\n",
            "        0002  GetLocal { to: 0, slot: 0 }  ; held\n",
            "        0003  SetGlobal { from: 0, name: 0 }  ; $total\n",
            "    3 | :done\n",
            "        0004  LoadSymbol { to: 0, name: 1 }  ; :done\n",
            "        0005  Return { from: 0 }\n",
        )
    );
}

#[test]
fn disassembly_without_the_source_writes_line_numbers_alone() {
    let text = metorex::bytecode::disassembly::disassemble(&unit_of("nil"), "eval", None);
    assert_eq!(
        text,
        "== eval\n    1 |\n        0000  LoadNil { to: 0 }\n        0001  Return { from: 0 }\n"
    );
}

#[test]
fn a_range_reads_its_ends_from_two_registers() {
    use metorex::bytecode::instruction::{RANGE_EXCLUSIVE, RANGE_LITERAL_ENDS};
    assert_eq!(
        instructions_of("1...3"),
        vec![
            LoadInteger { to: 1, value: 1 },
            LoadInteger { to: 2, value: 3 },
            NewRange {
                to: 0,
                first: 1,
                flags: RANGE_EXCLUSIVE | RANGE_LITERAL_ENDS,
            },
            Return { from: 0 },
        ]
    );
    let unit = unit_of("first = 1\nfirst..first");
    assert!(unit.instructions.contains(&NewRange {
        to: 0,
        first: 1,
        flags: 0,
    }));
}

#[test]
fn a_hash_lays_out_each_pair_and_spread_in_order() {
    use metorex::bytecode::unit::HashEntry;
    let unit = unit_of("{ a: 1, **{}, b: 2 }");
    let entries: Vec<usize> = unit.hashes[0]
        .entries
        .iter()
        .map(HashEntry::register_count)
        .collect();
    assert_eq!(unit.hashes[0].first, 1);
    assert_eq!(entries, vec![2, 2]);
    let unit = unit_of("extra = {}\n{ a: 1, **extra }");
    let spread = unit.hashes[1].entries[1].clone();
    assert!(matches!(spread, HashEntry::Spread { .. }));
}

#[test]
fn each_hole_of_an_interpolation_is_written_as_text_before_the_next_is_read() {
    use metorex::bytecode::unit::Piece;
    let unit = unit_of("\"a#{1}b#{2}\"");
    assert_eq!(
        unit.instructions,
        vec![
            LoadInteger { to: 1, value: 1 },
            ToText { to: 2, from: 1 },
            LoadInteger { to: 3, value: 2 },
            ToText { to: 4, from: 3 },
            Interpolate { to: 0, shape: 0 },
            Return { from: 0 },
        ]
    );
    let registers: Vec<Option<usize>> = unit.interpolations[0]
        .iter()
        .map(|piece| match piece {
            Piece::Text(_) => None,
            Piece::Value { register, .. } => Some(*register),
        })
        .collect();
    assert_eq!(registers, vec![None, Some(1), None, Some(3)]);
}

#[test]
fn a_scoped_constant_reads_its_namespace_in_the_register_it_answers_in() {
    assert_eq!(
        instructions_of("Math::PI"),
        vec![
            GetConstant { to: 0, name: 0 },
            GetScopedConstant {
                namespace: 0,
                name: 1,
            },
            Return { from: 0 },
        ]
    );
}

#[test]
fn magic_names_and_big_integers_load_what_they_name() {
    assert_eq!(
        instructions_of("__FILE__\n__dir__\n__LINE__\n"),
        vec![
            LoadFile { to: 0 },
            LoadDirectory { to: 0 },
            LoadInteger { to: 0, value: 3 },
            Return { from: 0 },
        ]
    );
    let unit = unit_of("123456789012345678901234567890");
    assert_eq!(
        unit.constants,
        vec![Object::integer(
            "123456789012345678901234567890"
                .parse::<num_bigint::BigInt>()
                .unwrap()
        )]
    );
}
