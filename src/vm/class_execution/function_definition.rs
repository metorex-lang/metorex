// Running a `def` written outside a class body.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Execute function definition - create a Method object and register it in the environment as a function.
    pub(crate) fn execute_function_def(
        &mut self,
        name: &str,
        parameters: &[crate::ast::Parameter],
        body: &crate::ast::Body,
        position: crate::lexer::Position,
        singleton_class: Option<&str>,
        end_line: usize,
    ) -> Result<ControlFlow, MetorexError> {
        // Create source location from position, naming the file the `def` was
        // read from the way that file names itself. Code handed to `eval`
        // with a name of its own reports that name rather than whatever is
        // running when the location is asked for.
        let mut source_location =
            crate::error::SourceLocation::new(position.line, position.column, position.offset);
        source_location.filename = self
            .loading_embedded_library
            .clone()
            .or_else(|| self.current_source_file.clone())
            .or_else(|| {
                self.reported_current_file()
                    .map(|file| file.display().to_string())
            });

        let mut function = build_method_from_params(
            name.to_string(),
            parameters,
            body.clone(),
            self.snapshot_active_refinements(),
            self.def_nesting(),
        );
        function.source_location = Some(source_location);
        function.end_line = Some(end_line);
        // A method defined outside every class is Object's, and a `def` run
        // while it runs installs on Object too.
        function.definee = self
            .running_method_def_scope()
            .or_else(|| self.def_scope_stack.last().cloned())
            .or_else(|| match self.globals().get("Object").view() {
                Some(View::Class(object_class)) => Some(object_class.clone()),
                _ => None,
            });
        // A `def` written outside every class belongs to Object, which is the
        // name a backtrace gives it however it is reached.
        if singleton_class.is_none()
            && self.def_scope_stack.is_empty()
            && let Some(View::Class(object_class)) = self.globals().get("Object").view()
        {
            function.owner = Some(object_class.name().to_string());
            function.owner_class = Some(object_class.clone());
        }
        let function = Rc::new(function);

        // Singleton method: define on the specific class (e.g., TrueClass)
        // or on a specific instance (`def x.foo` where `x` is an Object).
        if let Some(receiver_name) = singleton_class {
            let sole_instance_receiver =
                receiver_name.strip_prefix(crate::parser::SOLE_INSTANCE_RECEIVER);
            let receiver_name = sole_instance_receiver.unwrap_or(receiver_name);
            // `def @obj.name` reads the instance variable off the current
            // self; `def $stream.name` reads the global.
            let resolved = if let Some(variable) = receiver_name.strip_prefix('@') {
                self.eval_instance_var_read(variable, position).ok()
            } else if let Some(variable) = receiver_name.strip_prefix('$') {
                self.globals().get(variable)
            } else if receiver_name == "self" {
                // At the top level `self` is `main`, which is bound nowhere
                // in the environment and has to be asked for.
                self.eval_self(position).ok()
            } else {
                // A constant receiver is found the way the constant is read
                // anywhere, from the class body the `def` sits in outwards.
                self.environment()
                    .get(receiver_name)
                    .or_else(|| self.resolve_constant_in_scope(receiver_name))
                    .or_else(|| self.globals().get(receiver_name))
            };
            // A lowercase name no local holds is a method of `self`, and the
            // method is defined on the object it answers.
            let resolved = match resolved {
                None if receiver_name
                    .starts_with(|first: char| first.is_ascii_lowercase() || first == '_') =>
                {
                    let receiver = self.eval_self(position)?;
                    Some(self.send_to_object(receiver, receiver_name, Vec::new(), position)?)
                }
                found => found,
            };
            if sole_instance_receiver.is_some() {
                match resolved.view() {
                    Some(View::Class(target_class)) => {
                        target_class.define_method(name, Rc::clone(&function));
                    }
                    // A `def self.name` written where `self` is an object
                    // rather than a class names that object alone.
                    Some(receiver @ View::Instance(_)) => {
                        let singleton = self.singleton_class_of(&receiver.to_object());
                        singleton.define_method(name, Rc::clone(&function));
                        self.invoke_class_hook(
                            &singleton,
                            "singleton_method_added",
                            name,
                            position,
                        )?;
                    }
                    _ => {}
                }
                return Ok(ControlFlow::Value(Object::symbol(name)));
            }
            match resolved.view() {
                // `def Klass.name` / `def mod.name` defines a singleton
                // method, stored under the same `__class__` convention as
                // `def self.name` inside the body.
                Some(View::Class(target_class)) | Some(View::Module(target_class)) => {
                    // A singleton method is written on the singleton class,
                    // so either one being frozen refuses it.
                    let singleton_frozen = target_class
                        .singleton_class_slot()
                        .as_ref()
                        .is_some_and(|singleton| singleton.is_frozen());
                    if singleton_frozen {
                        let receiver = if target_class.is_module() {
                            Object::Module(Rc::clone(target_class))
                        } else {
                            Object::Class(Rc::clone(target_class))
                        };
                        return Err(self.frozen_modification_error(&receiver, position));
                    }
                    self.refuse_frozen_definee(target_class, position)?;
                    target_class.define_method(format!("__class__{}", name), Rc::clone(&function));
                    self.invoke_class_hook(target_class, "singleton_method_added", name, position)?;
                }
                // `def obj.name` installs on the object's singleton class,
                // the same place `define_singleton_method` and `class << obj`
                // put one, so `methods`, `undef_method`, and `remove_method`
                // all see it.
                // `def obj.name` installs on any receiver's singleton class,
                // exceptions included.
                Some(
                    receiver @ (View::Instance(_)
                    | View::Exception(_)
                    | View::Array(_)
                    | View::Dict(_)
                    | View::Set(_)
                    | View::String(_)
                    | View::Regex(_, _)
                    | View::Block(_)
                    | View::Method(_)
                    | View::Binding(_)),
                ) => {
                    if self.object_is_frozen(&receiver.to_object()) {
                        return Err(self.frozen_modification_error(&receiver.to_object(), position));
                    }
                    let singleton = self.singleton_class_of(&receiver.to_object());
                    singleton.define_method(name, Rc::clone(&function));
                    self.invoke_class_hook(&singleton, "singleton_method_added", name, position)?;
                }
                // A number, a symbol, and the like are the same object
                // wherever they turn up, so none of them can carry a method
                // of its own.
                Some(
                    receiver @ (View::Int(_) | View::BigInt(_) | View::Float(_) | View::Symbol(_)),
                ) => {
                    let message = format!(
                        "can't define singleton method \"{}\" for {}",
                        name,
                        self.builtins().class_of(&receiver.to_object()).name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: position_to_location(position),
                        message,
                    });
                }
                _ => {}
            }
            return Ok(ControlFlow::Value(Object::symbol(name)));
        }

        // Register the function in the environment (for immediate local access)
        self.environment_mut()
            .define(name.to_string(), Object::Method(Rc::clone(&function)));

        // Ruby installs the method on the current default definee: the
        // innermost lexical class/module if there is one, otherwise Object
        // (top-level `def`, globally accessible).
        // A `def` run from a method body installs where that method was
        // defined, and is public whatever visibility the class body left.
        let from_method_body = self.running_method_def_scope();
        if let Some(definee) = from_method_body
            .clone()
            .or_else(|| self.def_scope_stack.last().cloned())
        {
            self.refuse_frozen_definee(&definee, position)?;
            definee.define_method(name, Rc::clone(&function));
            if from_method_body.is_some() {
                definee.clear_method_visibility(name);
            } else {
                apply_current_visibility(&definee, name);
            }
            // A method written at the top level of a wrapped load belongs to
            // the module the load was wrapped in, and a top-level method is
            // private, so it is reached through that module rather than by
            // anything that mixes the module in.
            if self.load_wrap_depth > 0 && self.def_scope_stack.len() == 1 {
                definee.set_method_private(name);
            }
            let hook = Self::method_added_hook_for(&definee);
            self.invoke_class_hook(&definee, hook, name, position)?;
            if module_function_is_active(&definee) {
                self.copy_to_module_function(&definee, name, position)?;
            }
        } else if let Some(View::Class(object_class)) = self.globals().get("Object").view() {
            object_class.define_method(name, Rc::clone(&function));
            // A method defined in top-level code is private unless `public`
            // was called there. One defined when a method runs is public.
            let home = self.lexical_home_frame.unwrap_or(self.current_method_frame);
            let in_top_level_code = home == self.toplevel_frame;
            if in_top_level_code && !self.toplevel_public {
                object_class.set_method_private(name);
            } else {
                object_class.clear_method_visibility(name);
            }
        }

        Ok(ControlFlow::Value(Object::symbol(name)))
    }
}
