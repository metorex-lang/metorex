// A method held as an object, and whether the receiver answers a name
// at all.

use super::*;
use crate::object::{OptionView, View};

impl VirtualMachine {
    /// A method held as an object, and whether the receiver answers a name
    /// at all.
    pub(crate) fn call_object_method_lookup_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `obj.method(:name)` returns a Method object bound to obj.
            // `public_method` is the same lookup with private and protected
            // names refused.
            "method" | "public_method" => {
                // A class that defines `def self.method` or `def self.
                // public_method` of its own owns the name; the class-method
                // table records it under the `__class__` convention, which
                // the caller checks after this returns None.
                if let View::Class(class_rc) | View::Module(class_rc) = receiver.view()
                    && class_rc
                        .find_method(&format!("__class__{}", method_name))
                        .is_some()
                {
                    return Ok(None);
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = self.coerce_name_argument(&arguments[0], position)?;
                // Full lookup so singleton methods (`class << obj`,
                // `def self.foo`) and mixins resolve, not just the class's
                // own method table.
                let public_only = method_name == "public_method";
                // `public_method` refuses a method the receiver keeps to
                // itself, saying how it keeps it.
                if public_only && self.method_is_restricted(receiver, &name_str) {
                    let kept = match self.visibility_owner(receiver, &name_str) {
                        Some(owner) if owner.is_method_protected(&name_str) => "protected",
                        _ => "private",
                    };
                    let message = format!(
                        "method '{}' for class '{}' is {}",
                        name_str,
                        self.builtins().class_of(receiver).ruby_name(),
                        kept
                    );
                    return Err(crate::vm::errors::simple_exception(
                        "NameError",
                        &message,
                        position,
                    ));
                }
                // A method the receiver's class supplies natively is that
                // class's own, even when Enumerable declares the same name.
                let shadowed_by_enumerable = self.enumerable_stands_in(receiver, &name_str);
                // A refinement in force here is what the name stands for
                // while it lasts, so the method object answers to it.
                let refined = crate::vm::method_lookup::refinement_target_name(receiver, self)
                    .and_then(|target| self.find_refined_method(&target, &name_str));
                if let Some(refined) = refined {
                    let owner = self.builtins().class_of(receiver);
                    let mut bound = (*refined).clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    bound.owner = Some(owner.ruby_name());
                    bound.owner_class = Some(owner);
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
                // nil, true and false share Object's class record, so a method
                // their own class defines is found by that class's name.
                if let Some(named) = crate::builtin_classes::value_class_name(receiver)
                    && crate::vm::native_methods::object_methods::nearest_core_owner(
                        &[named.to_string()],
                        &name_str,
                    )
                    .is_some()
                    && self
                        .lookup_method(receiver, &name_str)
                        .is_none_or(|(_, found)| found.body.is_empty())
                    && let Some(View::Class(owner)) = self.globals().get(named).view()
                {
                    let mut stub = crate::object::Method::new(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                    );
                    stub.receiver = Some(Box::new(receiver.clone()));
                    stub.owner = Some(owner.ruby_name());
                    stub.provenance.original_name = Some(name_str.clone());
                    stub.owner_class = Some(owner.clone());
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                if let Some((resolved_class, method)) = self
                    .lookup_method(receiver, &name_str)
                    .filter(|_| !shadowed_by_enumerable)
                {
                    let mut bound = (*method).clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    // Two names that are the same native method answer the
                    // same Method object, which is what makes
                    // `method(:to_s) == method(:inspect)` hold for an alias.
                    if let Some(target) =
                        native_alias_target(self.builtins().class_of(receiver).name(), &name_str)
                    {
                        if bound.provenance.original_name.is_none() {
                            bound.provenance.original_name = Some(target.to_string());
                        }
                        // The receiver's own class answers it, whatever the
                        // lookup walked past to get here.
                        let owner = self.builtins().class_of(receiver);
                        bound.owner = Some(owner.ruby_name());
                        bound.owner_class = Some(owner);
                    }
                    // `public :name` inside a class makes that class the
                    // owner of the method it opened up, whichever ancestor
                    // wrote it. The ancestor that wrote it is kept, since
                    // that is where `super` carries on from.
                    if resolved_class.has_public_override(&name_str) {
                        if bound.origin_class.is_none() {
                            bound.origin_class = bound.owner_class.clone();
                        }
                        bound.owner = Some(resolved_class.ruby_name());
                        bound.owner_class = Some(std::rc::Rc::clone(&resolved_class));
                    }
                    if bound.owner_class.is_none() {
                        // The lookup answers the class it resolved through,
                        // which for a singleton class is where the walk began
                        // rather than where the method is written. The one
                        // that holds it is the one that owns it.
                        let holder = resolved_class
                            .find_method_with_owner(&name_str)
                            .map(|(held, _)| held)
                            .unwrap_or(resolved_class);
                        bound.owner = Some(holder.ruby_name());
                        bound.owner_class = Some(holder);
                    }
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
                // Module and Class instance methods are implemented natively
                // rather than living in a method table, so hand out a stub
                // carrying the right parameter list.
                if matches!(receiver.view(), View::Class(_) | View::Module(_))
                    && let Some(mut stub) =
                        crate::vm::native_methods::class_methods::native_module_method_stub(
                            &name_str,
                        )
                {
                    stub.receiver = Some(Box::new(receiver.clone()));
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // The Kernel methods every object carries are native too,
                // unless a class nearer the receiver defines the name, as
                // Symbol defines its own `inspect`.
                let receiver_class = Object::Class(self.builtins().class_of(receiver));
                let owners = self.ancestor_names(receiver_class, position)?;
                let kernel_owns_it = crate::vm::native_methods::object_methods::nearest_core_owner(
                    &owners, &name_str,
                )
                .is_none_or(|owner| matches!(owner, "Kernel" | "Object" | "BasicObject"));
                if kernel_owns_it
                    && let Some(mut stub) =
                        crate::vm::native_methods::class_methods::native_kernel_method_stub(
                            &name_str,
                        )
                {
                    stub.receiver = Some(Box::new(receiver.clone()));
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // A method the receiver's own class implements natively, such
                // as an operator on a number, is handed out as a stub bound to
                // the receiver. `invoke_method` runs the native one.
                let is_operator = matches!(
                    name_str.as_str(),
                    "+" | "-"
                        | "*"
                        | "/"
                        | "%"
                        | "**"
                        | "=="
                        | "!="
                        | "<"
                        | ">"
                        | "<="
                        | ">="
                        | "<=>"
                        | "==="
                        | "<<"
                        | ">>"
                        | "&"
                        | "|"
                        | "^"
                        | "[]"
                        | "[]="
                        | "-@"
                        | "+@"
                        | "~"
                );
                // `Klass.method(:new)` is Class#new, which takes any
                // arguments and hands them to `initialize`.
                if name_str == "new"
                    && let View::Class(_) = receiver.view()
                    && let Some(View::Class(class_class)) = self.globals().get("Class").view()
                {
                    let mut stub = crate::object::Method::new(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                    );
                    stub.signature.variadic_param = Some((0, "args".to_string()));
                    stub.receiver = Some(Box::new(receiver.clone()));
                    stub.owner = Some("Class".to_string());
                    stub.provenance.original_name = Some(name_str.clone());
                    stub.provenance.native_alias = Some(name_str.clone());
                    stub.owner_class = Some(class_class.clone());
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                if is_operator || self.responds_to(receiver, &name_str) {
                    let owner = self
                        .native_owner_of_exception_method(receiver, &name_str, position)?
                        .unwrap_or_else(|| self.builtins().class_of(receiver));
                    let mut stub = crate::object::Method::new(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                    );
                    stub.receiver = Some(Box::new(receiver.clone()));
                    stub.owner = Some(owner.ruby_name());
                    // Two names that are the same native method answer the
                    // same Method object, which is what makes `method(:to_s)
                    // == method(:inspect)` hold for an alias.
                    stub.provenance.original_name = Some(
                        native_alias_target(owner.name(), &name_str)
                            .unwrap_or(&name_str)
                            .to_string(),
                    );
                    stub.owner_class = Some(owner);
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // An object that answers `respond_to_missing?` for the name
                // has a method as far as Ruby is concerned, so hand out one
                // that routes through `method_missing`.
                if self.responds_via_missing(receiver, &name_str, !public_only, position)? {
                    let mut stub = method_missing_dispatcher(&name_str, receiver, position);
                    // The method belongs to the class that claimed the name,
                    // which is the receiver's own.
                    let owner = self.builtins().class_of(receiver);
                    stub.owner = Some(owner.ruby_name());
                    stub.owner_class = Some(owner);
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // A class or module is an object too, so Kernel's private
                // functions, such as `puts`, are methods it has.
                if matches!(receiver.view(), View::Class(_) | View::Module(_))
                    && !public_only
                    && crate::vm::native_methods::is_kernel_private_function(&name_str)
                    && let Some(View::Module(kernel)) = self.globals().get("Kernel").view()
                {
                    let mut stub = crate::object::Method::new(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                    );
                    stub.receiver = Some(Box::new(receiver.clone()));
                    stub.owner = Some("Kernel".to_string());
                    stub.provenance.original_name = Some(name_str.clone());
                    stub.owner_class = Some(kernel.clone());
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                let cls = self.builtins().class_of(receiver);
                let msg = format!("undefined method '{}' for class '{}'", name_str, cls.name());
                let exc = Object::exception("NameError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            // Kernel#respond_to_missing? — the default answer is false. A
            // class overrides it to claim names it handles through
            // `method_missing`.
            "respond_to_missing?" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(false)))
            }
            "respond_to?" => {
                // Accept String or Symbol method name, plus the optional
                // `include_private` flag.
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method_query = self.coerce_method_name(&arguments[0], method_name, position)?;
                let include_private = matches!(arguments.get(1), Some(value) if value.is_truthy());
                // A method a C extension defined as unimplemented on this
                // platform is there, and still answers false.
                let unimplemented =
                    self.lookup_method(receiver, &method_query)
                        .is_some_and(|(_, method)| {
                            method
                                .provenance
                                .c_function
                                .as_ref()
                                .is_some_and(crate::vm::capi::is_not_implemented)
                        });
                if unimplemented {
                    return Ok(Some(Object::Bool(false)));
                }
                if self.responds_to(receiver, &method_query)
                    && (include_private || !self.method_is_restricted(receiver, &method_query))
                {
                    return Ok(Some(Object::Bool(true)));
                }
                // A class or module has Kernel's functions as private methods,
                // as every object does.
                if include_private
                    && matches!(receiver.view(), View::Class(_) | View::Module(_))
                    && (crate::vm::native_methods::is_kernel_private_function(&method_query)
                        || crate::vm::native_methods::class_methods::BASIC_OBJECT_PRIVATE_METHODS
                            .contains(&method_query.as_str()))
                    && self
                        .lookup_method(receiver, &method_query)
                        .is_none_or(|(_, method)| !method.is_undefined)
                {
                    return Ok(Some(Object::Bool(true)));
                }
                // Ruby asks `respond_to_missing?` for anything the lookup
                // missed, passing the same private flag it was given.
                Ok(Some(Object::Bool(self.responds_via_missing(
                    receiver,
                    &method_query,
                    include_private,
                    position,
                )?)))
            }
            "nil?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(matches!(receiver.view(), View::Nil))))
            }
            "get_source" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let query = match arguments[0].view() {
                    View::String(s) => s.as_str().to_string(),
                    View::Symbol(s) => s.as_str().to_string(),
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                match self.lookup_method(receiver, &query) {
                    Some((_class, method)) => Ok(Some(Object::Method(method))),
                    None => Ok(Some(Object::Nil)),
                }
            }
            _ => Ok(None),
        }
    }
}

impl VirtualMachine {
    /// For an exception, the core class among its ancestors that answers
    /// `name` natively, as SystemExit does `status`, since every exception
    /// shares one class record.
    fn native_owner_of_exception_method(
        &mut self,
        receiver: &Object,
        name: &str,
        position: Position,
    ) -> Result<Option<std::rc::Rc<crate::class::Class>>, MetorexError> {
        let View::Exception(details) = receiver.view() else {
            return Ok(None);
        };
        let (class, type_name) = {
            let held = details.borrow();
            (held.class.clone(), held.exception_type.clone())
        };
        let class = match class {
            Some(class) => class,
            None => match self.globals().get(&type_name).view() {
                Some(View::Class(class)) => std::rc::Rc::clone(class),
                _ => return Ok(None),
            },
        };
        let owners = self.ancestor_names(Object::Class(class), position)?;
        let Some(owner) =
            crate::vm::native_methods::object_methods::nearest_core_owner(&owners, name)
        else {
            return Ok(None);
        };
        Ok(match self.globals().get(owner).view() {
            Some(View::Class(found)) => Some(std::rc::Rc::clone(found)),
            _ => None,
        })
    }
}
