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

#[test]
fn a_held_copy_writes_its_bodies_after_the_statements_around_them() {
    let held = encode_held(&vec![method_returning(42)]);
    let outer_length = u32::from_le_bytes(held[..4].try_into().unwrap()) as usize;
    let body_bytes = held.len() - 4 - outer_length;

    let encoded_body = rmp_serde::to_vec(&vec![Statement::Expression {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(2, 3),
        },
        position: pos(2, 3),
    }])
    .unwrap();

    assert_eq!(body_bytes, encoded_body.len());
}
