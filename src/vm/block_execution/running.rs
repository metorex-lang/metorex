// Running a block, with the scope it captured.

use super::*;
use crate::object::{OptionView, View};
use std::rc::Rc;

impl VirtualMachine {
    /// The values a lone argument spreads into when the block takes it apart
    /// across several parameters. `None` when it stays one argument.
    fn spread_lone_argument(
        &mut self,
        block: &BlockStatement,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Vec<Object>>, MetorexError> {
        match arguments {
            [single] if block.destructures_single_array() => {
                self.block_argument_spread(single, position)
            }
            _ => Ok(None),
        }
    }

    /// Execute a block callable within the VM, handling scope capture and return semantics.
    pub(crate) fn execute_block_callable(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some(name) = &block.from_symbol {
            return self.call_symbol_block(name, arguments, position);
        }
        // `{ |a,| }` destructures a lone array argument across its parameters,
        // discarding any elements it has no parameter for.
        let mut destructured = false;
        let arguments = match self.spread_lone_argument(block, &arguments, position)? {
            Some(elements) => {
                destructured = true;
                elements
            }
            None => arguments,
        };

        let parameters = block.binding_parameters();
        let expected = parameters.len();
        // A trailing keyword-argument hash feeds the keyword parameters, so
        // a lambda declaring one does not count it among the positionals.
        let takes_keywords = parameters.iter().any(|name| {
            name.starts_with("**") || name.starts_with(crate::object::KEYWORD_PARAM_PREFIX)
        });
        let found = if takes_keywords && matches!(arguments.last().view(), Some(View::Dict(_))) {
            arguments.len() - 1
        } else {
            arguments.len()
        };
        let has_variadic = parameters.iter().any(|p| p.starts_with('*'));
        let has_block_param = parameters.iter().any(|p| p.starts_with('&'));

        // Optional params (`|a, b = 1|`) widen the accepted count: `found`
        // may run from `expected - defaults` up to `expected`.
        let required = expected.saturating_sub(block.parameter_defaults.len());

        // Variadic params accept any number of args; skip strict arity check.
        // Only a lambda checks arity at all: a proc pads missing arguments
        // with nil and drops extras.
        if !destructured {
            let _ = (has_variadic, has_block_param, required, expected);
            // `|**nil|` says the block takes no keyword arguments at all,
            // whether it is a lambda or a proc.
            refuse_keywords_for_none_declared(block, &arguments, position)?;
            strict_arity_check(block, found, position)?;
        }

        let depth = block
            .written_depth
            .unwrap_or_else(|| self.block_nesting_depth());
        let frame_name = BLOCK_FRAME_NAME.with(Rc::clone);
        let frame = match &block.defining_method {
            Some((callee, defined)) => {
                CallFrame::method(frame_name, None, Rc::clone(callee), Rc::clone(defined))
            }
            // A block written outside any method is still entered from
            // somewhere, and that is the call site the frame records.
            None => CallFrame::boundary(frame_name),
        }
        .entered_at(position.line, position.column)
        .nested_in_a_block(depth)
        .written_in_scope(block.written_in.clone())
        .owned_by(block.defining_owner.clone())
        .owned_by_class(block.defining_class.clone())
        .with_source_file(self.frames_file());
        // The body runs in the file the block was written in, which is what a
        // backtrace entry for a call made from here has to name.
        let saved_file = self.enter_block_file(block);
        let execution_result =
            self.with_call_frame(frame, move |vm| vm.execute_block_body(block, arguments));
        self.leave_block_file(saved_file);

        match execution_result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.with_stack_frame(StackFrame::new(
                block.name().to_string(),
                position_to_location(position),
            ))),
        }
    }

    /// Execute the statements inside a block object with its captured scope.
    pub(crate) fn execute_block_body(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        self.in_home_frame_of(block, |vm| {
            vm.execute_block_body_in_home(block, arguments)
        })
    }

    fn execute_block_body_in_home(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
    ) -> Result<Object, MetorexError> {
        // A block with nothing in it still lets a limit on its caller pass.
        if !self.timeout_limits.is_empty() {
            let opened = crate::lexer::Position::new(
                block.opened_at.unwrap_or(0),
                block.opened_column.unwrap_or(0),
                0,
            );
            self.raise_expired_timeout(opened)?;
        }
        // A block body runs in its own scope seeded from `captured_vars`, not
        // chained to whatever method happens to be invoking it. Ruby's block
        // sees the locals of the scope it was written in, and nothing of its
        // caller's.
        self.environment_mut().push_isolated_scope();
        // Restore the lexical class/module nesting from the block's
        // definition site so an uppercase `Foo = ...` inside the body
        // assigns to the same enclosing module the surrounding code would
        // have. Saved/restored in pure stack fashion in case the caller's
        // current def_scope_stack is non-empty (e.g. block invoked from
        // inside a class body).
        let saved_def_scope =
            std::mem::replace(&mut self.def_scope_stack, block.captured_def_scope.clone());
        let carries_constant_home = self.enter_captured_constant_home(block);
        // A block opened inside this body belongs to the method this one was
        // written in, so a `return` written two blocks deep unwinds to the
        // method holding them both.
        let saved_lexical_home = self.lexical_home_frame.replace(block.home_frame);
        // The body belongs to the file the block was written in, whatever
        // file called it.
        let saved_file = self.enter_block_file(block);
        // A class variable written in the body belongs to the class or module
        // the block was written in, which is none at the top level.
        self.class_var_cref_stack.push(
            block
                .captured_nesting
                .first()
                .cloned()
                .or_else(|| block.captured_def_scope.last().cloned()),
        );
        // The scopes open around the block are the ones it was written in,
        // not the ones open in the method that called it, which is what
        // `Module.nesting` in the body reports and where an `eval` written
        // there opens what it defines.
        self.enter_block_nesting(block.captured_nesting.clone());

        // A trace sees a block body opening and closing, and reads the
        // parameters the block declared off either event.
        let (block_position, declared) = self.enter_block_body(block)?;

        // A lambda is something a `return` carried out of an eval can return
        // from, so its body says while it runs that one is there.
        if block.is_lambda {
            self.lambda_body_depth += 1;
        }
        self.running_block_breaks.push(running_break_flag(block));
        let result = (|| -> Result<Object, MetorexError> {
            // The names the block closed over are read through, not copied.
            self.environment_mut()
                .attach_captured(std::rc::Rc::clone(&block.captured_vars));
            self.environment_mut()
                .attach_reserved(std::rc::Rc::clone(&block.reserved_names));

            // A lambda takes its arguments the way a method does, so the
            // count has to match what it declared.
            if block.is_lambda {
                check_lambda_arity(block, &arguments, Position::new(0, 0, 0))?;
            }

            // Define parameters as regular variables (handles *args/&block prefixes)
            let arguments = marked_keyword_tail(block, arguments);
            bind_block_params(
                self,
                &block.binding_parameters(),
                &block.parameter_defaults,
                arguments,
                Position::new(0, 0, 0),
            )?;

            // A name written after the `;` in the parameter list is a local
            // of the block, which starts as nil however the outer scope reads.
            for name in block.block_locals() {
                self.environment_mut().define(name, Object::Nil);
            }

            let last_value;

            // `redo` runs the block's body again over the same arguments.
            'again: loop {
                {
                    let ran = self.run_compiled_unit(
                        Some(&block.unit),
                        block.body(),
                        crate::vm::units::UnitKind::Plain,
                    );
                    let flow = match ran {
                        // A `redo` inside a `begin` that has an ensure clause
                        // arrives as an unwinding signal once the clause ran.
                        Err(MetorexError::BlockRedo { .. }) => continue 'again,
                        other => other?,
                    };
                    match flow {
                        ControlFlow::Next => last_value = Object::Nil,
                        ControlFlow::Value(value) => last_value = value,
                        // `return` in a lambda returns from the lambda. In a
                        // proc or ordinary block it is a long return: it
                        // unwinds to the method that lexically created the
                        // block.
                        ControlFlow::Return { value, position } => {
                            if block.is_lambda {
                                last_value = value;
                                break 'again;
                            }
                            if let Some(home) = block.home_frame
                                && !self.live_frames.contains(&home)
                            {
                                return Err(orphaned_return_error(value, position));
                            }
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: position_to_location(position),
                                home_frame: block.home_frame,
                            });
                        }
                        // `break` in a lambda ends the lambda, and in any
                        // other block returns from the call the block was
                        // attached to.
                        ControlFlow::Break { value, position } => {
                            if block.is_lambda {
                                last_value = value;
                                break 'again;
                            }
                            return Err(self.break_signal(value, position_to_location(position)));
                        }
                        ControlFlow::Redo { .. } | ControlFlow::Retry { .. } => continue 'again,
                        // `next <value>` ends this run of the block with that
                        // value.
                        ControlFlow::Continue { value, .. } => last_value = value,
                    }
                    break 'again;
                }
            }

            Ok(last_value)
        })();
        self.running_block_breaks.pop();
        let result = match result {
            Ok(value) => self
                .leave_block_body(block, block_position, declared, &value)
                .map(|()| value),
            failed => {
                self.leave_running_code();
                failed
            }
        };

        if block.is_lambda {
            self.lambda_body_depth -= 1;
        }
        self.environment_mut().pop_scope();
        self.class_var_cref_stack.pop();
        self.leave_block_nesting();
        if carries_constant_home {
            self.constant_homes.pop();
        }
        self.def_scope_stack = saved_def_scope;
        self.lexical_home_frame = saved_lexical_home;
        // The exception says where it came from while the block's own file is
        // still the one in force, which is what a backtrace entry names.
        if let Err(MetorexError::UncaughtException {
            exception,
            location,
            ..
        }) = &result
        {
            self.note_exception_location(exception, location);
        }
        self.leave_block_file(saved_file);
        // A `break` leaving this body belongs to the invocation that was
        // handed the block, which is the call made from the frame the block
        // was written in.
        match result {
            // A `return` carried out of an eval stops at the lambda it was
            // written inside, and passing out of any other block leaves it
            // looking for the invocation it belongs to.
            Err(MetorexError::NonLocalReturn {
                value,
                location,
                home_frame,
            }) => {
                if block.is_lambda && home_frame.is_none() {
                    return Ok(value);
                }
                // Nothing is left to return from once the return has passed
                // out of every block, with no lambda around it and no method
                // running, which is what makes it a LocalJumpError.
                let nowhere_to_return_to = match block.home_frame {
                    // A block written outside any method belongs to the unit
                    // itself, which is not something to return from.
                    Some(home) => {
                        home == crate::vm::core::TOP_LEVEL_FRAME
                            || !self.live_frames.contains(&home)
                    }
                    None => true,
                };
                if home_frame.is_none() && self.lambda_body_depth == 0 && nowhere_to_return_to {
                    return Err(escaped_return_error(value, location));
                }
                Err(MetorexError::NonLocalReturn {
                    value,
                    location,
                    home_frame,
                })
            }
            // `next` written inside an expression unwinds to here, and ends
            // this run of the block with the value it carried.
            Err(MetorexError::BlockNext { value, .. }) => Ok(value),
            // A `break` returns from the call the block was attached to,
            // which it cannot do once that call has returned.
            Err(MetorexError::BlockBreak {
                home_frame: None,
                location,
                ..
            }) if !block.attached_call_running.get() => Err(break_from_proc_closure(location)),
            Err(MetorexError::BlockBreak {
                value,
                location,
                home_frame: None,
            }) => Err(MetorexError::BlockBreak {
                value,
                location,
                home_frame: block.home_frame,
            }),
            other => other,
        }
    }

    /// Execute a block body and return ControlFlow (for use in iterators like .each)
    /// This version propagates Break/Continue instead of converting them to errors
    pub(crate) fn execute_block_with_control_flow(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        self.in_home_frame_of(block, |vm| {
            vm.execute_block_with_control_flow_in_home(block, arguments, position)
        })
    }

    fn execute_block_with_control_flow_in_home(
        &mut self,
        block: &BlockStatement,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<ControlFlow, MetorexError> {
        // `{ |x, y| }` handed a single array spreads it across the parameters,
        // which is how `[[1, 2]].each { |x, y| }` binds x and y.
        let arguments = self
            .spread_lone_argument(block, &arguments, position)?
            .unwrap_or(arguments);
        // A lambda takes its arguments the way a method does, so a yield that
        // does not match its parameters is refused rather than padded.
        strict_arity_check(block, arguments.len(), Position::new(0, 0, 0))?;
        // A block body is a place of its own in a backtrace, named for the
        // scope it was written in and for how many blocks deep it sits.
        let depth = block
            .written_depth
            .unwrap_or_else(|| self.block_nesting_depth());
        // A block reached from a method is entered where that method was
        // called, which is the call site the frame records.
        let frame_name = BLOCK_FRAME_NAME.with(Rc::clone);
        let frame = match &block.defining_method {
            Some((callee, defined)) => {
                CallFrame::method(frame_name, None, Rc::clone(callee), Rc::clone(defined))
            }
            None => CallFrame::boundary(frame_name),
        }
        .entered_at(position.line, position.column)
        .nested_in_a_block(depth)
        .written_in_scope(block.written_in.clone())
        .owned_by(block.defining_owner.clone())
        .owned_by_class(block.defining_class.clone())
        .with_source_file(self.frames_file());
        self.call_stack_push(frame);
        self.environment_mut().push_isolated_scope();
        // The body belongs to the file the block was written in.
        let saved_file = self.enter_block_file(block);

        // The body reads constants and class variables in the scopes it was
        // written in, not those of the method that runs it, which a block
        // handed on with `&block` to a native iterator would otherwise see.
        self.class_var_cref_stack.push(
            block
                .captured_nesting
                .first()
                .cloned()
                .or_else(|| block.captured_def_scope.last().cloned()),
        );
        self.enter_block_nesting(block.captured_nesting.clone());
        self.running_block_breaks.push(running_break_flag(block));
        let result = match self.enter_block_body(block) {
            Err(error) => Err(error),
            Ok((block_position, declared)) => {
                let mut last_value = Object::Nil;
                let ran = (|| -> Result<ControlFlow, MetorexError> {
                    // The names the block closed over are read through, not copied.
                    self.environment_mut()
                        .attach_captured(std::rc::Rc::clone(&block.captured_vars));
                    self.environment_mut()
                        .attach_reserved(std::rc::Rc::clone(&block.reserved_names));

                    // Define parameters as regular variables (handles *args/&block prefixes)
                    bind_block_params(
                        self,
                        &block.binding_parameters(),
                        &block.parameter_defaults,
                        arguments,
                        Position::new(0, 0, 0),
                    )?;

                    // A name written after the `;` is a local of the block,
                    // which starts as nil however the outer scope reads.
                    for name in block.block_locals() {
                        self.environment_mut().define(name, Object::Nil);
                    }

                    // `redo` runs the block's body again over the same arguments,
                    // which are already bound in this scope.
                    loop {
                        {
                            let ran = self.run_compiled_unit(
                                Some(&block.unit),
                                block.body(),
                                crate::vm::units::UnitKind::Plain,
                            );
                            let flow = match ran {
                                Err(MetorexError::BlockRedo { .. }) => continue,
                                // A `break` raised by a C function called from
                                // this body, that has not yet passed a block
                                // boundary, belongs to this block.
                                Err(MetorexError::BlockBreak {
                                    value,
                                    home_frame: None,
                                    location,
                                }) => ControlFlow::Break {
                                    value,
                                    position: Position::new(location.line, location.column, 0),
                                },
                                other => other?,
                            };
                            match flow {
                                ControlFlow::Next => last_value = Object::Nil,
                                ControlFlow::Value(value) => last_value = value,
                                ControlFlow::Retry { .. } | ControlFlow::Redo { .. } => continue,
                                flow @ (ControlFlow::Return { .. }
                                | ControlFlow::Break { .. }
                                | ControlFlow::Continue { .. }) => return Ok(flow),
                            }
                            return Ok(ControlFlow::Next);
                        }
                    }
                })();
                match ran {
                    Ok(flow) => self
                        .leave_block_body(block, block_position, declared, &last_value)
                        .map(|()| flow),
                    failed => {
                        self.leave_running_code();
                        failed
                    }
                }
            }
        };
        self.running_block_breaks.pop();
        self.leave_block_nesting();
        self.class_var_cref_stack.pop();

        self.leave_block_file(saved_file);
        self.environment_mut().pop_scope();
        self.call_stack_pop();
        result
    }

    /// Make the file `block` was written in the one running, and a literal
    /// in it read in that file's encoding. Answers the file and encoding it
    /// replaced, or None when the block was written in the running file.
    fn enter_block_file(&mut self, block: &BlockStatement) -> Option<SavedFile> {
        let written_in = block.source_file.as_ref()?;
        if self.current_source_file.as_ref() == Some(written_in) {
            return None;
        }
        let encoding = self.file_encodings.get(written_in).cloned();
        Some((
            self.current_source_file.replace(written_in.clone()),
            std::mem::replace(&mut self.current_source_encoding, encoding),
        ))
    }

    /// Put back the file and encoding `enter_block_file` replaced.
    fn leave_block_file(&mut self, saved: Option<SavedFile>) {
        if let Some((file, encoding)) = saved {
            self.current_source_file = file;
            self.current_source_encoding = encoding;
        }
    }

    /// Start a block body the way a trace sees it: the body is running, and
    /// `b_call` fires at the line the block was opened on. Answers that
    /// position and the parameters the block declared, which `b_return`
    /// reports too.
    fn enter_block_body(
        &mut self,
        block: &BlockStatement,
    ) -> Result<(Position, Option<Object>), MetorexError> {
        let block_position = match block.opened_at {
            Some(line) => Position::new(line, 0, 0),
            None => block
                .body
                .first()
                .map(|held| held.position())
                .unwrap_or_else(|| Position::new(0, 0, 0)),
        };
        self.enter_running_code(
            std::rc::Rc::clone(&block.written_within),
            block_position.line,
        );
        if !self.events_wanted() {
            return Ok((block_position, None));
        }
        let declared = crate::vm::native_methods::block_parameter_list(block);
        let mut extra = vec![("parameters", declared.clone())];
        extra.extend(block_method_names(block));
        if let Err(error) = self.fire_event("b_call", block_position, extra) {
            self.leave_running_code();
            return Err(error);
        }
        Ok((block_position, Some(declared)))
    }

    /// Finish a block body that answered `value`, firing `b_return` at the
    /// line it ran last.
    fn leave_block_body(
        &mut self,
        block: &BlockStatement,
        block_position: Position,
        declared: Option<Object>,
        value: &Object,
    ) -> Result<(), MetorexError> {
        if !self.events_wanted() {
            self.leave_running_code();
            return Ok(());
        }
        let line = self
            .running_code
            .last()
            .map_or(block_position.line, |running| running.line);
        // A trace turned on while the body ran sees the parameters too.
        let declared =
            declared.unwrap_or_else(|| crate::vm::native_methods::block_parameter_list(block));
        let mut extra = vec![("parameters", declared), ("return_value", value.clone())];
        extra.extend(block_method_names(block));
        let fired = self.fire_event("b_return", Position::new(line, 0, 0), extra);
        self.leave_running_code();
        fired
    }

    /// Call the block `Symbol#to_proc` makes: the first argument is sent the
    /// name, with the rest as its arguments. Ruby's symbol proc runs without
    /// a frame of its own, so what it raises names the line that called it.
    #[inline(never)]
    fn call_symbol_block(
        &mut self,
        name: &str,
        mut arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() {
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                "no receiver given",
                position,
            ));
        }
        let receiver = arguments.remove(0);
        arguments.insert(0, Object::symbol(name.to_string()));
        self.send_to_object(
            receiver,
            crate::vm::native_methods::object_methods::SYMBOL_PROC_SEND,
            arguments,
            position,
        )
    }
}

thread_local! {
    /// The name every block's frame carries.
    static BLOCK_FRAME_NAME: Rc<str> = Rc::from("<block>");
}

/// The running file and encoding a block body replaced.
type SavedFile = (Option<String>, Option<String>);

/// The method a block was written in, as the names a block event reports
/// for `method_id` and `callee_id`.
fn block_method_names(block: &BlockStatement) -> Vec<(&'static str, Object)> {
    match &block.defining_method {
        Some((callee, defined)) => vec![
            ("method_id", Object::symbol(defined.to_string())),
            ("callee_id", Object::symbol(callee.to_string())),
        ],
        None => Vec::new(),
    }
}

/// The LocalJumpError a `break` raises from a block whose call has returned.
fn break_from_proc_closure(location: crate::error::SourceLocation) -> MetorexError {
    let message = "break from proc-closure".to_string();
    MetorexError::UncaughtException {
        exception: Object::exception("LocalJumpError", message.clone()),
        location,
        message,
    }
}

/// The flag a `break` in this block consults, which a lambda has none of.
fn running_break_flag(block: &BlockStatement) -> Option<std::rc::Rc<std::cell::Cell<bool>>> {
    (!block.is_lambda).then(|| std::rc::Rc::clone(&block.attached_call_running))
}
