// Whether two values are the same object, and the ids and names
// that follow from it.

use super::*;
use crate::object::View;

/// Whether two values are the same object. Reference types compare by
/// identity, immediates by value.
pub(crate) fn same_object(left: &Object, right: &Object) -> bool {
    use std::rc::Rc;
    match (left.view(), right.view()) {
        (View::Instance(a), View::Instance(b)) => Rc::ptr_eq(a, b),
        (View::String(a), View::String(b)) => Rc::ptr_eq(a, b),
        (View::Array(a), View::Array(b)) => Rc::ptr_eq(a, b),
        (View::Dict(a), View::Dict(b)) => Rc::ptr_eq(a, b),
        (View::Class(a), View::Class(b)) => Rc::ptr_eq(a, b),
        (View::Module(a), View::Module(b)) => Rc::ptr_eq(a, b),
        (View::Symbol(a), View::Symbol(b)) => a == b,
        (View::Int(a), View::Int(b)) => a == b,
        (View::Float(a), View::Float(b)) => a == b,
        (View::Bool(a), View::Bool(b)) => a == b,
        (View::Nil, View::Nil) => true,
        _ => false,
    }
}

/// A `Method` that forwards to the receiver's `method_missing`, for a name the
/// object claims through `respond_to_missing?`. The name goes in front of the
/// call's own arguments, so `method_missing`'s arity applies as written.
pub(crate) fn method_missing_dispatcher(
    name: &str,
    receiver: &Object,
    _position: Position,
) -> crate::object::Method {
    use crate::ast::{Expression, Statement};

    // The body is the same wherever the call asking for it was written, so
    // two of these for one name are equal the way Ruby's are.
    let position = Position::new(0, 0, 0);
    let call = Expression::MethodCall {
        receiver: Box::new(Expression::SelfExpr { position }),
        method: "method_missing".to_string(),
        arguments: vec![
            Expression::Symbol {
                value: name.to_string(),
                position,
            },
            Expression::Splat {
                expression: Box::new(Expression::Identifier {
                    name: crate::object::UNNAMED_PARAMETER.to_string(),
                    position,
                }),
                position,
            },
        ],
        trailing_block: None,
        position,
    };

    let mut method = crate::object::Method::new(
        name.to_string(),
        vec![crate::object::UNNAMED_PARAMETER.to_string()],
        vec![Statement::Expression {
            expression: call,
            position,
        }],
    );
    method.signature.variadic_param = Some((0, crate::object::UNNAMED_PARAMETER.to_string()));
    method.receiver = Some(Box::new(receiver.clone()));
    method
}

/// A stable, non-negative id derived from a value rather than an address, for
/// the objects metorex stores by value. Two equal values share an id.
pub(crate) fn value_object_id(tag: &str, value: &str) -> i64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tag.hash(&mut hasher);
    value.hash(&mut hasher);
    (hasher.finish() >> 1) as i64
}

/// One side of a `=~` comparison: the pattern, or the characters to search.
pub(crate) enum MatchSide {
    Pattern(String, String),
    Text(String),
}

/// Classify an operand of `=~`. A Symbol matches on its name, as Ruby's does.
pub(crate) fn matchable_text(object: &Object) -> Option<MatchSide> {
    match object.view() {
        View::Regex(pattern, flags) => Some(MatchSide::Pattern(
            pattern.as_str().to_string(),
            flags.as_str().to_string(),
        )),
        View::String(text) => Some(MatchSide::Text(
            crate::vm::native_methods::regexp_methods::match_text(text),
        )),
        View::Symbol(text) => Some(MatchSide::Text(text.as_str().to_string())),
        _ => None,
    }
}

/// The public method names a class defines: everything in its table that it
/// has not marked private or protected. `def self.name` is stored under the
/// `__class__` convention and belongs to the class object, not its instances.
pub(crate) fn public_method_names(class: &Class) -> Vec<String> {
    class
        .method_names()
        .into_iter()
        .filter(|name| !class.is_method_private(name) && !class.is_method_protected(name))
        .collect()
}

/// Whether a block body defines a method at its top level, which inside
/// `instance_eval` or `instance_exec` means a singleton method on the
/// receiver.
pub(crate) fn body_defines_a_method(body: &[crate::ast::Statement]) -> bool {
    body.iter().any(|statement| {
        matches!(
            statement,
            crate::ast::Statement::MethodDef { .. } | crate::ast::Statement::FunctionDef { .. }
        )
    })
}

/// The name a native method is really spelled with, for the aliases Ruby
/// documents as the same method rather than a separate one.
pub(crate) fn native_alias_target<'a>(class_name: &str, method_name: &'a str) -> Option<&'a str> {
    match (class_name, method_name) {
        // Integer answers these natively, so they are its own rather than
        // the Numeric versions the prelude also declares.
        ("Integer", "zero?") => Some("zero?"),
        // An Array renders itself the same way whichever of the two names
        // the call is written with.
        ("Array", "to_s") => Some("inspect"),
        ("Hash", "to_s") => Some("inspect"),
        ("Set", "to_s") => Some("inspect"),
        ("Set", "===") | ("Set", "member?") => Some("include?"),
        ("Set", "length") => Some("size"),
        _ => None,
    }
}
