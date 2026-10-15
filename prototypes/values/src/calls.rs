//! Two ways to lay out frames, running the same instructions over the tagged
//! value. `Stacked` keeps every frame's locals in one contiguous value stack
//! and passes arguments in place, so a call pushes a fixed-size frame record
//! and allocates nothing. `HeapFrames` gives each call a frame of its own on
//! the heap: an argument vector, a locals vector, and an operand stack.
//!
//! Locals a block reads are kept in an environment the unit makes when it
//! starts, which the block, a Proc made from it, and a binding share. A unit
//! with no such locals makes none.

use crate::machine::Representation;
use crate::tagged_value::{Tagged, TaggedValue};
use std::cell::RefCell;
use std::rc::Rc;

/// `Integer#times`, a native method that calls its block from Rust.
pub const TIMES: u16 = u16::MAX;
/// `Proc#call`.
pub const PROC_CALL: u16 = u16::MAX - 1;

/// Values the contiguous stack holds, CRuby's 1 MB thread VM stack.
const STACK_VALUES: usize = 1024 * 1024 / 8;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Integer(i64),
    Nil,
    SelfValue,
    Dup,
    Pop,
    /// A local in the frame's own slots.
    Load(u8),
    /// Pop a value into a local in the frame's own slots.
    Store(u8),
    /// A local kept in an environment `.0` units out: 0 for the running
    /// unit's own, 1 for the unit that wrote the running block.
    LoadCaptured(u8, u8),
    /// Pop a value into a local kept in an environment.
    StoreCaptured(u8, u8),
    Add,
    Multiply,
    LessThan,
    Jump(u32),
    JumpUnless(u32),
    /// Push one of self's instance variables.
    GetIvar(u8),
    /// Pop a value into one of self's instance variables.
    SetIvar(u8),
    NewObject(u8),
    /// Call method `.0` on the receiver under `.1` arguments, with the block
    /// unit `.2`.
    Send(u16, u8, Option<u16>),
    /// Call the frame's block with `.0` arguments.
    Yield(u8),
    /// Push a Proc of block unit `.0`.
    MakeProc(u16),
    Return,
}

pub struct Unit {
    pub code: Vec<Op>,
    pub parameters: u8,
    /// Slots for the unit's own locals, parameters first.
    pub locals: u8,
    /// Locals a block reads, kept in the unit's environment.
    pub captured: u8,
    /// A block finds the environment of the unit that wrote it one out.
    pub block: bool,
}

impl Unit {
    /// How many environments out from the frame's own the first one is: a
    /// block with no environment of its own starts at its writer's.
    fn environment_skip(&self) -> u8 {
        u8::from(self.block && self.captured == 0)
    }
}

pub struct Environment {
    slots: RefCell<Vec<Tagged>>,
    outer: Option<Rc<Environment>>,
}

fn environment_out(start: &Option<Rc<Environment>>, hops: u8) -> &Rc<Environment> {
    let mut held = start.as_ref().unwrap();
    for _ in 0..hops {
        held = held.outer.as_ref().unwrap();
    }
    held
}

#[derive(Clone)]
struct BlockHandler {
    unit: u16,
    receiver: Tagged,
    environment: Option<Rc<Environment>>,
}

struct Frame {
    unit: u16,
    /// Where the caller resumes.
    return_to: u32,
    /// The stack index of local 0.
    base: u32,
    /// The stack length to cut back to on return, below the receiver for a
    /// method.
    drop_to: u32,
    receiver: Tagged,
    block: Option<BlockHandler>,
    environment: Option<Rc<Environment>>,
}

pub struct Stacked<'program> {
    units: &'program [Unit],
    stack: Vec<Tagged>,
    frames: Vec<Frame>,
    procs: Vec<BlockHandler>,
}

impl<'program> Stacked<'program> {
    /// Run unit 0 and answer what it returns.
    pub fn run(units: &'program [Unit]) -> Tagged {
        let mut machine = Stacked {
            units,
            stack: Vec::with_capacity(STACK_VALUES),
            frames: Vec::with_capacity(1024),
            procs: Vec::new(),
        };
        machine.enter(0, TaggedValue::nil(), 0, 0, None, None, 0);
        machine.execute(0)
    }

    /// Push a frame for `unit` whose `arguments` are the top of the stack.
    #[allow(clippy::too_many_arguments)]
    fn enter(
        &mut self,
        unit: u16,
        receiver: Tagged,
        arguments: usize,
        drop_to: usize,
        block: Option<BlockHandler>,
        outer: Option<Rc<Environment>>,
        return_to: u32,
    ) {
        let callee = &self.units[unit as usize];
        let base = self.stack.len() - arguments;
        for _ in callee.parameters..callee.locals {
            self.stack.push(TaggedValue::nil());
        }
        let environment = if callee.captured > 0 {
            Some(Rc::new(Environment {
                slots: RefCell::new(vec![TaggedValue::nil(); callee.captured as usize]),
                outer,
            }))
        } else {
            outer
        };
        self.frames.push(Frame {
            unit,
            return_to,
            base: base as u32,
            drop_to: drop_to as u32,
            receiver,
            block,
            environment,
        });
    }

    /// Run until a return leaves `stop_at` frames, and answer what it
    /// returned.
    fn execute(&mut self, stop_at: usize) -> Tagged {
        let units = self.units;
        let mut running = &units[self.frames.last().unwrap().unit as usize];
        let mut code = &running.code[..];
        let mut base = self.frames.last().unwrap().base as usize;
        let mut at = 0usize;
        loop {
            let op = code[at];
            at += 1;
            match op {
                Op::Integer(number) => self.stack.push(TaggedValue::integer(number)),
                Op::Nil => self.stack.push(TaggedValue::nil()),
                Op::SelfValue => {
                    let receiver = self.frames.last().unwrap().receiver.clone();
                    self.stack.push(receiver);
                }
                Op::Dup => self.stack.push(self.stack.last().unwrap().clone()),
                Op::Pop => {
                    self.stack.pop();
                }
                Op::Load(slot) => self.stack.push(self.stack[base + slot as usize].clone()),
                Op::Store(slot) => {
                    let value = self.stack.pop().unwrap();
                    self.stack[base + slot as usize] = value;
                }
                Op::LoadCaptured(depth, slot) => {
                    let frame = self.frames.last().unwrap();
                    let held =
                        environment_out(&frame.environment, depth - running.environment_skip());
                    let value = held.slots.borrow()[slot as usize].clone();
                    self.stack.push(value);
                }
                Op::StoreCaptured(depth, slot) => {
                    let value = self.stack.pop().unwrap();
                    let frame = self.frames.last().unwrap();
                    let held =
                        environment_out(&frame.environment, depth - running.environment_skip());
                    let replaced =
                        std::mem::replace(&mut held.slots.borrow_mut()[slot as usize], value);
                    drop(replaced);
                }
                Op::Add | Op::Multiply | Op::LessThan => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(match op {
                        Op::Add => TaggedValue::add(left, right),
                        Op::Multiply => TaggedValue::multiply(left, right),
                        _ => TaggedValue::less_than(left, right),
                    });
                }
                Op::Jump(target) => at = target as usize,
                Op::JumpUnless(target) => {
                    if !TaggedValue::truthy(&self.stack.pop().unwrap()) {
                        at = target as usize;
                    }
                }
                Op::GetIvar(slot) => {
                    let value =
                        TaggedValue::get_slot(&self.frames.last().unwrap().receiver, slot as usize);
                    self.stack.push(value);
                }
                Op::SetIvar(slot) => {
                    let value = self.stack.pop().unwrap();
                    TaggedValue::set_slot(
                        &self.frames.last().unwrap().receiver,
                        slot as usize,
                        value,
                    );
                }
                Op::NewObject(slots) => self.stack.push(TaggedValue::new_object(slots as usize)),
                Op::Send(method, arguments, block) => {
                    let arguments = arguments as usize;
                    let receiver_at = self.stack.len() - arguments - 1;
                    let handler = block.map(|unit| {
                        let frame = self.frames.last().unwrap();
                        BlockHandler {
                            unit,
                            receiver: frame.receiver.clone(),
                            environment: frame.environment.clone(),
                        }
                    });
                    match method {
                        TIMES => {
                            let handler = handler.unwrap();
                            let count = integer_of(&self.stack[receiver_at]);
                            for step in 0..count {
                                self.stack.push(TaggedValue::integer(step));
                                let drop_to = self.stack.len() - 1;
                                self.enter(
                                    handler.unit,
                                    handler.receiver.clone(),
                                    1,
                                    drop_to,
                                    None,
                                    handler.environment.clone(),
                                    0,
                                );
                                self.execute(self.frames.len() - 1);
                            }
                            self.stack.truncate(receiver_at + 1);
                        }
                        PROC_CALL => {
                            let made =
                                self.procs[integer_of(&self.stack[receiver_at]) as usize].clone();
                            self.enter(
                                made.unit,
                                made.receiver,
                                arguments,
                                receiver_at,
                                None,
                                made.environment,
                                at as u32,
                            );
                            running = &units[made.unit as usize];
                            code = &running.code;
                            base = self.frames.last().unwrap().base as usize;
                            at = 0;
                        }
                        _ => {
                            let receiver = self.stack[receiver_at].clone();
                            self.enter(
                                method,
                                receiver,
                                arguments,
                                receiver_at,
                                handler,
                                None,
                                at as u32,
                            );
                            running = &units[method as usize];
                            code = &running.code;
                            base = self.frames.last().unwrap().base as usize;
                            at = 0;
                        }
                    }
                }
                Op::Yield(arguments) => {
                    let arguments = arguments as usize;
                    let handler = self.frames.last().unwrap().block.clone().unwrap();
                    let drop_to = self.stack.len() - arguments;
                    self.enter(
                        handler.unit,
                        handler.receiver,
                        arguments,
                        drop_to,
                        None,
                        handler.environment,
                        at as u32,
                    );
                    running = &units[handler.unit as usize];
                    code = &running.code;
                    base = self.frames.last().unwrap().base as usize;
                    at = 0;
                }
                Op::MakeProc(unit) => {
                    let frame = self.frames.last().unwrap();
                    self.procs.push(BlockHandler {
                        unit,
                        receiver: frame.receiver.clone(),
                        environment: frame.environment.clone(),
                    });
                    self.stack
                        .push(TaggedValue::integer(self.procs.len() as i64 - 1));
                }
                Op::Return => {
                    let answer = self.stack.pop().unwrap();
                    let finished = self.frames.pop().unwrap();
                    self.stack.truncate(finished.drop_to as usize);
                    if self.frames.len() == stop_at {
                        return answer;
                    }
                    self.stack.push(answer);
                    let resumed = self.frames.last().unwrap();
                    running = &units[resumed.unit as usize];
                    code = &running.code;
                    base = resumed.base as usize;
                    at = finished.return_to as usize;
                }
            }
        }
    }
}

fn integer_of(value: &Tagged) -> i64 {
    value.fixnum_value()
}

struct HeapFrame {
    locals: Vec<Tagged>,
    captured: Vec<Tagged>,
    /// The frame of the unit that wrote this block.
    outer: Option<Rc<RefCell<HeapFrame>>>,
}

#[derive(Clone)]
struct HeapBlock {
    unit: u16,
    receiver: Tagged,
    frame: Rc<RefCell<HeapFrame>>,
}

pub struct HeapFrames<'program> {
    units: &'program [Unit],
    procs: Vec<HeapBlock>,
}

impl<'program> HeapFrames<'program> {
    /// Run unit 0 and answer what it returns.
    pub fn run(units: &'program [Unit]) -> Tagged {
        let mut machine = HeapFrames {
            units,
            procs: Vec::new(),
        };
        machine.call(0, TaggedValue::nil(), Vec::new(), None, None)
    }

    fn call(
        &mut self,
        unit: u16,
        receiver: Tagged,
        arguments: Vec<Tagged>,
        block: Option<HeapBlock>,
        outer: Option<Rc<RefCell<HeapFrame>>>,
    ) -> Tagged {
        let running = &self.units[unit as usize];
        let mut locals = Vec::with_capacity(running.locals as usize);
        locals.extend(arguments);
        locals.resize(running.locals as usize, TaggedValue::nil());
        let frame = Rc::new(RefCell::new(HeapFrame {
            locals,
            captured: vec![TaggedValue::nil(); running.captured as usize],
            outer,
        }));
        let reach = |depth: u8| {
            let mut held = frame.clone();
            for _ in 0..depth {
                let next = held.borrow().outer.clone().unwrap();
                held = next;
            }
            held
        };
        let mut stack: Vec<Tagged> = Vec::with_capacity(8);
        let code = &running.code;
        let mut at = 0usize;
        loop {
            let op = code[at];
            at += 1;
            match op {
                Op::Integer(number) => stack.push(TaggedValue::integer(number)),
                Op::Nil => stack.push(TaggedValue::nil()),
                Op::SelfValue => stack.push(receiver.clone()),
                Op::Dup => stack.push(stack.last().unwrap().clone()),
                Op::Pop => {
                    stack.pop();
                }
                Op::Load(slot) => stack.push(frame.borrow().locals[slot as usize].clone()),
                Op::Store(slot) => {
                    let value = stack.pop().unwrap();
                    let replaced =
                        std::mem::replace(&mut frame.borrow_mut().locals[slot as usize], value);
                    drop(replaced);
                }
                Op::LoadCaptured(depth, slot) => {
                    let value = reach(depth).borrow().captured[slot as usize].clone();
                    stack.push(value);
                }
                Op::StoreCaptured(depth, slot) => {
                    let value = stack.pop().unwrap();
                    let held = reach(depth);
                    let replaced =
                        std::mem::replace(&mut held.borrow_mut().captured[slot as usize], value);
                    drop(replaced);
                }
                Op::Add | Op::Multiply | Op::LessThan => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(match op {
                        Op::Add => TaggedValue::add(left, right),
                        Op::Multiply => TaggedValue::multiply(left, right),
                        _ => TaggedValue::less_than(left, right),
                    });
                }
                Op::Jump(target) => at = target as usize,
                Op::JumpUnless(target) => {
                    if !TaggedValue::truthy(&stack.pop().unwrap()) {
                        at = target as usize;
                    }
                }
                Op::GetIvar(slot) => stack.push(TaggedValue::get_slot(&receiver, slot as usize)),
                Op::SetIvar(slot) => {
                    let value = stack.pop().unwrap();
                    TaggedValue::set_slot(&receiver, slot as usize, value);
                }
                Op::NewObject(slots) => stack.push(TaggedValue::new_object(slots as usize)),
                Op::Send(method, arguments, block_unit) => {
                    let handed = stack.split_off(stack.len() - arguments as usize);
                    let callee_receiver = stack.pop().unwrap();
                    let handler = block_unit.map(|unit| HeapBlock {
                        unit,
                        receiver: receiver.clone(),
                        frame: frame.clone(),
                    });
                    let answer = match method {
                        TIMES => {
                            let handler = handler.unwrap();
                            for step in 0..integer_of(&callee_receiver) {
                                self.call(
                                    handler.unit,
                                    handler.receiver.clone(),
                                    vec![TaggedValue::integer(step)],
                                    None,
                                    Some(handler.frame.clone()),
                                );
                            }
                            callee_receiver
                        }
                        PROC_CALL => {
                            let made = self.procs[integer_of(&callee_receiver) as usize].clone();
                            self.call(made.unit, made.receiver, handed, None, Some(made.frame))
                        }
                        _ => self.call(method, callee_receiver, handed, handler, None),
                    };
                    stack.push(answer);
                }
                Op::Yield(arguments) => {
                    let handed = stack.split_off(stack.len() - arguments as usize);
                    let handler = block.clone().unwrap();
                    let answer = self.call(
                        handler.unit,
                        handler.receiver,
                        handed,
                        None,
                        Some(handler.frame),
                    );
                    stack.push(answer);
                }
                Op::MakeProc(unit) => {
                    self.procs.push(HeapBlock {
                        unit,
                        receiver: receiver.clone(),
                        frame: frame.clone(),
                    });
                    stack.push(TaggedValue::integer(self.procs.len() as i64 - 1));
                }
                Op::Return => return stack.pop().unwrap(),
            }
        }
    }
}

/// A unit's instructions with named places to jump to.
#[derive(Default)]
struct Assembler {
    code: Vec<Op>,
    places: std::collections::HashMap<&'static str, u32>,
    jumps: Vec<(usize, &'static str)>,
}

impl Assembler {
    fn op(&mut self, op: Op) -> &mut Self {
        self.code.push(op);
        self
    }

    fn place(&mut self, name: &'static str) -> &mut Self {
        self.places.insert(name, self.code.len() as u32);
        self
    }

    fn jump(&mut self, name: &'static str) -> &mut Self {
        self.jumps.push((self.code.len(), name));
        self.op(Op::Jump(0))
    }

    fn jump_unless(&mut self, name: &'static str) -> &mut Self {
        self.jumps.push((self.code.len(), name));
        self.op(Op::JumpUnless(0))
    }

    fn finish(&mut self, parameters: u8, locals: u8, captured: u8, block: bool) -> Unit {
        let mut code = std::mem::take(&mut self.code);
        for (at, name) in &self.jumps {
            let target = self.places[name];
            code[*at] = match code[*at] {
                Op::Jump(_) => Op::Jump(target),
                _ => Op::JumpUnless(target),
            };
        }
        Unit {
            code,
            parameters,
            locals,
            captured,
            block,
        }
    }
}

pub struct CallKernel {
    pub name: &'static str,
    /// Method and block calls a run makes.
    pub calls: usize,
    pub units: Vec<Unit>,
}

/// `benchmarks/workloads/method_calls.rb` with `count` steps:
///
/// ```ruby
/// counter = Counter.new
/// count.times { |step| counter.add(step, 2) }
/// counter.total
/// ```
pub fn method_calls(count: i64) -> CallKernel {
    const INITIALIZE: u16 = 1;
    const ADD: u16 = 2;
    const TOTAL: u16 = 3;
    const STEP_BLOCK: u16 = 4;
    let main = Assembler::default()
        .op(Op::NewObject(1))
        .op(Op::Dup)
        .op(Op::Send(INITIALIZE, 0, None))
        .op(Op::Pop)
        .op(Op::StoreCaptured(0, 0))
        .op(Op::Integer(count))
        .op(Op::Send(TIMES, 0, Some(STEP_BLOCK)))
        .op(Op::Pop)
        .op(Op::LoadCaptured(0, 0))
        .op(Op::Send(TOTAL, 0, None))
        .op(Op::Return)
        .finish(0, 0, 1, false);
    let initialize = Assembler::default()
        .op(Op::Integer(0))
        .op(Op::SetIvar(0))
        .op(Op::Nil)
        .op(Op::Return)
        .finish(0, 0, 0, false);
    let add = Assembler::default()
        .op(Op::GetIvar(0))
        .op(Op::Load(0))
        .op(Op::Load(1))
        .op(Op::Multiply)
        .op(Op::Add)
        .op(Op::Dup)
        .op(Op::SetIvar(0))
        .op(Op::Return)
        .finish(2, 2, 0, false);
    let total = Assembler::default()
        .op(Op::GetIvar(0))
        .op(Op::Return)
        .finish(0, 0, 0, false);
    let step_block = Assembler::default()
        .op(Op::LoadCaptured(1, 0))
        .op(Op::Load(0))
        .op(Op::Integer(2))
        .op(Op::Send(ADD, 2, None))
        .op(Op::Return)
        .finish(1, 1, 0, true);
    CallKernel {
        name: "method_calls",
        calls: 2 * count as usize,
        units: vec![main, initialize, add, total, step_block],
    }
}

/// `benchmarks/workloads/block_calls.rb` with `count` yields and half as
/// many Proc calls:
///
/// ```ruby
/// def each_square(limit)
///   index = 0
///   while index < limit
///     yield index * index
///     index += 1
///   end
/// end
///
/// total = 0
/// each_square(count) { |square| total += square }
/// adder = proc { |value| total += value }
/// (count / 2).times { |step| adder.call(step) }
/// total
/// ```
pub fn block_calls(count: i64) -> CallKernel {
    const EACH_SQUARE: u16 = 1;
    const SQUARE_BLOCK: u16 = 2;
    const ADDER_BLOCK: u16 = 3;
    const STEP_BLOCK: u16 = 4;
    let main = Assembler::default()
        .op(Op::Integer(0))
        .op(Op::StoreCaptured(0, 0))
        .op(Op::SelfValue)
        .op(Op::Integer(count))
        .op(Op::Send(EACH_SQUARE, 1, Some(SQUARE_BLOCK)))
        .op(Op::Pop)
        .op(Op::MakeProc(ADDER_BLOCK))
        .op(Op::StoreCaptured(0, 1))
        .op(Op::Integer(count / 2))
        .op(Op::Send(TIMES, 0, Some(STEP_BLOCK)))
        .op(Op::Pop)
        .op(Op::LoadCaptured(0, 0))
        .op(Op::Return)
        .finish(0, 0, 2, false);
    let each_square = Assembler::default()
        .op(Op::Integer(0))
        .op(Op::Store(1))
        .place("top")
        .op(Op::Load(1))
        .op(Op::Load(0))
        .op(Op::LessThan)
        .jump_unless("done")
        .op(Op::Load(1))
        .op(Op::Load(1))
        .op(Op::Multiply)
        .op(Op::Yield(1))
        .op(Op::Pop)
        .op(Op::Load(1))
        .op(Op::Integer(1))
        .op(Op::Add)
        .op(Op::Store(1))
        .jump("top")
        .place("done")
        .op(Op::Nil)
        .op(Op::Return)
        .finish(1, 2, 0, false);
    let adds_to_total = || {
        Assembler::default()
            .op(Op::LoadCaptured(1, 0))
            .op(Op::Load(0))
            .op(Op::Add)
            .op(Op::Dup)
            .op(Op::StoreCaptured(1, 0))
            .op(Op::Return)
            .finish(1, 1, 0, true)
    };
    let step_block = Assembler::default()
        .op(Op::LoadCaptured(1, 1))
        .op(Op::Load(0))
        .op(Op::Send(PROC_CALL, 1, None))
        .op(Op::Return)
        .finish(1, 1, 0, true);
    CallKernel {
        name: "block_calls",
        calls: count as usize + 2 * (count / 2) as usize + 1,
        units: vec![
            main,
            each_square,
            adds_to_total(),
            adds_to_total(),
            step_block,
        ],
    }
}
