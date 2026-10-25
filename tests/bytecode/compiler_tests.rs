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
            Unhoist { slot: 0 },
            LoadInteger { to: 0, value: 1 },
            SetLocal { from: 0, slot: 0 },
            Unhoist { slot: 1 },
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
    let starts: Vec<(usize, Option<usize>, u32)> = unit
        .statement_starts
        .iter()
        .enumerate()
        .filter_map(|(at, start)| start.map(|start| (at, start.index, start.position.line)))
        .collect();
    assert_eq!(starts, vec![(0, Some(0), 1), (3, Some(1), 2)]);
}

#[test]
fn a_method_call_puts_its_arguments_after_the_receiver() {
    let unit = unit_of("1.between?(0, 2)");
    assert_eq!(
        unit.instructions,
        vec![
            LoadInteger { to: 0, value: 1 },
            LoadInteger { to: 1, value: 0 },
            LoadInteger { to: 2, value: 2 },
            Call { base: 0, site: 0 },
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
fn a_body_needing_more_registers_than_an_instruction_names_names_the_node_holding_them() {
    let power = format!("{}\n", vec!["held"; 300].join(" ** "));
    // Parsing and compiling a power nested this deep takes more stack than a
    // test thread starts with.
    let refused = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || compiled(&power).err())
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(refused, Some("BinaryOp"));
}

#[test]
fn a_begin_body_runs_ahead_of_the_statements_written_before_it() {
    let unit = unit_of("first = seen\nBEGIN { seen = 1 }\n");
    assert_eq!(unit.instructions[0].name(), "BeginBlockOnce");
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
    assert!(unit.instructions.contains(&AddToHash { hash: 0, shape: 1 }));
    assert_eq!(
        VirtualMachine::new().run_unit(&unit).unwrap(),
        Object::Int(100)
    );
}

#[test]
fn an_operator_assignment_on_a_receiver_reads_and_writes_one_target() {
    use metorex::bytecode::unit::Target;
    let unit = unit_of("held = []\nheld[0] += 1\n");
    let reads = unit
        .instructions
        .iter()
        .filter(|instruction| matches!(instruction, ReadTarget { .. }))
        .count();
    let stores = unit
        .instructions
        .iter()
        .filter(|instruction| matches!(instruction, StoreTarget { .. }))
        .count();
    assert_eq!((reads, stores), (1, 1));
    assert!(matches!(
        unit.targets[0],
        Target::Subscript {
            count: 1,
            sends: false,
            ..
        }
    ));
}

#[test]
fn a_safe_setter_skips_the_value_when_there_is_no_receiver() {
    let unit = unit_of("held = nil\nheld&.size = 3\n");
    assert!(
        unit.instructions
            .iter()
            .any(|instruction| matches!(instruction, JumpIfNil { .. }))
    );
    assert_eq!(VirtualMachine::new().run_unit(&unit).unwrap(), Object::Nil);
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
            "        0000  Unhoist { slot: 0 }  ; held\n",
            "        0001  LoadConstant { to: 0, constant: 0 }  ; 100000\n",
            "        0002  SetLocal { from: 0, slot: 0 }  ; held\n",
            "    2 | $total = held\n",
            "        0003  GetLocal { to: 0, slot: 0 }  ; held\n",
            "        0004  SetGlobal { from: 0, name: 0 }  ; $total\n",
            "    3 | :done\n",
            "        0005  LoadSymbol { to: 0, name: 1 }  ; :done\n",
            "        0006  Return { from: 0 }\n",
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

#[test]
fn constant_assignments_of_every_form_compile() {
    let unit = compiled(
        "LIMIT = 3\nLIMIT_TWICE ||= LIMIT * 2\nSettings::NESTED = 4\nSettings::NESTED ||= 5\n::TOP = 1\n",
    );
    assert_eq!(unit.err(), None);
}

#[test]
fn multiple_assignments_of_every_form_compile() {
    for source in [
        "first, second = 1, 2\n",
        "head, *rest = [1, 2, 3]\n",
        "(inner, other), last = [4, 5], 6\n",
        "*start, final = 7, 8, 9\n",
        "box.size, box.label = 10, \"ten\"\n",
        "values[0], values[1] = :x, :y\n",
    ] {
        assert_eq!(compiled(source).err(), None, "{source}");
    }
}

#[test]
fn declared_locals_are_bound_through_their_slots() {
    let declared = metorex::ast::Statement::DeclareLocals {
        names: vec!["item".to_string()],
        position: metorex::lexer::Position::new(1, 1, 0),
    };
    let unit = compile(std::slice::from_ref(&declared)).unwrap();
    assert_eq!(
        unit.instructions,
        vec![
            DeclareLocal { slot: 0 },
            LoadNil { to: 0 },
            Return { from: 0 },
        ]
    );
    assert_eq!(unit.locals[0].text, "item");
    let mut machine = VirtualMachine::new();
    machine.run_unit(&unit).unwrap();
    let read = Parser::new(Lexer::new("item").tokenize()).parse().unwrap();
    assert_eq!(machine.execute_program(&read).unwrap(), Some(Object::Nil));
}

#[test]
fn a_call_with_a_splat_gathers_its_arguments_in_a_list() {
    let unit = unit_of("held = [1]\nheld.push(*held) { }\n");
    let site = &unit.call_sites[0];
    assert!(site.arguments_in_list && site.has_block);
    assert_eq!(site.block_offset(), 1);
    let gathered: Vec<&str> = unit
        .instructions
        .iter()
        .filter(|instruction| {
            matches!(
                instruction,
                StartArguments | SplatArgument { .. } | MakeBlock { .. }
            )
        })
        .map(Instruction::name)
        .collect();
    assert_eq!(
        gathered,
        vec!["StartArguments", "SplatArgument", "MakeBlock"]
    );
}

#[test]
fn a_call_without_a_receiver_decides_its_route_before_its_arguments() {
    let names: Vec<&str> = instructions_of("format(\"%d\", 1)")
        .iter()
        .map(Instruction::name)
        .collect();
    assert_eq!(
        names,
        vec![
            "ResolveCall",
            "NewString",
            "LoadInteger",
            "CallResolved",
            "Move",
            "Return"
        ]
    );
}

#[test]
fn a_safe_call_skips_its_arguments_when_there_is_no_receiver() {
    let unit = unit_of("held = nil\nheld&.push(1)\n");
    assert!(
        unit.instructions
            .iter()
            .any(|instruction| matches!(instruction, SafeNavigation { .. }))
    );
    assert_eq!(VirtualMachine::new().run_unit(&unit).unwrap(), Object::Nil);
}

#[test]
fn branches_and_loops_of_every_form_compile() {
    for source in [
        "if held then 1 elsif other then 2 else 3 end\n",
        "unless held then 1 else 2 end\n",
        "found = if held then 1 end\n",
        "count = 0\nwhile count < 3\n  count += 1\n  next if count == 1\n  break if count > 2\n  redo if count == 0\nend\n",
        "count = 0\ncount += 1 until count > 3\n",
        "sum = 0\nfor number in [1, 2]\n  sum += number\nend\n",
        "named = case held\nwhen Integer then 1\nelse 2\nend\n",
        "[1].each { |number| next number }\n",
        "[1].each { |number| break number }\n",
    ] {
        assert_eq!(compiled(source).err(), None, "{source}");
    }
}

#[test]
fn a_do_while_loop_runs_its_body_before_it_reads_its_condition() {
    let parsed = Parser::new(Lexer::new("count = 0\ncount += 1\ncount < 0\ncount\n").tokenize())
        .parse()
        .unwrap();
    let metorex::ast::Statement::Expression {
        expression: condition,
        ..
    } = parsed[2].clone()
    else {
        panic!("not an expression: {:?}", parsed[2]);
    };
    let looped = metorex::ast::Statement::DoWhile {
        condition: Box::new(condition),
        body: vec![parsed[1].clone()],
        position: metorex::lexer::Position::new(2, 1, 0),
    };
    let unit = compile(&[parsed[0].clone(), looped, parsed[3].clone()]).unwrap();
    assert_eq!(
        VirtualMachine::new().run_unit(&unit).unwrap(),
        Object::Int(1)
    );
}

#[test]
fn a_while_loop_answers_what_break_carries() {
    let unit =
        unit_of("count = 0\nwhile true\n  count += 1\n  break count * 10 if count == 3\nend\n");
    assert_eq!(
        VirtualMachine::new().run_unit(&unit).unwrap(),
        Object::Int(30)
    );
}
