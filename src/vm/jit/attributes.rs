//! A `receiver.name = value` machine code stores with `StoreTarget` that
//! reaches a method writing one instance variable from its one parameter,
//! as `attr_writer` and `Struct` define them, writes the variable's slot
//! itself rather than calling the method, while the receiver's class, its
//! shape, and the method state are the ones the site found the method
//! under. A `receiver.name` machine code calls with `Call` that reaches a
//! method answering one instance variable, as `attr_reader` and `Struct`
//! define them, reads the variable itself the same way.

use crate::ast::{Expression, Statement};
use crate::bytecode::instruction::Instruction;
use crate::bytecode::unit::{Receiver, Target};
use crate::class::{Class, method_state};
use crate::object::shape::Shape;
use crate::object::{Method, View};
use crate::vm::VirtualMachine;
use crate::vm::bytecode_runner::UnitRun;
use crate::vm::native_methods::class_answers_natively;
use crate::vm::native_methods::{ARRAY_SUBCLASS_VAR, HASH_SUBCLASS_VAR};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// The writer one `StoreTarget` found last.
#[derive(Default)]
pub(super) struct AttributeWriteSite {
    found: RefCell<Option<AttributeWrite>>,
}

/// The slot the writer found on instances of `class` with `shape` writes,
/// under `method_state`. `listed` holds the variable's name when the
/// interpreter records it among the names the program wrote, which the
/// instance must hold already.
struct AttributeWrite {
    class: Weak<Class>,
    shape: Rc<Shape>,
    method_state: u64,
    slot: usize,
    listed: Option<String>,
}

impl AttributeWrite {
    fn holds(&self, class: &Rc<Class>, shape: &Rc<Shape>, state: u64) -> bool {
        self.method_state == state
            && std::ptr::eq(self.class.as_ptr(), Rc::as_ptr(class))
            && Rc::ptr_eq(&self.shape, shape)
    }
}

/// The reader one `Call` found last in its call site's cache, with the
/// class it was kept for, the method state it was found under, and the
/// variable it reads.
#[derive(Default)]
pub(super) struct AttributeReadSite {
    found: RefCell<Option<AttributeRead>>,
}

struct AttributeRead {
    class: Weak<Class>,
    method: Weak<Method>,
    method_state: u64,
    variable: String,
}

/// The variable `method` answers, when its body answers one instance
/// variable and it takes no parameters.
pub(super) fn plain_reader(method: &Method) -> Option<&str> {
    let plain = method.parameters.is_empty()
        && method.default_parameters.is_empty()
        && method.keyword_parameters.is_empty()
        && method.keyword_rest_parameter.is_none()
        && method.block_parameter.is_none()
        && method.variadic_param.is_none()
        && method.native_alias.is_none()
        && method.receiver.is_none()
        && method.bound_self.is_none()
        && method.captured_vars.is_none()
        && !method.lambda_body
        && !method.is_undefined;
    if !plain {
        return None;
    }
    match &*method.body {
        [
            Statement::Return {
                value: Some(Expression::InstanceVariable { name, .. }),
                ..
            }
            | Statement::Expression {
                expression: Expression::InstanceVariable { name, .. },
                ..
            },
        ] => Some(name),
        _ => None,
    }
}

/// The variable `method` writes and whether it was written in the core
/// library, when its body writes one instance variable from its one
/// parameter and nothing else.
fn plain_writer(method: &Method) -> Option<(&str, bool)> {
    let [parameter] = method.parameters.as_slice() else {
        return None;
    };
    let plain = method.default_parameters.is_empty()
        && method.keyword_parameters.is_empty()
        && method.keyword_rest_parameter.is_none()
        && method.block_parameter.is_none()
        && method.variadic_param.is_none()
        && method.native_alias.is_none()
        && method.receiver.is_none()
        && method.bound_self.is_none()
        && method.captured_vars.is_none()
        && !method.lambda_body
        && !method.is_undefined;
    if !plain {
        return None;
    }
    let [
        Statement::Assignment {
            target: Expression::InstanceVariable { name, position },
            value: Expression::Identifier { name: read, .. },
            ..
        },
    ] = &*method.body
    else {
        return None;
    };
    (read == parameter).then_some((name.as_str(), position.prelude))
}

impl VirtualMachine {
    /// Write the value of the `StoreTarget` at `at` into the slot its
    /// writer writes, when the target is a writer called on an instance
    /// its class alone decides the calls of, which is not frozen, while
    /// nothing traces calls. False leaves the store to the interpreter.
    pub(super) fn write_attribute_through(
        &self,
        site: &AttributeWriteSite,
        run: &UnitRun,
        at: usize,
    ) -> bool {
        let unit = run.unit();
        let Instruction::StoreTarget { from, target } = unit.instructions[at] else {
            return false;
        };
        let Target::Setter {
            receiver,
            reader,
            names_self,
            ..
        } = &unit.targets[usize::from(target)]
        else {
            return false;
        };
        if !self.tracepoints.is_empty() {
            return false;
        }
        let receiver = run.register(*receiver);
        let Some(class) = self.class_deciding_dispatch(&receiver) else {
            return false;
        };
        let View::Instance(instance) = receiver.view() else {
            return false;
        };
        let state = method_state();
        let shape = Rc::clone(instance.borrow().instance_vars.shape());
        let kept = site
            .found
            .borrow()
            .as_ref()
            .is_some_and(|write| write.holds(&class, &shape, state));
        if !kept {
            let Some(write) = self.attribute_write(&class, &shape, reader, *names_self) else {
                return false;
            };
            *site.found.borrow_mut() = Some(write);
        }
        let found = site.found.borrow();
        let write = found.as_ref().expect("a writer was found");
        let mut instance = instance.borrow_mut();
        let unlisted = write
            .listed
            .as_ref()
            .is_some_and(|name| !instance.program_dunder_names.contains(name));
        // A variable the core library gave the object becomes the program's
        // once the program writes it, which the slow path records.
        if instance.frozen || unlisted || !instance.interpreter_names.is_empty() {
            return false;
        }
        instance
            .instance_vars
            .set_slot(write.slot, run.register(usize::from(from)));
        true
    }

    /// The slot the writer `reader=` of `class` writes on an instance with
    /// `shape`, when an explicit receiver may call it unless `names_self`,
    /// it is a plain writer, and the shape holds the variable and no value
    /// the instance stands for.
    fn attribute_write(
        &self,
        class: &Rc<Class>,
        shape: &Rc<Shape>,
        reader: &str,
        names_self: bool,
    ) -> Option<AttributeWrite> {
        let setter = format!("{reader}=");
        let method = class.find_method(&setter)?;
        if !names_self && self.refuses_explicit_receiver(class, &setter)
            || class_answers_natively(class, &setter, true)
        {
            return None;
        }
        let (variable, prelude) = plain_writer(&method)?;
        if shape.slot_of(ARRAY_SUBCLASS_VAR).is_some() || shape.slot_of(HASH_SUBCLASS_VAR).is_some()
        {
            return None;
        }
        Some(AttributeWrite {
            class: Rc::downgrade(class),
            shape: Rc::clone(shape),
            method_state: method_state(),
            slot: shape.slot_of(variable)?,
            listed: (variable.starts_with("__") && !prelude).then(|| variable.to_string()),
        })
    }
}

impl VirtualMachine {
    /// Answer the `Call` at `at` with the instance variable its reader
    /// answers, when the call passes nothing, its receiver is an instance
    /// its class alone decides the calls of, the method the call site's
    /// cache holds for that class is a reader no native method answers in
    /// place of, and nothing traces calls. False leaves the call to the
    /// interpreter.
    pub(super) fn read_attribute_through(
        &self,
        site: &AttributeReadSite,
        run: &mut UnitRun,
        at: usize,
    ) -> bool {
        let unit = run.unit();
        let Instruction::Call { base, site: called } = unit.instructions[at] else {
            return false;
        };
        let called = &unit.call_sites[usize::from(called)];
        let passes_nothing = called.argument_count == 0
            && !called.arguments_in_list
            && !called.has_block
            && !called.block_argument
            && !called.sends_literal_name
            && called.receiver != Receiver::Sent;
        if !passes_nothing || !self.tracepoints.is_empty() {
            return false;
        }
        let receiver = run.register(usize::from(base));
        let Some(class) = self.class_deciding_dispatch(&receiver) else {
            return false;
        };
        let View::Instance(instance) = receiver.view() else {
            return false;
        };
        if self.was_moved(&receiver) {
            return false;
        }
        let Some(method) = called.cache.peek(&class) else {
            return false;
        };
        let state = method_state();
        let kept = site.found.borrow().as_ref().is_some_and(|read| {
            read.method_state == state
                && std::ptr::eq(read.class.as_ptr(), Rc::as_ptr(&class))
                && std::ptr::eq(read.method.as_ptr(), Rc::as_ptr(&method))
        });
        if !kept {
            let Some(variable) = plain_reader(&method) else {
                return false;
            };
            if class_answers_natively(&class, &called.name, true) {
                return false;
            }
            *site.found.borrow_mut() = Some(AttributeRead {
                class: Rc::downgrade(&class),
                method: Rc::downgrade(&method),
                method_state: state,
                variable: variable.to_string(),
            });
        }
        called.cache.count_hit();
        let found = site.found.borrow();
        let read = found.as_ref().expect("a reader was found");
        let value = instance
            .borrow()
            .instance_vars
            .get(&read.variable)
            .cloned()
            .unwrap_or(crate::object::Object::Nil);
        drop(found);
        run.set_register(usize::from(base), value);
        true
    }
}
