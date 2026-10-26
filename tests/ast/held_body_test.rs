use super::helpers::pos;
use metorex::ast::{Body, Expression, Statement, decode_held, encode_held};

fn method_returning(value: i64) -> Statement {
    Statement::MethodDef {
        name: "answer".to_string(),
        parameters: Vec::new(),
        body: Body::from(vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }]),
        is_class_method: false,
        position: pos(1, 1),
        end_position: pos(3, 1),
    }
}

#[test]
fn a_body_is_one_pointer_wide() {
    assert_eq!(std::mem::size_of::<Body>(), std::mem::size_of::<usize>());
}

#[test]
fn a_held_copy_decodes_to_the_statements_it_was_encoded_from() {
    let statements = vec![method_returning(42), method_returning(7)];
    let held: &'static [u8] = Vec::leak(encode_held(&statements));

    let decoded: Vec<Statement> = decode_held(held);

    assert_eq!(decoded, statements);
}

fn class_holding(statement: Statement) -> Statement {
    Statement::ClassDef {
        name: "Ledger".to_string(),
        namespace: None,
        superclass: None,
        superclass_expression: None,
        body: Body::from(vec![statement]),
        position: pos(1, 1),
    }
}

/// The lengths `encode_held` writes first: of the statements around the
/// bodies, and of the bodies that run as the copy loads.
fn area_lengths(held: &[u8]) -> (usize, usize) {
    let length = |at: usize| u32::from_le_bytes(held[at..at + 4].try_into().unwrap()) as usize;
    (length(0), length(4))
}

fn encoded_answer(value: i64) -> Vec<u8> {
    rmp_serde::to_vec(&vec![Statement::Expression {
        expression: Expression::IntLiteral {
            value,
            position: pos(2, 3),
        },
        position: pos(2, 3),
    }])
    .unwrap()
}

#[test]
fn a_held_copy_writes_a_method_body_after_the_bodies_that_run_as_it_loads() {
    let held = encode_held(&vec![class_holding(method_returning(42))]);
    let (outer_length, at_load_length) = area_lengths(&held);
    let when_called_length = held.len() - 8 - outer_length - at_load_length;

    assert_eq!(when_called_length, encoded_answer(42).len());
}

#[test]
fn a_held_copy_decodes_a_class_holding_a_method() {
    let statements = vec![class_holding(method_returning(7))];
    let held: &'static [u8] = Vec::leak(encode_held(&statements));

    let decoded: Vec<Statement> = decode_held(held);

    assert_eq!(decoded, statements);
}

#[test]
fn a_position_is_two_words_wide() {
    assert_eq!(std::mem::size_of::<metorex::lexer::Position>(), 16);
}

#[test]
fn an_expression_takes_88_bytes() {
    assert_eq!(std::mem::size_of::<Expression>(), 88);
}

#[test]
fn a_statement_takes_120_bytes() {
    assert_eq!(std::mem::size_of::<Statement>(), 120);
}

fn answer_at(position: metorex::lexer::Position) -> Statement {
    Statement::Expression {
        expression: Expression::IntLiteral {
            value: 42,
            position,
        },
        position,
    }
}

#[test]
fn a_held_body_reads_once_without_keeping_its_tree() {
    let body = Body::held(vec![answer_at(pos(2, 3))]);

    let read = body.read_once(<[Statement]>::to_vec);

    assert_eq!(
        (read, body.is_decoded()),
        (vec![answer_at(pos(2, 3))], false)
    );
}

#[test]
fn a_held_body_says_the_program_wrote_it_without_being_decoded() {
    let body = Body::held(vec![answer_at(pos(2, 3))]);

    assert_eq!(
        (body.opens_in_the_program(), body.is_decoded()),
        (true, false)
    );
}

#[test]
fn a_body_opening_in_the_prelude_says_the_program_did_not_write_it() {
    let mut position = pos(2, 3);
    position.prelude = true;

    assert!(!Body::held(vec![answer_at(position)]).opens_in_the_program());
}

#[test]
fn an_empty_body_says_the_program_did_not_write_it() {
    assert!(!Body::from(Vec::new()).opens_in_the_program());
}

#[test]
fn a_held_copy_keeps_whether_the_program_wrote_a_method_body() {
    let held: &'static [u8] = Vec::leak(encode_held(&vec![method_returning(42)]));

    let decoded: Vec<Statement> = decode_held(held);

    let Statement::MethodDef { body, .. } = &decoded[0] else {
        panic!("a method definition");
    };
    assert_eq!(
        (body.opens_in_the_program(), body.is_decoded()),
        (true, false)
    );
}

#[test]
fn a_parameter_takes_56_bytes() {
    assert_eq!(std::mem::size_of::<metorex::ast::Parameter>(), 56);
}
