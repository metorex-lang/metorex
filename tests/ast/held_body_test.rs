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
