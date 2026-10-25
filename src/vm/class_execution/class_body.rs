// The statements a class body is made of.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    pub(crate) fn apply_class_body(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A class body is not a method activation, so `__callee__` and
        // `__method__` inside one answer nil rather than reporting whichever
        // method happens to be running further down the stack.
        // Ruby names the body by the class alone, without the namespace it
        // was written in, and a singleton class body by what it is.
        // The body stands where the class was opened, which a backtrace
        // names as the place in the code around it.
        let frame = crate::vm::CallFrame::boundary(class_body_label(class))
            .with_location(Some(format!("{}:{}", position.line, position.column)))
            .with_source_file(self.file_for_frames());
        self.apply_class_body_in_frame(class, body, position, frame, None)
    }

    /// Run a class body in `frame`, which a block run as one, as
    /// `class_eval` runs it, names as the block. Such a block's `return`
    /// goes back to `block_home`.
    pub(crate) fn apply_class_body_in_frame(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
        frame: crate::vm::CallFrame,
        block_home: Option<Option<u64>>,
    ) -> Result<Object, MetorexError> {
        self.body_block_homes.push(block_home);
        let answer = self.apply_class_body_in_pushed_frame(class, body, position, frame);
        self.body_block_homes.pop();
        answer
    }

    fn apply_class_body_in_pushed_frame(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
        frame: crate::vm::CallFrame,
    ) -> Result<Object, MetorexError> {
        // Reset visibility to public at the start of every class body so
        // reopening doesn't carry over a stale state from a prior body.
        class.set_current_visibility("public");
        self.call_stack_push(frame);
        let body_result = self.apply_class_body_statements(class, body, position);
        self.call_stack_pop();
        body_result
    }

    /// Run a `class`, `module`, or `class << x` body. Such a body is a scope
    /// of its own: a local of the method or block holding it is not in scope
    /// inside, so a bare name there reads as a method the way Ruby reads it.
    /// A `class_eval` block is not one of these and keeps its own scope.
    pub(crate) fn apply_written_class_body(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let held_self = self.environment().get("self");
        self.environment_mut().push_isolated_scope();
        if let Some(receiver) = held_self {
            self.environment_mut().define_named("self", receiver);
        }
        // Ruby reserves every local the body assigns before any of it runs,
        // so one assigned in a branch not taken answers nil.
        for name in crate::ast::collect_assigned_locals(body) {
            self.environment_mut().hoist(name);
        }
        let answered = self.apply_class_body(class, body, position);
        self.environment_mut().pop_scope();
        answered
    }

    pub(crate) fn apply_class_body_statements(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A class variable written in this body belongs to this class, not to
        // whatever block the body happens to be running inside.
        let held_home = std::mem::take(&mut self.class_var_home);
        let answer = self.class_body_statements(class, body, position);
        self.class_var_home = held_home;
        // A `class << obj` body opened inside a method returns from it, and
        // so does a block run as the body.
        if class.is_singleton_class() && self.running_a_method()
            || matches!(self.body_block_homes.last(), Some(Some(_)))
        {
            return answer;
        }
        refuse_return_from_a_body(answer, position)
    }

    /// Whether a method body is running, rather than the top level of the
    /// program.
    fn running_a_method(&self) -> bool {
        self.current_method_frame
            .is_some_and(|frame| frame != crate::vm::core::TOP_LEVEL_FRAME)
    }

    pub(crate) fn class_body_statements(
        &mut self,
        class: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.run_compiled_body(class, body, position, false)
    }

    /// Run one statement of a class body, the way a class body runs it,
    /// with `last_value` the value the body answers so far.
    pub(crate) fn class_body_statement(
        &mut self,
        class: &Rc<Class>,
        statement: &Statement,
        position: Position,
        last_value: &mut Object,
    ) -> Result<(), MetorexError> {
        // Bare `private` / `public` / `protected` statements (no args)
        // toggle the default visibility for subsequent method defs.
        if let Statement::Expression {
            expression: Expression::Identifier { name, .. },
            ..
        } = statement
            && matches!(
                name.as_str(),
                "private" | "public" | "protected" | MODULE_FUNCTION_VISIBILITY
            )
        {
            class.set_current_visibility(name);
            return Ok(());
        }
        // A bare `freeze` in a class body freezes the class itself.
        if let Statement::Expression {
            expression: Expression::Identifier { name, .. },
            ..
        } = statement
            && name == "freeze"
        {
            class.freeze();
            return Ok(());
        }
        // `include`, `extend` and the `attr_*` statements are calls of
        // the methods of those names, so one the program wrote for this
        // class, as a module it extended writes `attr_accessor`, runs.
        if let Some(answered) = self.class_statement_as_call(class, statement)? {
            *last_value = answered;
            return Ok(());
        }
        match statement {
            Statement::FunctionDef {
                name: method_name,
                parameters,
                body: method_body,
                singleton_class: None,
                position: def_position,
                end_position: def_end,
            } => {
                let mut m = build_method_from_params(
                    method_name.clone(),
                    parameters,
                    method_body.clone(),
                    self.snapshot_active_refinements(),
                    self.def_nesting(),
                );
                m.owner = Some(class.name().to_string());
                m.owner_class = Some(Rc::clone(class));
                m.definee = Some(Rc::clone(class));
                m.source_location = Some(self.source_location_for(*def_position));
                m.end_line = Some(def_end.line);
                self.warn_redefined_optimized_method(class.name(), method_name, *def_position)?;
                self.refuse_frozen_definee(class, *def_position)?;
                class.define_method(method_name, Rc::new(m));
                apply_current_visibility(class, method_name);
                let hook = Self::method_added_hook_for(class);
                self.invoke_class_hook(class, hook, method_name, *def_position)?;
                if module_function_is_active(class) {
                    self.copy_to_module_function(class, method_name, *def_position)?;
                }
                *last_value = Object::symbol(method_name.clone());
            }
            // `def self.name` inside a `Class.new do ... end` block: the parser
            // emits a FunctionDef (not MethodDef, because `in_class_body` is
            // false inside a do-block). Route it through the same path as
            // `def self.name` in a regular class body — i.e. store as a
            // class-level method under the `__class__` prefix.
            Statement::FunctionDef {
                name: method_name,
                parameters,
                body: method_body,
                singleton_class: Some(receiver),
                position: def_position,
                end_position: def_end,
            } if receiver == "self" => {
                let mut m = build_method_from_params(
                    method_name.clone(),
                    parameters,
                    method_body.clone(),
                    self.snapshot_active_refinements(),
                    self.def_nesting(),
                );
                m.source_location = Some(self.source_location_for(*def_position));
                m.end_line = Some(def_end.line);
                class.define_method(format!("__class__{}", method_name), Rc::new(m));
                *last_value = Object::symbol(method_name.clone());
            }
            // `def Named.method` inside a body names the object it says
            // rather than the class the body opens, so the ordinary route
            // resolves the receiver.
            Statement::FunctionDef {
                singleton_class: Some(_),
                ..
            } => {
                if let ControlFlow::Value(value) = self.execute_statement(statement)? {
                    *last_value = value;
                }
            }
            Statement::MethodDef {
                name: method_name,
                parameters,
                body: method_body,
                is_class_method,
                position: def_position,
                end_position: def_end,
            } => {
                let mut m = build_method_from_params(
                    method_name.clone(),
                    parameters,
                    method_body.clone(),
                    self.snapshot_active_refinements(),
                    self.def_nesting(),
                );
                m.source_location = Some(self.source_location_for(*def_position));
                m.end_line = Some(def_end.line);
                m.owner = Some(class.name().to_string());
                m.owner_class = Some(Rc::clone(class));
                m.definee = Some(Rc::clone(class));
                let method = Rc::new(m);
                if *is_class_method {
                    // def self.method_name — store as class method with __class__ prefix
                    class.define_method(format!("__class__{}", method_name), method);
                    self.invoke_class_hook(
                        class,
                        "singleton_method_added",
                        method_name,
                        *def_position,
                    )?;
                } else {
                    self.warn_redefined_optimized_method(class.name(), method_name, *def_position)?;
                    self.refuse_frozen_definee(class, *def_position)?;
                    class.define_method(method_name, method);
                    apply_current_visibility(class, method_name);
                    // A method defined in a `class << obj` body is a
                    // singleton method of the attached object, so it
                    // fires the singleton hook rather than the plain one.
                    let hook = Self::method_added_hook_for(class);
                    self.invoke_class_hook(class, hook, method_name, *def_position)?;
                    if module_function_is_active(class) {
                        self.copy_to_module_function(class, method_name, *def_position)?;
                    }
                }
                if self.coverage.is_some() {
                    self.coverage_note_method(
                        Object::Class(Rc::clone(class)),
                        method_name,
                        *def_position,
                        *def_end,
                    );
                }
                *last_value = Object::symbol(method_name.clone());
            }
            Statement::Assignment {
                target: Expression::InstanceVariable { name: var_name, .. },
                value,
                ..
            } => {
                let initial_value = self.evaluate_expression(value)?;
                class_body_instance_variable(class, var_name, initial_value);
            }
            Statement::Expression {
                expression: Expression::InstanceVariable { name: var_name, .. },
                ..
            } => {
                // Instance variable declaration without assignment
                class.declare_instance_var(var_name);
            }
            Statement::AttrReader { attributes, .. } => {
                let names = self.resolve_attribute_names(attributes, position)?;
                self.define_attribute_methods(class, "attr_reader", &names, position)?;
            }
            Statement::AttrWriter { attributes, .. } => {
                let names = self.resolve_attribute_names(attributes, position)?;
                self.define_attribute_methods(class, "attr_writer", &names, position)?;
            }
            Statement::AttrAccessor { attributes, .. } => {
                let names = self.resolve_attribute_names(attributes, position)?;
                self.define_attribute_methods(class, "attr_accessor", &names, position)?;
            }
            Statement::Assignment {
                target:
                    Expression::Identifier {
                        name: const_name, ..
                    },
                value,
                ..
            } if const_name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase()) =>
            {
                // Constant assignment in class body (e.g., PI = 3.14159).
                // Lowercase identifiers fall through to the `_` arm and are
                // treated as normal local-variable assignments.
                // A block run as the body keeps its constants where the
                // block was written, which the plain assignment handles.
                if !self
                    .constant_home()
                    .is_some_and(|home| Rc::ptr_eq(&home, class))
                {
                    *last_value = match self.execute_statement(statement)? {
                        ControlFlow::Value(value) => value,
                        _ => Object::Nil,
                    };
                    return Ok(());
                }
                let assign_pos = statement.position();
                let const_value = match self.evaluate_constant_assignment(statement, value)? {
                    Ok(written) => written,
                    Err(kept) => {
                        *last_value = kept;
                        return Ok(());
                    }
                };
                self.class_body_constant(class, const_name, const_value.clone(), assign_pos)?;
                *last_value = const_value;
            }
            Statement::Include {
                module_name,
                position,
            } => {
                // include ModuleName: dispatch through `append_features`
                // so user overrides on the module's singleton class fire,
                // and so cyclic/frozen checks run.
                let resolved = self.resolve_constant_with_autoload(module_name)?;
                match resolved.view() {
                    Some(View::Module(module)) => {
                        self.apply_module_include(class, module, *position)?;
                    }
                    Some(_) => {
                        return Err(MetorexError::runtime_error(
                            format!("'{}' is not a module", module_name),
                            position_to_location(*position),
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            format!("Undefined module '{}'", module_name),
                            position_to_location(*position),
                        ));
                    }
                }
            }
            Statement::Extend {
                module_name,
                position,
            } => {
                // extend ModuleName: add module methods as class-level
                // methods, and record the mixin on the singleton class so
                // `kind_of?` reports it the way an `obj.extend` does.
                // `extend self` names the module being defined, which is
                // how a module makes its instance methods callable on
                // itself.
                let resolved = if module_name == "self" {
                    Some(if class.is_module() {
                        Object::Module(Rc::clone(class))
                    } else {
                        Object::Class(Rc::clone(class))
                    })
                } else {
                    self.resolve_constant_with_autoload(module_name)?
                };
                match resolved.view() {
                    Some(View::Module(module)) => {
                        for method_name in module.method_names() {
                            if let Some(method) = module.find_method(&method_name) {
                                class.set_class_var(
                                    format!("__ext__{}", method_name),
                                    Object::Method(method),
                                );
                                class.set_class_var(
                                    format!("__ext_from__{}", method_name),
                                    Object::Module(Rc::clone(module)),
                                );
                            }
                        }
                        let target = Object::Class(Rc::clone(class));
                        self.apply_module_extend(&target, module, *position)?;
                    }
                    Some(_) => {
                        return Err(MetorexError::runtime_error(
                            format!("'{}' is not a module", module_name),
                            position_to_location(*position),
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            format!("Undefined module '{}'", module_name),
                            position_to_location(*position),
                        ));
                    }
                }
            }
            // `alias $new $old` names a global, wherever it is written.
            Statement::Alias {
                new_name,
                old_name,
                position: alias_pos,
            } if !new_name.starts_with('$') => {
                self.install_alias(class, new_name, old_name, *alias_pos)?;
                let hook = Self::method_added_hook_for(class);
                self.invoke_class_hook(class, hook, new_name, *alias_pos)?;
            }
            // `class << <target>` inside a class body — open the target's
            // singleton class and apply the inner body there.
            Statement::Expression {
                expression:
                    Expression::SingletonClass {
                        target,
                        body: inner_body,
                        position: sc_pos,
                        ..
                    },
                ..
            } => {
                *last_value =
                    self.evaluate_singleton_class_expression(target, None, inner_body, *sc_pos)?;
            }
            // class << self block — treat inner statements as class-level
            Statement::Block { statements, .. } => {
                for inner_stmt in statements {
                    if let Statement::AttrAccessor { attributes, .. } = inner_stmt {
                        let names = self.resolve_attribute_names(attributes, position)?;
                        define_block_accessors(class, &names);
                    }
                }
            }
            _ => {
                let mut handled = false;
                // Handle alias_method in class body
                if let Statement::Expression {
                    expression:
                        Expression::Call {
                            callee,
                            arguments: call_args,
                            ..
                        },
                    ..
                } = statement
                    && let Expression::Identifier {
                        name: callee_name, ..
                    } = callee.as_ref()
                    && callee_name == "alias_method"
                    && call_args.len() == 2
                {
                    handled = true;
                    let new_value = self.evaluate_expression(&call_args[0])?;
                    let old_value = self.evaluate_expression(&call_args[1])?;
                    self.class_body_alias_method(class, &new_value, &old_value, position)?;
                }
                // Handle define_method(:name) { |args| body } calls in class body
                else if let Statement::Expression {
                    expression:
                        Expression::Call {
                            callee,
                            arguments: call_args,
                            trailing_block: Some(block_expr),
                            ..
                        },
                    ..
                } = statement
                    && let Expression::Identifier {
                        name: callee_name, ..
                    } = callee.as_ref()
                    && callee_name == "define_method"
                {
                    handled = true;
                    let mut define_args: Vec<Object> = Vec::new();
                    for arg_expr in call_args {
                        define_args.push(self.evaluate_expression(arg_expr)?);
                    }
                    let block = self.attach_trailing_block(block_expr)?;
                    *last_value = self.class_body_define_method(
                        class,
                        &define_args,
                        block,
                        statement.position(),
                    )?;
                }
                // `refine(target) { body }` inside a module body — dispatch
                // to the module's refine method, preserving the block.
                else if let Statement::Expression {
                    expression:
                        Expression::Call {
                            callee,
                            arguments: call_args,
                            trailing_block,
                            ..
                        },
                    ..
                } = statement
                    && let Expression::Identifier {
                        name: callee_name, ..
                    } = callee.as_ref()
                    && callee_name == "refine"
                {
                    handled = true;
                    self.evaluate_method_call(
                        &Expression::SelfExpr { position },
                        "refine",
                        call_args,
                        trailing_block.as_deref(),
                        position,
                    )?;
                }
                // Fall through to general statement execution for arbitrary
                // expressions and lowercase-identifier assignments (locals,
                // method calls, @ivars on the class, etc.). Ruby evaluates
                // any expression inside a class body with `self` bound to
                // the class; we've already pushed that self in
                // `apply_block_as_class_body_with_self` / `execute_class_def`.
                if !handled {
                    match statement {
                        // A bare expression statement (e.g. `1 + 1`, `self`,
                        // a constant reference) is the common tail of a
                        // `class_eval`/`module_eval` string or block — capture
                        // its value so it becomes the eval result.
                        Statement::Expression {
                            expression,
                            position: expr_pos,
                        } => {
                            let v = self.evaluate_expression(expression)?;
                            if matches!(expression, Expression::Identifier { .. })
                                && matches!(v.view(), View::Method(_))
                            {
                                *last_value = self.invoke_callable(v, vec![], *expr_pos)?;
                            } else {
                                *last_value = v;
                            }
                        }
                        _ => match self.execute_statement(statement)? {
                            ControlFlow::Value(v) => {
                                *last_value = v;
                            }
                            // A `class << obj` body opened inside a method
                            // returns from that method.
                            ControlFlow::Return { value, position }
                                if class.is_singleton_class() && self.running_a_method() =>
                            {
                                return Err(MetorexError::NonLocalReturn {
                                    value,
                                    location: position_to_location(position),
                                    home_frame: self.current_method_frame,
                                });
                            }
                            // A block run as the body returns from the
                            // method it was written in.
                            ControlFlow::Return { value, position }
                                if let Some(Some(home)) = self.body_block_homes.last().copied() =>
                            {
                                if let Some(frame) = home
                                    && !self.live_frames.contains(&frame)
                                {
                                    return Err(crate::vm::block_execution::orphaned_return_error(
                                        value, position,
                                    ));
                                }
                                return Err(MetorexError::NonLocalReturn {
                                    value,
                                    location: position_to_location(position),
                                    home_frame: home,
                                });
                            }
                            // Ruby's parser refuses a `return` written in
                            // a class or module body outright.
                            ControlFlow::Return { position, .. } => {
                                return Err(return_in_a_body_error(position));
                            }
                            // An exception the body raised leaves it.
                            _ => {}
                        },
                    }
                }
            }
        }
        Ok(())
    }
}

impl VirtualMachine {
    /// Run an `include`, `extend` or `attr_*` statement as the call of the
    /// method the program wrote for `class` under that name, or answer None
    /// when the class answers it with the core library's own.
    fn class_statement_as_call(
        &mut self,
        class: &Rc<Class>,
        statement: &Statement,
    ) -> Result<Option<Object>, MetorexError> {
        let (name, position) = match statement {
            Statement::AttrReader { position, .. } => ("attr_reader", *position),
            Statement::AttrWriter { position, .. } => ("attr_writer", *position),
            Statement::AttrAccessor { position, .. } => ("attr_accessor", *position),
            Statement::Include { position, .. } => ("include", *position),
            Statement::Extend { position, .. } => ("extend", *position),
            _ => return Ok(None),
        };
        if !self.class_answers_with_program_method(class, name) {
            return Ok(None);
        }
        let arguments = match statement {
            Statement::AttrReader { attributes, .. }
            | Statement::AttrWriter { attributes, .. }
            | Statement::AttrAccessor { attributes, .. } => {
                self.attribute_values(attributes, position)?
            }
            Statement::Include { module_name, .. } | Statement::Extend { module_name, .. } => {
                let found = self
                    .resolve_constant_with_autoload(module_name)?
                    .or_else(|| self.resolve_constant_in_scope(module_name));
                match found {
                    Some(module) => vec![module],
                    None => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
        let receiver = if class.is_module() {
            Object::Module(Rc::clone(class))
        } else {
            Object::Class(Rc::clone(class))
        };
        self.send_to_object(receiver, name, arguments, position)
            .map(Some)
    }
}

impl VirtualMachine {
    /// Whether `class` answers `name` with a method the program wrote: one on
    /// its singleton class, a superclass's, or a module it extended, however
    /// the extending was done, or one written `def self.name`.
    fn class_answers_with_program_method(&self, class: &Rc<Class>, name: &str) -> bool {
        let written = |method: &Rc<crate::object::Method>| {
            crate::vm::method_execution::written_by_the_program(method)
        };
        let receiver = if class.is_module() {
            Object::Module(Rc::clone(class))
        } else {
            Object::Class(Rc::clone(class))
        };
        class
            .singleton_class_slot()
            .as_ref()
            .and_then(|singleton| singleton.find_method(name))
            .is_some_and(|method| written(&method))
            || crate::vm::method_lookup::module_extended_method(class, name)
                .is_some_and(|method| written(&method))
            || crate::vm::method_lookup::module_own_method(class, name)
                .is_some_and(|method| written(&method))
            || crate::vm::method_lookup::class_level_method(class, name)
                .is_some_and(|(_, method)| written(&method))
            || self
                .lookup_method(&receiver, name)
                .is_some_and(|(_, method)| written(&method))
    }
}

impl VirtualMachine {
    /// Run the hook a class body's new methods are announced through:
    /// `singleton_method_added` on the object a singleton class belongs to,
    /// and `method_added` on any other class.
    fn announce_added_methods(
        &mut self,
        class: &Rc<Class>,
        names: Vec<String>,
        position: crate::lexer::Position,
    ) -> Result<(), MetorexError> {
        let hook = if class.get_class_var("__singleton__").is_some() {
            "singleton_method_added"
        } else {
            "method_added"
        };
        for name in &names {
            self.invoke_class_hook(class, hook, name, position)?;
        }
        Ok(())
    }
}

/// `@name = value` in a class body, which declares the instance variable
/// and binds the class's own value of it.
pub(crate) fn class_body_instance_variable(class: &Rc<Class>, name: &str, value: Object) {
    class.declare_instance_var(name);
    class.set_class_var(format!("@{}", name), value);
}

impl VirtualMachine {
    /// Bind a constant a class body assigns on the class.
    pub(crate) fn class_body_constant(
        &mut self,
        class: &Rc<Class>,
        const_name: &str,
        const_value: Object,
        assign_pos: Position,
    ) -> Result<(), MetorexError> {
        if class.get_class_var(const_name).is_some() {
            let owner = class.ruby_name();
            let owner = if owner.is_empty() {
                class.inspect_name()
            } else {
                owner
            };
            let msg = format!(
                "warning: already initialized constant {}::{}",
                owner, const_name
            );
            self.emit_warning_to_stderr(&msg, assign_pos);
        }
        // A value written over a pending autoload takes its place.
        class.remove_autoload(const_name);
        // An anonymous class or module takes the name it is bound to.
        crate::vm::statement::name_constant_value(class, const_name, &const_value);
        class.set_class_var(const_name, const_value);
        let assign_file = self
            .reported_current_file()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        class.set_const_location(const_name, assign_file, assign_pos.line as i64);
        self.trigger_const_added_hook(Object::Class(Rc::clone(class)), const_name, assign_pos)
    }

    /// Define the readers, writers, or both, `method` names for `names` on
    /// `class`, as `attr_reader`, `attr_writer`, or `attr_accessor` written
    /// in its body does.
    pub(crate) fn define_attribute_methods(
        &mut self,
        class: &Rc<Class>,
        method: &str,
        names: &[String],
        position: Position,
    ) -> Result<(), MetorexError> {
        let readers = method != "attr_writer";
        let writers = method != "attr_reader";
        for attr_name in names {
            if readers {
                let getter_body = vec![Statement::Return {
                    value: Some(Expression::InstanceVariable {
                        name: attr_name.clone(),
                        position,
                    }),
                    position,
                }];
                let method = Rc::new(Method::new(attr_name.clone(), vec![], getter_body));
                class.define_method(attr_name, method);
                apply_current_visibility(class, attr_name);
            }
            if writers {
                let setter_body = vec![Statement::Assignment {
                    target: Expression::InstanceVariable {
                        name: attr_name.clone(),
                        position,
                    },
                    value: Expression::Identifier {
                        name: crate::object::UNNAMED_PARAMETER.to_string(),
                        position,
                    },
                    position,
                }];
                let setter_name = format!("{}=", attr_name);
                let method = Rc::new(Method::new(
                    setter_name.clone(),
                    vec![crate::object::UNNAMED_PARAMETER.to_string()],
                    setter_body,
                ));
                class.define_method(&setter_name, method);
                apply_current_visibility(class, &setter_name);
            }
            class.declare_instance_var(attr_name);
        }
        let added = names
            .iter()
            .flat_map(|name| {
                let mut added = Vec::new();
                if readers {
                    added.push(name.clone());
                }
                if writers {
                    added.push(format!("{name}="));
                }
                added
            })
            .collect();
        self.announce_added_methods(class, added, position)
    }

    /// `attr_reader`, `attr_writer`, or `attr_accessor` in a class body with
    /// the names it was given evaluated: the call of the method the program
    /// wrote for the class under that name, when it wrote one, answering
    /// what the call answers, and otherwise the methods defined.
    pub(crate) fn class_body_attributes(
        &mut self,
        class: &Rc<Class>,
        method: &str,
        values: Vec<Object>,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if self.class_answers_with_program_method(class, method) {
            let receiver = if class.is_module() {
                Object::Module(Rc::clone(class))
            } else {
                Object::Class(Rc::clone(class))
            };
            return self
                .send_to_object(receiver, method, values, position)
                .map(Some);
        }
        let mut names = Vec::with_capacity(values.len());
        for value in &values {
            names.push(self.coerce_method_name(value, "attr_accessor", position)?);
        }
        self.define_attribute_methods(class, method, &names, position)?;
        Ok(None)
    }
}

impl VirtualMachine {
    /// `alias_method new, old` written in a class body, with the two names
    /// evaluated.
    pub(crate) fn class_body_alias_method(
        &mut self,
        class: &Rc<Class>,
        new_value: &Object,
        old_value: &Object,
        position: Position,
    ) -> Result<(), MetorexError> {
        let new_name = match new_value.view() {
            View::String(s) => s.as_str().to_string(),
            View::Symbol(s) => s.as_str().to_string(),
            _ => String::new(),
        };
        let old_name = match old_value.view() {
            View::String(s) => s.as_str().to_string(),
            View::Symbol(s) => s.as_str().to_string(),
            _ => String::new(),
        };

        if !new_name.is_empty() && !old_name.is_empty() {
            // A name the class holds no entry for, such as one
            // the interpreter answers natively, is aliased
            // through the full method so the stub that reaches
            // it is made.
            if class.alias_method(&new_name, &old_name) {
                let hook = Self::method_added_hook_for(class);
                self.invoke_class_hook(class, hook, &new_name, position)?;
            } else {
                let given = vec![
                    Object::symbol(new_name.clone()),
                    Object::symbol(old_name.clone()),
                ];
                self.call_class_methods(class, "alias_method", &given, position)?;
            }
        }
        Ok(())
    }

    /// `define_method(name) { ... }` written in a class body, with its
    /// arguments evaluated and its block made. The method stands where the
    /// call was written, not where the body it sits in opened.
    pub(crate) fn class_body_define_method(
        &mut self,
        class: &Rc<Class>,
        arguments: &[Object],
        block: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.pending_block = Some(block);
        self.pending_block_from_ampersand = false;
        self.module_define_method(class, arguments, position)
    }
}

impl VirtualMachine {
    /// A `return` that ended a class body: one in a `class << obj` body
    /// opened inside a method returns from that method, one in a block run
    /// as the body returns from the method the block was written in, and
    /// Ruby's parser refuses any other.
    pub(crate) fn class_body_return(
        &mut self,
        class: &Rc<Class>,
        value: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if class.is_singleton_class() && self.running_a_method() {
            return Err(MetorexError::NonLocalReturn {
                value,
                location: position_to_location(position),
                home_frame: self.current_method_frame,
            });
        }
        if let Some(Some(home)) = self.body_block_homes.last().copied() {
            if let Some(frame) = home
                && !self.live_frames.contains(&frame)
            {
                return Err(crate::vm::block_execution::orphaned_return_error(
                    value, position,
                ));
            }
            return Err(MetorexError::NonLocalReturn {
                value,
                location: position_to_location(position),
                home_frame: home,
            });
        }
        Err(return_in_a_body_error(position))
    }

    /// A `return` that ended a module body, which Ruby's parser refuses.
    pub(crate) fn module_body_return(&self, position: Position) -> MetorexError {
        return_in_a_body_error(position)
    }
}

/// The readers and writers an `attr_accessor` inside a `Block` of a class
/// body defines, on instance variables named with the `@` written in.
pub(crate) fn define_block_accessors(class: &Rc<Class>, names: &[String]) {
    for attr_name in names {
        // Getter
        let getter_body = vec![Statement::Expression {
            expression: Expression::InstanceVariable {
                name: format!("@{}", attr_name),
                position: crate::lexer::Position::default(),
            },
            position: crate::lexer::Position::default(),
        }];
        class.define_method(
            attr_name,
            Rc::new(Method::new(attr_name.to_string(), vec![], getter_body)),
        );
        // Setter
        let setter_body = vec![Statement::Assignment {
            target: Expression::InstanceVariable {
                name: format!("@{}", attr_name),
                position: crate::lexer::Position::default(),
            },
            value: Expression::Identifier {
                name: crate::object::UNNAMED_PARAMETER.to_string(),
                position: crate::lexer::Position::default(),
            },
            position: crate::lexer::Position::default(),
        }];
        class.define_method(
            format!("{}=", attr_name),
            Rc::new(Method::new(
                format!("{}=", attr_name),
                vec![crate::object::UNNAMED_PARAMETER.to_string()],
                setter_body,
            )),
        );
    }
}
