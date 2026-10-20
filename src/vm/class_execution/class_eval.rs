// Running a block or a string against a class as its body.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// Evaluate a block with `self` bound to the given class/module, executing
    /// the block's body as if it were a class/module definition body.
    pub(crate) fn apply_block_as_class_body(
        &mut self,
        class: &Rc<Class>,
        block: &crate::object::BlockStatement,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.apply_block_as_class_body_with_self(
            class,
            block,
            position,
            Object::Class(Rc::clone(class)),
        )
    }

    /// The class or module a constant written here lands in, None being the
    /// top level.
    /// Put back where a constant written in `block` lands, when the block
    /// was written in a block run as a class body, at the depth of the scopes
    /// it was written in. Answers whether it did, so the caller takes it off
    /// again when the block is done.
    pub(crate) fn enter_captured_constant_home(
        &mut self,
        block: &crate::object::BlockStatement,
    ) -> bool {
        let Some(home) = &block.captured_constant_home else {
            return false;
        };
        self.constant_homes
            .push((self.def_scope_stack.len(), home.clone()));
        true
    }

    pub(crate) fn constant_home(&self) -> Option<Rc<Class>> {
        match self.constant_homes.last() {
            Some((depth, home)) if *depth == self.def_scope_stack.len() => home.clone(),
            _ => self.def_scope_stack.last().cloned(),
        }
    }

    /// Run `block` as the body of `class` for `refine`, where Ruby makes the
    /// refinement the scope its constants land in as well.
    pub(crate) fn apply_refine_block(
        &mut self,
        class: &Rc<Class>,
        block: &crate::object::BlockStatement,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.run_block_as_class_body(
            class,
            block,
            position,
            Object::Class(Rc::clone(class)),
            false,
        )
    }

    /// Same as `apply_block_as_class_body` but explicit about the `self` kind
    /// (use `Object::Module(...)` when the receiver should behave as a module).
    pub(crate) fn apply_block_as_class_body_with_self(
        &mut self,
        class: &Rc<Class>,
        block: &crate::object::BlockStatement,
        position: Position,
        self_obj: Object,
    ) -> Result<Object, MetorexError> {
        self.run_block_as_class_body(class, block, position, self_obj, true)
    }

    fn run_block_as_class_body(
        &mut self,
        class: &Rc<Class>,
        block: &crate::object::BlockStatement,
        position: Position,
        self_obj: Object,
        constants_stay_lexical: bool,
    ) -> Result<Object, MetorexError> {
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define_named("self", self_obj.clone());
        // Share the outer RefCells so an assignment like `x = self` inside the
        // block mutates the caller's local, matching Ruby's closure semantics
        // for blocks (including `Class.new do ... end`).
        for (name, cell) in block.captured_vars.iter() {
            // A global such as the `method` function does not hide a local
            // the block captured under the same name.
            let global = self.environment().global_scope().borrow().own_var_ref(name);
            let held_only_globally = global.as_ref().is_some_and(|global| {
                !Rc::ptr_eq(cell, global)
                    && self
                        .environment()
                        .get_ref(name)
                        .is_some_and(|found| Rc::ptr_eq(&found, global))
            });
            if self.environment().get(name).is_none() || held_only_globally {
                self.environment_mut()
                    .define_shared(name.clone(), cell.clone());
            }
        }
        // Bind the block's first parameter to the class/module, matching Ruby:
        // `Class.new { |c| ... }` and `mod.class_eval { |m| ... }` both yield
        // the class/module as the sole block argument.
        if let Some(first) = block
            .parameters
            .iter()
            .find(|p| !p.starts_with('&') && !p.starts_with('*'))
        {
            let pname = first.trim_start_matches('|').to_string();
            self.environment_mut().define(pname, self_obj);
        }
        self.def_scope_stack.push(Rc::clone(class));
        self.class_var_cref_stack.push(Some(Rc::clone(class)));
        if constants_stay_lexical {
            self.constant_homes.push((
                self.def_scope_stack.len(),
                block.captured_def_scope.last().cloned(),
            ));
        }
        // A class/module body is not a method context, so `using` is permitted
        // even when this block runs deep inside method calls (e.g. mspec's
        // runner invoking `Class.new do using ...; end`). The refinements it
        // activates are lexical to the body, so they get their own scope.
        let saved_nesting = self.user_def_nesting;
        self.user_def_nesting = 0;
        self.push_refinement_scope();
        // The method running this block did not write it.
        self.borrowed_frames.push(self.current_method_frame);
        let frame = self.block_frame_at(block, position);
        let result = self.in_block_source_file(block, |vm| {
            vm.apply_class_body_in_frame(
                class,
                &block.body,
                position,
                frame,
                Some(block.home_frame),
            )
        });
        self.borrowed_frames.pop();
        self.pop_refinement_scope();
        self.user_def_nesting = saved_nesting;
        if constants_stay_lexical {
            self.constant_homes.pop();
        }
        self.def_scope_stack.pop();
        self.class_var_cref_stack.pop();
        if let Some(prev) = prev_self {
            self.environment_mut().define_named("self", prev);
        } else {
            self.environment_mut().undefine("self");
        }
        result
    }

    /// Runs `run` with the file and source encoding the block was written in
    /// as the current ones, so a call or a raise in the body is located in
    /// the block's own file rather than in the file that ran it.
    fn in_block_source_file<T>(
        &mut self,
        block: &crate::object::BlockStatement,
        run: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let body_source_file = block
            .source_file
            .clone()
            .or_else(|| self.current_source_file.clone());
        let saved_source_file = std::mem::replace(&mut self.current_source_file, body_source_file);
        let saved_source_encoding = std::mem::replace(
            &mut self.current_source_encoding,
            self.current_source_file
                .as_ref()
                .and_then(|named| self.file_encodings.get(named).cloned()),
        );
        let result = run(self);
        self.current_source_file = saved_source_file;
        self.current_source_encoding = saved_source_encoding;
        result
    }

    /// Implementation of `Module#class_exec` / `Module#module_exec`: evaluate the
    /// block with class-body semantics (so `def` lands on the receiver) and
    /// `self` bound to the receiver, but bind the block's parameters to the
    /// caller-supplied `args` rather than to the module. Returns the block's
    /// last value.
    pub(crate) fn class_exec_block(
        &mut self,
        class: &Rc<Class>,
        self_obj: Object,
        block: &crate::object::BlockStatement,
        args: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // The block reads the variables it captured where it was written,
        // in a scope of its own, so neither a local of the caller's nor a
        // parameter it binds here clashes with one of the caller's.
        self.environment_mut().push_isolated_scope();
        self.environment_mut()
            .attach_captured(Rc::clone(&block.captured_vars));
        self.environment_mut()
            .attach_reserved(Rc::clone(&block.reserved_names));
        self.environment_mut().define_named("self", self_obj);
        if let Err(error) = crate::vm::block_execution::bind_block_params(
            self,
            &block.parameters,
            &block.parameter_defaults,
            args,
            position,
        ) {
            self.environment_mut().pop_scope();
            return Err(error);
        }
        // The body reads constants through the scopes the block was written
        // in, not those of the method that called `class_exec`.
        let saved_def_scope =
            std::mem::replace(&mut self.def_scope_stack, block.captured_def_scope.clone());
        self.enter_block_nesting(block.captured_nesting.clone());
        self.def_scope_stack.push(Rc::clone(class));
        self.class_var_cref_stack.push(Some(Rc::clone(class)));
        self.constant_homes.push((
            self.def_scope_stack.len(),
            block.captured_def_scope.last().cloned(),
        ));
        let saved_nesting = self.user_def_nesting;
        self.user_def_nesting = 0;
        // The method running this block did not write it.
        self.borrowed_frames.push(self.current_method_frame);
        let frame = self.block_frame_at(block, position);
        let result = self.in_block_source_file(block, |vm| {
            vm.apply_class_body_in_frame(
                class,
                &block.body,
                position,
                frame,
                Some(block.home_frame),
            )
        });
        self.borrowed_frames.pop();
        self.user_def_nesting = saved_nesting;
        self.constant_homes.pop();
        self.def_scope_stack = saved_def_scope;
        self.leave_block_nesting();
        self.class_var_cref_stack.pop();
        self.environment_mut().pop_scope();
        result
    }

    /// Shared implementation of `Module#class_eval` / `Module#module_eval` (and
    /// the `Class` variants). Handles both the block form and the string form
    /// (`class_eval(code, filename = "...", lineno = 1)`), returning the value
    /// of the last evaluated statement. `self_obj` is the receiver as it should
    /// appear inside the body (`Object::Class` or `Object::Module`).
    pub(crate) fn class_eval_with_args(
        &mut self,
        class_rc: &Rc<Class>,
        self_obj: Object,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A `private` or `module_function` toggle inside the eval belongs to
        // the eval, so the surrounding body's state is restored on the way
        // out. (The block form starts fresh; `apply_class_body` resets it.)
        let outer_visibility = class_rc.current_visibility();
        let result = self.class_eval_body(class_rc, self_obj, arguments, position);
        class_rc.set_current_visibility(outer_visibility);
        result
    }

    pub(crate) fn class_eval_body(
        &mut self,
        class_rc: &Rc<Class>,
        self_obj: Object,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Block form wins when a block is present.
        if let Some(View::Block(block)) = self.pending_block.take().view() {
            if !arguments.is_empty() {
                return Err(self.arg_count_error(
                    format!(
                        "wrong number of arguments (given {}, expected 0)",
                        arguments.len()
                    ),
                    position,
                ));
            }
            return self.apply_block_as_class_body_with_self(class_rc, block, position, self_obj);
        }

        // String form expects 1..3 positional arguments.
        if arguments.is_empty() {
            return Err(self.arg_count_error(
                "wrong number of arguments (given 0, expected 1..3)".to_string(),
                position,
            ));
        }
        if arguments.len() > 3 {
            return Err(self.arg_count_error(
                format!(
                    "wrong number of arguments (given {}, expected 1..3)",
                    arguments.len()
                ),
                position,
            ));
        }

        let code = self.coerce_eval_string(&arguments[0], position)?;
        let filename = match arguments.get(1) {
            Some(obj) => self.coerce_eval_string(obj, position)?,
            None => {
                let caller = self
                    .current_source_file
                    .clone()
                    .or_else(|| {
                        self.reported_current_file()
                            .map(|path| path.display().to_string())
                    })
                    .unwrap_or_else(|| "(eval)".to_string());
                format!("(eval at {}:{})", caller, position.line)
            }
        };
        let lineno = match arguments.get(2).view() {
            Some(View::Int(n)) => n.max(1) as usize,
            _ => 1,
        };

        let tokens = crate::lexer::Lexer::with_start_line(&code, lineno).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .inside_eval()
            .with_outer_locals(self.visible_local_names())
            .parse()
            .map_err(|errors| {
                MetorexError::runtime_error(
                    format!(
                        "{}: parse error: {}",
                        filename,
                        errors
                            .iter()
                            .map(|e| e.to_string())
                            .collect::<Vec<_>>()
                            .join("; ")
                    ),
                    position_to_location(position),
                )
            })?;

        // Evaluate the parsed body with `self` bound to the receiver and the
        // receiver pushed as the current definee, so `def` lands on it and
        // constants resolve in its scope. `__FILE__` reflects `filename` for
        // the duration. Refinements active at the call site stay active so
        // `class_eval` can see `using` from the eval scope.
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define_named("self", self_obj.clone());
        // Ruby resolves constants in a string eval using the caller's lexical
        // scope, which for a named receiver includes the namespaces it is
        // nested in. Push those enclosing modules (outermost first) so e.g.
        // `ModuleSpecs::ClassEvalTest.module_eval("Lookup")` finds
        // `ModuleSpecs::Lookup`.
        let mut enclosing_pushed = 0usize;
        let full_name = class_rc.name().to_string();
        if let Some(idx) = full_name.rfind("::") {
            let mut cumulative = String::new();
            for segment in full_name[..idx].split("::") {
                if cumulative.is_empty() {
                    cumulative = segment.to_string();
                } else {
                    cumulative = format!("{}::{}", cumulative, segment);
                }
                if let Some(View::Module(m) | View::Class(m)) =
                    self.resolve_constant_in_scope(&cumulative).view()
                {
                    self.def_scope_stack.push(m.clone());
                    enclosing_pushed += 1;
                }
            }
        }
        self.def_scope_stack.push(Rc::clone(class_rc));
        self.class_var_cref_stack.push(Some(Rc::clone(class_rc)));
        let prev_file = self.current_file.clone();
        self.current_file = Some(std::path::PathBuf::from(&filename));
        let prev_source_file = self.current_source_file.replace(filename.clone());
        let saved_nesting = self.user_def_nesting;
        self.user_def_nesting = 0;
        // Code handed to `class_eval` is counted into the file it names when
        // the run was started with `eval` coverage on.
        if self.coverage_counts_eval() {
            self.coverage_note_eval(&filename, &statements);
        }
        // The code is a place of its own in a backtrace, named as the code
        // that ran it and standing where the call was made, as Kernel#eval's
        // code is, rather than as a class body.
        let written_in = self
            .call_stack()
            .last()
            .cloned()
            .unwrap_or_else(|| crate::vm::CallFrame::boundary("<main>"));
        self.call_stack_push(
            written_in
                .with_location(Some(format!("{}:{}", position.line, position.column)))
                .with_source_file(
                    prev_source_file
                        .clone()
                        .or_else(|| prev_file.as_ref().map(|file| file.display().to_string())),
                ),
        );
        class_rc.set_current_visibility("public");
        let result = self.apply_class_body_statements(class_rc, &statements, position);
        if let Err(error) = &result {
            self.trace_error_leaving_frame(error);
        }
        self.call_stack_pop();
        self.user_def_nesting = saved_nesting;
        self.current_file = prev_file;
        self.current_source_file = prev_source_file;
        self.def_scope_stack.pop();
        self.class_var_cref_stack.pop();
        for _ in 0..enclosing_pushed {
            self.def_scope_stack.pop();
        }
        if let Some(prev) = prev_self {
            self.environment_mut().define_named("self", prev);
        } else {
            self.environment_mut().undefine("self");
        }
        result
    }

    /// Coerce an eval argument (code string or filename) to a `String`,
    /// invoking `#to_str` for non-strings. Mirrors MRI's error messages:
    /// a missing `#to_str` raises "no implicit conversion of X into String",
    /// and a `#to_str` returning a non-string raises "can't convert X into
    /// String".
    fn coerce_eval_string(
        &mut self,
        arg: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if let View::String(s) = arg.view() {
            return Ok(s.as_str().to_string());
        }
        let class_name = match arg.view() {
            View::Class(_) | View::Module(_) => "Module".to_string(),
            _ => self.builtins().class_of(arg).name().to_string(),
        };
        if let Some((cls, m)) = self.lookup_method(arg, "to_str")
            && !m.is_undefined
        {
            let result = self.invoke_method(cls, m, arg.clone(), vec![], position)?;
            if let View::String(s) = result.view() {
                return Ok(s.as_str().to_string());
            }
            let msg = format!("can't convert {} into String", class_name);
            let exc = Object::exception("TypeError", msg.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: msg,
            });
        }
        let msg = format!("no implicit conversion of {} into String", class_name);
        let exc = Object::exception("TypeError", msg.clone());
        Err(MetorexError::UncaughtException {
            exception: exc,
            location: position_to_location(position),
            message: msg,
        })
    }

    /// Build an `ArgumentError` with the given message.
    fn arg_count_error(&self, msg: String, position: Position) -> MetorexError {
        let exc = Object::exception("ArgumentError", msg.clone());
        MetorexError::UncaughtException {
            exception: exc,
            location: position_to_location(position),
            message: msg,
        }
    }

    /// Apply a class-body statement list to the given class (also used for
    /// `Class.new { ... }` / anonymous classes). Returns the value of the last
    /// evaluated statement, which `class_eval` / `module_eval` surface as their
    /// result (ordinary class definitions discard it).
    /// Which hook a newly defined method fires. A method added to a
    /// singleton class is a singleton method of the object it is attached
    /// to, so Ruby reports it through `singleton_method_added`.
    pub(crate) fn method_added_hook_for(class: &Rc<Class>) -> &'static str {
        if class.get_class_var("__singleton__").is_some() {
            "singleton_method_added"
        } else {
            "method_added"
        }
    }
}
