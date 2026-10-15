//! The inner loops of ruby-bench benchmarks, written in the prototype's
//! instructions.

use crate::machine::{Function, Op};
use std::collections::HashMap;

/// Instructions with named places to jump to, resolved when finished.
#[derive(Default)]
struct Assembler {
    code: Vec<Op>,
    places: HashMap<&'static str, u32>,
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

    /// The function, which finds its `parameters` arguments on the stack and
    /// gives `locals` more slots past them.
    fn finish(&mut self, _parameters: u8, locals: u8) -> Function {
        let mut code = std::mem::take(&mut self.code);
        for (at, name) in &self.jumps {
            let target = self.places[name];
            code[*at] = match code[*at] {
                Op::Jump(_) => Op::Jump(target),
                _ => Op::JumpUnless(target),
            };
        }
        Function { code, locals }
    }
}

/// `while i < count` around `body`, with `i` in local `counter`.
fn counted_loop(code: &mut Assembler, counter: u8, count: i64, body: impl FnOnce(&mut Assembler)) {
    code.op(Op::Integer(0)).op(Op::Store(counter));
    code.place("top")
        .op(Op::Load(counter))
        .op(Op::Integer(count))
        .op(Op::LessThan)
        .jump_unless("done");
    body(code);
    code.op(Op::Load(counter))
        .op(Op::Integer(1))
        .op(Op::Add)
        .op(Op::Store(counter))
        .jump("top")
        .place("done");
}

pub struct Kernel {
    pub name: &'static str,
    pub from: &'static str,
    pub program: Vec<Function>,
}

/// `fib(30)`, as ruby-bench's fib computes `fib(32)`.
fn fib() -> Kernel {
    let mut main = Assembler::default();
    main.op(Op::Integer(30)).op(Op::Call(1, 1)).op(Op::Return);
    let mut fib = Assembler::default();
    fib.op(Op::Load(0))
        .op(Op::Integer(2))
        .op(Op::LessThan)
        .jump_unless("recurse")
        .op(Op::Load(0))
        .op(Op::Return)
        .place("recurse")
        .op(Op::Load(0))
        .op(Op::Integer(1))
        .op(Op::Subtract)
        .op(Op::Call(1, 1))
        .op(Op::Load(0))
        .op(Op::Integer(2))
        .op(Op::Subtract)
        .op(Op::Call(1, 1))
        .op(Op::Add)
        .op(Op::Return);
    Kernel {
        name: "fib",
        from: "fib",
        program: vec![main.finish(0, 0), fib.finish(1, 0)],
    }
}

/// A sum over a counted loop, as loops-times and the `while` loops of the
/// micro benchmarks run.
fn while_loop() -> Kernel {
    let mut main = Assembler::default();
    main.op(Op::Integer(0)).op(Op::Store(1));
    counted_loop(&mut main, 0, 50_000_000, |code| {
        code.op(Op::Load(1))
            .op(Op::Load(0))
            .op(Op::Add)
            .op(Op::Store(1));
    });
    main.op(Op::Load(1)).op(Op::Return);
    Kernel {
        name: "while_loop",
        from: "loops-times",
        program: vec![main.finish(0, 2)],
    }
}

/// Reading one of an object's four instance variables in a loop.
fn getivar() -> Kernel {
    let mut main = Assembler::default();
    main.op(Op::NewObject(4)).op(Op::Store(0));
    for slot in 0..4u8 {
        main.op(Op::Load(0))
            .op(Op::Integer(slot as i64 + 1))
            .op(Op::SetSlot(slot));
    }
    main.op(Op::Integer(0)).op(Op::Store(2));
    counted_loop(&mut main, 1, 20_000_000, |code| {
        code.op(Op::Load(2))
            .op(Op::Load(0))
            .op(Op::GetSlot(3))
            .op(Op::Add)
            .op(Op::Store(2));
    });
    main.op(Op::Load(2)).op(Op::Return);
    Kernel {
        name: "getivar",
        from: "getivar",
        program: vec![main.finish(0, 3)],
    }
}

/// Making an object with two instance variables in a loop and dropping it.
fn object_new() -> Kernel {
    let mut main = Assembler::default();
    counted_loop(&mut main, 0, 10_000_000, |code| {
        code.op(Op::NewObject(2))
            .op(Op::Store(1))
            .op(Op::Load(1))
            .op(Op::Load(0))
            .op(Op::SetSlot(0))
            .op(Op::Load(1))
            .op(Op::Load(0))
            .op(Op::SetSlot(1));
    });
    main.op(Op::Load(0)).op(Op::Return);
    Kernel {
        name: "object_new",
        from: "object-new-initialize",
        program: vec![main.finish(0, 2)],
    }
}

/// Float multiplication and addition in a loop, as nbody's inner loop runs.
fn float_arithmetic() -> Kernel {
    let mut main = Assembler::default();
    main.op(Op::Float(0.0)).op(Op::Store(0));
    counted_loop(&mut main, 1, 20_000_000, |code| {
        code.op(Op::Load(0))
            .op(Op::Float(0.5))
            .op(Op::Multiply)
            .op(Op::Float(1.25))
            .op(Op::Add)
            .op(Op::Store(0));
    });
    main.op(Op::Load(0)).op(Op::Return);
    Kernel {
        name: "float_arithmetic",
        from: "nbody",
        program: vec![main.finish(0, 2)],
    }
}

/// Filling an Array with 5 million Integers, then summing it, as matmul and
/// the array-walking benchmarks do.
fn array_sum() -> Kernel {
    let mut main = Assembler::default();
    main.op(Op::NewArray).op(Op::Store(0));
    counted_loop(&mut main, 1, 5_000_000, |code| {
        code.op(Op::Load(0)).op(Op::Load(1)).op(Op::ArrayPush);
    });
    let mut second = Assembler::default();
    second.op(Op::Integer(0)).op(Op::Store(2));
    counted_loop(&mut second, 1, 5_000_000, |code| {
        code.op(Op::Load(2))
            .op(Op::Load(0))
            .op(Op::Load(1))
            .op(Op::ArrayGet)
            .op(Op::Add)
            .op(Op::Store(2));
    });
    second.op(Op::Load(2)).op(Op::Return);
    // The second loop runs as a function of its own, called with the array,
    // so each loop's places are named apart.
    main.op(Op::Load(0)).op(Op::Call(1, 1)).op(Op::Return);
    Kernel {
        name: "array_sum",
        from: "matmul",
        program: vec![main.finish(0, 2), second.finish(1, 2)],
    }
}

pub fn all() -> Vec<Kernel> {
    vec![
        fib(),
        while_loop(),
        getivar(),
        object_new(),
        float_arithmetic(),
        array_sum(),
    ]
}
