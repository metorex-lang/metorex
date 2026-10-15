//! A stack interpreter generic over how a value is represented, so the two
//! representations run the same instructions, frames, and object layout and
//! differ only in the value itself.

/// What the interpreter needs of a value representation.
pub trait Representation {
    type Value: Clone;

    fn nil() -> Self::Value;
    fn integer(number: i64) -> Self::Value;
    fn float(number: f64) -> Self::Value;
    fn truthy(value: &Self::Value) -> bool;
    fn add(left: Self::Value, right: Self::Value) -> Self::Value;
    fn subtract(left: Self::Value, right: Self::Value) -> Self::Value;
    fn multiply(left: Self::Value, right: Self::Value) -> Self::Value;
    fn less_than(left: Self::Value, right: Self::Value) -> Self::Value;
    /// An object with `slots` instance variables, each nil.
    fn new_object(slots: usize) -> Self::Value;
    fn get_slot(object: &Self::Value, slot: usize) -> Self::Value;
    fn set_slot(object: &Self::Value, slot: usize, value: Self::Value);
    fn new_array() -> Self::Value;
    fn array_push(array: &Self::Value, value: Self::Value);
    fn array_get(array: &Self::Value, index: &Self::Value) -> Self::Value;
    /// A place a collector may run, after an object is made, with every
    /// value the interpreter holds on `stack`.
    fn safepoint(_stack: &[Self::Value]) {}
    /// The value written out, to check both representations computed the
    /// same thing.
    fn describe(value: &Self::Value) -> String;
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Integer(i64),
    Float(f64),
    Load(u8),
    Store(u8),
    Add,
    Subtract,
    Multiply,
    LessThan,
    JumpUnless(u32),
    Jump(u32),
    /// Call function `.0` with `.1` arguments on the stack.
    Call(u16, u8),
    Return,
    NewObject(u8),
    /// Pop an object and push its slot.
    GetSlot(u8),
    /// Pop a value and an object and store the value in the object's slot.
    SetSlot(u8),
    NewArray,
    /// Pop a value and an array and push the value onto the array.
    ArrayPush,
    /// Pop an index and an array and push the element.
    ArrayGet,
}

pub struct Function {
    pub code: Vec<Op>,
    /// Locals past the parameters, each nil when the function starts.
    pub locals: u8,
}

struct Frame {
    function: usize,
    return_to: usize,
    base: usize,
}

/// Run function 0 of `program` with no arguments and answer what it returns.
pub fn run<R: Representation>(program: &[Function]) -> R::Value {
    let mut stack: Vec<R::Value> = Vec::with_capacity(1024);
    for _ in 0..program[0].locals {
        stack.push(R::nil());
    }
    let mut frames = vec![Frame {
        function: 0,
        return_to: 0,
        base: 0,
    }];
    let mut code = &program[0].code;
    let mut base = 0;
    let mut at = 0;
    loop {
        let op = code[at];
        at += 1;
        match op {
            Op::Integer(number) => stack.push(R::integer(number)),
            Op::Float(number) => stack.push(R::float(number)),
            Op::Load(slot) => stack.push(stack[base + slot as usize].clone()),
            Op::Store(slot) => {
                let value = stack.pop().unwrap();
                stack[base + slot as usize] = value;
            }
            Op::Add | Op::Subtract | Op::Multiply | Op::LessThan => {
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                stack.push(match op {
                    Op::Add => R::add(left, right),
                    Op::Subtract => R::subtract(left, right),
                    Op::Multiply => R::multiply(left, right),
                    _ => R::less_than(left, right),
                });
            }
            Op::JumpUnless(target) => {
                if !R::truthy(&stack.pop().unwrap()) {
                    at = target as usize;
                }
            }
            Op::Jump(target) => at = target as usize,
            Op::Call(function, arguments) => {
                let callee = &program[function as usize];
                frames.push(Frame {
                    function: function as usize,
                    return_to: at,
                    base,
                });
                base = stack.len() - arguments as usize;
                for _ in 0..callee.locals {
                    stack.push(R::nil());
                }
                code = &callee.code;
                at = 0;
            }
            Op::Return => {
                let answer = stack.pop().unwrap();
                let finished = frames.pop().unwrap();
                if frames.is_empty() {
                    return answer;
                }
                stack.truncate(base);
                stack.push(answer);
                base = finished.base;
                at = finished.return_to;
                code = &program[frames.last().unwrap().function].code;
            }
            Op::NewObject(slots) => {
                stack.push(R::new_object(slots as usize));
                R::safepoint(&stack);
            }
            Op::GetSlot(slot) => {
                let object = stack.pop().unwrap();
                stack.push(R::get_slot(&object, slot as usize));
            }
            Op::SetSlot(slot) => {
                let value = stack.pop().unwrap();
                let object = stack.pop().unwrap();
                R::set_slot(&object, slot as usize, value);
            }
            Op::NewArray => {
                stack.push(R::new_array());
                R::safepoint(&stack);
            }
            Op::ArrayPush => {
                let value = stack.pop().unwrap();
                let array = stack.pop().unwrap();
                R::array_push(&array, value);
            }
            Op::ArrayGet => {
                let index = stack.pop().unwrap();
                let array = stack.pop().unwrap();
                stack.push(R::array_get(&array, &index));
            }
        }
    }
}
