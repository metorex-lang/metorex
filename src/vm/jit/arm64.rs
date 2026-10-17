//! Machine code for arm64, under the AAPCS64 calling convention. The
//! context is kept in x19, `step` in x20, and the registers' start in x21,
//! which a call preserves.

use super::{
    AT_OFFSET, COMPILED_IN_OFFSET, Comparison, DEOPTIMIZE, EPOCH_OFFSET, Emitted, FALSE_BITS,
    IntegerOperation, LARGEST_INLINE_WORD, REGISTERS_OFFSET, STEP_OFFSET, Source, Step,
    TABLE_OFFSET, TRUE_BITS,
};

const X0: u32 = 0;
const X1: u32 = 1;
const X9: u32 = 9;
const X10: u32 = 10;
const X11: u32 = 11;
const X12: u32 = 12;
const X13: u32 = 13;
const X14: u32 = 14;
const X15: u32 = 15;
const X16: u32 = 16;
const CONTEXT: u32 = 19;
const STEP: u32 = 20;
const REGISTERS: u32 = 21;
const SPARE: u32 = 22;
const FRAME: u32 = 29;
const LINK: u32 = 30;
const SP: u32 = 31;
const ZERO: u32 = 31;

/// Condition codes.
const EQUAL: u32 = 0x0;
const NOT_EQUAL: u32 = 0x1;
const OVERFLOW: u32 = 0x6;
const HIGHER: u32 = 0x8;
const GREATER_EQUAL: u32 = 0xA;
const LESS: u32 = 0xB;
const GREATER: u32 = 0xC;
const LESS_EQUAL: u32 = 0xD;

/// Where a branch goes once its target's place is known.
#[derive(Clone, Copy)]
enum Label {
    Instruction(usize),
    Dispatch,
    DispatchJump,
    Finish,
    /// A place inside one instruction's code, by the number `local` gave.
    Local(usize),
}

/// The kinds of branch, by how their offset is encoded.
enum BranchKind {
    /// `b`: 26 bits of words.
    Unconditional,
    /// `cbz`, `b.cond`: 19 bits of words.
    Short,
    /// `tbnz`: 14 bits of words.
    Test,
}

struct Emitter {
    words: Vec<u32>,
    fixups: Vec<(usize, Label, BranchKind)>,
    locals: Vec<usize>,
}

impl Emitter {
    fn word(&mut self, word: u32) {
        self.words.push(word);
    }

    fn branch(&mut self, word: u32, label: Label, kind: BranchKind) {
        self.fixups.push((self.words.len(), label, kind));
        self.words.push(word);
    }

    fn here(&self) -> usize {
        self.words.len()
    }

    /// A new label inside an instruction's code, placed by `place`.
    fn local(&mut self) -> Label {
        self.locals.push(usize::MAX);
        Label::Local(self.locals.len() - 1)
    }

    fn place(&mut self, label: Label) {
        if let Label::Local(number) = label {
            self.locals[number] = self.here();
        }
    }

    fn jump(&mut self, label: Label) {
        self.branch(0x1400_0000, label, BranchKind::Unconditional);
    }

    fn branch_if(&mut self, condition: u32, label: Label) {
        self.branch(0x5400_0000 | condition, label, BranchKind::Short);
    }

    /// `mov to, from`, as `orr to, xzr, from`.
    fn move_register(&mut self, to: u32, from: u32) {
        self.word(0xAA00_0000 | from << 16 | ZERO << 5 | to);
    }

    /// `ldr x<to>, [x<base>, #offset]`.
    fn load_64(&mut self, to: u32, base: u32, offset: u32) {
        self.word(0xF940_0000 | (offset / 8) << 10 | base << 5 | to);
    }

    /// `str x<from>, [x<base>, #offset]`.
    fn store_64(&mut self, from: u32, base: u32, offset: u32) {
        self.word(0xF900_0000 | (offset / 8) << 10 | base << 5 | from);
    }

    /// `ldr w<to>, [x<base>, #offset]`.
    fn load_32(&mut self, to: u32, base: u32, offset: u32) {
        self.word(0xB940_0000 | (offset / 4) << 10 | base << 5 | to);
    }

    /// `str w<from>, [x<base>, #offset]`.
    fn store_32(&mut self, from: u32, base: u32, offset: u32) {
        self.word(0xB900_0000 | (offset / 4) << 10 | base << 5 | from);
    }

    /// `mov w<to>, #value`, as `movz` and, when the high half is not zero,
    /// `movk`.
    fn load_immediate_32(&mut self, to: u32, value: u32) {
        self.word(0x5280_0000 | (value & 0xFFFF) << 5 | to);
        if value >> 16 != 0 {
            self.word(0x72A0_0000 | (value >> 16) << 5 | to);
        }
    }

    /// `mov x<to>, #value`, as `movz` and a `movk` for each further
    /// 16 bits that are not zero.
    fn load_immediate_64(&mut self, to: u32, value: u64) {
        self.word(0xD280_0000 | ((value & 0xFFFF) as u32) << 5 | to);
        for half in 1..4 {
            let bits = ((value >> (16 * half)) & 0xFFFF) as u32;
            if bits != 0 {
                self.word(0xF280_0000 | half << 21 | bits << 5 | to);
            }
        }
    }

    /// `cmp x<left>, x<right>`.
    fn compare(&mut self, left: u32, right: u32) {
        self.word(0xEB00_001F | right << 16 | left << 5);
    }

    fn load_source(&mut self, to: u32, source: Source) {
        match source {
            Source::Register(index) => self.load_64(to, REGISTERS, u32::from(index) * 8),
            Source::Bits(bits) => self.load_immediate_64(to, bits),
        }
    }

    /// Go back to the interpreter at instruction `at`.
    fn deoptimize(&mut self, at: usize) {
        self.load_immediate_32(X9, at as u32);
        self.store_32(X9, CONTEXT, AT_OFFSET);
        self.load_immediate_64(X0, DEOPTIMIZE);
        self.jump(Label::Finish);
    }

    /// Go back to the interpreter at instruction `at` if the epoch has
    /// moved on since the unit was compiled.
    fn check_epoch(&mut self, at: usize) {
        let holds = self.local();
        self.load_64(X9, CONTEXT, EPOCH_OFFSET);
        self.load_64(X9, X9, 0);
        self.load_64(X10, CONTEXT, COMPILED_IN_OFFSET);
        self.compare(X9, X10);
        self.branch_if(EQUAL, holds);
        self.deoptimize(at);
        self.place(holds);
    }

    /// Call `step` for instruction `at` and go on as it says.
    fn call_step(&mut self, at: usize) {
        self.move_register(X0, CONTEXT);
        self.load_immediate_32(X1, at as u32);
        // blr x20
        self.word(0xD63F_0000 | STEP << 5);
        // cbz x0, the next instruction's code
        self.branch(
            0xB400_0000 | X0,
            Label::Instruction(at + 1),
            BranchKind::Short,
        );
        self.jump(Label::Dispatch);
    }

    fn integer(
        &mut self,
        at: usize,
        operation: IntegerOperation,
        to: u8,
        left: Source,
        right: Source,
    ) {
        let integers = self.local();
        let store = self.local();
        let slow = self.local();
        self.check_epoch(at);
        self.load_source(X11, left);
        self.load_source(X12, right);
        // and x13, x11, x12; tbnz x13, #0, integers
        self.word(0x8A00_0000 | X12 << 16 | X11 << 5 | X13);
        self.branch(0x3700_0000 | X13, integers, BranchKind::Test);
        self.deoptimize(at);
        self.place(integers);
        match operation {
            IntegerOperation::Add => {
                // sub x12, x12, #1; adds x14, x11, x12
                self.word(0xD100_0000 | 1 << 10 | X12 << 5 | X12);
                self.word(0xAB00_0000 | X12 << 16 | X11 << 5 | X14);
                self.branch_if(OVERFLOW, slow);
            }
            IntegerOperation::Subtract => {
                // subs x14, x11, x12; add x14, x14, #1
                self.word(0xEB00_0000 | X12 << 16 | X11 << 5 | X14);
                self.branch_if(OVERFLOW, slow);
                self.word(0x9100_0000 | 1 << 10 | X14 << 5 | X14);
            }
            IntegerOperation::Compare(comparison) => {
                let condition = match comparison {
                    Comparison::Equal => EQUAL,
                    Comparison::NotEqual => NOT_EQUAL,
                    Comparison::Less => LESS,
                    Comparison::LessEqual => LESS_EQUAL,
                    Comparison::Greater => GREATER,
                    Comparison::GreaterEqual => GREATER_EQUAL,
                };
                self.load_immediate_64(X16, TRUE_BITS);
                self.load_immediate_64(X15, FALSE_BITS);
                self.compare(X11, X12);
                // csel x14, x16, x15, condition
                self.word(0x9A80_0000 | X15 << 16 | condition << 12 | X16 << 5 | X14);
            }
        }
        // A register holding a heap value is released by `step`.
        let destination = u32::from(to) * 8;
        self.load_64(X15, REGISTERS, destination);
        // tst x15, #7; b.ne store
        self.word(0xF240_0000 | 1 << 22 | 2 << 10 | X15 << 5 | ZERO);
        self.branch_if(NOT_EQUAL, store);
        // cmp x15, #LARGEST_INLINE_WORD; b.hi slow
        self.word(0xF100_001F | (LARGEST_INLINE_WORD as u32) << 10 | X15 << 5);
        self.branch_if(HIGHER, slow);
        self.place(store);
        self.store_64(X14, REGISTERS, destination);
        self.jump(Label::Instruction(at + 1));
        self.place(slow);
        self.call_step(at);
    }
}

/// Lay out the machine code for a unit whose instructions run as `steps`.
pub(super) fn emit(steps: &[Step]) -> Emitted {
    let mut code = Emitter {
        words: Vec::new(),
        fixups: Vec::new(),
        locals: Vec::new(),
    };
    // stp x29, x30, [sp, #-48]!
    code.word(0xA980_0000 | (0x7A << 15) | LINK << 10 | SP << 5 | FRAME);
    // mov x29, sp
    code.word(0x9100_0000 | SP << 5 | FRAME);
    // stp x19, x20, [sp, #16]; stp x21, x22, [sp, #32]
    code.word(0xA900_0000 | (2 << 15) | STEP << 10 | SP << 5 | CONTEXT);
    code.word(0xA900_0000 | (4 << 15) | SPARE << 10 | SP << 5 | REGISTERS);
    code.move_register(CONTEXT, X0);
    code.load_64(STEP, X0, STEP_OFFSET);
    code.load_64(REGISTERS, X0, REGISTERS_OFFSET);
    code.load_32(X9, X0, AT_OFFSET);
    code.jump(Label::DispatchJump);

    let mut starts = Vec::with_capacity(steps.len());
    for (at, step) in steps.iter().enumerate() {
        starts.push(code.here());
        match step {
            Step::Branch(target) => code.jump(Label::Instruction(*target)),
            Step::Call { checked } => {
                if *checked {
                    code.check_epoch(at);
                }
                code.call_step(at);
            }
            Step::Integer {
                operation,
                to,
                left,
                right,
            } => code.integer(at, *operation, *to, *left, *right),
        }
    }
    // Past the last instruction, which always finishes the unit: brk #0.
    starts.push(code.here());
    code.word(0xD420_0000);

    let dispatch = code.here();
    // cmp x0, #JUMP
    code.word(0xF100_001F | (super::JUMP as u32) << 10 | X0 << 5);
    code.branch_if(NOT_EQUAL, Label::Finish);
    code.load_32(X9, CONTEXT, AT_OFFSET);
    let dispatch_jump = code.here();
    code.load_64(X10, CONTEXT, TABLE_OFFSET);
    // ldrsw x11, [x10, x9, lsl #2]
    code.word(0xB8A0_7800 | X9 << 16 | X10 << 5 | X11);
    // add x11, x10, x11
    code.word(0x8B00_0000 | X11 << 16 | X10 << 5 | X11);
    // br x11
    code.word(0xD61F_0000 | X11 << 5);

    let finish = code.here();
    // ldp x21, x22, [sp, #32]; ldp x19, x20, [sp, #16]
    code.word(0xA940_0000 | (4 << 15) | SPARE << 10 | SP << 5 | REGISTERS);
    code.word(0xA940_0000 | (2 << 15) | STEP << 10 | SP << 5 | CONTEXT);
    // ldp x29, x30, [sp], #48
    code.word(0xA8C0_0000 | (6 << 15) | LINK << 10 | SP << 5 | FRAME);
    // ret
    code.word(0xD65F_03C0);

    for (at, label, kind) in std::mem::take(&mut code.fixups) {
        let target = match label {
            Label::Instruction(index) => starts[index],
            Label::Dispatch => dispatch,
            Label::DispatchJump => dispatch_jump,
            Label::Finish => finish,
            Label::Local(number) => code.locals[number],
        };
        let words = target as i64 - at as i64;
        code.words[at] |= match kind {
            BranchKind::Unconditional => (words as u32) & 0x03FF_FFFF,
            BranchKind::Short => ((words as u32) & 0x7_FFFF) << 5,
            BranchKind::Test => ((words as u32) & 0x3FFF) << 5,
        };
    }

    let table_offset = code.words.len() * 4;
    let mut bytes: Vec<u8> = code
        .words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect();
    for start in &starts[..steps.len()] {
        let offset = (start * 4) as i64 - table_offset as i64;
        bytes.extend_from_slice(&(offset as i32).to_le_bytes());
    }
    Emitted {
        bytes,
        table_offset,
    }
}
