//! An instance variable machine code writes with `SetInstanceVariable` is
//! written straight into the slot of `self` its site found last, when
//! `self` has the shape the site found it with, including a write that
//! adds the variable and moves `self` on to the next shape.

use crate::bytecode::instruction::Instruction;
use crate::object::View;
use crate::object::shape::Shape;
use crate::vm::VirtualMachine;
use crate::vm::bytecode_runner::UnitRun;
use std::cell::RefCell;
use std::rc::Rc;

/// The shape one `SetInstanceVariable` found `self` with, the shape the
/// write left it with, and the slot it wrote. The two shapes are the same
/// for a variable `self` held already.
#[derive(Default)]
pub(super) struct SlotWriteSite {
    found: RefCell<Option<SlotWrite>>,
}

struct SlotWrite {
    before: Rc<Shape>,
    after: Rc<Shape>,
    slot: usize,
}

impl VirtualMachine {
    /// Write the value of the `SetInstanceVariable` at `at` into its slot of
    /// `self`, when `self` is an instance that may be changed and the name
    /// is none of the interpreter's own. False leaves the write to the
    /// interpreter.
    pub(super) fn write_slot_through(
        &self,
        site: &SlotWriteSite,
        run: &UnitRun,
        at: usize,
    ) -> bool {
        let unit = run.unit();
        let Instruction::SetInstanceVariable { from, name } = unit.instructions[at] else {
            return false;
        };
        let name = &unit.names[usize::from(name)].text;
        if name.starts_with("__") {
            return false;
        }
        let Some(held) = self.environment().get("self") else {
            return false;
        };
        let View::Instance(instance) = held.view() else {
            return false;
        };
        let mut instance = instance.borrow_mut();
        if instance.frozen {
            return false;
        }
        let value = run.register(usize::from(from));
        let variables = &mut instance.instance_vars;
        let mut found = site.found.borrow_mut();
        match found.as_ref() {
            Some(write) if Rc::ptr_eq(&write.before, variables.shape()) => {
                if Rc::ptr_eq(&write.before, &write.after) {
                    variables.set_slot(write.slot, value);
                } else {
                    variables.push_into_shape(Rc::clone(&write.after), value);
                }
            }
            _ => {
                let before = Rc::clone(variables.shape());
                variables.insert(name.clone(), value);
                let slot = variables
                    .shape()
                    .slot_of(name)
                    .expect("a variable just written has a slot");
                *found = Some(SlotWrite {
                    before,
                    after: Rc::clone(variables.shape()),
                    slot,
                });
            }
        }
        true
    }
}
