// Call expression evaluation: bare-id self-method dispatch and callable invocation.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::object::{OptionView, View};

use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// Whether `self` answers a method of this name that the program wrote,
    /// which stands in place of the Kernel function of the same name.
    fn self_defines_method(&mut self, name: &str) -> bool {
        let Ok(current_self) = self.eval_self(crate::lexer::Position::default()) else {
            return false;
        };
        self.lookup_method(&current_self, name)
            .is_some_and(|(_, method)| !method.is_undefined)
    }
}

/// How a call written without a receiver, `name(arguments)`, is made, which
/// is decided before its arguments are evaluated.
pub(crate) enum CallRoute {
    /// A method call on `self`.
    OnSelf,
    /// A method call on a receiver evaluated already, written as `written`.
    OnReceiver {
        receiver: Object,
        written: crate::vm::method_lookup::WrittenReceiver,
    },
    /// A Kernel function. `catches_break` is set where a `break` in the
    /// call's block returns from the call.
    Native { native: String, catches_break: bool },
    /// `Hash(x)` and the other Kernel conversions, which pass a call they do
    /// not convert on to the routes after them.
    KernelConversion,
    /// A value the name or the callee evaluated to, called with the
    /// arguments.
    Callable(Object),
    /// Evaluating the callee raised this, which the call raises once its
    /// arguments are evaluated.
    Failed(MetorexError),
}

impl VirtualMachine {
    /// How `name(arguments)` is made, from what the scope binds `name` to
    /// and what `self` answers.
    pub(crate) fn resolve_bare_call(
        &mut self,
        name: &str,
        has_arguments: bool,
        has_block: bool,
        has_keyword_splat: bool,
        position: Position,
    ) -> Result<CallRoute, MetorexError> {
        // A call written with parentheses names a method, so a local of the
        // same name that holds no callable does not stand in the way of
        // `foobar()` reaching `foobar`, or raising NoMethodError without one.
        if let Some(held) = self.environment().get(name)
            && !matches!(
                held.view(),
                View::Method(_) | View::Block(_) | View::NativeFunction(_)
            )
        {
            let receiver_defines = self.eval_self(position).ok().is_some_and(|receiver| {
                self.lookup_method(&receiver, name)
                    .is_some_and(|(_, found)| !found.is_undefined)
            });
            if !receiver_defines
                && let Some(View::NativeFunction(native)) = self.globals().get(name).view()
            {
                return Ok(CallRoute::Native {
                    native: native.to_string(),
                    catches_break: true,
                });
            }
            return Ok(CallRoute::OnSelf);
        }

        // If the name is not a local variable, the call is a method call
        // with the supplied arguments. Self-method dispatch also wins when
        // the binding is a global NativeFunction (e.g. `define_method`) but
        // the receiver has a method by that name, which lets `class_eval {
        // define_method ... }` hit the class's own define_method.
        if self.environment().get("self").is_some() {
            let env_val = self.environment().get(name);
            let dispatch_to_self = match env_val.view() {
                None => true,
                // `define_method`, `private`, `public` live in globals as
                // top-level conveniences, but inside a class/module body or
                // class_eval they should resolve to the receiver's method so
                // `private :foo` marks Tally#foo (not Object#foo) as private.
                Some(View::NativeFunction(fn_name))
                    if matches!(
                        fn_name.as_str(),
                        "define_method" | "private" | "public" | "protected" | "module_function"
                    ) =>
                {
                    matches!(
                        self.environment().get("self").view(),
                        Some(View::Class(_)) | Some(View::Module(_))
                    )
                }
                // A `def` registers its name in the environment as a Method
                // so the function is reachable. A call form written with
                // arguments names that method, so it dispatches rather than
                // calling whatever the bare name would answer.
                Some(held) if self.name_is_a_definition(name, &held.to_object()) => true,
                _ => false,
            };
            if dispatch_to_self {
                return Ok(CallRoute::OnSelf);
            }
        }

        // `using` is a special native whose identifier resolution auto-invokes
        // with zero args; when args are present we must bypass that and call
        // directly with the evaluated args.
        if name == "using"
            && let Some(View::NativeFunction(fn_name)) = self.environment().get("using").view()
            && fn_name == "using"
        {
            return Ok(CallRoute::Native {
                native: "using".to_string(),
                catches_break: false,
            });
        }

        // A program that writes its own `p` or `puts` means that one, whichever
        // module it opened to define it, so a bare call reaches the method
        // rather than the Kernel function of the same name.
        if self.named_native_function(name).is_some() && self.self_defines_method(name) {
            return Ok(CallRoute::OnSelf);
        }

        // `__method__()`, `__callee__()`, `abort`, `exit` and `binding` are
        // auto-invoked when named bare, so a call form has to reach the native
        // function rather than calling whatever the bare name evaluated to.
        if matches!(
            name,
            "__method__"
                | "__callee__"
                | "abort"
                | "at_exit"
                | "caller"
                | "caller_locations"
                | "chomp"
                | "chop"
                | "fork"
                | "loop"
                | "open"
                | "exit"
                | "exit!"
                | "fail"
                | "gets"
                | "global_variables"
                | "local_variables"
                | "p"
                | "pp"
                | "proc"
                | "print"
                | "puts"
                | "rand"
                | "readline"
                | "readlines"
                | "srand"
                | "throw"
                | "binding"
        ) && let Some(View::NativeFunction(native_name)) =
            self.named_native_function(name).view()
        {
            return Ok(CallRoute::Native {
                native: native_name.to_string(),
                catches_break: false,
            });
        }

        // A Kernel function named by a call form is reached even where a
        // local of its name holds something that cannot be called.
        if matches!(
            self.environment().get(name),
            Some(held)
                if !matches!(
                    held.view(),
                    View::Block(_)
                        | View::Method(_)
                        | View::NativeFunction(_)
                        | View::Class(_)
                        | View::Module(_)
                )
        ) && !self.self_defines_method(name)
            && let Some(View::NativeFunction(native_name)) = self.named_native_function(name).view()
        {
            return Ok(CallRoute::Native {
                native: native_name.to_string(),
                catches_break: false,
            });
        }

        // A call form written with arguments or a block names a method. A
        // local of that name holding something that cannot be called is not
        // what the call meant: Ruby reads `name(...)` as a method call
        // whatever locals are in scope, which is what lets an example write
        // `mock = mock("held")`.
        if (has_arguments || has_block)
            && !has_keyword_splat
            && matches!(
                self.environment().get(name),
                Some(held)
                    if !matches!(
                        held.view(),
                        View::Block(_)
                            | View::Method(_)
                            | View::NativeFunction(_)
                            // A name holding a class or a module is the name
                            // of that class, and a call form on it builds one.
                            | View::Class(_)
                            | View::Module(_)
                    )
            )
        {
            return Ok(CallRoute::OnSelf);
        }

        // A `def` registers its name in the environment as a Method so the
        // function is reachable. A call form naming it dispatches as a method
        // call, whatever the bare name on its own would answer. At the top
        // level nothing binds `self`, so the case is taken here.
        if self.environment().get("self").is_none()
            // A `**held` argument is passed as keywords by the callable
            // itself, so a call carrying one keeps the route that knows that.
            && !has_keyword_splat
            && match self.environment().get(name) {
                None => true,
                Some(held) => self.name_is_a_definition(name, &held),
            }
        {
            return Ok(CallRoute::OnSelf);
        }

        // `Hash(x)` and the other Kernel conversion functions share a name
        // with a constant, so they are resolved here rather than by letting
        // the identifier fall through to the class it collides with.
        if crate::vm::native_methods::kernel_conversion::is_kernel_conversion(name) {
            return Ok(CallRoute::KernelConversion);
        }
        self.resolve_after_conversion(name, position)
    }

    /// The routes a bare call takes when it is no Kernel conversion.
    fn resolve_after_conversion(
        &mut self,
        name: &str,
        position: Position,
    ) -> Result<CallRoute, MetorexError> {
        // A method defined on `self` beats a same-named Kernel function.
        // `lambda` is the case the specs exercise: a class that defines its
        // own `lambda` must have it reached by a receiverless call too.
        if matches!(
            self.environment().get(name).view(),
            Some(View::NativeFunction(_))
        ) && let Some(current_self) = self.environment().get("self")
            && let Some((_, method)) = self.lookup_method(&current_self, name)
            && !method.is_undefined
        {
            return Ok(CallRoute::OnSelf);
        }

        // A Kernel function that runs when its bare name is evaluated has to
        // be reached here with the arguments the call carries, since
        // evaluating the name on its own would run it with none.
        if let Some(View::NativeFunction(native)) = self.named_native_function(name).view()
            && (crate::vm::eval::identifier::runs_when_named_bare(native)
                || matches!(
                    native.as_str(),
                    "private" | "public" | "protected" | "module_function"
                ))
        {
            return Ok(CallRoute::Native {
                native: native.to_string(),
                catches_break: true,
            });
        }

        // A bare identifier naming a zero-argument `def` is a call, so
        // evaluating it as the callee would run the method and then try to
        // call its result. Invoke the definition with this call's arguments
        // instead.
        if let Some(value) = self.environment().get(name)
            && self.name_is_a_definition(name, &value)
        {
            return Ok(CallRoute::Callable(value));
        }

        let callable = self.evaluate_expression(&Expression::Identifier {
            name: name.to_string(),
            position,
        });
        // A method may share its name with a constant, the way `URI(text)`
        // shares one with the URI module. A call form reaches the method
        // rather than trying to call what the constant holds.
        if let Ok(View::Class(_) | View::Module(_)) = callable.as_ref().map(Object::view) {
            let current = self.eval_self(position)?;
            if matches!(
                self.lookup_method(&current, name),
                Some((_, found)) if !found.is_undefined
            ) {
                return Ok(CallRoute::OnSelf);
            }
        }
        Ok(match callable {
            Ok(value) => CallRoute::Callable(value),
            Err(_) if self.environment().get("self").is_some() => CallRoute::OnSelf,
            Err(error) => CallRoute::Failed(error),
        })
    }

    /// Make a call `resolve_bare_call` routed, with its arguments evaluated
    /// and its block, if one was written, evaluated.
    pub(crate) fn run_call_route(
        &mut self,
        route: CallRoute,
        name: &str,
        arguments: Vec<Object>,
        block: Option<Object>,
        position: Position,
        cache: Option<&std::rc::Rc<crate::bytecode::call_cache::CallCache>>,
    ) -> Result<Object, MetorexError> {
        let has_block = block.is_some();
        match route {
            CallRoute::OnSelf => {
                let receiver = self.eval_self(position)?;
                let written = crate::vm::method_lookup::WrittenReceiver::SelfKeyword;
                self.call_with_block_through(
                    receiver, name, arguments, block, written, position, cache,
                )
            }
            CallRoute::OnReceiver { receiver, written } => self.call_with_block_through(
                receiver, name, arguments, block, written, position, cache,
            ),
            CallRoute::Native {
                native,
                catches_break,
            } => {
                self.attach_block(block);
                match self.call_native_function(&native, arguments, position) {
                    Err(MetorexError::BlockBreak { value, .. }) if catches_break && has_block => {
                        Ok(value)
                    }
                    other => other,
                }
            }
            CallRoute::KernelConversion => {
                if let Some(result) =
                    self.call_bare_kernel_conversion(name, &arguments, position)?
                {
                    return Ok(result);
                }
                let route = self.resolve_after_conversion(name, position)?;
                self.run_call_route(route, name, arguments, block, position, cache)
            }
            CallRoute::Callable(value) => {
                self.attach_block(block);
                match self.invoke_callable(value, arguments, position) {
                    // `break <value>` in the attached block unwinds here.
                    Err(MetorexError::BlockBreak { value, .. }) if has_block => Ok(value),
                    other => other,
                }
            }
            CallRoute::Failed(error) => Err(error),
        }
    }

    /// Call `name` on a receiver evaluated already, as a method call
    /// written with a receiver is made: a `break` in the block attached here
    /// returns from this call when the block was written in this frame.
    pub(crate) fn call_with_block(
        &mut self,
        receiver: Object,
        name: &str,
        arguments: Vec<Object>,
        block: Option<Object>,
        written: crate::vm::method_lookup::WrittenReceiver,
        position: Position,
    ) -> Result<Object, MetorexError> {
        self.call_with_block_through(receiver, name, arguments, block, written, position, None)
    }

    /// `call_with_block`, answering from the call site's `cache` when one is
    /// given.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn call_with_block_through(
        &mut self,
        receiver: Object,
        name: &str,
        arguments: Vec<Object>,
        block: Option<Object>,
        written: crate::vm::method_lookup::WrittenReceiver,
        position: Position,
        cache: Option<&std::rc::Rc<crate::bytecode::call_cache::CallCache>>,
    ) -> Result<Object, MetorexError> {
        let has_block = block.is_some();
        let calling_frame = self.current_method_frame;
        self.refuse_moved_receiver(&receiver, position)?;
        self.attach_block(block);
        let reached = match cache {
            Some(cache) => {
                self.call_through_cache(receiver, name, arguments, written, position, cache)
            }
            None => self.call_evaluated_method(receiver, name, arguments, written, position),
        };
        match reached {
            Err(MetorexError::BlockBreak {
                value, home_frame, ..
            }) if has_block && home_frame == calling_frame => Ok(value),
            other => other,
        }
    }

    /// Make `block`, a block written on a call, the block the call passes.
    pub(crate) fn attach_block(&mut self, block: Option<Object>) {
        let Some(block) = block else {
            return;
        };
        if let View::Block(held) = block.view() {
            held.attached_call_running.set(true);
            self.attached_block_flags
                .push(std::rc::Rc::clone(&held.attached_call_running));
        }
        self.pending_block = Some(block);
        self.pending_block_from_ampersand = false;
    }
}

impl VirtualMachine {
    /// The Kernel function a bare name stands for. A fiber runs in a scope of
    /// its own rather than in the one the program started from, and a local
    /// of the same name hides the function only when it is not called, so
    /// the name is looked for among the globals when the scope holds
    /// anything else.
    fn named_native_function(&self, name: &str) -> Option<Object> {
        match self.environment().get(name) {
            Some(held) if matches!(held.view(), View::NativeFunction(_)) => Some(held),
            _ => match self.globals().get(name) {
                Some(held) if matches!(held.view(), View::NativeFunction(_)) => Some(held),
                _ => None,
            },
        }
    }
}
