//! Instruction formats and ways to dispatch them, run on the kernels of
//! kernels.rs over the 8-byte tagged value:
//!
//! - a stack machine whose instructions are an opcode byte followed by its
//!   operand bytes, as `src/bytecode` lays them out,
//! - the same stack machine with 4-byte instructions,
//! - a register machine with 4-byte instructions, dispatched by `match`,
//! - the register machine's instructions decoded once into a function
//!   pointer and operands each, dispatched by calling the pointer.

use crate::kernels::Kernel;
use crate::machine::{self, Representation};
use crate::tagged_value::{Tagged, TaggedValue};
use std::collections::HashMap;

struct Frame {
    function: usize,
    return_to: usize,
    base: usize,
}

/// Each literal is an index into its function's constants.
#[derive(Clone, Copy)]
pub enum StackOp {
    Constant(u16),
    Load(u8),
    Store(u8),
    Add,
    Subtract,
    Multiply,
    LessThan,
    JumpUnless(u16),
    Jump(u16),
    /// Call function `.0` with `.1` arguments on the stack.
    Call(u8, u8),
    Return,
    NewObject(u8),
    GetSlot(u8),
    SetSlot(u8),
    NewArray,
    ArrayPush,
    ArrayGet,
}

pub struct StackFunction {
    code: Vec<StackOp>,
    bytes: Vec<u8>,
    constants: Vec<Tagged>,
    locals: u8,
}

/// Constants a function's instructions name by index, each kept once.
#[derive(Default)]
struct Constants {
    values: Vec<Tagged>,
    indexes: HashMap<u64, u16>,
}

impl Constants {
    fn index(&mut self, value: Tagged) -> u16 {
        let next = self.values.len() as u16;
        *self.indexes.entry(value.bits()).or_insert_with(|| {
            self.values.push(value);
            next
        })
    }
}

/// The kernel's program with each literal moved to its function's
/// constants, in 4-byte instructions and as a byte stream.
pub fn stack_program(kernel: &Kernel) -> Vec<StackFunction> {
    kernel
        .program
        .iter()
        .map(|function| {
            let mut constants = Constants::default();
            let code: Vec<StackOp> = function
                .code
                .iter()
                .map(|op| match *op {
                    machine::Op::Integer(number) => {
                        StackOp::Constant(constants.index(TaggedValue::integer(number)))
                    }
                    machine::Op::Float(number) => {
                        StackOp::Constant(constants.index(TaggedValue::float(number)))
                    }
                    machine::Op::Load(slot) => StackOp::Load(slot),
                    machine::Op::Store(slot) => StackOp::Store(slot),
                    machine::Op::Add => StackOp::Add,
                    machine::Op::Subtract => StackOp::Subtract,
                    machine::Op::Multiply => StackOp::Multiply,
                    machine::Op::LessThan => StackOp::LessThan,
                    machine::Op::JumpUnless(target) => StackOp::JumpUnless(target as u16),
                    machine::Op::Jump(target) => StackOp::Jump(target as u16),
                    machine::Op::Call(function, arguments) => {
                        StackOp::Call(function as u8, arguments)
                    }
                    machine::Op::Return => StackOp::Return,
                    machine::Op::NewObject(slots) => StackOp::NewObject(slots),
                    machine::Op::GetSlot(slot) => StackOp::GetSlot(slot),
                    machine::Op::SetSlot(slot) => StackOp::SetSlot(slot),
                    machine::Op::NewArray => StackOp::NewArray,
                    machine::Op::ArrayPush => StackOp::ArrayPush,
                    machine::Op::ArrayGet => StackOp::ArrayGet,
                })
                .collect();
            StackFunction {
                bytes: byte_stream(&code),
                code,
                constants: constants.values,
                locals: function.locals,
            }
        })
        .collect()
}

mod opcode {
    pub const CONSTANT: u8 = 0;
    pub const LOAD: u8 = 1;
    pub const STORE: u8 = 2;
    pub const ADD: u8 = 3;
    pub const SUBTRACT: u8 = 4;
    pub const MULTIPLY: u8 = 5;
    pub const LESS_THAN: u8 = 6;
    pub const JUMP_UNLESS: u8 = 7;
    pub const JUMP: u8 = 8;
    pub const CALL: u8 = 9;
    pub const RETURN: u8 = 10;
    pub const NEW_OBJECT: u8 = 11;
    pub const GET_SLOT: u8 = 12;
    pub const SET_SLOT: u8 = 13;
    pub const NEW_ARRAY: u8 = 14;
    pub const ARRAY_PUSH: u8 = 15;
    pub const ARRAY_GET: u8 = 16;
}

/// The bytes one instruction takes, its opcode and its operands.
fn byte_length(op: &StackOp) -> usize {
    match op {
        StackOp::Constant(_) | StackOp::JumpUnless(_) | StackOp::Jump(_) | StackOp::Call(..) => 3,
        StackOp::Load(_)
        | StackOp::Store(_)
        | StackOp::NewObject(_)
        | StackOp::GetSlot(_)
        | StackOp::SetSlot(_) => 2,
        _ => 1,
    }
}

/// An opcode byte followed by its operands, a wide one big-endian, with
/// jump targets turned from instruction indexes into byte offsets.
fn byte_stream(code: &[StackOp]) -> Vec<u8> {
    let mut offsets = Vec::with_capacity(code.len());
    let mut offset = 0;
    for op in code {
        offsets.push(offset as u16);
        offset += byte_length(op);
    }
    let mut bytes = Vec::with_capacity(offset);
    for op in code {
        match *op {
            StackOp::Constant(index) => {
                bytes.push(opcode::CONSTANT);
                bytes.extend(index.to_be_bytes());
            }
            StackOp::Load(slot) => bytes.extend([opcode::LOAD, slot]),
            StackOp::Store(slot) => bytes.extend([opcode::STORE, slot]),
            StackOp::Add => bytes.push(opcode::ADD),
            StackOp::Subtract => bytes.push(opcode::SUBTRACT),
            StackOp::Multiply => bytes.push(opcode::MULTIPLY),
            StackOp::LessThan => bytes.push(opcode::LESS_THAN),
            StackOp::JumpUnless(target) => {
                bytes.push(opcode::JUMP_UNLESS);
                bytes.extend(offsets[target as usize].to_be_bytes());
            }
            StackOp::Jump(target) => {
                bytes.push(opcode::JUMP);
                bytes.extend(offsets[target as usize].to_be_bytes());
            }
            StackOp::Call(function, arguments) => bytes.extend([opcode::CALL, function, arguments]),
            StackOp::Return => bytes.push(opcode::RETURN),
            StackOp::NewObject(slots) => bytes.extend([opcode::NEW_OBJECT, slots]),
            StackOp::GetSlot(slot) => bytes.extend([opcode::GET_SLOT, slot]),
            StackOp::SetSlot(slot) => bytes.extend([opcode::SET_SLOT, slot]),
            StackOp::NewArray => bytes.push(opcode::NEW_ARRAY),
            StackOp::ArrayPush => bytes.push(opcode::ARRAY_PUSH),
            StackOp::ArrayGet => bytes.push(opcode::ARRAY_GET),
        }
    }
    bytes
}

fn binary(operation: u8, left: Tagged, right: Tagged) -> Tagged {
    match operation {
        opcode::ADD => TaggedValue::add(left, right),
        opcode::SUBTRACT => TaggedValue::subtract(left, right),
        opcode::MULTIPLY => TaggedValue::multiply(left, right),
        _ => TaggedValue::less_than(left, right),
    }
}

/// Run the 4-byte stack instructions, answering what function 0 returns
/// and, when `COUNT`, how many instructions ran.
pub fn run_stack<const COUNT: bool>(program: &[StackFunction]) -> (Tagged, u64) {
    let mut stack: Vec<Tagged> = Vec::with_capacity(1024);
    for _ in 0..program[0].locals {
        stack.push(TaggedValue::nil());
    }
    let mut frames = vec![Frame {
        function: 0,
        return_to: 0,
        base: 0,
    }];
    let mut function = &program[0];
    let mut base = 0;
    let mut at = 0;
    let mut executed = 0;
    loop {
        let op = function.code[at];
        at += 1;
        if COUNT {
            executed += 1;
        }
        match op {
            StackOp::Constant(index) => stack.push(function.constants[index as usize].clone()),
            StackOp::Load(slot) => stack.push(stack[base + slot as usize].clone()),
            StackOp::Store(slot) => {
                let value = stack.pop().unwrap();
                stack[base + slot as usize] = value;
            }
            StackOp::Add | StackOp::Subtract | StackOp::Multiply | StackOp::LessThan => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                let operation = match op {
                    StackOp::Add => opcode::ADD,
                    StackOp::Subtract => opcode::SUBTRACT,
                    StackOp::Multiply => opcode::MULTIPLY,
                    _ => opcode::LESS_THAN,
                };
                stack.push(binary(operation, left, right));
            }
            StackOp::JumpUnless(target) => {
                if !TaggedValue::truthy(&stack.pop().unwrap()) {
                    at = target as usize;
                }
            }
            StackOp::Jump(target) => at = target as usize,
            StackOp::Call(callee, arguments) => {
                frames.push(Frame {
                    function: callee as usize,
                    return_to: at,
                    base,
                });
                function = &program[callee as usize];
                base = stack.len() - arguments as usize;
                for _ in 0..function.locals {
                    stack.push(TaggedValue::nil());
                }
                at = 0;
            }
            StackOp::Return => {
                let answer = stack.pop().unwrap();
                let finished = frames.pop().unwrap();
                if frames.is_empty() {
                    return (answer, executed);
                }
                stack.truncate(base);
                stack.push(answer);
                base = finished.base;
                at = finished.return_to;
                function = &program[frames.last().unwrap().function];
            }
            StackOp::NewObject(slots) => stack.push(TaggedValue::new_object(slots as usize)),
            StackOp::GetSlot(slot) => {
                let object = stack.pop().unwrap();
                stack.push(TaggedValue::get_slot(&object, slot as usize));
            }
            StackOp::SetSlot(slot) => {
                let value = stack.pop().unwrap();
                let object = stack.pop().unwrap();
                TaggedValue::set_slot(&object, slot as usize, value);
            }
            StackOp::NewArray => stack.push(TaggedValue::new_array()),
            StackOp::ArrayPush => {
                let value = stack.pop().unwrap();
                let array = stack.pop().unwrap();
                TaggedValue::array_push(&array, value);
            }
            StackOp::ArrayGet => {
                let index = stack.pop().unwrap();
                let array = stack.pop().unwrap();
                stack.push(TaggedValue::array_get(&array, &index));
            }
        }
    }
}

/// Run the stack instructions as a byte stream, decoding each opcode and
/// its operands as it reaches them.
pub fn run_bytes(program: &[StackFunction]) -> Tagged {
    let mut stack: Vec<Tagged> = Vec::with_capacity(1024);
    for _ in 0..program[0].locals {
        stack.push(TaggedValue::nil());
    }
    let mut frames = vec![Frame {
        function: 0,
        return_to: 0,
        base: 0,
    }];
    let mut function = &program[0];
    let mut base = 0;
    let mut at = 0;
    loop {
        let bytes = &function.bytes;
        let code = bytes[at];
        at += 1;
        match code {
            opcode::CONSTANT => {
                let index = u16::from_be_bytes([bytes[at], bytes[at + 1]]);
                at += 2;
                stack.push(function.constants[index as usize].clone());
            }
            opcode::LOAD => {
                let slot = bytes[at];
                at += 1;
                stack.push(stack[base + slot as usize].clone());
            }
            opcode::STORE => {
                let slot = bytes[at];
                at += 1;
                let value = stack.pop().unwrap();
                stack[base + slot as usize] = value;
            }
            opcode::ADD | opcode::SUBTRACT | opcode::MULTIPLY | opcode::LESS_THAN => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                stack.push(binary(code, left, right));
            }
            opcode::JUMP_UNLESS => {
                let target = u16::from_be_bytes([bytes[at], bytes[at + 1]]);
                at += 2;
                if !TaggedValue::truthy(&stack.pop().unwrap()) {
                    at = target as usize;
                }
            }
            opcode::JUMP => at = u16::from_be_bytes([bytes[at], bytes[at + 1]]) as usize,
            opcode::CALL => {
                let callee = bytes[at] as usize;
                let arguments = bytes[at + 1] as usize;
                at += 2;
                frames.push(Frame {
                    function: callee,
                    return_to: at,
                    base,
                });
                function = &program[callee];
                base = stack.len() - arguments;
                for _ in 0..function.locals {
                    stack.push(TaggedValue::nil());
                }
                at = 0;
            }
            opcode::RETURN => {
                let answer = stack.pop().unwrap();
                let finished = frames.pop().unwrap();
                if frames.is_empty() {
                    return answer;
                }
                stack.truncate(base);
                stack.push(answer);
                base = finished.base;
                at = finished.return_to;
                function = &program[frames.last().unwrap().function];
            }
            opcode::NEW_OBJECT => {
                let slots = bytes[at];
                at += 1;
                stack.push(TaggedValue::new_object(slots as usize));
            }
            opcode::GET_SLOT => {
                let slot = bytes[at];
                at += 1;
                let object = stack.pop().unwrap();
                stack.push(TaggedValue::get_slot(&object, slot as usize));
            }
            opcode::SET_SLOT => {
                let slot = bytes[at];
                at += 1;
                let value = stack.pop().unwrap();
                let object = stack.pop().unwrap();
                TaggedValue::set_slot(&object, slot as usize, value);
            }
            opcode::NEW_ARRAY => stack.push(TaggedValue::new_array()),
            opcode::ARRAY_PUSH => {
                let value = stack.pop().unwrap();
                let array = stack.pop().unwrap();
                TaggedValue::array_push(&array, value);
            }
            opcode::ARRAY_GET => {
                let index = stack.pop().unwrap();
                let array = stack.pop().unwrap();
                stack.push(TaggedValue::array_get(&array, &index));
            }
            _ => panic!("no opcode {code}"),
        }
    }
}

/// Every operand names a register of the frame, numbered from its first
/// argument. A literal is loaded into a register from the function's
/// constants.
#[derive(Clone, Copy)]
pub enum RegisterOp {
    /// Register `.0` gets constant `.1`.
    Constant(u8, u16),
    /// Register `.0` gets register `.1`.
    Move(u8, u8),
    /// Register `.0` gets register `.1` plus register `.2`.
    Add(u8, u8, u8),
    Subtract(u8, u8, u8),
    Multiply(u8, u8, u8),
    LessThan(u8, u8, u8),
    /// Jump to `.1` unless register `.0` is true.
    JumpUnless(u8, u16),
    Jump(u16),
    /// Call function `.1` with its arguments in registers from `.0` on,
    /// which become the callee's first registers. The answer goes to `.0`.
    Call(u8, u8),
    Return(u8),
    NewObject(u8, u8),
    /// Register `.0` gets slot `.2` of the object in register `.1`.
    GetSlot(u8, u8, u8),
    /// Slot `.1` of the object in register `.0` gets register `.2`.
    SetSlot(u8, u8, u8),
    NewArray(u8),
    /// Push register `.1` onto the array in register `.0`.
    ArrayPush(u8, u8),
    /// Register `.0` gets the element of the array in register `.1` at the
    /// index in register `.2`.
    ArrayGet(u8, u8, u8),
}

pub struct RegisterFunction {
    code: Vec<RegisterOp>,
    constants: Vec<Tagged>,
    parameters: u8,
    registers: u8,
}

/// The register file: CRuby 4.0's `thread_vm_stack_size` in 8-byte values.
const REGISTER_FILE: usize = 1024 * 1024 / 8;

/// An operand of an arithmetic or comparison instruction at or past this
/// names constant `operand - CONSTANT_OPERAND` when the program is built
/// with constant operands.
const CONSTANT_OPERAND: u8 = 128;

/// Run the register instructions, answering what function 0 returns and,
/// when `COUNT`, how many instructions ran. With `CONSTANT_OPERANDS`, an
/// arithmetic or comparison operand may name a constant.
pub fn run_registers<const COUNT: bool, const CONSTANT_OPERANDS: bool>(
    program: &[RegisterFunction],
) -> (Tagged, u64) {
    let mut registers: Vec<Tagged> = (0..REGISTER_FILE).map(|_| TaggedValue::nil()).collect();
    let mut frames = vec![Frame {
        function: 0,
        return_to: 0,
        base: 0,
    }];
    let mut function = &program[0];
    let mut base = 0;
    let mut at = 0;
    let mut executed = 0;
    loop {
        let op = function.code[at];
        at += 1;
        if COUNT {
            executed += 1;
        }
        match op {
            RegisterOp::Constant(to, index) => {
                registers[base + to as usize] = function.constants[index as usize].clone()
            }
            RegisterOp::Move(to, from) => {
                registers[base + to as usize] = registers[base + from as usize].clone()
            }
            RegisterOp::Add(to, left, right)
            | RegisterOp::Subtract(to, left, right)
            | RegisterOp::Multiply(to, left, right)
            | RegisterOp::LessThan(to, left, right) => {
                let operand = |number: u8| {
                    if CONSTANT_OPERANDS && number >= CONSTANT_OPERAND {
                        function.constants[(number - CONSTANT_OPERAND) as usize].clone()
                    } else {
                        registers[base + number as usize].clone()
                    }
                };
                let (left, right) = (operand(left), operand(right));
                registers[base + to as usize] = match op {
                    RegisterOp::Add(..) => TaggedValue::add(left, right),
                    RegisterOp::Subtract(..) => TaggedValue::subtract(left, right),
                    RegisterOp::Multiply(..) => TaggedValue::multiply(left, right),
                    _ => TaggedValue::less_than(left, right),
                };
            }
            RegisterOp::JumpUnless(condition, target) => {
                if !TaggedValue::truthy(&registers[base + condition as usize]) {
                    at = target as usize;
                }
            }
            RegisterOp::Jump(target) => at = target as usize,
            RegisterOp::Call(first, callee) => {
                frames.push(Frame {
                    function: callee as usize,
                    return_to: at,
                    base,
                });
                function = &program[callee as usize];
                base += first as usize;
                for slot in function.parameters..function.registers {
                    registers[base + slot as usize] = TaggedValue::nil();
                }
                at = 0;
            }
            RegisterOp::Return(from) => {
                let answer = registers[base + from as usize].clone();
                let finished = frames.pop().unwrap();
                if frames.is_empty() {
                    return (answer, executed);
                }
                registers[base] = answer;
                base = finished.base;
                at = finished.return_to;
                function = &program[frames.last().unwrap().function];
            }
            RegisterOp::NewObject(to, slots) => {
                registers[base + to as usize] = TaggedValue::new_object(slots as usize)
            }
            RegisterOp::GetSlot(to, object, slot) => {
                let value =
                    TaggedValue::get_slot(&registers[base + object as usize], slot as usize);
                registers[base + to as usize] = value;
            }
            RegisterOp::SetSlot(object, slot, from) => {
                let value = registers[base + from as usize].clone();
                TaggedValue::set_slot(&registers[base + object as usize], slot as usize, value);
            }
            RegisterOp::NewArray(to) => registers[base + to as usize] = TaggedValue::new_array(),
            RegisterOp::ArrayPush(array, from) => {
                let value = registers[base + from as usize].clone();
                TaggedValue::array_push(&registers[base + array as usize], value);
            }
            RegisterOp::ArrayGet(to, array, index) => {
                let value = TaggedValue::array_get(
                    &registers[base + array as usize],
                    &registers[base + index as usize],
                );
                registers[base + to as usize] = value;
            }
        }
    }
}

/// A register instruction decoded once: the function that runs it and its
/// operands.
#[derive(Clone, Copy)]
pub struct Decoded {
    handler: Handler,
    first: u8,
    second: u8,
    third: u8,
    wide: u16,
}

/// Runs one instruction, answering false when the program has finished.
type Handler = fn(&mut Machine, Decoded) -> bool;

pub struct Machine<'program> {
    program: &'program [RegisterFunction],
    decoded: &'program [Vec<Decoded>],
    registers: Vec<Tagged>,
    frames: Vec<Frame>,
    code: &'program [Decoded],
    constants: &'program [Tagged],
    base: usize,
    at: usize,
    answer: Option<Tagged>,
}

impl Machine<'_> {
    #[inline(always)]
    fn register(&self, number: u8) -> &Tagged {
        &self.registers[self.base + number as usize]
    }

    #[inline(always)]
    fn set(&mut self, number: u8, value: Tagged) {
        self.registers[self.base + number as usize] = value;
    }

    #[inline(always)]
    fn binary(&mut self, instruction: Decoded, operation: fn(Tagged, Tagged) -> Tagged) -> bool {
        let left = self.register(instruction.second).clone();
        let right = self.register(instruction.third).clone();
        self.set(instruction.first, operation(left, right));
        true
    }
}

fn constant(machine: &mut Machine, instruction: Decoded) -> bool {
    let value = machine.constants[instruction.wide as usize].clone();
    machine.set(instruction.first, value);
    true
}

fn move_register(machine: &mut Machine, instruction: Decoded) -> bool {
    let value = machine.register(instruction.second).clone();
    machine.set(instruction.first, value);
    true
}

fn add(machine: &mut Machine, instruction: Decoded) -> bool {
    machine.binary(instruction, TaggedValue::add)
}

fn subtract(machine: &mut Machine, instruction: Decoded) -> bool {
    machine.binary(instruction, TaggedValue::subtract)
}

fn multiply(machine: &mut Machine, instruction: Decoded) -> bool {
    machine.binary(instruction, TaggedValue::multiply)
}

fn less_than(machine: &mut Machine, instruction: Decoded) -> bool {
    machine.binary(instruction, TaggedValue::less_than)
}

fn jump_unless(machine: &mut Machine, instruction: Decoded) -> bool {
    if !TaggedValue::truthy(machine.register(instruction.first)) {
        machine.at = instruction.wide as usize;
    }
    true
}

fn jump(machine: &mut Machine, instruction: Decoded) -> bool {
    machine.at = instruction.wide as usize;
    true
}

fn call(machine: &mut Machine, instruction: Decoded) -> bool {
    let callee = instruction.second as usize;
    machine.frames.push(Frame {
        function: callee,
        return_to: machine.at,
        base: machine.base,
    });
    let function = &machine.program[callee];
    machine.base += instruction.first as usize;
    for slot in function.parameters..function.registers {
        machine.set(slot, TaggedValue::nil());
    }
    machine.code = &machine.decoded[callee];
    machine.constants = &function.constants;
    machine.at = 0;
    true
}

fn return_from(machine: &mut Machine, instruction: Decoded) -> bool {
    let answer = machine.register(instruction.first).clone();
    let finished = machine.frames.pop().unwrap();
    let Some(caller) = machine.frames.last() else {
        machine.answer = Some(answer);
        return false;
    };
    let caller = caller.function;
    machine.set(0, answer);
    machine.base = finished.base;
    machine.at = finished.return_to;
    machine.code = &machine.decoded[caller];
    machine.constants = &machine.program[caller].constants;
    true
}

fn new_object(machine: &mut Machine, instruction: Decoded) -> bool {
    machine.set(
        instruction.first,
        TaggedValue::new_object(instruction.second as usize),
    );
    true
}

fn get_slot(machine: &mut Machine, instruction: Decoded) -> bool {
    let value = TaggedValue::get_slot(
        machine.register(instruction.second),
        instruction.third as usize,
    );
    machine.set(instruction.first, value);
    true
}

fn set_slot(machine: &mut Machine, instruction: Decoded) -> bool {
    let value = machine.register(instruction.third).clone();
    TaggedValue::set_slot(
        machine.register(instruction.first),
        instruction.second as usize,
        value,
    );
    true
}

fn new_array(machine: &mut Machine, instruction: Decoded) -> bool {
    machine.set(instruction.first, TaggedValue::new_array());
    true
}

fn array_push(machine: &mut Machine, instruction: Decoded) -> bool {
    let value = machine.register(instruction.second).clone();
    TaggedValue::array_push(machine.register(instruction.first), value);
    true
}

fn array_get(machine: &mut Machine, instruction: Decoded) -> bool {
    let value = TaggedValue::array_get(
        machine.register(instruction.second),
        machine.register(instruction.third),
    );
    machine.set(instruction.first, value);
    true
}

/// Each function's instructions decoded into handlers.
pub fn decode(program: &[RegisterFunction]) -> Vec<Vec<Decoded>> {
    let decoded = |handler: Handler, first: u8, second: u8, third: u8, wide: u16| Decoded {
        handler,
        first,
        second,
        third,
        wide,
    };
    program
        .iter()
        .map(|function| {
            function
                .code
                .iter()
                .map(|op| match *op {
                    RegisterOp::Constant(to, index) => decoded(constant, to, 0, 0, index),
                    RegisterOp::Move(to, from) => decoded(move_register, to, from, 0, 0),
                    RegisterOp::Add(to, left, right) => decoded(add, to, left, right, 0),
                    RegisterOp::Subtract(to, left, right) => decoded(subtract, to, left, right, 0),
                    RegisterOp::Multiply(to, left, right) => decoded(multiply, to, left, right, 0),
                    RegisterOp::LessThan(to, left, right) => decoded(less_than, to, left, right, 0),
                    RegisterOp::JumpUnless(condition, target) => {
                        decoded(jump_unless, condition, 0, 0, target)
                    }
                    RegisterOp::Jump(target) => decoded(jump, 0, 0, 0, target),
                    RegisterOp::Call(first, callee) => decoded(call, first, callee, 0, 0),
                    RegisterOp::Return(from) => decoded(return_from, from, 0, 0, 0),
                    RegisterOp::NewObject(to, slots) => decoded(new_object, to, slots, 0, 0),
                    RegisterOp::GetSlot(to, object, slot) => decoded(get_slot, to, object, slot, 0),
                    RegisterOp::SetSlot(object, slot, from) => {
                        decoded(set_slot, object, slot, from, 0)
                    }
                    RegisterOp::NewArray(to) => decoded(new_array, to, 0, 0, 0),
                    RegisterOp::ArrayPush(array, from) => decoded(array_push, array, from, 0, 0),
                    RegisterOp::ArrayGet(to, array, index) => {
                        decoded(array_get, to, array, index, 0)
                    }
                })
                .collect()
        })
        .collect()
}

/// Run the decoded instructions, calling each one's handler.
pub fn run_handlers(program: &[RegisterFunction], decoded: &[Vec<Decoded>]) -> Tagged {
    let mut machine = Machine {
        program,
        decoded,
        registers: (0..REGISTER_FILE).map(|_| TaggedValue::nil()).collect(),
        frames: vec![Frame {
            function: 0,
            return_to: 0,
            base: 0,
        }],
        code: &decoded[0],
        constants: &program[0].constants,
        base: 0,
        at: 0,
        answer: None,
    };
    loop {
        let instruction = machine.code[machine.at];
        machine.at += 1;
        if !(instruction.handler)(&mut machine, instruction) {
            return machine.answer.take().unwrap();
        }
    }
}

/// Register instructions with named places to jump to, resolved when
/// finished.
#[derive(Default)]
struct Assembler {
    constant_operands: bool,
    code: Vec<RegisterOp>,
    constants: Constants,
    places: HashMap<&'static str, u16>,
    jumps: Vec<(usize, &'static str)>,
}

impl Assembler {
    fn op(&mut self, op: RegisterOp) -> &mut Self {
        self.code.push(op);
        self
    }

    fn integer(&mut self, to: u8, number: i64) -> &mut Self {
        let index = self.constants.index(TaggedValue::integer(number));
        self.op(RegisterOp::Constant(to, index))
    }

    fn float(&mut self, to: u8, number: f64) -> &mut Self {
        let index = self.constants.index(TaggedValue::float(number));
        self.op(RegisterOp::Constant(to, index))
    }

    /// An operand for the integer: the constant itself with constant
    /// operands, or else register `scratch` with the constant loaded into
    /// it.
    fn integer_operand(&mut self, scratch: u8, number: i64) -> u8 {
        let index = self.constants.index(TaggedValue::integer(number));
        self.operand(scratch, index)
    }

    fn float_operand(&mut self, scratch: u8, number: f64) -> u8 {
        let index = self.constants.index(TaggedValue::float(number));
        self.operand(scratch, index)
    }

    fn operand(&mut self, scratch: u8, index: u16) -> u8 {
        if self.constant_operands {
            return CONSTANT_OPERAND + index as u8;
        }
        self.op(RegisterOp::Constant(scratch, index));
        scratch
    }

    fn place(&mut self, name: &'static str) -> &mut Self {
        self.places.insert(name, self.code.len() as u16);
        self
    }

    fn jump(&mut self, name: &'static str) -> &mut Self {
        self.jumps.push((self.code.len(), name));
        self.op(RegisterOp::Jump(0))
    }

    fn jump_unless(&mut self, condition: u8, name: &'static str) -> &mut Self {
        self.jumps.push((self.code.len(), name));
        self.op(RegisterOp::JumpUnless(condition, 0))
    }

    fn finish(&mut self, parameters: u8, registers: u8) -> RegisterFunction {
        let mut code = std::mem::take(&mut self.code);
        for (at, name) in &self.jumps {
            let target = self.places[name];
            code[*at] = match code[*at] {
                RegisterOp::JumpUnless(condition, _) => RegisterOp::JumpUnless(condition, target),
                _ => RegisterOp::Jump(target),
            };
        }
        RegisterFunction {
            code,
            constants: std::mem::take(&mut self.constants).values,
            parameters,
            registers,
        }
    }
}

/// `while i < count` around `body`, with `i` in register `counter` and
/// register `scratch` holding the comparison and, without constant
/// operands, each literal where it is used.
fn counted_loop(
    code: &mut Assembler,
    counter: u8,
    scratch: u8,
    count: i64,
    body: impl FnOnce(&mut Assembler),
) {
    code.integer(counter, 0).place("top");
    let limit = code.integer_operand(scratch, count);
    code.op(RegisterOp::LessThan(scratch, counter, limit))
        .jump_unless(scratch, "done");
    body(code);
    let one = code.integer_operand(scratch, 1);
    code.op(RegisterOp::Add(counter, counter, one))
        .jump("top")
        .place("done");
}

/// The register program for one of kernels.rs's kernels, computing what it
/// computes, with or without constant operands.
pub fn register_program(name: &str, constant_operands: bool) -> Vec<RegisterFunction> {
    let assembler = || Assembler {
        constant_operands,
        ..Assembler::default()
    };
    match name {
        "fib" => {
            let mut main = assembler();
            main.integer(0, 30)
                .op(RegisterOp::Call(0, 1))
                .op(RegisterOp::Return(0));
            let mut fib = assembler();
            let two = fib.integer_operand(1, 2);
            fib.op(RegisterOp::LessThan(2, 0, two))
                .jump_unless(2, "recurse")
                .op(RegisterOp::Return(0))
                .place("recurse");
            let one = fib.integer_operand(1, 1);
            fib.op(RegisterOp::Subtract(1, 0, one))
                .op(RegisterOp::Call(1, 1));
            let two = fib.integer_operand(2, 2);
            fib.op(RegisterOp::Subtract(2, 0, two))
                .op(RegisterOp::Call(2, 1))
                .op(RegisterOp::Add(1, 1, 2))
                .op(RegisterOp::Return(1));
            vec![main.finish(0, 1), fib.finish(1, 3)]
        }
        "while_loop" => {
            let mut main = assembler();
            main.integer(1, 0);
            counted_loop(&mut main, 0, 2, 50_000_000, |code| {
                code.op(RegisterOp::Add(1, 1, 0));
            });
            main.op(RegisterOp::Return(1));
            vec![main.finish(0, 3)]
        }
        "getivar" => {
            let mut main = assembler();
            main.op(RegisterOp::NewObject(0, 4));
            for slot in 0..4u8 {
                main.integer(3, slot as i64 + 1)
                    .op(RegisterOp::SetSlot(0, slot, 3));
            }
            main.integer(2, 0);
            counted_loop(&mut main, 1, 3, 20_000_000, |code| {
                code.op(RegisterOp::GetSlot(3, 0, 3))
                    .op(RegisterOp::Add(2, 2, 3));
            });
            main.op(RegisterOp::Return(2));
            vec![main.finish(0, 4)]
        }
        "object_new" => {
            let mut main = assembler();
            counted_loop(&mut main, 0, 2, 10_000_000, |code| {
                code.op(RegisterOp::NewObject(1, 2))
                    .op(RegisterOp::SetSlot(1, 0, 0))
                    .op(RegisterOp::SetSlot(1, 1, 0));
            });
            main.op(RegisterOp::Return(0));
            vec![main.finish(0, 3)]
        }
        "float_arithmetic" => {
            let mut main = assembler();
            main.float(0, 0.0);
            counted_loop(&mut main, 1, 2, 20_000_000, |code| {
                let half = code.float_operand(2, 0.5);
                code.op(RegisterOp::Multiply(0, 0, half));
                let step = code.float_operand(2, 1.25);
                code.op(RegisterOp::Add(0, 0, step));
            });
            main.op(RegisterOp::Return(0));
            vec![main.finish(0, 3)]
        }
        "array_sum" => {
            let mut main = assembler();
            main.op(RegisterOp::NewArray(0));
            counted_loop(&mut main, 1, 2, 5_000_000, |code| {
                code.op(RegisterOp::ArrayPush(0, 1));
            });
            main.op(RegisterOp::Move(3, 0))
                .op(RegisterOp::Call(3, 1))
                .op(RegisterOp::Return(3));
            let mut second = assembler();
            second.integer(2, 0);
            counted_loop(&mut second, 1, 3, 5_000_000, |code| {
                code.op(RegisterOp::ArrayGet(3, 0, 1))
                    .op(RegisterOp::Add(2, 2, 3));
            });
            second.op(RegisterOp::Return(2));
            vec![main.finish(0, 4), second.finish(1, 4)]
        }
        _ => panic!("no register program for {name}"),
    }
}
