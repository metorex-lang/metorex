//! `Module#define_method` — installs an instance method from a block, a Proc,
//! a bound `Method`, or an `UnboundMethod`.

use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{BlockStatement, Method, Object};
use crate::object::{OptionView, View};
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    /// Implements `Module#define_method` for a class or module receiver.
    ///
    /// `arguments[0]` is the new method's name; `arguments[1]`, when present,
    /// supplies the body and takes precedence over a literal block.
    pub(crate) fn module_define_method(
        &mut self,
        class_rc: &Rc<Class>,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let pending_block = self.pending_block.take();

        if arguments.is_empty() {
            return Err(raise(
                "ArgumentError",
                "wrong number of arguments (given 0, expected 1..2)",
                position,
            ));
        }
        let method_name = self.coerce_method_name(&arguments[0], "define_method", position)?;

        if class_rc.is_frozen() {
            return Err(raise(
                "FrozenError",
                &format!("can't modify frozen class: {}", class_rc.inspect_name()),
                position,
            ));
        }

        // An explicit second argument wins over a literal block, so
        // `define_method(:m, some_proc) { ... }` uses `some_proc`.
        let body_source = match arguments.get(1) {
            Some(object) => object.clone(),
            None => match pending_block {
                Some(object) => object,
                None => {
                    return Err(raise(
                        "ArgumentError",
                        "tried to create Proc object without a block",
                        position,
                    ));
                }
            },
        };

        let method = match body_source.view() {
            View::Block(block) => {
                let mut made = self.method_from_block(&method_name, block);
                // The method stands where the block was written, which is what
                // `source_location` reports for one built this way.
                let mut written = self.source_location_for(position);
                if let Some(named) = &block.source_file {
                    written.filename = Some(named.clone());
                }
                made.source_location = Some(written);
                // The method belongs to the class it was defined on, which is
                // where `super` in its body starts from, whatever class the
                // receiver is.
                if made.owner_class.is_none() {
                    made.owner = Some(class_rc.name().to_string());
                    made.owner_class = Some(Rc::clone(class_rc));
                }
                made
            }
            View::Method(source) => {
                self.rebindable_method(class_rc, &method_name, source, position)?
            }
            _ => {
                return Err(raise(
                    "TypeError",
                    &format!(
                        "wrong argument type {} (expected Proc/Method/UnboundMethod)",
                        ruby_class_name(&body_source)
                    ),
                    position,
                ));
            }
        };

        class_rc.define_method(&method_name, Rc::new(method));
        self.apply_define_method_visibility(class_rc, &method_name);

        let hook = if class_rc.is_singleton_class() {
            "singleton_method_added"
        } else {
            "method_added"
        };
        self.invoke_class_hook(class_rc, hook, &method_name, position)?;
        if class_rc.current_visibility() == crate::vm::native_methods::MODULE_FUNCTION_VISIBILITY {
            self.copy_to_module_function(class_rc, &method_name, position)?;
        }

        Ok(Object::symbol(method_name))
    }

    /// Implements `Object#define_singleton_method`: the method is installed on
    /// the receiver's singleton class, so only that object responds to it.
    pub(crate) fn object_define_singleton_method(
        &mut self,
        receiver: &Object,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        if self.object_is_frozen(receiver) {
            return Err(raise(
                "FrozenError",
                &format!("can't modify frozen object: {}", receiver),
                position,
            ));
        }
        let singleton = self.singleton_class_of(receiver);
        self.module_define_method(&singleton, arguments, position)
    }

    /// Convert a block/Proc into a method body, carrying over its parameter
    /// shape and the variables it closed over.
    pub(crate) fn method_from_block(
        &mut self,
        method_name: &str,
        block: &Rc<BlockStatement>,
    ) -> Method {
        let mut regular_params: Vec<String> = Vec::new();
        let mut variadic_param: Option<(usize, String)> = None;
        let mut block_parameter: Option<String> = None;
        let mut keyword_rest_parameter: Option<String> = None;
        let mut keyword_parameters: Vec<(String, Option<crate::ast::Expression>)> = Vec::new();
        for (index, param) in block.parameters.iter().enumerate() {
            if param == crate::object::TRAILING_COMMA_PARAM {
                // A method built from `{ |a,| }` keeps the plain `|a|` arity;
                // only procs destructure on a trailing comma.
                continue;
            }
            if param == crate::object::NO_KEYWORDS_PARAM {
                keyword_rest_parameter = Some(param.clone());
            } else if let Some(name) = param.strip_prefix("**") {
                keyword_rest_parameter = Some(name.to_string());
            } else if let Some(name) = param.strip_prefix('*') {
                variadic_param = Some((index, name.to_string()));
                regular_params.push(name.to_string());
            } else if let Some(name) = param.strip_prefix('&') {
                block_parameter = Some(name.to_string());
            } else if let Some(name) = param.strip_prefix(crate::object::KEYWORD_PARAM_PREFIX) {
                // A keyword the block declares carries its default alongside
                // the name, which the block records separately.
                keyword_parameters.push((name.to_string(), None));
            } else {
                regular_params.push(param.clone());
            }
        }

        let mut method = Method::new(method_name.to_string(), regular_params, block.body.to_vec());
        method.variadic_param = variadic_param;
        method.block_parameter = block_parameter;
        method.keyword_rest_parameter = keyword_rest_parameter;
        method.keyword_parameters = keyword_parameters;
        // Optional block params (`|a, b = 1|`) become the method's default
        // parameters, keyed by positional index.
        for (original_index, expression) in block.parameter_defaults.iter() {
            let positional_index = block.parameters[..*original_index]
                .iter()
                .filter(|p| !p.starts_with('&'))
                .count();
            method
                .default_parameters
                .push((positional_index, expression.clone()));
        }
        method.lambda_body = true;
        // A block written at the top level records no scope, and is named
        // for `<main>`.
        let scope = block.written_in.as_deref().unwrap_or("<main>");
        let label = crate::vm::native_functions::backtrace_label(scope);
        method.block_label = Some(
            match block.written_depth.unwrap_or(1) {
                0 | 1 => format!("block in {label}"),
                counted => format!("block ({counted} levels) in {label}"),
            }
            .into(),
        );
        method.captured_def_scope = block.captured_def_scope.clone();
        method.captured_constant_home = block.captured_constant_home.clone();
        // A constant in the body is looked up through the modules the block
        // was written in, as it is while the block runs as a block.
        method.captured_nesting = block.captured_nesting.clone();
        method.captured_vars = Some(if block.captured_vars.is_empty() {
            self.environment().current_scope_var_refs()
        } else {
            (*block.captured_vars).clone()
        });
        method
    }

    /// Copy a `Method`/`UnboundMethod` under a new name, first checking that
    /// its original owner permits rebinding onto `class_rc`.
    fn rebindable_method(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        source: &Rc<Method>,
        position: Position,
    ) -> Result<Method, MetorexError> {
        // A Proc built by `Method#to_proc` carries its own receiver, so there
        // is nothing to rebind and nothing to validate.
        if source.bound_self.is_none()
            && let Some(owner) = &source.owner_class
        {
            if owner.is_singleton_class() {
                if !singleton_rebind_allowed(owner, class_rc) {
                    return Err(raise(
                        "TypeError",
                        "can't bind singleton method to a different class",
                        position,
                    ));
                }
            } else if owner.superclass().is_some()
                && !class_rc.has_ancestor(owner)
                && !singleton_of_descendant(class_rc, owner)
                && !singleton_class_ends_in(class_rc, owner)
            {
                // Only class-owned methods are restricted; a module's methods
                // may be installed anywhere.
                return Err(raise(
                    "TypeError",
                    &format!(
                        "bind argument must be a subclass of {}",
                        owner.inspect_name()
                    ),
                    position,
                ));
            }
        }

        let mut method = (**source).clone();
        // The copy remembers the name it was cut from: a body-less stub needs
        // it to reach the native method, and two names for one definition are
        // equal because of it.
        method.original_name = Some(
            method
                .original_name
                .clone()
                .unwrap_or_else(|| method.name.clone()),
        );
        method.name = method_name.to_string();
        method.receiver = None;
        // The copy still stands where the original was written, which is what
        // `source_location` reports for a method defined from another.
        method.source_location = source.source_location.clone();
        method.owner = Some(class_rc.name().to_string());
        method.owner_class = Some(Rc::clone(class_rc));
        Ok(method)
    }

    /// Ruby marks `initialize` and friends private no matter the current
    /// visibility. Otherwise the new method inherits the caller's visibility,
    /// but only when the call is made from inside the target module itself.
    fn apply_define_method_visibility(&mut self, class_rc: &Rc<Class>, method_name: &str) {
        let always_private = matches!(
            method_name,
            "initialize" | "initialize_copy" | "initialize_clone" | "initialize_dup"
        );
        let defined_in_target = matches!(
            self.environment().get("self").view(),
            Some(View::Class(current)) | Some(View::Module(current))
                if Rc::ptr_eq(current, class_rc)
        );

        let current = class_rc.current_visibility();
        let inherits_private = defined_in_target
            && matches!(
                current.as_str(),
                "private" | crate::vm::native_methods::MODULE_FUNCTION_VISIBILITY
            );
        if always_private || inherits_private {
            class_rc.set_method_private(method_name.to_string());
        } else {
            class_rc.set_method_public(method_name);
        }
    }
}

/// A singleton method may only be reinstalled on the singleton class it came
/// from, or on the singleton class of a descendant of its attached class —
/// mirroring how Ruby's metaclass chain parallels the class chain.
fn singleton_rebind_allowed(owner: &Rc<Class>, target: &Rc<Class>) -> bool {
    if Rc::ptr_eq(owner, target) {
        return true;
    }
    if !target.is_singleton_class() {
        return false;
    }
    match (attached_class(owner), attached_class(target)) {
        (Some(owner_attached), Some(target_attached)) => {
            target_attached.has_ancestor(&owner_attached)
        }
        _ => false,
    }
}

/// Whether `target` is the singleton class of a class that inherits from
/// `owner`. Metorex stores class methods on the class itself rather than on a
/// metaclass, so a parent's class method has the parent as its owner where
/// Ruby would name the parent's singleton class.
fn singleton_of_descendant(target: &Rc<Class>, owner: &Rc<Class>) -> bool {
    target.is_singleton_class()
        && attached_class(target).is_some_and(|attached| attached.has_ancestor(owner))
}

/// The class or module a singleton class was created for, if it has one.
fn attached_class(singleton: &Rc<Class>) -> Option<Rc<Class>> {
    match singleton.get_class_var("__attached__").view() {
        Some(View::Class(attached)) | Some(View::Module(attached)) => Some(attached.clone()),
        _ => None,
    }
}

/// Build an exception error of the given class with `message`.
fn raise(class_name: &str, message: &str, position: Position) -> MetorexError {
    MetorexError::UncaughtException {
        exception: Object::exception(class_name, message.to_string()),
        location: position_to_location(position),
        message: message.to_string(),
    }
}

/// Ruby-visible class name, used wherever a message names an object's class.
pub(crate) fn ruby_class_name(object: &Object) -> &'static str {
    match object.view() {
        View::Nil => "NilClass",
        View::Bool(true) => "TrueClass",
        View::Bool(false) => "FalseClass",
        View::Int(_) | View::BigInt(_) => "Integer",
        View::Float(_) => "Float",
        View::String(_) => "String",
        View::Symbol(_) => "Symbol",
        View::Array(_) => "Array",
        View::Dict(_) => "Hash",
        View::Set(_) => "Set",
        View::Range { .. } => "Range",
        View::Class(_) => "Class",
        View::Module(_) => "Module",
        View::Exception(_) => "Exception",
        View::Regex(_, _) => "Regexp",
        _ => "Object",
    }
}

/// Whether `class_rc` is the singleton class of a class or module, whose
/// ancestors end in Class, Module, Object, and BasicObject, and `owner` is
/// one of those.
fn singleton_class_ends_in(class_rc: &Rc<Class>, owner: &Rc<Class>) -> bool {
    class_rc.is_singleton_class()
        && matches!(
            class_rc.get_class_var("__attached__").view(),
            Some(View::Class(_) | View::Module(_))
        )
        && matches!(owner.name(), "Class" | "Module" | "Object" | "BasicObject")
}
