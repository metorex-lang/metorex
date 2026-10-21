//! Method lookup and dispatch for the virtual machine.
//!
//! This module handles resolving method calls on receiver objects and dispatching
//! to the appropriate method implementation.

use super::VirtualMachine;
use crate::object::{OptionView, View};

use crate::ast::Expression;
use crate::bytecode::call_cache::CallCache;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    /// Whether deciding a call's visibility asked where the call was made
    /// from, which a protected method does. Such a decision holds for that
    /// caller alone, so the call site does not keep it.
    static CALLER_CONSULTED: Cell<bool> = const { Cell::new(false) };
}

/// Build the refinement lookup key for a receiver. For user-defined class
/// instances we key by pointer (to avoid anonymous-class name collisions);
/// for builtin types we use the well-known class name alone.
pub(crate) fn refinement_target_name(
    receiver: &Object,
    vm: &crate::vm::VirtualMachine,
) -> Option<String> {
    match receiver.view() {
        View::String(_) => Some(builtin_key(vm, "String")),
        View::Int(_) => Some(builtin_key(vm, "Integer")),
        View::Float(_) => Some(builtin_key(vm, "Float")),
        View::Array(_) => Some(builtin_key(vm, "Array")),
        View::Dict(_) => Some(builtin_key(vm, "Hash")),
        View::Symbol(_) => Some(builtin_key(vm, "Symbol")),
        View::Instance(inst) => {
            let cls = &inst.borrow().class;
            Some(format!("__refine__{}@{:p}", cls.name(), Rc::as_ptr(cls)))
        }
        // A class or module is itself an object, and `refine Foo.singleton_class`
        // is how its own methods are refined. The singleton class is where
        // those land, so it is what the key names.
        View::Class(held) | View::Module(held) => {
            let singleton = held.singleton_class_slot().clone()?;
            Some(format!(
                "__refine__{}@{:p}",
                singleton.name(),
                Rc::as_ptr(&singleton)
            ))
        }
        _ => None,
    }
}

/// The refinement that covers this object for this name, if one is in force.
/// A refinement on a class reaches its subclasses, and one on a module reaches
/// what takes that module in, so the walk goes up the chain. It stops where a
/// class answers the name itself, since what a descendant writes stands ahead
/// of what an ancestor's refinement says.
pub(crate) fn find_refinement(
    receiver: &Object,
    name: &str,
    vm: &crate::vm::VirtualMachine,
) -> Option<Rc<crate::object::Method>> {
    if !vm.any_refinement_active() || singleton_answers(receiver, name) {
        return None;
    }
    if let Some(key) = refinement_target_name(receiver, vm)
        && let Some(found) = vm.find_refined_method(&key, name)
    {
        return Some(found);
    }
    let mut cursor = match receiver.view() {
        View::Instance(inst) => Some(Rc::clone(&inst.borrow().class)),
        _ => Some(vm.builtins().class_of(receiver)),
    };
    // The core classes reach Object through a record of their own, so a
    // refinement of the Object the program names is looked for under that
    // record as well.
    let program_object = match vm.globals().get("Object").view() {
        Some(View::Class(object)) => Some(Rc::clone(object)),
        _ => None,
    };
    while let Some(class) = cursor {
        let key = format!("__refine__{}@{:p}", class.name(), Rc::as_ptr(&class));
        if let Some(found) = vm.find_refined_method(&key, name) {
            return Some(found);
        }
        if class.name() == "Object"
            && let Some(object) = &program_object
            && !Rc::ptr_eq(object, &class)
            && let Some(found) =
                vm.find_refined_method(&format!("__refine__Object@{:p}", Rc::as_ptr(object)), name)
        {
            return Some(found);
        }
        for mixin in class.transitive_mixins() {
            let key = format!("__refine__{}@{:p}", mixin.name(), Rc::as_ptr(&mixin));
            if let Some(found) = vm.find_refined_method(&key, name) {
                return Some(found);
            }
        }
        // A class that writes the name itself settles it: the ordinary walk
        // reaches that one before it reaches any ancestor's refinement.
        if class.find_own_method(name).is_some() {
            return None;
        }
        cursor = class.superclass();
    }
    None
}

/// Whether the object itself answers to the name, put there by `class << obj`
/// or `define_singleton_method`. Ruby looks there before it looks at any
/// refinement in force.
pub(crate) fn singleton_answers(receiver: &Object, name: &str) -> bool {
    let View::Instance(inst) = receiver.view() else {
        return false;
    };
    let held = inst.borrow().singleton_class.borrow().clone();
    held.is_some_and(|singleton| {
        singleton
            .find_own_method(name)
            .is_some_and(|method| !method.is_undefined)
    })
}

fn builtin_key(vm: &crate::vm::VirtualMachine, name: &str) -> String {
    if let Some(View::Class(c)) = vm.globals().get(name).view() {
        format!("__refine__{}@{:p}", name, Rc::as_ptr(c))
    } else {
        format!("__refine__{}", name)
    }
}

impl VirtualMachine {
    /// Evaluate a method call expression on a receiver object.
    pub(crate) fn evaluate_method_call(
        &mut self,
        receiver_expr: &Expression,
        method_name: &str,
        argument_exprs: &[Expression],
        trailing_block: Option<&Expression>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let has_block = trailing_block.is_some();
        let calling_frame = self.current_method_frame;
        let result = self.evaluate_method_call_inner(
            receiver_expr,
            method_name,
            argument_exprs,
            trailing_block,
            position,
        );
        // Ruby: `break <value>` inside the block passed to this call unwinds
        // to *this* method call and makes the call return `value`. Only catch
        // when a block was attached here and the break came from a block
        // written in this frame, so a nested call does not absorb a break
        // meant for an outer one. Top-level blocks carry no frame, and match
        // only a call made at top level.
        match result {
            Err(MetorexError::BlockBreak {
                value, home_frame, ..
            }) if has_block && home_frame == calling_frame => Ok(value),
            other => other,
        }
    }

    fn evaluate_method_call_inner(
        &mut self,
        receiver_expr: &Expression,
        method_name: &str,
        argument_exprs: &[Expression],
        trailing_block: Option<&Expression>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // `native_fn[args]` — a call with the bracketed args wrapped as an
        // Array, matching Ruby's `private [:foo, :bar]` where `[...]` is the
        // sole argument. Natives that auto-invoke when their name is evaluated
        // (`private` inside a class body) make this check precede evaluating
        // the receiver expression.
        if method_name == "[]"
            && let Expression::Identifier { name, .. } = receiver_expr
            && let Some(View::NativeFunction(native_name)) = self.environment().get(name).view()
        {
            let arguments = self.evaluate_arguments(argument_exprs)?;
            let array_arg = Object::Array(Rc::new(RefCell::new(arguments)));
            return self.call_native_function(native_name, vec![array_arg], position);
        }

        let receiver = self.evaluate_expression(receiver_expr)?;
        self.refuse_moved_receiver(&receiver, position)?;
        // `held&.name(args) { block }` runs none of what follows when there
        // is no receiver, so the arguments are never evaluated.
        if method_name == crate::parser::SAFE_CALL {
            let nil_receiver = matches!(receiver.view(), View::Nil);
            self.coverage_branch(position, usize::from(nil_receiver));
            if nil_receiver {
                return Ok(Object::Nil);
            }
        }
        let arguments = self.evaluate_arguments(argument_exprs)?;

        // If there's a trailing block, evaluate it and store as pending_block.
        // Native methods (each, map, etc.) will take it from self.pending_block.
        if let Some(block_expr) = trailing_block {
            self.pending_block = Some(self.attach_trailing_block(block_expr)?);
            self.pending_block_from_ampersand = false;
        }

        let written = WrittenReceiver::of(receiver_expr);
        self.call_evaluated_method(receiver, method_name, arguments, written, position)
    }

    /// Call `method_name` on a receiver and arguments already evaluated.
    /// `written` says whether the receiver was written as `self`, which is
    /// what lets a private method be called.
    pub(crate) fn call_through_cache(
        &mut self,
        receiver: Object,
        method_name: &str,
        arguments: Vec<Object>,
        written: WrittenReceiver,
        position: Position,
        cache: &Rc<CallCache>,
    ) -> Result<Object, MetorexError> {
        if let Some(class) = self.class_deciding_dispatch(&receiver) {
            if let Some(method) = cache.find(&class) {
                return self.invoke_method(class, method, receiver, arguments, position);
            }
            self.recording_call_cache = Some(Rc::clone(cache));
        }
        self.call_evaluated_method(receiver, method_name, arguments, written, position)
    }

    /// The class of an object whose calls its class alone decides: an
    /// instance with no methods of its own, while no refinement is active.
    pub(crate) fn class_deciding_dispatch(&self, receiver: &Object) -> Option<Rc<Class>> {
        let View::Instance(instance) = receiver.view() else {
            return None;
        };
        if self.any_refinement_active() {
            return None;
        }
        let instance = instance.borrow();
        if !instance.singleton_methods.borrow().is_empty()
            || instance.singleton_class.borrow().is_some()
        {
            return None;
        }
        Some(Rc::clone(&instance.class))
    }

    pub(crate) fn call_evaluated_method(
        &mut self,
        receiver: Object,
        method_name: &str,
        arguments: Vec<Object>,
        written: WrittenReceiver,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let mut recording = self.recording_call_cache.take();
        // Refinement dispatch: if an active refinement covers this receiver's
        // class and defines this method, use it.
        if let Some(method) = find_refinement(&receiver, method_name, self) {
            let class = self.builtins().class_of(&receiver);
            return self.invoke_method(class, method, receiver, arguments, position);
        }

        if let Some(result) = self.call_undefined_name(&receiver, method_name, &arguments, position)
        {
            return result;
        }

        // For Class/Module receivers, a module-level copy of a method wins
        // over the instance method of the same name: `module_function :foo`
        // and `extend` both leave the instance method private, and `Mod.foo`
        // must reach the copy rather than trip visibility enforcement.
        if let View::Class(class_rc) | View::Module(class_rc) = receiver.view() {
            let class_rc = Rc::clone(class_rc);
            // A method put on the class's own singleton class stands ahead of
            // one written as `def self.name`, which is how a later
            // `define_singleton_method` replaces it, and a module prepended
            // to the singleton class stands ahead of both.
            let own = singleton_prepended_method(&class_rc, method_name)
                .map(|(_, method)| method)
                .or_else(|| {
                    class_rc
                        .singleton_class_slot()
                        .as_ref()
                        .and_then(|singleton| singleton.find_own_method(method_name))
                })
                .or_else(|| class_level_method(&class_rc, method_name).map(|(_, method)| method));
            if let Some(method) = own {
                let is_explicit_receiver = !written.names_self();
                if is_explicit_receiver && self.method_is_restricted(&receiver, method_name) {
                    if let Some(handled) = self.restricted_call_via_method_missing(
                        &receiver,
                        method_name,
                        &arguments,
                        position,
                    ) {
                        return handled;
                    }
                    let msg = format!(
                        "{} method '{}' called for {} {}",
                        self.visibility_word(&receiver, method_name),
                        method_name,
                        class_rc.kind_name().to_lowercase(),
                        class_rc.inspect_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: crate::vm::errors::no_method_error(
                            &msg,
                            method_name,
                            &receiver,
                            &arguments,
                        ),
                        location: crate::vm::utils::position_to_location(position),
                        message: msg,
                    });
                }
                return self.invoke_method(class_rc, method, receiver.clone(), arguments, position);
            }
        }

        // Enumerable is written over `each`, so its methods stand in only
        // where the receiver's class has none of its own. A native method
        // counts as the class's own, which is what puts Struct#to_h ahead of
        // Enumerable#to_h the way Ruby's ancestor order does.
        if self.enumerable_stands_in(&receiver, method_name) {
            recording = None;
            let class = self.builtins().class_of(&receiver);
            if let Some(result) =
                self.call_native_method(&class, &receiver, method_name, &arguments, position)?
            {
                return Ok(result);
            }
        }

        // A class's own methods come ahead of the instance methods it has as
        // an object, so a native `Dir.mkdir` answers ahead of a top-level
        // `def mkdir`, which is a private method of Object.
        if self.object_method_behind_native(&receiver, method_name) {
            let class = self.builtins().class_of(&receiver);
            if let Some(result) =
                self.call_native_method(&class, &receiver, method_name, &arguments, position)?
            {
                return Ok(result);
            }
        }

        // Try user-defined method lookup first
        match self.lookup_method(&receiver, method_name) {
            Some((class, method)) if !method.is_undefined => {
                // Visibility check: an *explicit-receiver* call (e.g.
                // `obj.foo`, anything other than `self.foo` or a bare ident)
                // can only invoke public methods. Private/protected methods
                // raise NoMethodError when called externally. We treat
                // `Expression::SelfExpr` as implicit self even though it's
                // syntactically present, matching Ruby — `self.foo` is
                // allowed regardless of visibility.
                let is_explicit_receiver = !written.names_self();
                CALLER_CONSULTED.set(false);
                let mut is_private = self.method_is_restricted(&receiver, method_name);
                // A prepended module sits ahead of the class, so when one of
                // them supplies the method its visibility is the one in force
                // and the class's own marking does not apply.
                let prepended_owner = class
                    .transitive_prepends()
                    .into_iter()
                    .find(|prepended| prepended.find_own_method(method_name).is_some());
                if class.has_public_override(method_name) {
                    is_private = false;
                } else if let Some(owner) = prepended_owner {
                    is_private = self.refuses_explicit_receiver(&owner, method_name);
                } else if self.refuses_explicit_receiver(&class, method_name) {
                    is_private = true;
                } else if class.find_own_method(method_name).is_some() {
                    // The class that supplies the method decides how it may
                    // be called. A marking further up the chain belongs to
                    // another definition of the same name.
                    is_private = false;
                } else if !is_private {
                    let mut current = class.superclass();
                    while let Some(sc) = current {
                        if sc.has_public_override(method_name) {
                            is_private = false;
                            break;
                        }
                        if sc.find_method(method_name).is_some() {
                            is_private = self.refuses_explicit_receiver(&sc, method_name);
                            break;
                        }
                        current = sc.superclass();
                    }
                }
                if is_explicit_receiver && is_private {
                    if let Some(handled) = self.restricted_call_via_method_missing(
                        &receiver,
                        method_name,
                        &arguments,
                        position,
                    ) {
                        return handled;
                    }
                    let named = match receiver.view() {
                        View::Nil => "nil".to_string(),
                        View::Bool(held) => held.to_string(),
                        _ if self.is_the_main_object(&receiver) => "main".to_string(),
                        _ => format!("an instance of {}", class.inspect_name()),
                    };
                    let msg = format!(
                        "{} method '{}' called for {}",
                        self.visibility_word(&receiver, method_name),
                        method_name,
                        named
                    );
                    let exc = crate::vm::errors::no_method_error(
                        &msg,
                        method_name,
                        &receiver,
                        &arguments,
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: crate::vm::utils::position_to_location(position),
                        message: msg,
                    });
                }
                if let Some(cache) = recording
                    && !CALLER_CONSULTED.get()
                {
                    cache.store(&class, &method);
                }
                return self.invoke_method(class, method, receiver, arguments, position);
            }
            _ => {}
        }

        // A natively-implemented class method carries visibility too, so
        // `private_class_method :new` refuses an explicit-receiver call the
        // way a table-backed one does.
        if let View::Class(class_rc) | View::Module(class_rc) = receiver.view()
            && written != WrittenReceiver::SelfKeyword
            && self.method_is_restricted(&receiver, method_name)
        {
            let class_rc = Rc::clone(class_rc);
            if let Some(handled) = self.restricted_call_via_method_missing(
                &receiver,
                method_name,
                &arguments,
                position,
            ) {
                return handled;
            }
            let msg = format!(
                "private method '{}' called for {} {}",
                method_name,
                class_rc.kind_name().to_lowercase(),
                class_rc.inspect_name()
            );
            return Err(MetorexError::UncaughtException {
                exception: crate::vm::errors::no_method_error(
                    &msg,
                    method_name,
                    &receiver,
                    &arguments,
                ),
                location: crate::vm::utils::position_to_location(position),
                message: msg,
            });
        }

        // A private method of every object refuses a call written with a
        // receiver other than `self`.
        if !written.names_self()
            && let Some(answer) =
                self.refuse_private_object_method(&receiver, method_name, &arguments, position)
        {
            return answer;
        }
        // A method the object answers natively, which its class or singleton
        // class made private or protected, refuses a call written with a
        // receiver.
        if !written.names_self()
            && let Some(refused) =
                self.refuse_marked_native_method(&receiver, method_name, &arguments, position)
        {
            return refused;
        }
        // Try native method as fallback
        let class = self.builtins().class_of(&receiver);
        let native_result =
            self.call_native_method(&class, &receiver, method_name, &arguments, position)?;

        if let Some(result) = native_result {
            return Ok(result);
        }

        // For user-defined class instances, fall back to base Object methods
        let object_result =
            self.call_object_method(&receiver, method_name, &arguments, position)?;

        if let Some(result) = object_result {
            return Ok(result);
        }

        // Fallback: methods defined on the global Object class (mspec injects
        // describe/it/before/after there).
        if let Some((object_class, method)) = self.object_table_method(method_name) {
            // A private one, as every top-level method is, refuses a call
            // written with a receiver.
            if !written.names_self() && object_class.is_method_private(method_name) {
                return Err(self.private_object_method_error(
                    &receiver,
                    method_name,
                    &arguments,
                    position,
                ));
            }
            return self.invoke_method(object_class, method, receiver, arguments, position);
        }

        // Kernel carries its functions as module functions as well as
        // private instance methods, so `Kernel.rand` reaches the same
        // implementation a bare `rand` does.
        if let View::Module(module_rc) = receiver.view()
            && module_rc.ruby_name_is("Kernel")
            && crate::vm::native_methods::is_kernel_private_function(method_name)
        {
            let held = self
                .kernel_function_receiver
                .replace(Object::Module(Rc::clone(module_rc)));
            let answered = self.call_native_function(method_name, arguments, position);
            self.kernel_function_receiver = held;
            return answered;
        }

        // Try method_missing as a final fallback
        if let Some((method_missing_class, method_missing_method)) =
            self.lookup_method(&receiver, "method_missing")
        {
            // Ruby hands `method_missing` the name as a Symbol followed by
            // the call's own arguments, one by one, so a handler written as
            // `def method_missing(name, path)` reads the path as itself.
            let mut method_missing_args = Vec::with_capacity(arguments.len() + 1);
            method_missing_args.push(Object::symbol(method_name.to_string()));
            method_missing_args.extend(arguments.iter().cloned());
            self.invoke_method(
                method_missing_class,
                method_missing_method,
                receiver,
                method_missing_args,
                position,
            )
        } else {
            let wording = self.receiver_wording_for(&receiver, position);
            Err(crate::vm::errors::undefined_method_error_worded(
                method_name,
                &receiver,
                &arguments,
                wording,
                position,
            ))
        }
    }

    /// A call of a name the receiver's class retired with `undef_method`,
    /// native methods such as `inspect` and `<=>` included, reaches
    /// `method_missing`, or raises the NoMethodError a name nothing defines
    /// raises. Answers None when the name was not undefined.
    pub(crate) fn call_undefined_name(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        if !matches!(receiver.view(), View::Instance(_)) {
            return None;
        }
        let (_, method) = self.lookup_method(receiver, method_name)?;
        if !method.is_undefined {
            return None;
        }
        if method_name != "method_missing"
            && let Some(handled) =
                self.restricted_call_via_method_missing(receiver, method_name, arguments, position)
        {
            return Some(handled);
        }
        let wording = self.receiver_wording_for(receiver, position);
        Some(Err(crate::vm::errors::undefined_method_error_worded(
            method_name,
            receiver,
            arguments,
            wording,
            position,
        )))
    }

    /// A call that visibility refuses still reaches a user-defined
    /// `method_missing`, which is where Ruby's default implementation turns
    /// it into the NoMethodError. Answers None when nothing defines one, so
    /// the caller raises the error itself.
    fn restricted_call_via_method_missing(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        let (owner, handler) = self.lookup_method(receiver, "method_missing")?;
        if handler.is_undefined {
            return None;
        }
        let mut handler_arguments = vec![Object::symbol(method_name.to_string())];
        handler_arguments.extend(arguments.iter().cloned());
        Some(self.invoke_method(
            owner,
            handler,
            receiver.clone(),
            handler_arguments,
            position,
        ))
    }

    /// Look up a method on the receiver and return its class and method definition.
    /// Whether the current `self` is a class or module object, i.e. execution
    /// is inside a class or module body rather than at the top level.
    pub(crate) fn self_is_class_or_module(&self) -> bool {
        matches!(
            self.environment().get("self").view(),
            Some(View::Class(_) | View::Module(_))
        ) || self.def_scope_stack.last().is_some()
    }

    /// The class or module a receiverless declaration applies to: the current
    /// `self` when it is one, otherwise the innermost lexical class or module
    /// body, which is where an `eval`'d declaration lands.
    /// The class or module a `def` run here installs in, when it is written
    /// in a running method's body rather than in a class body opened since.
    /// Inside a block, the method is the one the block was written in, and
    /// inside a block `instance_exec` runs, it is the receiver's singleton
    /// class.
    pub(crate) fn running_method_def_scope(&mut self) -> Option<Rc<Class>> {
        match self.written_in_method() {
            Some((depth, definee)) if depth == self.def_scope_stack.len() => match definee {
                crate::vm::core::Definee::Class(class) => Some(class),
                crate::vm::core::Definee::SingletonOf(receiver) => {
                    Some(self.singleton_class_of(&receiver))
                }
            },
            _ => None,
        }
    }

    /// Where a `def` written in the running method around the code here
    /// installs, with the def scope depth the method started at.
    pub(crate) fn written_in_method(&self) -> Option<(usize, crate::vm::core::Definee)> {
        let frame = self.home_method_frame()?;
        if self.home_method_frame_is_borrowed() {
            return None;
        }
        self.method_definees.get(&frame).cloned()
    }

    fn home_method_frame(&self) -> Option<u64> {
        match self.lexical_home_frame {
            Some(home) => home,
            None => self.current_method_frame,
        }
    }

    /// A file loaded from inside a method runs its top level in that
    /// method's frame, and a block run by `class_exec` runs in the
    /// caller's. Neither is written in that method.
    pub(crate) fn home_method_frame_is_borrowed(&self) -> bool {
        self.home_method_frame().is_some_and(|frame| {
            self.borrowed_frames.contains(&Some(frame))
                && self.exec_definee_borrows.get(&frame) != Some(&self.borrowed_frames.len())
        })
    }

    pub(crate) fn current_definee(&self) -> Option<Rc<Class>> {
        if let Some(definee) = self.def_scope_stack.last() {
            return Some(Rc::clone(definee));
        }
        match self.environment().get("self").view() {
            Some(View::Class(class) | View::Module(class)) => Some(class.clone()),
            _ => None,
        }
    }

    /// Whether `receiver` answers to `name`. A class or module object answers
    /// to its module-level and singleton methods, never to its own instance
    /// methods, which belong to the objects it describes rather than to it.
    pub(crate) fn responds_to(&self, receiver: &Object, name: &str) -> bool {
        // A refinement in force here answers for the receiver too, which is
        // what `respond_to?` reports while the refinement is active.
        if self.any_refinement_active()
            && let Some(target) = refinement_target_name(receiver, self)
            && self.find_refined_method(&target, name).is_some()
        {
            return true;
        }
        // The object a program runs against at the top level answers the
        // methods its singleton class defines, which are native.
        if self.is_the_main_object(receiver) && main_object_method(name) {
            return true;
        }
        let (View::Class(class_rc) | View::Module(class_rc)) = receiver.view() else {
            // A tombstone left by `undef_method` is not something the object
            // responds to.
            // Kernel methods live in the native dispatch tables rather than
            // in any class's method map, so they are checked separately.
            return match self.lookup_method(receiver, name) {
                Some((_, method)) => !method.is_undefined,
                None => {
                    (matches!(receiver.view(), View::Method(_))
                        && crate::vm::native_methods::method_object_methods::NATIVE_METHOD_OBJECT_METHODS
                            .contains(&name))
                        || (matches!(receiver.view(), View::Exception(_))
                            && crate::vm::native_methods::EVERY_EXCEPTION_METHOD
                                .contains(&name))
                        || self.exception_answers_natively(receiver, name)
                        || crate::vm::native_methods::class_methods::BASIC_OBJECT_PRIVATE_METHODS
                            .contains(&name)
                        || crate::vm::native_methods::is_native_kernel_method(name)
                        || self.answers_natively(receiver, name)
                        || core_class_of_instance_defines(receiver, name)
                        || self.object_table_method(name).is_some()
                        || (self.builtins().class_of(receiver).name() == "File"
                            && crate::vm::native_methods::class_methods::is_native_io_method(name))
                }
            };
        };
        // A method put on the class's own singleton class stands ahead of one
        // written as `def self.name`, as it does when the method is called.
        if let Some(method) = class_rc
            .singleton_class_slot()
            .as_ref()
            .and_then(|singleton| singleton.find_own_method(name))
            .or_else(|| class_level_method(class_rc, name).map(|(_, method)| method))
        {
            return !method.is_undefined;
        }
        // Kernel carries its functions as module functions as well as private
        // instance methods, so it answers to the names a bare call reaches.
        if class_rc.ruby_name_is("Kernel")
            && crate::vm::native_methods::is_kernel_private_function(name)
        {
            return true;
        }
        if class_rc.ruby_name_is("Process")
            && crate::vm::native_methods::PROCESS_NATIVE_METHODS.contains(&name)
        {
            return true;
        }
        let mut cursor = Some(Rc::clone(class_rc));
        while let Some(current) = cursor {
            // A method undefined on the singleton class answers no further up.
            if let Some(sc) = current.singleton_class_slot().clone()
                && let Some(found) = sc.find_method(name)
            {
                return !found.is_undefined;
            }
            if core_singleton_method(&current.ruby_name(), name) {
                return true;
            }
            cursor = current.superclass();
        }
        // A module made by a subclass of Module answers what that subclass
        // defines for its instances.
        if let Some(made_from) = class_rc.instance_of()
            && made_from
                .find_method_with_owner(name)
                .is_some_and(|(owner, method)| {
                    !method.is_undefined && !owner.is_method_private(name)
                })
        {
            return true;
        }
        // What Class or Module, and their ancestors, define for their
        // instances, unless it is private there, as Kernel's functions are.
        if self
            .builtins()
            .class_of(receiver)
            .find_method_with_owner(name)
            .is_some_and(|(owner, _)| !owner.is_method_private(name))
        {
            return true;
        }
        // A class answers to `new` and to Module's own natives, which live in
        // the dispatch tables rather than in a method map. It also carries
        // Kernel's default `respond_to_missing?` the way any object does.
        if name == "respond_to_missing?"
            || (name == "new" && matches!(receiver.view(), View::Class(_)))
        {
            return true;
        }
        // Class's own public methods, such as `allocate` and `superclass`.
        if matches!(receiver.view(), View::Class(_)) && class_public_method(name) {
            return true;
        }
        // Kernel's and BasicObject's public methods, such as
        // `singleton_class` and `is_a?`, which a class or module answers as
        // any object does, unless one of them was undefined on the way.
        if (core_public_method("Kernel", name) || core_public_method("BasicObject", name))
            && self
                .lookup_method(receiver, name)
                .is_none_or(|(_, method)| !method.is_undefined)
        {
            return true;
        }
        // An exception class builds one with `exception` as well as `new`.
        if name == "exception"
            && matches!(receiver.view(), View::Class(_))
            && self.is_exception_class(class_rc)
        {
            return true;
        }
        // Reopening `Class` or `Module` adds a method every class and module
        // answers to, the same chain `lookup_method` walks for the call.
        for global_name in ["Class", "Module"] {
            if let Some(View::Class(global_class)) = self.globals().get(global_name).view()
                && let Some(method) = global_class.find_method(name)
            {
                return !method.is_undefined;
            }
        }
        // The hooks Module and Class define privately, such as
        // `method_added` and `inherited`, are methods a class or module has,
        // private as they are.
        if core_private_module_method(receiver, name) {
            return true;
        }
        crate::vm::native_methods::native_module_method_stub(name).is_some()
    }

    /// Whether `name` resolves to a private or protected method on
    /// `receiver`, so an explicit-receiver call would be refused.
    /// Whether `name` already carries `visibility` through `class_rc`'s
    /// ancestors, so declaring it again records nothing.
    pub(crate) fn inherits_visibility(
        &self,
        class_rc: &Rc<Class>,
        name: &str,
        visibility: &str,
    ) -> bool {
        match visibility {
            "private" => self.inherits_private(class_rc, name),
            "protected" => self.inherits_protected(class_rc, name),
            _ => {
                class_rc.find_method(name).is_some()
                    && self.inherited_visibility(class_rc, name).is_none()
            }
        }
    }

    /// Whether `name` is already private through `class_rc`'s ancestors.
    pub(crate) fn inherits_private(&self, class_rc: &Rc<Class>, name: &str) -> bool {
        matches!(self.inherited_visibility(class_rc, name), Some(true))
    }

    /// Whether `name` is already protected through `class_rc`'s ancestors.
    pub(crate) fn inherits_protected(&self, class_rc: &Rc<Class>, name: &str) -> bool {
        matches!(self.inherited_visibility(class_rc, name), Some(false))
    }

    /// The visibility `name` carries from the nearest ancestor that marks or
    /// defines it: `Some(true)` for private, `Some(false)` for protected,
    /// `None` for public or undefined.
    fn inherited_visibility(&self, class_rc: &Rc<Class>, name: &str) -> Option<bool> {
        // A module mixed in through another one counts, so a method private
        // in a module that `extend self` reaches through an include is seen.
        for ancestor in class_rc
            .transitive_mixins()
            .into_iter()
            .chain(std::iter::successors(class_rc.superclass(), |current| {
                current.superclass()
            }))
        {
            if ancestor.has_public_override(name) {
                return None;
            }
            if ancestor.is_method_private(name) {
                return Some(true);
            }
            if ancestor.is_method_protected(name) {
                return Some(false);
            }
            if ancestor.find_own_method(name).is_some() {
                return None;
            }
        }
        None
    }

    pub(crate) fn method_is_restricted(&self, receiver: &Object, name: &str) -> bool {
        // A Kernel function is a private method of every object, so a caller
        // outside the object cannot write it with a receiver. A module that
        // carries it as a module function is the exception, since that copy
        // is public.
        if (crate::vm::native_methods::is_kernel_private_function(name)
            || crate::vm::native_methods::class_methods::BASIC_OBJECT_PRIVATE_METHODS
                .contains(&name))
            && !matches!(receiver.view(), View::Class(_) | View::Module(_))
            && self.visibility_owner(receiver, name).is_none()
            && !self.answers_natively(receiver, name)
            && !self.exception_answers_natively(receiver, name)
        {
            return true;
        }
        let owner = self.visibility_owner(receiver, name);
        // A singleton class is where class-method visibility is recorded, so
        // its answer settles the question.
        if let Some(owner) = &owner
            && owner.is_singleton_class()
        {
            if owner.has_public_override(name) || !owner.is_method_restricted(name) {
                return false;
            }
            return owner.is_method_private(name) || !self.calling_from_class_under(owner);
        }
        // Otherwise a module-level copy is public even when the instance
        // method it was copied from is private, as `module_function` leaves it.
        if let View::Class(class_rc) | View::Module(class_rc) = receiver.view()
            && module_level_method(class_rc, name).is_some()
        {
            return false;
        }
        match owner {
            Some(owner) => self.refuses_explicit_receiver(&owner, name),
            // A hook Module or Class defines privately, which nothing in the
            // program defined over again.
            None => {
                core_private_module_method(receiver, name)
                    && self.lookup_method(receiver, name).is_none()
            }
        }
    }

    /// How a refused call names the marking that refused it, which Ruby
    /// spells out as either private or protected.
    fn visibility_word(&self, receiver: &Object, name: &str) -> &'static str {
        match self.visibility_owner(receiver, name) {
            Some(owner) if owner.is_method_protected(name) && !owner.is_method_private(name) => {
                "protected"
            }
            _ => "private",
        }
    }

    /// Whether `owner`'s marking on `name` refuses an explicit-receiver call
    /// from where this call is being made. A private method always refuses;
    /// a protected one refuses only from outside the class.
    pub(crate) fn refuses_explicit_receiver(&self, owner: &Rc<Class>, name: &str) -> bool {
        if owner.has_public_override(name) {
            return false;
        }
        if owner.is_method_private(name) {
            return true;
        }
        owner.is_method_protected(name) && !self.calling_from_within(owner)
    }

    /// Whether the code making this call is running as the class whose
    /// singleton class is `singleton`, or a class below it, which is what a
    /// protected class method asks.
    fn calling_from_class_under(&self, singleton: &Rc<Class>) -> bool {
        CALLER_CONSULTED.set(true);
        let caller = self.environment().get("self");
        let Some(View::Class(class_rc) | View::Module(class_rc)) = caller.view() else {
            return false;
        };
        let mut cursor = Some(Rc::clone(class_rc));
        while let Some(current) = cursor {
            if current
                .singleton_class_slot()
                .as_ref()
                .is_some_and(|held| Rc::ptr_eq(held, singleton))
            {
                return true;
            }
            cursor = current.superclass();
        }
        false
    }

    /// Whether the code making this call is running as an object the given
    /// class or module answers for, which is what a protected method asks.
    fn calling_from_within(&self, owner: &Rc<Class>) -> bool {
        CALLER_CONSULTED.set(true);
        let Some(caller) = self.environment().get("self") else {
            return false;
        };
        let mut cursor = match caller.view() {
            View::Class(class_rc) | View::Module(class_rc) => Some(Rc::clone(class_rc)),
            View::Instance(instance) => Some(Rc::clone(&instance.borrow().class)),
            _ => Some(self.builtins().class_of(&caller)),
        };
        while let Some(current) = cursor {
            if Rc::ptr_eq(&current, owner) || current.name() == owner.name() {
                return true;
            }
            if current
                .transitive_mixins()
                .iter()
                .any(|mixin| Rc::ptr_eq(mixin, owner))
            {
                return true;
            }
            cursor = current.superclass();
        }
        false
    }

    /// The class or module that actually defines `name` for `receiver`, which
    /// is where its visibility is recorded. `lookup_method` reports the class
    /// it dispatched through, and for a mixed-in method that is the including
    /// class rather than the module carrying the private marking.
    pub(crate) fn visibility_owner(&self, receiver: &Object, name: &str) -> Option<Rc<Class>> {
        // A class method's visibility is recorded on the singleton class,
        // even though the method itself may live in the class's own table
        // under the `__class__` convention.
        if let View::Class(class_rc) | View::Module(class_rc) = receiver.view() {
            let mut defining_singleton = None;
            let mut cursor = Some(Rc::clone(class_rc));
            while let Some(current) = cursor {
                if let Some(sc) = current.singleton_class_slot().clone() {
                    // The nearest singleton with its own marking for this
                    // name settles it; a subclass's `private_class_method`
                    // marks the name without redefining the method. A
                    // natively-implemented name such as `new` is marked the
                    // same way with no method map entry to find.
                    if sc.has_public_override(name) || sc.is_method_restricted(name) {
                        return Some(sc);
                    }
                    if let Some((owner, _)) = sc.find_method_with_owner(name) {
                        defining_singleton.get_or_insert(owner);
                    }
                }
                cursor = current.superclass();
            }
            if defining_singleton.is_some() {
                return defining_singleton;
            }
            // A module's own instance methods are its instances' business,
            // so what decides it is Module and the classes above it.
            if let View::Module(_) = receiver.view()
                && let Some(View::Class(module_class)) = self.globals().get("Module").view()
                && let Some((owner, _)) = module_class.find_method_with_owner(name)
            {
                return Some(owner);
            }
        }
        // A value's lookup answers with its own class, so the class that
        // defines the method is found from there, which is where a top-level
        // method's privacy is recorded.
        let View::Instance(instance_rc) = receiver.view() else {
            return self
                .lookup_method(receiver, name)
                .map(|(class, _)| {
                    class
                        .find_method_with_owner(name)
                        .map(|(owner, _)| owner)
                        .unwrap_or(class)
                })
                .or_else(|| self.object_table_method(name).map(|(owner, _)| owner));
        };
        let instance_ref = instance_rc.borrow();
        let singleton = instance_ref.singleton_class.borrow().clone();
        let class = Rc::clone(&instance_ref.class);
        drop(instance_ref);
        // A name the object answers natively has no table entry, so the
        // nearest class marking its visibility owns it.
        let marks =
            |held: &Rc<Class>| held.has_public_override(name) || held.is_method_restricted(name);
        if let Some(sc) = &singleton
            && sc.find_own_method(name).is_none()
            && marks(sc)
        {
            return Some(Rc::clone(sc));
        }
        singleton
            .and_then(|sc| sc.find_method_with_owner(name))
            .or_else(|| class.find_method_with_owner(name))
            .map(|(owner, _)| owner)
            .or_else(|| {
                let mut cursor = Some(class);
                while let Some(current) = cursor {
                    if marks(&current) {
                        return Some(current);
                    }
                    cursor = current.superclass();
                }
                None
            })
    }

    /// The answer to a call with a receiver to a private method every object
    /// answers natively, such as `puts` or `initialize`: the receiver's own
    /// `method_missing`, or the NoMethodError, unless the receiver answers it
    /// publicly, which leaves the call to go ahead.
    #[inline(never)]
    fn refuse_private_object_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        let private_everywhere = crate::vm::native_methods::is_kernel_private_function(method_name)
            || crate::vm::native_methods::class_methods::BASIC_OBJECT_PRIVATE_METHODS
                .contains(&method_name);
        if !private_everywhere {
            return None;
        }
        let made_public = self.public_in_chain(receiver, method_name, true);
        // A Kernel function a class or singleton made public, which nothing
        // native answers on this receiver, is called as the function it is.
        if made_public
            && crate::vm::native_methods::is_kernel_private_function(method_name)
            && !self.answers_natively(receiver, method_name)
        {
            return Some(self.call_native_function(method_name, arguments.to_vec(), position));
        }
        let refused = self.method_is_restricted(receiver, method_name)
            && !made_public
            && !self.public_in_chain(receiver, method_name, false);
        if !refused {
            return None;
        }
        // A receiver with a `method_missing` of its own is handed the call,
        // as a Delegator is to pass it on.
        if let Some(handled) =
            self.restricted_call_via_method_missing(receiver, method_name, arguments, position)
        {
            return Some(handled);
        }
        Some(Err(self.private_object_method_error(
            receiver,
            method_name,
            arguments,
            position,
        )))
    }

    /// Whether a class or module the receiver answers through, or its own
    /// singleton class, makes `name` public: by the program's own `public`,
    /// with `by_program`, or otherwise as a core class that defines it as a
    /// public method of its own, as Binding does `eval`.
    fn public_in_chain(&self, receiver: &Object, name: &str, by_program: bool) -> bool {
        let singleton = match receiver.view() {
            View::Instance(instance) => instance.borrow().singleton_class.borrow().clone(),
            _ => None,
        };
        let class = match receiver.view() {
            View::Instance(instance) => Rc::clone(&instance.borrow().class),
            _ => self.builtins().class_of(receiver),
        };
        let publicly = |held: &Rc<crate::class::Class>| {
            if by_program {
                held.has_public_override(name)
            } else {
                crate::vm::native_methods::core_method_names::CORE_METHODS
                    .iter()
                    .any(|row| row.owner == held.ruby_name() && row.public.contains(&name))
            }
        };
        let mut cursor = singleton.or(Some(class));
        while let Some(current) = cursor {
            if publicly(&current) || current.transitive_mixins().iter().any(publicly) {
                return true;
            }
            cursor = current.superclass();
        }
        false
    }

    /// The answer to a call with a receiver to a method an instance answers
    /// natively, when a class or the singleton class marked the name private
    /// or protected: the receiver's own `method_missing`, or the
    /// NoMethodError. None when no marking refuses the call.
    #[inline(never)]
    fn refuse_marked_native_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        let View::Instance(instance) = receiver.view() else {
            return None;
        };
        let marks = |held: &Rc<Class>| {
            held.has_public_override(method_name) || held.is_method_restricted(method_name)
        };
        let singleton = instance.borrow().singleton_class.borrow().clone();
        let mut owner = singleton.filter(|held| marks(held));
        let mut cursor = Some(Rc::clone(&instance.borrow().class));
        while owner.is_none()
            && let Some(current) = cursor
        {
            if marks(&current) {
                owner = Some(Rc::clone(&current));
            }
            cursor = current.superclass();
        }
        let owner = owner?;
        if !self.refuses_explicit_receiver(&owner, method_name) {
            return None;
        }
        if let Some(handled) =
            self.restricted_call_via_method_missing(receiver, method_name, arguments, position)
        {
            return Some(handled);
        }
        let wording = self.receiver_wording_for(receiver, position);
        let word =
            if owner.is_method_protected(method_name) && !owner.is_method_private(method_name) {
                "protected"
            } else {
                "private"
            };
        let msg = format!("{word} method '{method_name}' called for {wording}");
        Some(Err(MetorexError::UncaughtException {
            exception: crate::vm::errors::no_method_error(&msg, method_name, receiver, arguments),
            location: crate::vm::utils::position_to_location(position),
            message: msg,
        }))
    }

    /// The NoMethodError for a private method of Object called with a
    /// receiver. Kept out of the method-call path, which runs once for every
    /// level of a Ruby call and would carry this one's locals in each frame.
    #[inline(never)]
    fn private_object_method_error(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> MetorexError {
        let wording = match receiver.view() {
            View::Nil => "nil".to_string(),
            View::Bool(held) => held.to_string(),
            _ => self.receiver_wording_for(receiver, position),
        };
        let msg = format!("private method '{method_name}' called for {wording}");
        MetorexError::UncaughtException {
            exception: crate::vm::errors::no_method_error(&msg, method_name, receiver, arguments),
            location: crate::vm::utils::position_to_location(position),
            message: msg,
        }
    }

    /// A method written on Object, such as a top-level `def`, which a value
    /// whose own class lookup does not reach Object still answers to.
    pub(crate) fn object_table_method(&self, method_name: &str) -> Option<(Rc<Class>, Rc<Method>)> {
        let viewed = self.globals().get("Object");
        let Some(View::Class(object_class)) = viewed.view() else {
            return None;
        };
        let method = object_class.find_method(method_name)?;
        (!method.is_undefined).then_some((object_class.clone(), method))
    }

    /// Whether `method`, found for an exception, nil, true or false, is one
    /// every object has from Object, Kernel or BasicObject, while the value's
    /// own class answers `method_name` natively ahead of it.
    fn object_method_hides_native(
        &self,
        receiver: &Object,
        method_name: &str,
        method: &Rc<Method>,
    ) -> bool {
        let answered_natively = match receiver.view() {
            View::Exception(_) => {
                crate::vm::native_methods::EVERY_EXCEPTION_METHOD.contains(&method_name)
            }
            View::Nil | View::Bool(_) => {
                crate::vm::native_methods::value_answers_natively(receiver, method_name)
            }
            _ => false,
        };
        answered_natively
            && self
                .object_table_method(method_name)
                .is_some_and(|(_, every_object_has)| Rc::ptr_eq(&every_object_has, method))
    }

    /// Whether a class or module receiver would answer `method_name` with an
    /// instance method every object has from Object, Kernel or BasicObject.
    pub(crate) fn object_method_behind_native(&self, receiver: &Object, method_name: &str) -> bool {
        if !matches!(receiver.view(), View::Class(_) | View::Module(_)) {
            return false;
        }
        let Some((_, found)) = self.lookup_method(receiver, method_name) else {
            return false;
        };
        let viewed = self.globals().get("Object");
        let Some(View::Class(object_class)) = viewed.view() else {
            return false;
        };
        object_class
            .find_method(method_name)
            .is_some_and(|every_object_has| Rc::ptr_eq(&every_object_has, &found))
    }

    /// Whether the method the receiver would answer comes from the Enumerable
    /// module rather than from its own class.
    pub(crate) fn enumerable_stands_in(&self, receiver: &Object, method_name: &str) -> bool {
        let class = match receiver.view() {
            View::Instance(instance_rc) => Rc::clone(&instance_rc.borrow().class),
            View::Class(_) | View::Module(_) => return false,
            _ => self.builtins().class_of(receiver),
        };
        class
            .find_method_with_owner(method_name)
            .is_some_and(|(owner, _)| owner.ruby_name_is("Enumerable"))
    }

    /// An operator method written on a built-in value's own class, on one of
    /// the modules prepended to it, or on the value's singleton class. Used
    /// when `1 + 2` has to reach a reopened `Integer#+` rather than the native
    /// arithmetic. An ancestor's copy does not count: the core library writes
    /// several operators in Ruby on `Numeric` and on `Comparable`, and those
    /// stand for the native implementation rather than replacing it. A
    /// body-less stub does not count either, for the same reason.
    pub(crate) fn lookup_own_operator_method(
        &self,
        receiver: &Object,
        method_name: &str,
    ) -> Option<(Rc<Class>, Rc<Method>)> {
        let answers = |owner: &Rc<Class>| {
            owner
                .find_own_method(method_name)
                .filter(|method| !method.is_undefined && !method.body.is_empty())
                .map(|method| (Rc::clone(owner), method))
        };
        if let Some(found) = self
            .existing_singleton_class(receiver)
            .and_then(|singleton| answers(&singleton))
        {
            return Some(found);
        }
        let class = self.builtins().class_of(receiver);
        if class.has_prepends()
            && let Some(found) = class.prepend_chain().iter().find_map(answers)
        {
            return Some(found);
        }
        answers(&class)
    }

    pub(crate) fn lookup_method(
        &self,
        receiver: &Object,
        method_name: &str,
    ) -> Option<(Rc<Class>, Rc<Method>)> {
        match receiver.view() {
            View::Instance(instance_rc) => {
                let instance_ref = instance_rc.borrow();
                let class = Rc::clone(&instance_ref.class);
                if let Some(sing) = instance_ref.find_singleton_method(method_name) {
                    return Some((class, sing));
                }
                if let Some(sc) = instance_ref.singleton_class.borrow().clone()
                    && let Some(method) = sc.find_method(method_name)
                {
                    return Some((sc, method));
                }
                drop(instance_ref);
                class.find_method(method_name).map(|method| (class, method))
            }
            View::Class(class_rc) => {
                // A module-level method (`def self.name`, or one copied in by
                // `extend`) is checked before walking the singleton class's
                // superclass chain, so a method from the singleton's ancestors
                // (e.g. Object#describe, when Object has been reopened) does
                // not shadow the class's own class-level method.
                // A method put on the class's own singleton class stands
                // ahead of one written as `def self.name`, which is how a
                // later `define_singleton_method` replaces it.
                if let Some(found) = singleton_prepended_method(class_rc, method_name) {
                    return Some(found);
                }
                if let Some(sc) = class_rc.singleton_class_slot().clone()
                    && let Some(method) = sc.find_own_method(method_name)
                {
                    return Some((sc, method));
                }
                if let Some(found) = class_level_method(class_rc, method_name) {
                    return Some(found);
                }
                // Walk the receiver's superclass chain, checking each
                // ancestor's singleton class. Mirrors Ruby's metaclass
                // chain so `class << Parent; attr_accessor :x; end` is
                // visible on `Child.x` when `Child < Parent`.
                let mut cursor = Some(Rc::clone(class_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && let Some(method) = sc.find_method(method_name)
                    {
                        return Some((sc, method));
                    }
                    cursor = current.superclass();
                }
                // What Class and Module themselves define answers before an
                // instance method of the class does, the way Ruby's singleton
                // chain reaches Class and Module before Object. A name Module
                // answers natively counts as one of its own.
                for global_name in ["Class", "Module"] {
                    if let Some(View::Class(global_class)) = self.globals().get(global_name).view()
                        && let Some(method) = global_class.find_own_method(method_name)
                    {
                        return Some((global_class.clone(), method));
                    }
                }
                if crate::vm::native_methods::is_native_module_method(method_name)
                    && let Some(View::Class(module_class)) = self.globals().get("Module").view()
                {
                    // The stub carries the parameters Module's method takes,
                    // which a block made from it with `&method(name)` counts.
                    let stub = crate::vm::native_methods::native_module_method_stub(method_name)
                        .unwrap_or_else(|| {
                            crate::object::Method::with_owner(
                                method_name.to_string(),
                                Vec::new(),
                                Vec::new(),
                                "Module".to_string(),
                            )
                        });
                    return Some((module_class.clone(), Rc::new(stub)));
                }
                // A class answers what Class and its ancestors define for
                // their instances, never its own instance methods. An
                // encoding is the exception: each one is kept as a class
                // below Encoding, and answers what Encoding defines for its
                // instances.
                if class_rc
                    .superclass()
                    .is_some_and(|parent| parent.name() == "Encoding")
                    && let Some(method) = class_rc.find_method(method_name)
                {
                    return Some((Rc::clone(class_rc), method));
                }
                // Every class is an instance of `Class`, which descends from
                // `Module`, so reopening either (`class Module; def
                // const_added(name); ...`) adds a method every class answers.
                for global_name in ["Class", "Module"] {
                    if let Some(View::Class(global_class)) = self.globals().get(global_name).view()
                        && let Some(method) = global_class.find_method(method_name)
                    {
                        return Some((global_class.clone(), method));
                    }
                }
                None
            }
            View::Module(module_rc) => {
                if let Some(sc) = module_rc.singleton_class_slot().clone()
                    && let Some(method) = sc.find_own_method(method_name)
                {
                    return Some((sc, method));
                }
                if let Some(method) = module_level_method(module_rc, method_name) {
                    return Some((Rc::clone(module_rc), method));
                }
                let mut cursor = Some(Rc::clone(module_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && let Some(method) = sc.find_method(method_name)
                    {
                        return Some((sc, method));
                    }
                    cursor = current.superclass();
                }
                // A module made by `new` on a subclass of Module answers the
                // methods that subclass writes.
                if let Some(made_from) = module_rc.instance_of()
                    && let Some(method) = made_from.find_method(method_name)
                {
                    return Some((made_from, method));
                }
                // User-defined modules are instances of the global `Module`
                // class. Methods mixed into Module via `class Module; include
                // X; end` (mspec does this for matchers like `be_nil`) live
                // on that global class, not on the user module itself, so
                // fall back to `Module`'s method table when the receiver's
                // own chain comes up empty.
                if let Some(View::Class(global_module)) = self.globals().get("Module").view()
                    && let Some(method) = global_module.find_method(method_name)
                {
                    return Some((global_module.clone(), method));
                }
                None
            }
            _ => {
                // A value kind with its own singleton class, such as an
                // exception given a `def obj.name`, answers from there first.
                if let Some(singleton) = self.existing_singleton_class(receiver)
                    && let Some(method) = singleton.find_method(method_name)
                {
                    return Some((singleton, method));
                }
                // A program that reopens `NilClass` or one of the other
                // immediate classes makes a class of its own in globals,
                // which is where the method it wrote lives.
                if let Some(named) = reopened_class_name(receiver)
                    && let Some(View::Class(reopened)) = self.globals().get(named).view()
                    && let Some(method) = reopened.find_method(method_name)
                    && !method.body.is_empty()
                    && !self.object_method_hides_native(receiver, method_name, &method)
                {
                    return Some((reopened.clone(), method));
                }
                // An exception built from a user-defined subclass looks the
                // method up on that class, which `class_of` cannot report.
                if let View::Exception(details) = receiver.view()
                    && let Some(class) = details.borrow().class.clone()
                    && let Some(method) = class.find_method(method_name)
                    // A body-less stub stands in for a native method, so it
                    // must not shadow the implementation, unless it is an
                    // alias reaching one under a name of its own.
                    && (!method.body.is_empty() || aliases_another_name(&method, method_name))
                    && !self.object_method_hides_native(receiver, method_name, &method)
                {
                    return Some((class, method));
                }
                // An exception the interpreter raised carries only its class
                // name, and the class that name resolves to holds what a
                // program reopened or prepended there.
                if let View::Exception(details) = receiver.view()
                    && details.borrow().class.is_none()
                {
                    let exception_type = details.borrow().exception_type.clone();
                    let named = match self.globals().get(&exception_type).view() {
                        Some(View::Class(class)) => Some(Object::Class(class.clone())),
                        _ => self.resolve_qualified_constant(&exception_type),
                    };
                    if let Some(View::Class(class)) = named.view()
                        && let Some(method) = class.find_method(method_name)
                        && !method.body.is_empty()
                        && !self.object_method_hides_native(receiver, method_name, &method)
                    {
                        return Some((class.clone(), method));
                    }
                }
                let class = self.builtins().class_of(receiver);
                class
                    .find_method(method_name)
                    .filter(|method| {
                        !self.object_method_hides_native(receiver, method_name, method)
                    })
                    .map(|method| (class, method))
            }
        }
    }
}

/// The class name a program writes to reopen one of the immediate values,
/// whose methods live in a class of their own rather than on the builtin.
pub(crate) fn reopened_class_name(receiver: &Object) -> Option<&'static str> {
    match receiver.view() {
        View::Nil => Some("NilClass"),
        View::Bool(true) => Some("TrueClass"),
        View::Bool(false) => Some("FalseClass"),
        View::Symbol(_) => Some("Symbol"),
        // The core library reopens Regexp, so what a program writes there
        // lands on the class in globals rather than on the builtin.
        View::Regex(_, _) => Some("Regexp"),
        _ => None,
    }
}

/// A module-level method on a class or module object: one stored under the
/// `__class__` convention by `def self.name` or `module_function`, or copied
/// in by `extend` under the `__ext__` convention.
/// The method a module prepended to `class_rc`'s singleton class gives it,
/// which stands in front of the class's own class methods, with the module.
pub(crate) fn singleton_prepended_method(
    class_rc: &Rc<Class>,
    method_name: &str,
) -> Option<(Rc<Class>, Rc<Method>)> {
    let singleton = class_rc.singleton_class_slot().clone()?;
    singleton.prepend_chain().into_iter().find_map(|prepended| {
        prepended
            .find_method(method_name)
            .map(|method| (prepended, method))
    })
}

/// A class method `class_rc` answers from its own `def self.name`, a module
/// it extended, or a superclass. A method put on a superclass's singleton
/// class stands ahead of the `def self.name` that superclass wrote, as one
/// `define_method` there replaces it.
pub(crate) fn class_level_method(
    class_rc: &Rc<Class>,
    method_name: &str,
) -> Option<(Rc<Class>, Rc<Method>)> {
    if let Some(method) = class_rc
        .find_own_method(&format!("__class__{}", method_name))
        .or_else(|| module_extended_method(class_rc, method_name))
    {
        return Some((Rc::clone(class_rc), method));
    }
    if let Some(found) = extended_method(class_rc, method_name) {
        return Some(found);
    }
    let mut cursor = class_rc.superclass();
    while let Some(current) = cursor {
        if let Some(singleton) = current.singleton_class_slot().clone()
            && let Some(method) = singleton.find_own_method(method_name)
        {
            return Some((singleton, method));
        }
        if let Some(method) = current.find_own_method(&format!("__class__{}", method_name)) {
            return Some((Rc::clone(class_rc), method));
        }
        if let Some(found) = extended_method(&current, method_name) {
            return Some(found);
        }
        cursor = current.superclass();
    }
    None
}

/// A method of a module `extend` put on the class's singleton class.
pub(crate) fn extended_method(
    class_rc: &Rc<Class>,
    method_name: &str,
) -> Option<(Rc<Class>, Rc<Method>)> {
    let singleton = class_rc.singleton_class_slot().clone()?;
    let mixins = singleton.mixins.borrow();
    mixins
        .iter()
        .find_map(|module| module.find_method_with_owner(method_name))
}

pub(crate) fn module_level_method(class_rc: &Rc<Class>, method_name: &str) -> Option<Rc<Method>> {
    module_own_method(class_rc, method_name)
        .or_else(|| module_extended_method(class_rc, method_name))
}

/// A method written on the module itself with `def self.name` or `def Mod.name`.
pub(crate) fn module_own_method(class_rc: &Rc<Class>, method_name: &str) -> Option<Rc<Method>> {
    // Classes holding no `def self.name` anywhere up the chain answer none.
    let mut cursor = Some(Rc::clone(class_rc));
    let mut any_held = false;
    while let Some(current) = cursor {
        if current.holds_class_methods.get() {
            any_held = true;
            break;
        }
        cursor = current.superclass();
    }
    if !any_held {
        return None;
    }
    class_rc.find_inherited_method(&format!("__class__{}", method_name))
}

/// A method the module answers to because it extended a module, its own
/// instance methods included when it extended itself. The extended module is
/// asked again, so a method it was given later answers in place of the one
/// it held when it was extended.
pub(crate) fn module_extended_method(
    class_rc: &Rc<Class>,
    method_name: &str,
) -> Option<Rc<Method>> {
    if !class_rc.holds_extended_copies.get() {
        return None;
    }
    let viewed = class_rc.get_class_var(&format!("__ext__{}", method_name));
    let copied = match viewed.view() {
        Some(View::Method(method)) => method,
        _ => return None,
    };
    match class_rc
        .get_class_var(&format!("__ext_from__{}", method_name))
        .view()
    {
        Some(View::Module(extended)) => extended.find_method(method_name).or(Some(copied.clone())),
        _ => Some(copied.clone()),
    }
}

/// Whether a receiver expression names `self`, which Ruby treats as an
/// implicit receiver: `self.name` reaches a private method the way a bare
/// `name` does. The lexer reads `self` as an identifier, so the keyword form
/// and the parser's own `SelfExpr` both have to be recognized here.
/// How a call's receiver was written, as far as visibility asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WrittenReceiver {
    /// The keyword `self`.
    SelfKeyword,
    /// An identifier spelled `self`.
    SelfName,
    /// Anything else.
    Other,
}

impl WrittenReceiver {
    pub(crate) fn of(receiver_expr: &Expression) -> Self {
        match receiver_expr {
            Expression::SelfExpr { .. } => WrittenReceiver::SelfKeyword,
            Expression::Identifier { name, .. } if name == "self" => WrittenReceiver::SelfName,
            _ => WrittenReceiver::Other,
        }
    }

    /// Whether the receiver was written as `self`, by keyword or by name.
    pub(crate) fn names_self(self) -> bool {
        self != WrittenReceiver::Other
    }
}

pub(crate) fn names_self(receiver_expr: &Expression) -> bool {
    match receiver_expr {
        Expression::SelfExpr { .. } => true,
        Expression::Identifier { name, .. } => name == "self",
        _ => false,
    }
}

/// Whether `name` is one of the public instance methods the core class or
/// module named `owner` defines.
pub(crate) fn core_public_method(owner: &str, name: &str) -> bool {
    crate::vm::native_methods::core_method_names::CORE_METHODS
        .iter()
        .find(|row| row.owner == owner)
        .is_some_and(|row| row.public.contains(&name))
}

/// Whether a core class an instance belongs to, below Object, defines
/// `name` as a public method of its own, as Thread::Queue does `empty?`.
/// Instances of those classes are answered by native dispatch, which no
/// method table records.
fn core_class_of_instance_defines(receiver: &Object, name: &str) -> bool {
    let View::Instance(instance) = receiver.view() else {
        return false;
    };
    let mut cursor = Some(Rc::clone(&instance.borrow().class));
    while let Some(current) = cursor {
        let named = current.ruby_name();
        if named == "Object" || named == "BasicObject" {
            return false;
        }
        if core_public_method(&named, name) {
            return true;
        }
        cursor = current.superclass();
    }
    false
}

/// Whether `name` is one of the private instance methods the core class or
/// module named `owner` defines.
pub(crate) fn core_private_method(owner: &str, name: &str) -> bool {
    crate::vm::native_methods::core_method_names::CORE_METHODS
        .iter()
        .find(|row| row.owner == owner)
        .is_some_and(|row| row.private.contains(&name))
}

/// The private methods of Module that Class undefines, which a class does
/// not have.
const MODULE_PRIVATE_METHODS_CLASS_UNDEFINES: &[&str] = &[
    "append_features",
    "extend_object",
    "module_function",
    "prepend_features",
    "refine",
];

/// Whether `name` is a private method `receiver`, a class or a module, has
/// from Module or Class.
fn core_private_module_method(receiver: &Object, name: &str) -> bool {
    match receiver.view() {
        View::Module(_) => core_private_method("Module", name),
        View::Class(_) => {
            core_private_method("Class", name)
                || (core_private_method("Module", name)
                    && !MODULE_PRIVATE_METHODS_CLASS_UNDEFINES.contains(&name))
        }
        _ => false,
    }
}

/// Whether `name` is one of the public instance methods Class defines.
pub(crate) fn class_public_method(name: &str) -> bool {
    crate::vm::native_methods::core_method_names::CORE_METHODS
        .iter()
        .find(|row| row.owner == "Class")
        .is_some_and(|row| row.public.contains(&name))
}

/// Whether the core class or module named `owner` answers `name` on
/// itself, as Dir answers `pwd`.
pub(crate) fn core_singleton_method(owner: &str, name: &str) -> bool {
    crate::vm::native_methods::core_method_names::CORE_SINGLETON_METHODS
        .iter()
        .find(|row| row.owner == owner)
        .is_some_and(|row| row.public.contains(&name))
}

/// Whether `name` is one of the methods the singleton class of the object a
/// program runs against at the top level defines.
pub(crate) fn main_object_method(name: &str) -> bool {
    crate::vm::native_methods::core_method_names::CORE_METHODS
        .iter()
        .find(|row| row.owner == "main")
        .is_some_and(|row| row.public.contains(&name) || row.private.contains(&name))
}

/// Whether a body-less stub is an alias reaching a native method under a
/// name other than its own.
fn aliases_another_name(method: &Method, name: &str) -> bool {
    method
        .native_alias
        .as_deref()
        .is_some_and(|target| target != name)
}
