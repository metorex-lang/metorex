// Expression-evaluation dispatch: a thin `match` over `Expression` variants
// that delegates each variant to a helper in the sibling modules.

use crate::object::{OptionView, View};
use std::rc::Rc;

use crate::ast::{BinaryOp, Expression};
use crate::error::MetorexError;
use crate::object::{BlockStatement, Object};

use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// The Block a block or lambda literal makes, closing over the variables
    /// in scope where it was written. Kept out of `evaluate_expression_inner`,
    /// whose frame is on the stack once for every expression a nested call is
    /// in the middle of. A block that closes over every variable in scope
    /// takes what `closure` keeps for the scopes open here, when it keeps
    /// it, and leaves what it read there otherwise.
    #[inline(never)]
    pub(crate) fn evaluate_block_literal(
        &mut self,
        expression: &Expression,
        shared: &crate::bytecode::unit::SharedBody,
        closure: Option<&crate::vm::block_reuse::ClosureCache>,
    ) -> Result<Object, MetorexError> {
        let Expression::Lambda {
            parameters,
            parameter_defaults,
            captured_vars,
            outer_locals: _,
            is_lambda,
            position,
            ..
        } = expression
        else {
            unreachable!("only a Lambda expression is handed here")
        };
        if let Some(block) = self.reusable_block(&shared.kept) {
            return Ok(Object::Block(block));
        }
        let captures_everything = captured_vars.as_ref().is_none_or(|names| names.is_empty());
        let closure = closure.filter(|closure| captures_everything && !closure.gave_up());
        let kept = closure
            .and_then(|closure| self.kept_closure(closure))
            .or_else(|| {
                captures_everything
                    .then(|| self.kept_closure(&self.closure_here))
                    .flatten()
            });
        let mut captured = crate::scope::Captures::default();
        let shared_here = kept.is_some();
        if shared_here {
            crate::vm::counters::count_closure_shared();
        }
        if kept.is_some() {
        } else if let Some(names) = captured_vars {
            if names.is_empty() {
                // Empty vec signals automatic capture of all current scope variables.
                // This is used for true lambdas (lambda do ... end, arrow syntax).
                captured = self.environment().current_scope_var_refs();
            } else {
                // Explicit list of variables to capture
                let mut listed: Vec<(Rc<str>, _)> = Vec::new();
                for name in names.iter() {
                    if listed.iter().any(|(held, _)| **held == **name) {
                        continue;
                    }
                    if let Some(value_ref) = self.environment().get_ref(name) {
                        listed.push((crate::fast_hash::shared_name(name), value_ref));
                    }
                }
                captured = crate::scope::Captures::new(listed);
            }
        }
        // If captured_vars is None, capture all current scope variables
        // so blocks work correctly across method boundaries (which use
        // isolated scopes).
        if kept.is_none() && captured.is_empty() && captured_vars.is_none() {
            captured = self.environment().current_scope_var_refs();
        }
        let origin = self.block_origin(parameters, parameter_defaults, shared, *position);
        let mut block = BlockStatement::from_origin(origin, Rc::new(captured), *is_lambda);
        match kept {
            Some((captured, reserved)) => {
                debug_assert!(
                    {
                        let fresh = self.environment().current_scope_var_refs();
                        fresh.len() == captured.len()
                            && fresh
                                .iter()
                                .zip(captured.iter())
                                .all(|(made, held)| made.0 == held.0 && Rc::ptr_eq(made.1, held.1))
                            && *reserved == self.visible_local_names()
                    },
                    "a kept closure is what the scopes open here close over"
                );
                block.captured_vars = captured;
                block.reserved_names = reserved;
            }
            None => {
                let names = self.visible_local_names();
                let mut last = shared.reserved_names.borrow_mut();
                block.reserved_names = match last.as_ref() {
                    Some(held) if **held == names => Rc::clone(held),
                    _ => {
                        let made = Rc::new(names);
                        *last = Some(Rc::clone(&made));
                        made
                    }
                };
                drop(last);
                if let Some(closure) = closure {
                    self.keep_closure(closure, &block.captured_vars, &block.reserved_names);
                }
            }
        }
        if captures_everything && !shared_here {
            self.keep_closure(
                &self.closure_here,
                &block.captured_vars,
                &block.reserved_names,
            );
        }
        block.home_frame = self.lexical_home_frame.unwrap_or(self.current_method_frame);
        let block = Rc::new(block);
        crate::vm::counters::count_block_built();
        self.keep_block(&shared.kept, &block);
        Ok(Object::Block(block))
    }

    /// The origin of a block the literal `shared` belongs to makes here,
    /// which is the one its last block was given when that block was made
    /// in the same place.
    fn block_origin(
        &self,
        parameters: &[String],
        parameter_defaults: &[(usize, Expression)],
        shared: &crate::bytecode::unit::SharedBody,
        position: crate::lexer::Position,
    ) -> Rc<crate::object::BlockOrigin> {
        let defining_method = self.enclosing_method_names();
        let defining_class = if defining_method.is_some() {
            self.method_owner_stack.last().cloned().flatten()
        } else {
            None
        };
        let captured_constant_home = self.applying_constant_home().map(|home| home.cloned());
        let defining_owner = self.enclosing_method_owner().and_then(|(owner, _)| owner);
        // The block's body belongs to the file it was written in,
        // wherever it is later called from.
        // A block the core library opens stands in for Ruby's C code,
        // which names no source location at all.
        let source_file = if position.prelude {
            None
        } else {
            // The main script is named the way it was given, as
            // `__FILE__` names it.
            let file = self.current_source_file.clone().or_else(|| {
                self.reported_current_file()
                    .map(|path| path.display().to_string())
            });
            file.map(|file| {
                let mut last = shared.file.borrow_mut();
                match last.as_ref() {
                    Some(held) if **held == *file => Rc::clone(held),
                    _ => {
                        let made: Rc<str> = file.into();
                        *last = Some(Rc::clone(&made));
                        made
                    }
                }
            })
        };
        let written_within = match &source_file {
            Some(file) => self.places_within(shared, file, position.line as usize),
            None => crate::object::BlockOrigin::no_places(),
        };
        let class_var_home = Some(self.class_var_cref_stack.last().cloned().flatten());
        let written_in = Some(self.enclosing_scope_label());
        let written_depth = Some(self.block_nesting_depth());
        let method_nesting = self.method_nesting_stack.last();
        if let Some(last) = shared.origin.borrow().as_ref()
            && same_classes(&last.captured_def_scope, self.def_scope_stack.iter())
            && match method_nesting {
                Some(nesting) => Rc::ptr_eq(nesting, &last.captured_nesting),
                None => same_classes(&last.captured_nesting, self.def_scope_stack.iter().rev()),
            }
            && last.defining_method == defining_method
            && same_home(&last.defining_class, &defining_class)
            && match (&last.captured_constant_home, &captured_constant_home) {
                (Some(held), Some(home)) => same_home(held, home),
                (None, None) => true,
                _ => false,
            }
            && match (&last.class_var_home, &class_var_home) {
                (Some(held), Some(home)) => same_home(held, home),
                (None, None) => true,
                _ => false,
            }
            && last.defining_owner == defining_owner
            && last.source_file == source_file
            && Rc::ptr_eq(&last.written_within, &written_within)
            && last.written_in == written_in
            && last.written_depth == written_depth
        {
            return Rc::clone(last);
        }
        // The scopes open here are what the body reads its own
        // lexical nesting as, whatever scope it is later called from.
        let captured_nesting = match method_nesting {
            Some(captured) => captured.clone(),
            None => self.snapshot_lexical_nesting().into(),
        };
        let made = Rc::new(crate::object::BlockOrigin {
            parameters: parameters.to_vec(),
            outer_locals: Rc::clone(&shared.outer_locals),
            parameter_defaults: parameter_defaults.to_vec(),
            body: shared.body.clone(),
            captured_def_scope: self.def_scope_stack.clone(),
            captured_constant_home,
            class_var_home,
            captured_nesting,
            defining_method,
            defining_owner,
            defining_class,
            // Where the block was opened, which is the line
            // `source_location` names however far down the body starts.
            opened_at: (!position.prelude).then_some(position.line as usize),
            opened_column: (!position.prelude)
                .then_some(position.column.saturating_sub(1) as usize),
            source_file,
            from_symbol: None,
            written_in,
            written_depth,
            written_within,
            unit: shared.unit.clone(),
        });
        *shared.origin.borrow_mut() = Some(Rc::clone(&made));
        made
    }

    /// The places a block written at `line` of `file`, in the code running
    /// now, was written within: those of the code with its own added. The
    /// list the literal's last block was given is shared while the code and
    /// the file are the same.
    fn places_within(
        &self,
        shared: &crate::bytecode::unit::SharedBody,
        file: &str,
        line: usize,
    ) -> crate::object::CodePlaces {
        let enclosing = self.running_code.last().map(|running| &running.within);
        if let Some(kept) = shared.places.borrow().as_ref()
            && kept.file == file
            && match (&kept.enclosing, enclosing) {
                (None, None) => true,
                (Some(kept), Some(open)) => Rc::ptr_eq(kept, open),
                _ => false,
            }
        {
            return Rc::clone(&kept.made);
        }
        let mut within = enclosing
            .map(|running| running.as_ref().clone())
            .unwrap_or_default();
        within.push((file.to_string(), line));
        let made = Rc::new(within);
        *shared.places.borrow_mut() = Some(crate::bytecode::unit::KeptPlaces {
            enclosing: enclosing.cloned(),
            file: file.to_string(),
            made: Rc::clone(&made),
        });
        made
    }

    /// Apply a binary operator to operands already evaluated, which is how a
    /// compound assignment combines what it read with its right-hand side.
    /// Evaluate the block written after a call, marking the call it is
    /// attached to as running until `release_attached_blocks` clears it.
    pub(crate) fn attach_trailing_block(
        &mut self,
        block_expr: &Expression,
    ) -> Result<Object, MetorexError> {
        let block = self.evaluate_expression(block_expr)?;
        if let View::Block(held) = block.view() {
            held.flags.attached_call_running.set(true);
            self.attached_block_flags.push(Rc::clone(&held.flags));
        }
        Ok(block)
    }

    /// The signal a `break` in a block raises: a BlockBreak that returns
    /// from the call the block was attached to, or a LocalJumpError when
    /// that call has already returned.
    pub(crate) fn break_signal(
        &self,
        value: Object,
        location: crate::error::SourceLocation,
    ) -> MetorexError {
        if let Some(Some(running)) = self.running_block_breaks.last()
            && !running.attached_call_running.get()
        {
            let message = "break from proc-closure".to_string();
            return MetorexError::UncaughtException {
                exception: Object::exception("LocalJumpError", message.clone()),
                location,
                message,
            };
        }
        MetorexError::BlockBreak {
            value,
            location,
            home_frame: None,
        }
    }

    /// Mark every block attached since the stack stood at `depth` as no
    /// longer running, since the call it was attached to has returned.
    pub(crate) fn release_attached_blocks(&mut self, depth: usize) {
        while self.attached_block_flags.len() > depth {
            if let Some(flag) = self.attached_block_flags.pop() {
                flag.attached_call_running.set(false);
            }
        }
    }

    /// Run an operator while a TracePoint listens. One a built-in value
    /// answers itself fires `c_call` and `c_return` around it, as the C
    /// method MRI runs for it does, and one written in Ruby fires the
    /// events of the method it calls.
    pub(crate) fn traced_operator(
        &mut self,
        op: &BinaryOp,
        left_value: Object,
        right_value: Object,
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        let Some(name) = binary_op_method_name(op).filter(|_| !position.prelude) else {
            return self.operate_on_values(op, left_value, right_value, &position);
        };
        self.traced_native_operation(&left_value.clone(), name, position, |vm| {
            vm.operate_on_values(op, left_value, right_value, &position)
        })
    }

    /// Run `operation`, which calls `name` on `receiver`, firing `c_call`
    /// and `c_return` around it when the receiver answers it natively.
    pub(crate) fn traced_native_operation(
        &mut self,
        receiver: &Object,
        name: &str,
        position: crate::lexer::Position,
        operation: impl FnOnce(&mut Self) -> Result<Object, MetorexError>,
    ) -> Result<Object, MetorexError> {
        self.program_call_pending = true;
        let traced = self.begin_native_trace(receiver, name, position)?;
        let answered = operation(self);
        let reported = answered.clone().map(Some);
        self.end_native_trace(traced, receiver, name, &reported, position)?;
        answered
    }

    pub(crate) fn operate_on_values(
        &mut self,
        op: &BinaryOp,
        left_value: Object,
        right_value: Object,
        position: &crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        if self.refinement_scopes.iter().any(|scope| !scope.is_empty())
            && let Some(op_name) = binary_op_method_name(op)
            && let Some(method) =
                crate::vm::method_lookup::find_refinement(&left_value, op_name, self)
        {
            let class = self.builtins().class_of(&left_value);
            return self.invoke_method(class, method, left_value, vec![right_value], *position);
        }
        // A reopened core class, or a module prepended to one,
        // holds the operator in a method table rather than in the
        // native one, so `1 + 2` consults it before the built-in
        // arithmetic runs. Only the receiver's own class and its
        // prepended modules are asked: an ancestor's definition, such
        // as the core library's Numeric#%, describes the operator the
        // native path already implements, and answering from there
        // would replace every built-in operator with its Ruby-level
        // spelling.
        if !matches!(left_value.view(), View::Instance(_))
            && let Some(op_name) = binary_op_method_name(op)
            && let Some((class, method)) = self.lookup_own_operator_method(&left_value, op_name)
        {
            return self.invoke_method(
                class,
                method,
                left_value.clone(),
                vec![right_value],
                *position,
            );
        }
        // Check for user-defined operator methods on instances. Walk
        // via lookup_method so per-instance singleton-class overrides
        // (used by mspec mocks, among other things) win over the
        // underlying class definition.
        if let (Some(op_name), View::Instance(_)) = (binary_op_method_name(op), left_value.view()) {
            if let Some((class, method)) = self.lookup_method(&left_value, op_name) {
                if !method.is_undefined {
                    return self.invoke_method(
                        class,
                        method,
                        left_value.clone(),
                        vec![right_value],
                        *position,
                    );
                }
                // An operator the class undefined is answered by its
                // `method_missing`, and raises without one.
                if let Some(answer) = self.undefined_hook_via_method_missing(
                    &left_value,
                    op_name,
                    &right_value,
                    *position,
                ) {
                    return answer;
                }
                let arguments = [right_value];
                let wording = self.receiver_wording_for(&left_value, *position);
                return Err(crate::vm::errors::undefined_method_error_worded(
                    op_name,
                    &left_value,
                    &arguments,
                    wording,
                    *position,
                ));
            }
            // A generated struct class answers `==` from Struct's
            // native table, which the method map above does not hold.
            let instance_class = match left_value.view() {
                View::Instance(instance) => Some(std::rc::Rc::clone(&instance.borrow().class)),
                _ => None,
            };
            if let Some(class) = &instance_class
                && let Some(members) = crate::vm::native_methods::struct_members(class)
                && let Some(result) = self.call_struct_instance_method(
                    class,
                    &members,
                    &left_value,
                    op_name,
                    std::slice::from_ref(&right_value),
                    *position,
                )?
            {
                return Ok(result);
            }
            // Rational arithmetic likewise lives in a native table.
            if let Some(class) = &instance_class
                && class.name() == "Rational"
                && let Some(result) = self.call_rational_method(
                    &left_value,
                    op_name,
                    std::slice::from_ref(&right_value),
                    *position,
                )?
            {
                return Ok(result);
            }
            // Comparable derives `<`, `<=`, `>` and `>=` from `<=>` for a
            // class that includes it; metorex answers them here rather than
            // through methods on the module.
            if matches!(
                op,
                BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual
            ) && (!matches!(left_value.view(), View::Instance(_))
                || self.is_comparable(&left_value))
                && let Some((class, spaceship)) = self.lookup_method(&left_value, "<=>")
            {
                let cmp = self.invoke_method(
                    class,
                    spaceship,
                    left_value.clone(),
                    vec![right_value.clone()],
                    *position,
                )?;
                if let View::Int(c) = cmp.view() {
                    let result = match op {
                        BinaryOp::Less => c < 0,
                        BinaryOp::LessEqual => c <= 0,
                        BinaryOp::Greater => c > 0,
                        BinaryOp::GreaterEqual => c >= 0,
                        _ => unreachable!(),
                    };
                    return Ok(Object::Bool(result));
                }
            }
        }
        // `1 / 2r` — an Integer or Float on the left of a Rational is
        // promoted so the Rational's own arithmetic runs.
        if let (Some(op_name), View::Int(_) | View::Float(_)) =
            (binary_op_method_name(op), left_value.view())
            && crate::vm::native_methods::rational_parts(&right_value).is_some()
        {
            let promoted = self.promote_to_rational(&left_value, *position)?;
            if let Some(result) = self.call_rational_method(
                &promoted,
                op_name,
                std::slice::from_ref(&right_value),
                *position,
            )? {
                return Ok(result);
            }
        }
        self.evaluate_binary_operation(op, left_value, right_value, *position)
    }

    /// Read `name` out of a namespace already evaluated, which is how
    /// `Ns::Name` resolves and how `Ns::Name += 1` reads before it writes.
    pub(crate) fn read_scoped_constant(
        &mut self,
        ns_value: Object,
        name: &str,
        position: &crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        match ns_value.view() {
            View::Class(class_rc) | View::Module(class_rc) => {
                // Own constants first, then (like Ruby's qualified
                // lookup) the ancestor chain — but not top-level
                // constants, which a qualified reference must not
                // reach. Registered autoloads fire on their owner.
                // Top-level constants are Object's own, so a name
                // written as `Object::Name` reaches one of them before
                // any module mixed into Object.
                let top_level = if class_rc.name() == "Object"
                    && name.starts_with(|held: char| held.is_ascii_uppercase())
                {
                    self.globals()
                        .get(name)
                        .filter(|held| !matches!(held.view(), View::NativeFunction(_)))
                } else {
                    None
                };
                let entry = match top_level {
                    Some(held) => Some((Rc::clone(class_rc), Some(held))),
                    None => self.const_entry_on(class_rc, name, true, false),
                };
                let owner = match &entry {
                    Some((owner, _)) => Rc::clone(owner),
                    None => Rc::clone(class_rc),
                };
                let value = match entry {
                    Some((_, Some(v))) => Some(v),
                    Some((owner, None)) => self.try_autoload_constant(&owner, name)?,
                    None => self.try_autoload_constant(class_rc, name)?,
                };
                if let Some(v) = value {
                    // A private constant is read with its scope written out
                    // only from inside the body of the class that owns it.
                    if owner.is_private_constant(name)
                        && !self
                            .def_scope_stack
                            .iter()
                            .any(|open| Rc::ptr_eq(open, &owner))
                    {
                        return self.private_constant_refused(class_rc, &owner, name, *position);
                    }
                    self.warn_deprecated_constant(class_rc, name, *position);
                    return Ok(v);
                }
                // Uninitialized constants dispatch const_missing —
                // the default implementation raises NameError.
                // Autoload's "loaded but didn't define" path lands
                // here too.
                self.dispatch_const_missing(class_rc, name, *position)
            }
            // Only a class or module holds constants, so reading one
            // out of anything else is refused by its type.
            _ => {
                let shown = self.get_inspect_representation(&ns_value, *position)?;
                Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &format!("{shown} is not a class/module"),
                    *position,
                ))
            }
        }
    }
}

/// Map a binary operator to its operator method name for user-defined dispatch.
impl VirtualMachine {
    /// Apply a unary operator to a value. An object that defines `-@`,
    /// `+@`, or `!` names what the operator in front of it does, which is
    /// how a number a program writes negates and how a delegator passes `!`
    /// on.
    pub(crate) fn apply_unary_operator(
        &mut self,
        op: &crate::ast::UnaryOp,
        value: Object,
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        if let View::Instance(_) = value.view()
            && let Some(name) = match op {
                crate::ast::UnaryOp::Minus => Some("-@"),
                crate::ast::UnaryOp::Plus => Some("+@"),
                crate::ast::UnaryOp::Not => Some("!"),
            }
            && (self
                .lookup_method(&value, name)
                .is_some_and(|(_, method)| !method.is_undefined)
                || crate::vm::native_methods::rational_parts(&value).is_some()
                || crate::vm::native_methods::complex_parts(&value).is_some())
        {
            return self.send_to_object(value, name, vec![], position);
        }
        self.evaluate_unary_operation(op, value, position)
    }
}

pub(crate) fn binary_op_method_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("+"),
        BinaryOp::Subtract => Some("-"),
        BinaryOp::Multiply => Some("*"),
        BinaryOp::Divide => Some("/"),
        BinaryOp::Modulo => Some("%"),
        BinaryOp::Power => Some("**"),
        BinaryOp::Equal => Some("=="),
        BinaryOp::CaseEqual => Some("==="),
        BinaryOp::NotEqual => Some("!="),
        BinaryOp::Less => Some("<"),
        BinaryOp::Greater => Some(">"),
        BinaryOp::LessEqual => Some("<="),
        BinaryOp::GreaterEqual => Some(">="),
        BinaryOp::Spaceship => Some("<=>"),
        BinaryOp::BitwiseAnd => Some("&"),
        BinaryOp::BitwiseOr => Some("|"),
        BinaryOp::Xor => Some("^"),
        _ => None,
    }
}

/// Whether `$<digits>` names a capture past the highest one a pattern can
/// number, which Ruby reads as nil and says so about.
fn number_variable_is_too_big(name: &str) -> bool {
    /// The highest group a pattern can number, above which Ruby reads the
    /// variable as nil.
    const HIGHEST_GROUP: u128 = (1 << 30) - 1;
    if name.is_empty() || !name.bytes().all(|held| held.is_ascii_digit()) {
        return false;
    }
    name.parse::<u128>().is_ok_and(|held| held > HIGHEST_GROUP)
}

impl VirtualMachine {
    /// The value the global `$name` holds, read the way `$name` written in
    /// the program reads it.
    pub(crate) fn read_global_variable(
        &mut self,
        name: &str,
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        self.refuse_global_in_ractor(name, position)?;
        // `$?` is the status of the last child waited for, which lives
        // with the process rather than in the global table.
        // A global given a second name reads what the first holds,
        // whatever that first name is: `$MATCH` reads the match `$&`
        // names the same way `$&` does.
        let name = self
            .global_aliases
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string());
        // A global a C extension defined is read through its getter.
        if let Some(hooked) = self.hooked_globals.get(&name).copied() {
            return crate::vm::capi::read_hooked_global(self, &name, hooked, position);
        }
        if name == "?" {
            return Ok(self.process_last_status());
        }
        // `$=` once made matches ignore case, and reading it says it
        // no longer does once the deprecated category is asked for.
        if name == "=" && self.warning_category_enabled("deprecated") {
            let message = format!(
                "{}variable $= is no longer effective\n",
                self.warning_prefix(0, position)
            );
            self.warn_through_warning_module(message, position)?;
        }
        // A number past what a capture can be numbered names no
        // group at all, which Ruby says so about and reads as nil.
        if number_variable_is_too_big(&name) {
            let at = (
                self.current_source_file.clone().unwrap_or_default(),
                position.line,
                position.column,
            );
            if self.reported_big_number_variables.insert(at) {
                let message =
                    format!("warning: '${name}' is too big for a number variable, always nil");
                self.emit_warning_to_stderr(&message, position);
            }
            return Ok(Object::Nil);
        }
        // `$1` through `$9` name the captures of the last match, and
        // `` $` `` and `$'` the text on either side of it.
        if let Some(group) = crate::vm::native_methods::capture_reference(&name) {
            return self.last_match_part(group, position);
        }
        // `$@` is where the exception being handled was raised, which
        // is the backtrace that exception carries.
        if name == "@" {
            let raised = self.globals().get("!").unwrap_or(Object::Nil);
            if matches!(raised.view(), View::Nil) {
                return Ok(Object::Nil);
            }
            return self.send_to_object(raised, "backtrace", Vec::new(), position);
        }
        match self.globals().get(&name) {
            Some(held) => Ok(held),
            None => {
                // Reading a global nothing has set says so in
                // verbose mode.
                if matches!(self.globals().get("VERBOSE").view(), Some(View::Bool(true))) {
                    let message = format!(
                        "{}global variable '${name}' not initialized\n",
                        self.warning_prefix(0, position)
                    );
                    self.warn_through_warning_module(message, position)?;
                }
                Ok(Object::Nil)
            }
        }
    }
}

/// Whether a Range end is written as a literal, which makes the Range one
/// object however many times its line runs.
pub(crate) fn literal_range_end(end: &Expression) -> bool {
    matches!(
        end,
        Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::NilLiteral { .. }
    )
}

/// Whether `held` lists the classes `open` does, one for one.
fn same_classes<'a>(
    held: &[Rc<crate::class::Class>],
    open: impl ExactSizeIterator<Item = &'a Rc<crate::class::Class>>,
) -> bool {
    held.len() == open.len()
        && held
            .iter()
            .zip(open)
            .all(|(one, other)| Rc::ptr_eq(one, other))
}

/// Whether two optional classes are the same class, or both none.
fn same_home(
    held: &Option<Rc<crate::class::Class>>,
    other: &Option<Rc<crate::class::Class>>,
) -> bool {
    match (held, other) {
        (Some(one), Some(other)) => Rc::ptr_eq(one, other),
        (None, None) => true,
        _ => false,
    }
}
