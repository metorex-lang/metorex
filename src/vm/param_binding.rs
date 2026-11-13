//! Parameter binding utilities for the virtual machine.
//!
//! This module provides functions for binding positional, keyword, variadic,
//! and block parameters when invoking methods and functions.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::object::Object;
use crate::object::{OptionView, View};
use std::cell::RefCell;
use std::rc::Rc;

use super::VirtualMachine;

/// The key the parser adds to a keyword-argument dict so it can be told
/// apart from a Hash the caller passed positionally.
pub(crate) const KWARGS_MARKER: &str = "__MX_KWARGS__";

/// What a keyword hash key that is not a Symbol is marked with once the
/// keywords are split from the arguments. No keyword name holds it.
pub(crate) const NON_SYMBOL_KEY_MARK: char = '\0';

/// The value a parameter's default expression gives. An instance variable
/// or a literal is read here, without compiling the expression to run alone.
fn default_value(vm: &mut VirtualMachine, default: &Expression) -> Result<Object, MetorexError> {
    match default {
        Expression::InstanceVariable { name, position } => {
            vm.eval_instance_var_read(name, *position)
        }
        Expression::NilLiteral { .. } => Ok(Object::Nil),
        Expression::BoolLiteral { value, .. } => Ok(Object::Bool(*value)),
        Expression::IntLiteral { value, .. } => Ok(Object::Int(*value)),
        _ => vm.evaluate_expression(default),
    }
}

/// Count the number of positional arguments, excluding a trailing kwargs dict.
pub(crate) fn positional_arg_count(arguments: &[Object]) -> usize {
    positional_arg_count_for(arguments, true)
}

/// The positional count, given whether the callee declares any keyword
/// parameters. A callee that declares none takes a trailing keyword hash as an
/// ordinary positional argument, which is what Ruby counts it as.
pub(crate) fn positional_arg_count_for(arguments: &[Object], takes_keywords: bool) -> usize {
    if takes_keywords && let Some(View::Dict(dict_rc)) = arguments.last().view() {
        let dict = dict_rc.borrow();
        if dict.contains_key(KWARGS_MARKER) {
            return arguments.len() - 1;
        }
    }
    arguments.len()
}

/// Bind one positional parameter. A `def f((a, b))` group spreads the value
/// it is given across the names in the group, the same way a block's does.
/// The local a positional parameter is bound to. A name written twice, as
/// `def m(_, _)` does, is read back as its first, so each later one is kept
/// under a name no program can spell, where a bare `super` finds it.
pub(crate) fn parameter_local(params: &[String], index: usize) -> std::borrow::Cow<'_, str> {
    let name = &params[index];
    if params[..index].contains(name) {
        std::borrow::Cow::Owned(format!("{name}#{index}"))
    } else {
        std::borrow::Cow::Borrowed(name)
    }
}

fn define_positional_param(
    vm: &mut VirtualMachine,
    param: &str,
    value: Object,
) -> Result<(), MetorexError> {
    match param.strip_prefix(crate::object::DESTRUCTURED_GROUP_PREFIX) {
        Some(names) => crate::vm::block_execution::bind_group_names(vm, names, value),
        None => {
            vm.environment_mut().define_named(param, value);
            Ok(())
        }
    }
}

/// `define_positional_param` for a parameter bound to nil before the
/// defaults ran, which keeps the binding a default's block closed over.
fn fill_positional_param(
    vm: &mut VirtualMachine,
    param: &str,
    value: Object,
) -> Result<(), MetorexError> {
    if param.starts_with(crate::object::DESTRUCTURED_GROUP_PREFIX) {
        return define_positional_param(vm, param, value);
    }
    if !vm.environment_mut().assign_own(param, value.clone()) {
        vm.environment_mut().define_named(param, value);
    }
    Ok(())
}

/// Bind the parameter `params` names at `index` to `value`, in its place
/// among the method's locals when the scope holds them by place. `refill`
/// says the parameter was bound to nil before the defaults ran.
fn bind_positional_param(
    vm: &mut VirtualMachine,
    params: &[String],
    index: usize,
    value: Object,
    refill: bool,
) -> Result<(), MetorexError> {
    let name = parameter_local(params, index);
    match bind_frame_place(vm, index, &name, value) {
        Ok(()) => Ok(()),
        Err(value) if refill => fill_positional_param(vm, &name, value),
        Err(value) => define_positional_param(vm, &name, value),
    }
}

/// Bind the local at `place` among the method's locals to `value` when the
/// scope holds them by place and `name` is the one there, answering the
/// value back when not.
fn bind_frame_place(
    vm: &mut VirtualMachine,
    place: usize,
    name: &str,
    value: Object,
) -> Result<(), Object> {
    vm.environment()
        .current_scope_ref()
        .borrow_mut()
        .bind_frame_place(place, name, value)
}

/// Bind positional parameters to arguments, handling variadic (splat) parameters.
///
/// When a variadic param is present at index `vi`, parameters before it get one arg each,
/// the variadic param collects remaining args as an Array, and parameters after it
/// get args from the end.
pub(crate) fn bind_params(
    vm: &mut VirtualMachine,
    params: &[String],
    positional: &[Object],
    default_parameters: &[(usize, Expression)],
    variadic_param: &Option<(usize, String)>,
) -> Result<(), MetorexError> {
    // A default that assigns a local, as `def f(a = b = 1)` does, names that
    // local for the whole body whether or not the default is reached.
    for (_, default_expr) in default_parameters {
        for name in crate::ast::collect_assigned_locals_in(default_expr) {
            if vm.environment().get(&name).is_none() {
                vm.environment_mut().define(name, Object::Nil);
            }
        }
    }
    // Every parameter is a local of the method before any default runs, so
    // a default that reads its own parameter, `def f(a = a)`, reads nil.
    if !default_parameters.is_empty() {
        for (place, param) in params.iter().enumerate() {
            if let Err(nil) = bind_frame_place(vm, place, param, Object::Nil) {
                vm.environment_mut().define_named(param, nil);
            }
        }
    }
    if let Some((vi, _)) = variadic_param {
        let vi = *vi;
        let params_after_splat = params.len() - vi - 1;
        let has_a_default =
            |index: usize| default_parameters.iter().any(|(held, _)| *held == index);
        // Every required parameter is filled before any optional one, whether
        // it stands before the splat or after it, so `def f(a, b = 9, *r, q)`
        // called with two values gives `q` the second and leaves `b` at its
        // default.
        let required_before = (0..vi).filter(|index| !has_a_default(*index)).count();
        let mut for_optionals = positional
            .len()
            .saturating_sub(required_before + params_after_splat);
        let mut cursor = 0;

        for i in 0..params.len() {
            let value = if i < vi {
                // Before the splat, in the order they were written: an
                // optional one takes a value only while any are left over.
                let takes_one = if has_a_default(i) {
                    let spare = for_optionals > 0;
                    for_optionals = for_optionals.saturating_sub(1);
                    spare
                } else {
                    true
                };
                match positional.get(cursor).filter(|_| takes_one) {
                    Some(value) => {
                        cursor += 1;
                        value.clone()
                    }
                    None => match default_parameters.iter().find(|(index, _)| *index == i) {
                        Some((_, default_expr)) => default_value(vm, default_expr)?,
                        None => Object::Nil,
                    },
                }
            } else if i == vi {
                // The splat parameter: collect middle args into an array
                let upto = positional
                    .len()
                    .saturating_sub(params_after_splat)
                    .max(cursor);
                let rest: Vec<Object> = positional.get(cursor..upto).unwrap_or(&[]).to_vec();
                Object::Array(Rc::new(RefCell::new(rest)))
            } else {
                // After the splat, the last values, and where there are too
                // few they start right after what came before and run out
                // into nil.
                let from = positional
                    .len()
                    .saturating_sub(params_after_splat)
                    .max(cursor);
                positional
                    .get(from + i - vi - 1)
                    .cloned()
                    .unwrap_or(Object::Nil)
            };
            bind_positional_param(vm, params, i, value, !default_parameters.is_empty())?;
        }
    } else if let (Some(first_optional), Some(last_optional)) = (
        default_parameters.iter().map(|(index, _)| *index).min(),
        default_parameters.iter().map(|(index, _)| *index).max(),
    ) {
        // `def f(a, b = 1, c)` fills the required parameters on either side
        // first, from the front and from the back, and gives what is left to
        // the optional ones in the middle.
        let trailing_required = params.len() - last_optional - 1;
        let for_optionals = positional
            .len()
            .saturating_sub(first_optional + trailing_required);
        for i in 0..params.len() {
            let value = if i < first_optional {
                positional.get(i).cloned().unwrap_or(Object::Nil)
            } else if i <= last_optional {
                let rank = i - first_optional;
                match positional
                    .get(first_optional + rank)
                    .filter(|_| rank < for_optionals)
                {
                    Some(value) => value.clone(),
                    None => match default_parameters.iter().find(|(index, _)| *index == i) {
                        Some((_, default_expr)) => default_value(vm, default_expr)?,
                        None => Object::Nil,
                    },
                }
            } else {
                let offset_from_end = params.len() - i;
                let index = positional.len().saturating_sub(offset_from_end);
                positional.get(index).cloned().unwrap_or(Object::Nil)
            };
            bind_positional_param(vm, params, i, value, true)?;
        }
    } else {
        let by_place = vm
            .environment()
            .current_scope_ref()
            .borrow()
            .frame_leads_with(params);
        for i in 0..params.len() {
            let value = if i < positional.len() {
                positional[i].clone()
            } else {
                Object::Nil
            };
            if !by_place {
                bind_positional_param(vm, params, i, value, false)?;
                continue;
            }
            let unbound = vm
                .environment()
                .current_scope_ref()
                .borrow_mut()
                .bind_frame_parameter(i, value);
            if let Err(value) = unbound {
                define_positional_param(vm, &params[i], value)?;
            }
        }
    }
    Ok(())
}

/// Split a list of evaluated arguments into positional args and keyword args.
/// If the last argument is a Dict with symbol-style keys, it's treated as keyword args.
///
/// `has_keyword_params` indicates whether the callee declared any keyword
/// parameters. When false, the trailing kwargs dict (if any) is kept as a
/// positional argument with its `__MX_KWARGS__` marker stripped — matching
/// Ruby's behavior of folding trailing `key: value` syntax into a Hash that
/// fills the last positional parameter.
pub(crate) fn split_keyword_args(
    arguments: Vec<Object>,
    has_keyword_params: bool,
) -> (Vec<Object>, crate::object::HashEntries) {
    split_keyword_args_for(arguments, has_keyword_params, false)
}

/// The same split, told whether the callee was named by `ruby2_keywords`. A
/// method that was keeps the keyword hash marked, so the splat it lands in
/// can pass it on as keywords again.
pub(crate) fn split_keyword_args_for(
    mut arguments: Vec<Object>,
    has_keyword_params: bool,
    keeps_keywords: bool,
) -> (Vec<Object>, crate::object::HashEntries) {
    // Only split if the trailing dict carries the parser-emitted kwargs marker.
    if let Some(View::Dict(dict_rc)) = arguments.last().view() {
        let dict = dict_rc.borrow();
        if dict.contains_key("__MX_KWARGS__") {
            if !has_keyword_params {
                // Promote the kwargs dict to a regular Hash positional arg.
                let mut cleaned: crate::object::HashEntries = dict
                    .iter()
                    .filter(|(k, _)| {
                        !matches!(
                            k.as_str(),
                            "__MX_KWARGS__"
                                | crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                        )
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                if keeps_keywords {
                    cleaned.insert(
                        crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                            .to_string()
                            .into(),
                        Object::Bool(true),
                    );
                }
                drop(dict);
                arguments.pop();
                arguments.push(Object::Dict(Rc::new(RefCell::new(cleaned))));
                return (arguments, Default::default());
            }
            // The table of key objects travels with the keys that are not
            // Symbols, so a `**` parameter gives back the objects they were.
            let kwargs: crate::object::HashEntries = dict
                .iter()
                .filter(|(k, _)| {
                    !matches!(
                        k.as_str(),
                        "__MX_KWARGS__"
                            | crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                    )
                })
                .map(|(k, v)| {
                    // A Symbol key names a keyword. Any other key stays
                    // marked, so it names no keyword and a `**` parameter
                    // takes it back as the key it was.
                    let name = if let Some(stripped) = k.strip_prefix(':') {
                        stripped.to_string()
                    } else {
                        format!("{NON_SYMBOL_KEY_MARK}{k}")
                    };
                    (name.into(), v.clone())
                })
                .collect();
            drop(dict);
            arguments.pop();
            return (arguments, kwargs);
        }
    }
    (arguments, Default::default())
}
