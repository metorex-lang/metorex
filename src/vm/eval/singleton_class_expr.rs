// Singleton-class expression support: `class << target; body; end`.
//
// - Resolves (or lazily materializes) the singleton class for the target.
// - Executes `body` with the singleton class as `self` and as the enclosing
//   lexical scope, so method defs inside land on the singleton class.
// - Returns the last expression value from the body (or nil).

use crate::object::{OptionView, View};
use std::rc::Rc;

use crate::ast::{Expression, Statement};
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::core::VirtualMachine;

/// What the singleton class of a receiver that has no slot of its own for
/// one is kept under: its kind, and for one held by reference, its address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SingletonKey {
    Nil,
    True,
    False,
    Array(usize),
    Dict(usize),
    Set(usize),
    String(usize),
    Regex(usize),
    Block(usize),
    Method(usize),
    Binding(usize),
    Exception(usize),
    /// Every other value of the kind named, which share one.
    Other(&'static str),
}

/// The key the singleton class of a value-kind receiver that has no
/// per-object storage slot of its own is kept under. None for receivers
/// that *do* own a slot, which are handled directly from the object, and
/// for an Integer or a Float.
fn primitive_singleton_key(receiver: &Object) -> Option<SingletonKey> {
    let address = |pointer: *const ()| pointer as usize;
    match receiver.view() {
        View::Nil => Some(SingletonKey::Nil),
        View::Bool(true) => Some(SingletonKey::True),
        View::Bool(false) => Some(SingletonKey::False),
        View::Int(_) | View::Float(_) => None,
        // A collection and a string are each held by reference, so every one
        // of them can carry methods of its own rather than sharing a table
        // with every other collection of its kind.
        View::Array(held) => Some(SingletonKey::Array(address(Rc::as_ptr(held).cast()))),
        View::Dict(held) => Some(SingletonKey::Dict(address(Rc::as_ptr(held).cast()))),
        View::Set(held) => Some(SingletonKey::Set(address(Rc::as_ptr(held).cast()))),
        View::String(held) => Some(SingletonKey::String(address(Rc::as_ptr(held).cast()))),
        // A Regexp is held by reference too, so each one carries its own
        // methods rather than sharing a table with every other pattern.
        View::Regex(pattern, _) => Some(SingletonKey::Regex(address(Rc::as_ptr(pattern).cast()))),
        // A Proc, a Method and a Binding are each an object of their own.
        View::Block(held) => Some(SingletonKey::Block(address(Rc::as_ptr(held).cast()))),
        View::Method(held) => Some(SingletonKey::Method(address(Rc::as_ptr(held).cast()))),
        View::Binding(held) => Some(SingletonKey::Binding(address(Rc::as_ptr(held).cast()))),
        _ => None,
    }
}

/// Display-name the singleton class should carry. Ruby prints singleton
/// classes as `#<Class:<target-inspect>>`; we approximate with a stable
/// tag so ancestors/inspect output is predictable.
fn singleton_class_display_name(receiver: &Object) -> String {
    match receiver.view() {
        View::Nil => "#<Class:NilClass>".to_string(),
        View::Bool(true) => "#<Class:TrueClass>".to_string(),
        View::Bool(false) => "#<Class:FalseClass>".to_string(),
        View::Class(c) => format!("#<Class:{}>", c.inspect_name()),
        View::Module(m) => format!("#<Class:{}>", m.inspect_name()),
        // The attached object's own display, so the two agree.
        _ => format!("#<Class:{}>", receiver),
    }
}

impl VirtualMachine {
    /// Look up (and, if necessary, create) the singleton class of `receiver`.
    /// The singleton class records its attached object via the `__attached__`
    /// class-variable so `attached_object` can return it later.
    pub(crate) fn singleton_class_of(&mut self, receiver: &Object) -> Rc<Class> {
        match receiver.view() {
            View::Instance(inst_rc) => {
                let inst = inst_rc.borrow();
                if let Some(existing) = inst.singleton_class.borrow().clone() {
                    return existing;
                }
                let superclass = Some(Rc::clone(&inst.class));
                let sc = Class::new(singleton_class_display_name(receiver), superclass);
                sc.set_class_var("__singleton__", Object::Bool(true));
                sc.set_class_var("__attached__", receiver.clone());
                *inst.singleton_class.borrow_mut() = Some(Rc::clone(&sc));
                sc
            }
            View::Class(class_rc) | View::Module(class_rc) => {
                if let Some(existing) = class_rc.singleton_class_slot().clone() {
                    return existing;
                }
                let is_module = matches!(receiver.view(), View::Module(_));
                // Per Ruby: the singleton class of a Class K has K's parent's
                // singleton class as its superclass (not K's parent itself).
                // Build that chain on demand by recursing through `superclass`.
                // For BasicObject (no superclass), the singleton inherits from
                // `Class` itself, which gives the chain Class → Module → Object
                // → Kernel → BasicObject when walking a class's singleton
                // ancestors. A standalone module's singleton inherits from
                // `Module`, or from the Module subclass that made it, matching
                // MRI. Other module-kind classes with no
                // superclass stay ungrounded to avoid unbounded lookup loops.
                let parent_singleton = match class_rc.superclass() {
                    Some(sup) => Some(self.singleton_class_of(&Object::Class(sup))),
                    None if class_rc.name() == "BasicObject" => {
                        (match self.globals().get("Class").view() {
                            Some(View::Class(c)) => Some(c),
                            _ => None,
                        })
                        .cloned()
                    }
                    // A module made by `new` on a subclass of Module is an
                    // instance of that subclass, which its singleton class
                    // inherits from.
                    None if is_module && class_rc.instance_of().is_some() => class_rc.instance_of(),
                    None if is_module => (match self.globals().get("Module").view() {
                        Some(View::Class(c)) => Some(c),
                        _ => None,
                    })
                    .cloned(),
                    None => None,
                };
                let sc = Class::new(singleton_class_display_name(receiver), parent_singleton);
                sc.set_class_var("__singleton__", Object::Bool(true));
                sc.set_class_var("__attached__", receiver.clone());
                class_rc.set_singleton_class(Rc::clone(&sc));
                sc
            }
            other => {
                let key = primitive_singleton_key(receiver).unwrap_or_else(|| {
                    match other {
                        // An exception is a reference type, so each one gets its
                        // own singleton class rather than sharing one per kind.
                        View::Exception(details) => {
                            SingletonKey::Exception(Rc::as_ptr(details).cast::<()>() as usize)
                        }
                        _ => SingletonKey::Other(other.type_name()),
                    }
                });
                if let Some(existing) = self.primitive_singleton_classes.get(&key) {
                    return Rc::clone(existing);
                }
                let sc = Class::new(singleton_class_display_name(receiver), None);
                sc.set_class_var("__singleton__", Object::Bool(true));
                sc.set_class_var("__attached__", receiver.clone());
                self.primitive_singleton_classes.insert(key, Rc::clone(&sc));
                sc
            }
        }
    }

    /// The singleton class a value-kind receiver already has, without
    /// creating one. Used by method lookup, which must not materialize a
    /// singleton class just by asking.
    pub(crate) fn existing_singleton_class(&self, receiver: &Object) -> Option<Rc<Class>> {
        // Most programs give no value of these kinds a singleton class, which
        // spares building the key to look one up by.
        if self.primitive_singleton_classes.is_empty() {
            return None;
        }
        let key = primitive_singleton_key(receiver).or_else(|| match receiver.view() {
            View::Exception(details) => Some(SingletonKey::Exception(
                Rc::as_ptr(details).cast::<()>() as usize,
            )),
            View::Instance(_) | View::Class(_) | View::Module(_) => None,
            other => Some(SingletonKey::Other(other.type_name())),
        })?;
        self.primitive_singleton_classes.get(&key).map(Rc::clone)
    }

    /// Whether `receiver` was given a singleton class, without making one.
    pub(crate) fn singleton_given(&self, receiver: &Object) -> bool {
        match receiver.view() {
            View::Instance(held) => held.borrow().singleton_class.borrow().is_some(),
            View::Class(held) | View::Module(held) => held.singleton_class_slot().is_some(),
            _ => self.existing_singleton_class(receiver).is_some(),
        }
    }

    /// Carry the methods an object was given of its own over to a copy of it,
    /// which is what `clone` does and `dup` does not.
    pub(crate) fn copy_singleton_methods(&mut self, source: &Object, copy: &Object) {
        let Some(singleton) = self.existing_singleton_class(source) else {
            return;
        };
        let names = singleton.method_names();
        if names.is_empty() {
            return;
        }
        let destination = self.singleton_class_of(copy);
        for name in names {
            if let Some(method) = singleton.find_own_method(&name) {
                destination.define_method(name, method);
            }
        }
    }

    /// Evaluate `class << target; body; end`.
    /// Materializes the singleton class, runs the body with it as the
    /// enclosing lexical scope and `self`, and returns the body's last value.
    pub(crate) fn evaluate_singleton_class_expression(
        &mut self,
        target: &Expression,
        assign_to: Option<&Expression>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let target_obj = self.evaluate_expression(target)?;
        self.open_singleton_class(target_obj, assign_to, body, position)
    }

    /// Run `class << target; body; end` with the target evaluated already.
    /// `assign_to` is the place `class << place = target` stores it, None
    /// when the code compiled the store itself.
    pub(crate) fn open_singleton_class(
        &mut self,
        target_obj: Object,
        assign_to: Option<&Expression>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        // `true`, `false`, and `nil` each have exactly one instance, so the
        // class itself is the singleton class. A number or a symbol has no
        // singleton class at all.
        match target_obj.view() {
            View::Bool(_) | View::Nil => {
                let named = match target_obj.view() {
                    View::Bool(true) => "TrueClass",
                    View::Bool(false) => "FalseClass",
                    _ => "NilClass",
                };
                if let Some(View::Class(held)) = self.globals().get(named).view() {
                    let singleton = Rc::clone(held);
                    if let Some(destination) = assign_to {
                        self.assign_value(destination, target_obj.clone())?;
                    }
                    return self.run_singleton_class_body(&singleton, body, position);
                }
            }
            View::Int(_) | View::BigInt(_) | View::Float(_) | View::Symbol(_) => {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "can't define singleton",
                    position,
                ));
            }
            _ => {}
        }
        // `class << @receiver = Object.new` stores the object first, so the
        // body's `def`s land on the same one the name now holds.
        if let Some(destination) = assign_to {
            self.assign_value(destination, target_obj.clone())?;
        }
        let singleton_cls = self.singleton_class_of(&target_obj);
        // A frozen object's singleton class is frozen with it.
        if self.object_is_frozen(&target_obj) {
            singleton_cls.freeze();
        }
        self.run_singleton_class_body(&singleton_cls, body, position)
    }

    /// Run a `class << x` body with the singleton class as both the lexical
    /// def scope and `self`, so a nested `def` lands on it. The body's own
    /// last value is the result.
    fn run_singleton_class_body(
        &mut self,
        singleton_cls: &Rc<Class>,
        body: &[Statement],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let prev_self = self.environment().get("self");
        self.environment_mut()
            .define_named("self", Object::Class(Rc::clone(singleton_cls)));
        self.def_scope_stack.push(Rc::clone(singleton_cls));

        let apply_result = self.apply_written_class_body(singleton_cls, body, position);
        let last = match &apply_result {
            Ok(value) => value.clone(),
            Err(_) => Object::Nil,
        };

        self.def_scope_stack.pop();
        if let Some(prev) = prev_self {
            self.environment_mut().define_named("self", prev);
        } else {
            self.environment_mut().undefine("self");
        }

        apply_result?;
        Ok(last)
    }
}
