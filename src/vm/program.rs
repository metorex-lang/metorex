// Top-level program execution and the expression-evaluation entrypoint.

use super::core::VirtualMachine;
use super::utils::*;
use crate::object::{OptionView, View};

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{BlockStatement, Object};

/// Build a `BlockStatement` for `&:symbol` (symbol-to-proc): a one-arg block
/// `|x| x.send(:symbol)`.
fn symbol_to_proc_block(sym: &str) -> BlockStatement {
    let pos = Position::new(0, 0, 0);
    let body = vec![Statement::Expression {
        expression: Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "x".to_string(),
                position: pos,
            }),
            // `public_send` is what Ruby's symbol proc uses, so a name the
            // receiver keeps to itself is refused rather than reached. It is
            // also not `send`, which a class of the program's own may name
            // for something else, as a socket does.
            method: crate::vm::native_methods::object_methods::SYMBOL_PROC_SEND.to_string(),
            arguments: vec![Expression::Symbol {
                value: sym.to_string(),
                position: pos,
            }],
            trailing_block: None,
            position: pos,
        },
        position: pos,
    }];
    let mut made = BlockStatement::new(vec!["x".to_string()], body, indexmap::IndexMap::new());
    // What the callable stands for, which is what it says of itself in place
    // of a file and a line.
    made.from_symbol = Some(sym.to_string());
    made
}

/// Build the block `&some_method` hands over:
/// `{ |*args, &block| target.call(*args, &block) }`, where `target` is the
/// Method itself. The call keeps the method's own arity,
/// so a callable given the wrong number of arguments still says so.
fn method_to_proc_block(target: &Object, position: Position) -> BlockStatement {
    let mut parameters = Vec::new();
    let mut parameter_defaults = Vec::new();
    let mut forwarded = Vec::new();
    let written_in_ruby = |method: &crate::object::Method| {
        method.source_location.is_some() || !method.body.is_empty()
    };
    if let View::Method(method) = target.view()
        && written_in_ruby(method)
    {
        let optional: std::collections::HashSet<usize> = method
            .signature
            .default_parameters
            .iter()
            .map(|(index, _)| *index)
            .collect();
        // The splat's own name sits among the positional ones, and it is
        // forwarded as a splat rather than as one more positional.
        let splat_at = method
            .signature
            .variadic_param
            .as_ref()
            .map(|(index, _)| *index);
        for index in 0..method.parameters.len() {
            if splat_at == Some(index) {
                continue;
            }
            let name = format!("__method_proc_p{index}");
            if optional.contains(&index) {
                parameter_defaults.push((parameters.len(), Expression::NilLiteral { position }));
            }
            parameters.push(name.clone());
            forwarded.push(Expression::Identifier { name, position });
        }
        if method.signature.variadic_param.is_some() {
            let name = "__method_proc_rest".to_string();
            parameters.push(format!("*{name}"));
            forwarded.push(Expression::Splat {
                expression: Box::new(Expression::Identifier { name, position }),
                position,
            });
        }
    } else {
        // A method the core library writes natively checks its own
        // arguments, so the block hands it everything it is given.
        let name = "__method_proc_rest".to_string();
        parameters.push(format!("*{name}"));
        forwarded.push(Expression::Splat {
            expression: Box::new(Expression::Identifier { name, position }),
            position,
        });
    }
    let handed = "__method_proc_block".to_string();
    parameters.push(format!("&{handed}"));
    forwarded.push(Expression::BlockArg {
        expression: Box::new(Expression::Identifier {
            name: handed,
            position,
        }),
        position,
    });
    let call = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "__method_proc_target".to_string(),
            position,
        }),
        method: "call".to_string(),
        arguments: forwarded,
        trailing_block: None,
        position,
    };
    let mut made = BlockStatement::new(
        parameters,
        vec![Statement::Expression {
            expression: call,
            position,
        }],
        indexmap::IndexMap::new(),
    );
    made.parameter_defaults = parameter_defaults;
    // A Method takes its arguments exactly, so the block standing for one is
    // a lambda rather than an ordinary block.
    made.is_lambda = true;
    made
}

/// The Proc `rb_proc_new` makes around a C block function: not a lambda,
/// written nowhere in the program, taking any number of values and the block
/// it is called with, and handing them all to `target`.
pub(crate) fn proc_for_c_function(target: Object) -> Object {
    let position = Position::default();
    let rest = "__c_function_values".to_string();
    let handed = "__c_function_block".to_string();
    let call = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "__method_proc_target".to_string(),
            position,
        }),
        method: "call".to_string(),
        arguments: vec![
            Expression::Splat {
                expression: Box::new(Expression::Identifier {
                    name: rest.clone(),
                    position,
                }),
                position,
            },
            Expression::BlockArg {
                expression: Box::new(Expression::Identifier {
                    name: handed.clone(),
                    position,
                }),
                position,
            },
        ],
        trailing_block: None,
        position,
    };
    let mut made = BlockStatement::new(
        vec![format!("*{rest}"), format!("&{handed}")],
        vec![Statement::Expression {
            expression: call,
            position,
        }],
        indexmap::IndexMap::new(),
    );
    std::rc::Rc::make_mut(&mut made.captured_vars).insert(
        "__method_proc_target".to_string(),
        std::rc::Rc::new(std::cell::RefCell::new(target)),
    );
    Object::Block(std::rc::Rc::new(made))
}

/// The block `&target` hands over for a Method, which calls the Method with
/// what the block is given.
pub(crate) fn block_for_method(target: Object, position: Position) -> Object {
    let mut block = method_to_proc_block(&target, position);
    std::rc::Rc::make_mut(&mut block.captured_vars).insert(
        "__method_proc_target".to_string(),
        std::rc::Rc::new(std::cell::RefCell::new(target)),
    );
    Object::Block(std::rc::Rc::new(block))
}

impl VirtualMachine {
    /// Execute a sequence of statements and return an optional result (from return statements).
    pub fn execute_program(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        // Each file's top level starts with its methods private.
        let caller_toplevel_public = std::mem::replace(&mut self.toplevel_public, false);
        let caller_toplevel_frame =
            std::mem::replace(&mut self.toplevel_frame, self.current_method_frame);
        self.borrowed_frames.push(self.current_method_frame);
        // A file's top level sits in no block, whatever block loaded it.
        let caller_lexical_home = self.lexical_home_frame.take();
        // A `return` written at the top level ends the program, carrying its
        // value out however deep in blocks it was written.
        let ran = self.run_compiled_program(statements, false);
        let result = match ran {
            Err(MetorexError::NonLocalReturn { value, .. }) => Ok(Some(value)),
            other => other,
        };
        self.toplevel_public = caller_toplevel_public;
        self.toplevel_frame = caller_toplevel_frame;
        self.borrowed_frames.pop();
        self.lexical_home_frame = caller_lexical_home;
        result
    }

    /// Run a unit whose `return` belongs to the scope around it rather than
    /// ending the unit, which is what code handed to `eval` does.
    pub(crate) fn run_eval_statements(
        &mut self,
        statements: &[Statement],
    ) -> Result<Option<Object>, MetorexError> {
        self.run_compiled_program(statements, true)
    }

    /// A `return` leaving an eval, which unwinds to the invocation the code
    /// around the eval belongs to.
    pub(crate) fn unwinding_return(
        &self,
        value: Object,
        at: crate::lexer::Position,
    ) -> MetorexError {
        // The return belongs to the innermost lambda or method around the
        // eval, whichever the unwinding reaches first, so it names no frame
        // of its own.
        MetorexError::NonLocalReturn {
            value,
            location: position_to_location(at),
            home_frame: None,
        }
    }

    /// Evaluate a list of argument expressions, expanding any splat (`*expr`)
    /// arguments and routing block-arg (`&expr`) arguments to `pending_block`.
    pub(crate) fn evaluate_arguments(
        &mut self,
        argument_exprs: &[Expression],
    ) -> Result<Vec<Object>, MetorexError> {
        let mut arguments = ArgumentList::with_capacity(argument_exprs.len());
        for arg in argument_exprs {
            match arg {
                Expression::Splat {
                    expression,
                    position,
                } => {
                    let value = self.evaluate_expression(expression)?;
                    arguments.splat(self, value, *position)?;
                }
                Expression::KeywordSplat {
                    expression,
                    position,
                } => {
                    let value = self.evaluate_expression(expression)?;
                    arguments.keyword_splat(self, value, *position)?;
                }
                Expression::BlockArg {
                    expression,
                    position,
                } => {
                    let value = self.evaluate_expression(expression)?;
                    arguments.block_argument(self, value, *position)?;
                }
                _ => arguments.push(self.evaluate_expression(arg)?),
            }
        }
        Ok(arguments.finish())
    }

    /// Run an expression alone, compiled, answering its value.
    pub(crate) fn evaluate_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<Object, MetorexError> {
        enter_nesting(expression.position())?;
        let result = self.run_expression(expression);
        leave_nesting();
        if result.is_err() {
            self.note_error_spot(&result, expression);
        }
        result
    }
}

impl VirtualMachine {
    /// Run the `at_exit` handlers, last registered first, once the program is
    /// over. Each one runs even if an earlier raised, and a handler that calls
    /// `exit` decides the status over the one the script was ending with.
    ///
    /// Answers the exit status to end with, and whether anything reported an
    /// error, given the status the program had reached and the exception that
    /// ended it, if any.
    /// Say how many frames under the top one a report writes out. The value
    /// is also what `Thread::Backtrace.limit` answers.
    pub fn set_backtrace_limit(&mut self, limit: i64) {
        self.backtrace_limit = limit;
        self.globals_mut()
            .set("__backtrace_limit__", Object::Int(limit));
    }

    pub fn run_at_exit_handlers(&mut self, status: i32, ending: Option<Object>) -> i32 {
        if let Some(exception) = ending {
            self.set_current_exception(exception);
        }
        let mut status = status;
        // The main thread's scheduler is closed as the main program ends,
        // which runs whatever it still holds.
        if let Err(error) = self.close_thread_scheduler(crate::lexer::Position::new(0, 0, 0)) {
            self.report_ending_error(error);
            status = 1;
        }
        // A program that is ending takes its threads with it, and each one
        // unwinds where it stands.
        self.end_live_threads();
        // `Signal.trap(:EXIT, ...)` names what to run as the program ends,
        // ahead of everything `at_exit` left.
        if let Some(held) = self.signal_handlers.get("EXIT").cloned()
            && !matches!(held.view(), View::String(text) if matches!(
                &*text.as_str(),
                "DEFAULT" | "SYSTEM_DEFAULT" | "IGNORE"
            ))
            && !matches!(held.view(), View::Nil)
        {
            let position = crate::lexer::Position::new(0, 0, 0);
            let _ = self.send_to_object(held, "call", Vec::new(), position);
            use std::io::Write as _;
            let _ = std::io::stdout().flush();
        }
        while let Some(handler) = self.at_exit_handlers.pop() {
            let View::Block(block) = handler.view() else {
                continue;
            };
            let position = crate::lexer::Position {
                line: 0,
                column: 0,
                offset: 0,
                prelude: false,
            };
            let outcome = self.execute_block_callable(block, Vec::new(), position);
            // What a handler printed belongs before whatever the next one
            // reports, and stdout would otherwise hold it until exit.
            use std::io::Write as _;
            let _ = std::io::stdout().flush();
            match &outcome {
                Ok(_) => {}
                Err(crate::error::MetorexError::UncaughtException { exception, .. })
                    if let View::Exception(details) = exception.view()
                        && details.borrow().is_system_exit() =>
                {
                    // `exit` inside a handler ends that handler and settles
                    // the status the program leaves with.
                    if let Some(carried) = details.borrow().status {
                        status = carried as i32;
                    }
                    self.set_current_exception(Object::Exception(std::rc::Rc::clone(details)));
                }
                Err(error) => {
                    self.report_ending_error(error.clone());
                    status = 1;
                }
            }
        }
        // What C wrapped in an object is freed once nothing else will run.
        self.free_remaining_data();
        status
    }
}

impl VirtualMachine {
    /// Report what something run as the program ends raised, the way the
    /// program reports what it ends with: where it was raised and the class
    /// it is.
    fn report_ending_error(&mut self, error: crate::error::MetorexError) {
        if let crate::error::MetorexError::UncaughtException { exception, .. } = &error {
            let held = exception.clone();
            match self.uncaught_report(&held) {
                Some(report) => eprint!("{}", report),
                None => eprintln!("{}", error),
            }
            self.set_current_exception(held);
        } else {
            eprintln!("{}", error);
        }
    }
}

/// How deeply expressions and method calls may nest, together, before one
/// raises SystemStackError rather than running out of stack.
const MOST_NESTED: usize = 1200;

thread_local! {
    /// How deeply the program running on this thread is nested. It belongs
    /// to the thread, so two virtual machines running side by side do not
    /// add their nesting together.
    static NESTING: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Go one level deeper, or raise SystemStackError when the program is as deep
/// as it may go, or as near the end of the stack it runs on. A level entered
/// is left with `leave_nesting`.
pub(crate) fn enter_nesting(position: crate::lexer::Position) -> Result<(), MetorexError> {
    let reached = NESTING.with(|depth| depth.get());
    if reached >= MOST_NESTED || near_the_end_of_the_stack() {
        return Err(crate::vm::errors::simple_exception(
            "SystemStackError",
            "stack level too deep",
            position,
        ));
    }
    NESTING.with(|depth| depth.set(reached + 1));
    Ok(())
}

/// The room a level may take on the native stack after its nesting is
/// counted, which differs with the build and with what the level runs.
const STACK_KEPT_FREE: usize = 2 * 1024 * 1024;

/// A stack this thread runs on, from its lowest address to its highest.
#[derive(Clone, Copy)]
struct StackRange {
    low: usize,
    high: usize,
}

thread_local! {
    /// The stacks this thread's program runs on: the thread's own, and the
    /// stack of each fiber made on it, the newest last. The one found last
    /// is kept, since the next check is most often on the same one.
    static STACKS: std::cell::RefCell<Vec<StackRange>> = const { std::cell::RefCell::new(Vec::new()) };
    static LAST_STACK: std::cell::Cell<Option<StackRange>> = const { std::cell::Cell::new(None) };
}

/// Note a fiber's stack, from `low` to `high`, which the fiber's body runs
/// on. A stack it takes the place of, in memory a fiber freed, is dropped.
pub(crate) fn note_fiber_stack(low: usize, high: usize) {
    with_stacks(|stacks| {
        stacks.retain(|held| held.high <= low || held.low >= high);
        stacks.push(StackRange { low, high });
    });
    LAST_STACK.with(|last| last.set(None));
}

/// Run `body` over this thread's stacks, with the thread's own noted first.
fn with_stacks<T>(body: impl FnOnce(&mut Vec<StackRange>) -> T) -> T {
    STACKS.with(|stacks| {
        let mut stacks = stacks.borrow_mut();
        if stacks.is_empty()
            && let Some(own) = thread_stack()
        {
            stacks.push(own);
        }
        body(&mut stacks)
    })
}

/// Whether what is left of the stack this runs on, below here, is less
/// than `STACK_KEPT_FREE`, or than a sixteenth of the stack when that is
/// less.
fn near_the_end_of_the_stack() -> bool {
    let marker = 0u8;
    let here = std::ptr::addr_of!(marker) as usize;
    let holds_here = |range: &StackRange| (range.low..range.high).contains(&here);
    let range = LAST_STACK
        .with(|last| last.get())
        .filter(holds_here)
        .or_else(|| {
            let found =
                with_stacks(|stacks| stacks.iter().rev().find(|range| holds_here(range)).copied());
            LAST_STACK.with(|last| last.set(found));
            found
        });
    let Some(range) = range else {
        return false;
    };
    let kept = STACK_KEPT_FREE.min((range.high - range.low) / 16);
    here - range.low < kept
}

/// This thread's own stack, as the operating system reports it.
#[cfg(target_os = "macos")]
fn thread_stack() -> Option<StackRange> {
    // SAFETY: both calls read the calling thread's attributes.
    let (high, size) = unsafe {
        let thread = libc::pthread_self();
        (
            libc::pthread_get_stackaddr_np(thread) as usize,
            libc::pthread_get_stacksize_np(thread),
        )
    };
    Some(StackRange {
        low: high.checked_sub(size)?,
        high,
    })
}

/// This thread's own stack, as the operating system reports it.
#[cfg(target_os = "linux")]
fn thread_stack() -> Option<StackRange> {
    // SAFETY: the attributes are initialized by `pthread_getattr_np` before
    // they are read, and destroyed after.
    unsafe {
        let mut attributes: libc::pthread_attr_t = std::mem::zeroed();
        if libc::pthread_getattr_np(libc::pthread_self(), &mut attributes) != 0 {
            return None;
        }
        let mut low: *mut libc::c_void = std::ptr::null_mut();
        let mut size: libc::size_t = 0;
        let read = libc::pthread_attr_getstack(&attributes, &mut low, &mut size);
        libc::pthread_attr_destroy(&mut attributes);
        (read == 0).then(|| StackRange {
            low: low as usize,
            high: low as usize + size,
        })
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn thread_stack() -> Option<StackRange> {
    None
}

/// Come back up the level `enter_nesting` went down.
pub(crate) fn leave_nesting() {
    NESTING.with(|depth| depth.set(depth.get() - 1));
}

/// A call's arguments as they are evaluated, one at a time.
#[derive(Default)]
pub(crate) struct ArgumentList {
    args: Vec<Object>,
    /// Whether the last argument was spread out of a splat, which is what
    /// lets a hash marked by `ruby2_keywords` be passed on as keywords.
    tail_from_splat: bool,
}

impl ArgumentList {
    pub(crate) fn with_capacity(capacity: usize) -> ArgumentList {
        ArgumentList {
            args: Vec::with_capacity(capacity),
            tail_from_splat: false,
        }
    }

    /// Add a plain argument.
    pub(crate) fn push(&mut self, value: Object) {
        self.tail_from_splat = false;
        self.args.push(value);
    }

    /// Add what `*value` spreads.
    pub(crate) fn splat(
        &mut self,
        vm: &mut VirtualMachine,
        value: Object,
        splat_at: Position,
    ) -> Result<(), MetorexError> {
        self.tail_from_splat = true;
        match value.view() {
            View::Array(arr) => {
                self.args.extend(arr.borrow().iter().cloned());
            }
            // `*nil` spreads into nothing, the way `[*nil]` does,
            // so a call written with one passes no argument there.
            View::Nil => {}
            _ => match vm.splat_through_to_a(value.clone(), splat_at)? {
                Ok(items) => self.args.extend(items),
                Err(held) => self.args.push(held),
            },
        }
        Ok(())
    }

    /// Add the keywords `**value` passes.
    pub(crate) fn keyword_splat(
        &mut self,
        vm: &mut VirtualMachine,
        value: Object,
        splat_position: Position,
    ) -> Result<(), MetorexError> {
        self.tail_from_splat = false;
        let mut value = value;
        // Anything else is asked for a Hash of its own, which is
        // how an object stands in for keyword arguments.
        if !matches!(value.view(), View::Dict(_)) && vm.lookup_method(&value, "to_hash").is_some() {
            value = vm.send_to_object(value, "to_hash", vec![], splat_position)?;
        }
        match value.view() {
            // `**nil` passes no keywords, the same as `**{}`.
            View::Nil => {}
            View::Dict(entries) if entries.borrow().is_empty() => {}
            View::Dict(entries) => {
                // Keywords spread after other keywords join them,
                // a later key taking the place of an earlier one.
                if let Some(View::Dict(gathered)) = self.args.last().view()
                    && gathered.borrow().contains_key("__MX_KWARGS__")
                {
                    let mut joined = gathered.borrow().clone();
                    for (key, value) in entries.borrow().iter() {
                        joined.insert(key.clone(), value.clone());
                    }
                    self.args.pop();
                    self.args
                        .push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                            joined,
                        ))));
                    return Ok(());
                }
                let mut keywords = entries.borrow().clone();
                keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
                self.args
                    .push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                        keywords,
                    ))));
            }
            _ => self.args.push(value),
        }
        Ok(())
    }

    /// Take `&value` as the call's block.
    pub(crate) fn block_argument(
        &mut self,
        vm: &mut VirtualMachine,
        value: Object,
        other_position: Position,
    ) -> Result<(), MetorexError> {
        // A refinement in force may say how an object becomes a
        // block, which stands ahead of whatever its own kind does.
        let refined_to_proc = vm.any_refinement_active()
            && crate::vm::method_lookup::refinement_target_name(&value, vm)
                .and_then(|target| vm.find_refined_method(&target, "to_proc"))
                .is_some();
        match value.view() {
            View::Nil => {}
            View::Block(_) => {
                vm.pending_block = Some(value);
                vm.pending_block_from_ampersand = true;
            }
            _ if refined_to_proc => {
                let made =
                    vm.send_to_object(value.clone(), "to_proc", Vec::new(), other_position)?;
                vm.pending_block = Some(made);
                vm.pending_block_from_ampersand = true;
                vm.pending_block_source = Some(value.clone());
            }
            View::Symbol(sym) => {
                // `&:method` is symbol-to-proc: synthesize a block
                // `|x| x.send(:method)`. The block has no captured
                // vars and a one-statement body that calls .send
                // on the parameter.
                vm.pending_block = Some(Object::Block(std::rc::Rc::new(symbol_to_proc_block(
                    &sym.as_str(),
                ))));
                vm.pending_block_from_ampersand = true;
            }
            // `&some_method` hands the method over as the block,
            // which is what `to_proc` on a Method answers.
            View::Method(_) => {
                vm.pending_block = Some(block_for_method(value.clone(), other_position));
                vm.pending_block_from_ampersand = true;
            }
            // An object of the program's own becomes a block
            // through `to_proc`, which is how a Yielder reaches
            // a method that takes one.
            View::Instance(_)
                if vm
                    .send_to_object(
                        value.clone(),
                        "respond_to?",
                        vec![Object::symbol("to_proc".to_string())],
                        other_position,
                    )?
                    .is_truthy() =>
            {
                let made =
                    vm.send_to_object(value.clone(), "to_proc", Vec::new(), other_position)?;
                match made.view() {
                    View::Block(_) => {
                        vm.pending_block = Some(made);
                        vm.pending_block_from_ampersand = true;
                        vm.pending_block_source = Some(value.clone());
                    }
                    // A `to_proc` that answers a Method stands
                    // for a block the same way `&method` does.
                    View::Method(_) => {
                        vm.pending_block = Some(block_for_method(made.clone(), other_position));
                        vm.pending_block_from_ampersand = true;
                    }
                    // Anything else is no block at all, which
                    // Ruby names both sides of.
                    _ => {
                        let named = vm.builtins().class_of(&value).ruby_name();
                        let gives = vm.builtins().class_of(&made).ruby_name();
                        let message = format!(
                            "can't convert {} into Proc ({}#to_proc gives {})",
                            named, named, gives
                        );
                        return Err(crate::vm::errors::simple_exception(
                            "TypeError",
                            &message,
                            other_position,
                        ));
                    }
                }
            }
            // An object with no `to_proc` of its own is no block,
            // and Ruby refuses it rather than counting it among
            // the arguments.
            View::Instance(_) => {
                let named = vm.builtins().class_of(&value).ruby_name();
                let message = format!("no implicit conversion of {} into Proc", named);
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    &message,
                    other_position,
                ));
            }
            _ => {
                // Non-block, non-nil &arg: push as positional so
                // the existing trailing-block-extraction in
                // `invoke_method` can pick it up if it happens to
                // be a Method/Proc that the user wants to coerce.
                self.tail_from_splat = false;
                self.args.push(value.clone());
            }
        }
        Ok(())
    }

    /// The arguments to call with.
    pub(crate) fn finish(mut self) -> Vec<Object> {
        // A hash marked by `ruby2_keywords` and spread out of a splat goes on
        // as the keyword arguments it was gathered from.
        if self.tail_from_splat
            && let Some(View::Dict(entries)) = self.args.last().view()
            && entries
                .borrow()
                .contains_key(crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY)
        {
            let mut keywords: indexmap::IndexMap<String, Object> = entries
                .borrow()
                .iter()
                .filter(|(key, _)| {
                    key.as_str() != crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                })
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            keywords.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
            self.args.pop();
            self.args
                .push(Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                    keywords,
                ))));
        }
        self.args
    }
}
