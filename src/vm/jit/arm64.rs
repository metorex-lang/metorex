//! Machine code for arm64, under the AAPCS64 calling convention. A call
//! preserves x19 to x28, which hold the context, `step`, the registers'
//! start, `run_instruction`, where the epoch is, the epoch the unit was
//! compiled in, and where the signal flag, the collection flag, and the
//! statement count are.
//!
//! What runs on the usual path is laid out first, instruction by
//! instruction, and what runs when a check fails is laid out after it, so
//! the usual path stays short. Starting, finishing, deciding where to go
//! after a call, and calling into Rust are in code every unit shares, which
//! a call reaches through a trampoline that loads the context and jumps to
//! the function, so each call site is three words.

use super::{
    AT_OFFSET, Action, COLLECTION_DUE_OFFSET, COMPILED_IN_OFFSET, Callee, Comparison, DEOPTIMIZE,
    EPOCH_OFFSET, Emitted, FALSE_BITS, Helper, Holding, IntegerOperation, LARGEST_INLINE_WORD,
    Link, NIL_BITS, Planned, REGISTERS_OFFSET, RETURNED, RUN_INSTRUCTION_OFFSET,
    SIGNAL_PENDING_OFFSET, STATEMENT_INDEX_OFFSET, STATEMENTS_BETWEEN_CLOCK_READS,
    STATEMENTS_RUN_OFFSET, STEP_OFFSET, Shared, SharedLayout, Source, Start, TABLE_OFFSET,
    TRUE_BITS,
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
const RUN_INSTRUCTION: u32 = 22;
const EPOCH: u32 = 23;
const COMPILED_IN: u32 = 24;
const SIGNAL_PENDING: u32 = 25;
const COLLECTION_DUE: u32 = 26;
const STATEMENTS_RUN: u32 = 27;
const FRAME: u32 = 29;
const LINK: u32 = 30;
const SP: u32 = 31;
const ZERO: u32 = 31;

/// The bytes the prologue pushes: x29 and x30, then x19 to x28.
const FRAME_BYTES: u32 = 96;

/// The first register of each pair the prologue saves.
const SAVED_PAIRS: [u32; 5] = [19, 21, 23, 25, 27];

/// The most registers a statement start empties one by one rather than in
/// a loop.
const MOST_REGISTERS_UNROLLED: u32 = 2;

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
    Shared(Shared),
    /// A place inside one instruction's code, by the number `local` gave.
    Local(usize),
}

/// The kinds of branch, by how their offset is encoded.
#[derive(Clone, Copy)]
enum BranchKind {
    /// `b`, `bl`: 26 bits of words.
    Unconditional,
    /// `cbz`, `cbnz`, `b.cond`: 19 bits of words.
    Short,
}

/// A word's place: in the usual path, or laid out after it.
#[derive(Clone, Copy)]
struct Place {
    cold: bool,
    index: usize,
}

#[derive(Default)]
struct Emitter {
    hot: Vec<u32>,
    cold: Vec<u32>,
    writing_cold: bool,
    fixups: Vec<(Place, Label, BranchKind)>,
    locals: Vec<Option<Place>>,
    /// For each part of the shared code a short branch goes to, the
    /// `b` in this unit that it goes through, since a short branch reaches
    /// only 1 MB.
    veneers: Vec<(Shared, Label)>,
    /// The statement start subroutines called, laid out by `place_starts`.
    starts: Vec<(StartLayout, Label)>,
}

/// What a statement start subroutine does: empty registers `live` up to
/// `count`, and when `checks`, test the flags and count the statement.
#[derive(Clone, Copy, PartialEq)]
struct StartLayout {
    live: u32,
    count: u32,
    checks: bool,
}

impl Emitter {
    fn here(&self) -> Place {
        if self.writing_cold {
            Place {
                cold: true,
                index: self.cold.len(),
            }
        } else {
            Place {
                cold: false,
                index: self.hot.len(),
            }
        }
    }

    fn word(&mut self, word: u32) {
        if self.writing_cold {
            self.cold.push(word);
        } else {
            self.hot.push(word);
        }
    }

    fn branch(&mut self, word: u32, label: Label, kind: BranchKind) {
        let label = match (label, kind) {
            (Label::Shared(to), BranchKind::Short) => self.veneer(to),
            _ => label,
        };
        self.fixups.push((self.here(), label, kind));
        self.word(word);
    }

    /// A `b` to `to` for short branches, laid out by `place_veneers`.
    fn veneer(&mut self, to: Shared) -> Label {
        if let Some((_, label)) = self.veneers.iter().find(|(held, _)| *held == to) {
            return *label;
        }
        let label = self.local();
        self.veneers.push((to, label));
        label
    }

    /// Lay out each veneer after everything else.
    fn place_veneers(&mut self) {
        self.writing_cold = true;
        for (to, label) in std::mem::take(&mut self.veneers) {
            self.place(label);
            self.jump(Label::Shared(to));
        }
    }

    /// A new label inside an instruction's code, placed by `place`.
    fn local(&mut self) -> Label {
        self.locals.push(None);
        Label::Local(self.locals.len() - 1)
    }

    fn place(&mut self, label: Label) {
        if let Label::Local(number) = label {
            self.locals[number] = Some(self.here());
        }
    }

    /// Lay out `code` after the usual path, starting at the label answered.
    fn cold_block(&mut self, code: impl FnOnce(&mut Self)) -> Label {
        let label = self.local();
        let was = std::mem::replace(&mut self.writing_cold, true);
        self.place(label);
        code(self);
        self.writing_cold = was;
        label
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

    /// `stp x29, x30, [sp, #-FRAME_BYTES]!`, the frame the shared finish
    /// takes down.
    fn save_frame(&mut self) {
        self.word(
            0xA980_0000
                | (((-(FRAME_BYTES as i32) / 8) as u32) & 0x7F) << 15
                | LINK << 10
                | SP << 5
                | FRAME,
        );
    }

    /// `ldrb w<to>, [x<base>]`.
    fn load_byte(&mut self, to: u32, base: u32) {
        self.word(0x3940_0000 | base << 5 | to);
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

    /// `cmp x<value>, #immediate`.
    fn compare_immediate(&mut self, value: u32, immediate: u64) {
        self.word(0xF100_001F | (immediate as u32) << 10 | value << 5);
    }

    fn load_register(&mut self, to: u32, register: u32) {
        self.load_64(to, REGISTERS, register * 8);
    }

    fn store_register(&mut self, from: u32, register: u32) {
        self.store_64(from, REGISTERS, register * 8);
    }

    fn load_source(&mut self, to: u32, source: Source) {
        match source {
            Source::Register(index) => self.load_register(to, u32::from(index)),
            Source::Bits(bits) => self.load_immediate_64(to, bits),
        }
    }

    /// Call `callee` for instruction `at` through its trampoline, and go on
    /// past the call when it answers NEXT.
    fn call(&mut self, at: usize, callee: Callee) {
        self.load_immediate_32(X1, at as u32);
        // bl trampoline
        self.branch(
            0x9400_0000,
            Label::Shared(Shared::Call(callee)),
            BranchKind::Unconditional,
        );
        // cbnz x0, dispatch
        self.branch(
            0xB500_0000 | X0,
            Label::Shared(Shared::Dispatch),
            BranchKind::Short,
        );
    }

    /// Call `callee` for instruction `at` from code laid out after the usual
    /// path, then go on with instruction `at + 1`.
    fn cold_call(&mut self, at: usize, callee: Callee) -> Label {
        self.cold_block(|code| {
            code.call(at, callee);
            code.jump(Label::Instruction(at + 1));
        })
    }

    /// Code that goes back to the interpreter at instruction `at`.
    fn deoptimization(&mut self, at: usize) -> Label {
        self.cold_block(|code| {
            code.load_immediate_32(X1, at as u32);
            code.jump(Label::Shared(Shared::Deoptimize));
        })
    }

    /// Go back to the interpreter at instruction `at` if the epoch has
    /// moved on since the unit was compiled.
    fn check_epoch(&mut self, at: usize) {
        let moved = self.deoptimization(at);
        self.load_64(X9, EPOCH, 0);
        self.compare(X9, COMPILED_IN);
        self.branch_if(NOT_EQUAL, moved);
    }

    /// Branch to `slow` unless register `value` holds an inline value.
    fn unless_inline(&mut self, value: u32, slow: Label) {
        let inline = self.local();
        // tst x<value>, #7; b.ne inline
        self.word(0xF240_0000 | 1 << 22 | 2 << 10 | value << 5 | ZERO);
        self.branch_if(NOT_EQUAL, inline);
        self.compare_immediate(value, LARGEST_INLINE_WORD);
        self.branch_if(HIGHER, slow);
        self.place(inline);
    }

    /// `x<to>` = the address of register `register`.
    fn registers_from(&mut self, to: u32, register: u32) {
        self.load_immediate_32(to, register * 8);
        // add x<to>, x21, x<to>
        self.word(0x8B00_0000 | to << 16 | REGISTERS << 5 | to);
    }

    /// Start a statement as `Start::Inline` says, with the epoch checked
    /// first, through the unit's subroutine for its registers and checks.
    fn start_inline(&mut self, at: usize, live: u32, count: u32, index: Option<u32>, checks: bool) {
        let layout = StartLayout {
            live,
            count,
            checks,
        };
        let subroutine = match self.starts.iter().find(|(held, _)| *held == layout) {
            Some((_, label)) => *label,
            None => {
                let label = self.local();
                self.starts.push((layout, label));
                label
            }
        };
        self.load_immediate_32(X1, at as u32);
        // bl subroutine; cbnz x0, dispatch
        self.branch(0x9400_0000, subroutine, BranchKind::Unconditional);
        self.branch(
            0xB500_0000 | X0,
            Label::Shared(Shared::Dispatch),
            BranchKind::Short,
        );
        if let Some(index) = index {
            self.load_immediate_32(X10, index);
            self.store_64(X10, CONTEXT, STATEMENT_INDEX_OFFSET);
        }
    }

    /// Lay out each statement start subroutine `start_inline` called. One
    /// is called with the instruction's place in w1, and answers in x0 as
    /// a helper does. It goes back to the interpreter when the epoch has
    /// moved, and calls `start_statement` when a register to empty holds a
    /// heap value, a flag is set, or the count is due for reading the clock.
    fn place_starts(&mut self) {
        self.writing_cold = true;
        for (layout, label) in std::mem::take(&mut self.starts) {
            let StartLayout {
                live,
                count,
                checks,
            } = layout;
            let slow = self.local();
            self.place(label);
            self.load_64(X9, EPOCH, 0);
            self.compare(X9, COMPILED_IN);
            self.branch_if(NOT_EQUAL, Label::Shared(Shared::Deoptimize));
            let emptied = count.saturating_sub(live);
            let looped = emptied > MOST_REGISTERS_UNROLLED;
            if looped {
                let check = self.local();
                self.registers_from(X12, live);
                self.load_immediate_32(X13, emptied);
                self.place(check);
                // ldr x9, [x12], #8
                self.word(0xF840_8400 | X12 << 5 | X9);
                self.unless_inline(X9, slow);
                // subs w13, w13, #1; b.ne check
                self.word(0x7100_0400 | X13 << 5 | X13);
                self.branch_if(NOT_EQUAL, check);
            } else {
                for register in live..count {
                    self.load_register(X9, register);
                    self.unless_inline(X9, slow);
                }
            }
            if checks {
                self.load_byte(X9, SIGNAL_PENDING);
                self.load_byte(X10, COLLECTION_DUE);
                // orr w9, w9, w10; cbnz w9, slow
                self.word(0x2A00_0000 | X10 << 16 | X9 << 5 | X9);
                self.branch(0x3500_0000 | X9, slow, BranchKind::Short);
                self.load_64(X9, STATEMENTS_RUN, 0);
                // add x9, x9, #1
                self.word(0x9100_0000 | 1 << 10 | X9 << 5 | X9);
                // tst x9, #(STATEMENTS_BETWEEN_CLOCK_READS - 1); b.eq slow
                let ones = STATEMENTS_BETWEEN_CLOCK_READS.trailing_zeros();
                self.word(0xF240_0000 | 1 << 22 | (ones - 1) << 10 | X9 << 5 | ZERO);
                self.branch_if(EQUAL, slow);
                self.store_64(X9, STATEMENTS_RUN, 0);
            }
            if looped {
                let empty = self.local();
                self.registers_from(X12, live);
                self.load_immediate_32(X13, emptied);
                self.load_immediate_64(X9, NIL_BITS);
                self.place(empty);
                // str x9, [x12], #8
                self.word(0xF800_8400 | X12 << 5 | X9);
                // subs w13, w13, #1; b.ne empty
                self.word(0x7100_0400 | X13 << 5 | X13);
                self.branch_if(NOT_EQUAL, empty);
            } else if live < count {
                self.load_immediate_64(X9, NIL_BITS);
                for register in live..count {
                    self.store_register(X9, register);
                }
            }
            self.load_immediate_64(X0, super::NEXT);
            // ret
            self.word(0xD65F_03C0);
            self.place(slow);
            // str x30, [sp, #-16]!
            self.word(0xF800_0C00 | 0x1F0 << 12 | SP << 5 | LINK);
            self.branch(
                0x9400_0000,
                Label::Shared(Shared::Call(Callee::Helper(Helper::StartStatement))),
                BranchKind::Unconditional,
            );
            // ldr x30, [sp], #16; ret
            self.word(0xF840_0400 | 16 << 12 | SP << 5 | LINK);
            self.word(0xD65F_03C0);
        }
    }

    fn store(&mut self, at: usize, to: u8, bits: u64) {
        let slow = self.cold_call(at, Callee::RunInstruction);
        self.load_register(X15, u32::from(to));
        self.unless_inline(X15, slow);
        self.load_immediate_64(X14, bits);
        self.store_register(X14, u32::from(to));
    }

    fn move_value(&mut self, at: usize, to: u8, from: u8) {
        let slow = self.cold_call(at, Callee::RunInstruction);
        self.load_register(X14, u32::from(from));
        self.unless_inline(X14, slow);
        self.load_register(X15, u32::from(to));
        self.unless_inline(X15, slow);
        self.store_register(X14, u32::from(to));
    }

    fn branch_if_holding(&mut self, test: u8, when: Holding, target: usize) {
        self.load_register(X9, u32::from(test));
        match when {
            Holding::Nil => {
                self.compare_immediate(X9, NIL_BITS);
                self.branch_if(EQUAL, Label::Instruction(target));
            }
            Holding::Falsy => {
                // cbz x9, target
                self.branch(
                    0xB400_0000 | X9,
                    Label::Instruction(target),
                    BranchKind::Short,
                );
                self.compare_immediate(X9, NIL_BITS);
                self.branch_if(EQUAL, Label::Instruction(target));
            }
            Holding::Truthy => {
                let falsy = self.local();
                // cbz x9, falsy
                self.branch(0xB400_0000 | X9, falsy, BranchKind::Short);
                self.compare_immediate(X9, NIL_BITS);
                self.branch_if(NOT_EQUAL, Label::Instruction(target));
                self.place(falsy);
            }
        }
    }

    /// Make the Integer in x14 a value, going to `slow` when it does not
    /// fit in one.
    fn tag_integer(&mut self, slow: Label) {
        // adds x14, x14, x14; b.vs slow; orr x14, x14, #1
        self.word(0xAB00_0000 | X14 << 16 | X14 << 5 | X14);
        self.branch_if(OVERFLOW, slow);
        self.word(0xB240_0000 | X14 << 5 | X14);
    }

    fn integer(
        &mut self,
        at: usize,
        operation: IntegerOperation,
        to: u8,
        left: Source,
        right: Source,
        checked: bool,
    ) {
        if !checked {
            self.check_epoch(at);
        }
        let not_integers = self.deoptimization(at);
        let slow = self.cold_call(at, Callee::RunInstruction);
        self.load_source(X11, left);
        self.load_source(X12, right);
        // and x13, x11, x12; tst x13, #1; b.eq not_integers
        self.word(0x8A00_0000 | X12 << 16 | X11 << 5 | X13);
        self.word(0xF240_0000 | 1 << 22 | X13 << 5 | ZERO);
        self.branch_if(EQUAL, not_integers);
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
            IntegerOperation::Multiply => {
                // asr x11, x11, #1; asr x12, x12, #1
                self.word(0x9341_FC00 | X11 << 5 | X11);
                self.word(0x9341_FC00 | X12 << 5 | X12);
                // mul x14, x11, x12; smulh x15, x11, x12
                self.word(0x9B00_7C00 | X12 << 16 | X11 << 5 | X14);
                self.word(0x9B40_7C00 | X12 << 16 | X11 << 5 | X15);
                // cmp x15, x14, asr #63; b.ne slow
                self.word(0xEB80_0000 | X14 << 16 | 63 << 10 | X15 << 5 | ZERO);
                self.branch_if(NOT_EQUAL, slow);
                self.tag_integer(slow);
            }
            IntegerOperation::Modulo => {
                let settled = self.local();
                // asr x11, x11, #1; asr x12, x12, #1; cbz x12, slow
                self.word(0x9341_FC00 | X11 << 5 | X11);
                self.word(0x9341_FC00 | X12 << 5 | X12);
                self.branch(0xB400_0000 | X12, slow, BranchKind::Short);
                // sdiv x13, x11, x12; msub x14, x13, x12, x11
                self.word(0x9AC0_0C00 | X12 << 16 | X11 << 5 | X13);
                self.word(0x9B00_8000 | X12 << 16 | X11 << 10 | X13 << 5 | X14);
                // cbz x14, settled
                self.branch(0xB400_0000 | X14, settled, BranchKind::Short);
                // eor x15, x14, x12; cmp x15, #0; b.ge settled
                self.word(0xCA00_0000 | X12 << 16 | X14 << 5 | X15);
                self.compare_immediate(X15, 0);
                self.branch_if(GREATER_EQUAL, settled);
                // add x14, x14, x12
                self.word(0x8B00_0000 | X12 << 16 | X14 << 5 | X14);
                self.place(settled);
                self.tag_integer(slow);
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
        // A register holding a heap value is released by `run_instruction`.
        self.load_register(X15, u32::from(to));
        self.unless_inline(X15, slow);
        self.store_register(X14, u32::from(to));
    }
}

/// Lay out the machine code for a unit whose instructions run as `steps`,
/// or None when a branch is too far for its encoding.
pub(super) fn emit(steps: &[Planned]) -> Option<Emitted> {
    let mut code = Emitter::default();
    // A run from the first instruction starts here: save the frame, then
    // the rest in the shared code.
    code.save_frame();
    code.branch(
        0x9400_0000,
        Label::Shared(Shared::Setup),
        BranchKind::Unconditional,
    );
    let mut starts = Vec::with_capacity(steps.len() + 1);
    for (at, planned) in steps.iter().enumerate() {
        starts.push(code.hot.len());
        let starts_statement = planned.start.starts();
        let inline_start = matches!(planned.start, Start::Inline { .. })
            && !matches!(planned.action, Action::Step);
        if starts_statement && !inline_start {
            code.check_epoch(at);
        }
        // `step` starts the statement itself.
        if !matches!(planned.action, Action::Step) {
            match planned.start {
                Start::None => {}
                Start::Call => code.call(at, Callee::Helper(Helper::StartStatement)),
                Start::Inline {
                    live,
                    count,
                    index,
                    checks,
                } => code.start_inline(at, live, count, index, checks),
            }
        }
        match &planned.action {
            Action::Step => code.call(at, Callee::Step),
            Action::Call(callee) => code.call(at, *callee),
            Action::Branch(target) => code.jump(Label::Instruction(*target)),
            Action::Integer {
                operation,
                to,
                left,
                right,
            } => code.integer(at, *operation, *to, *left, *right, starts_statement),
            Action::Store { to, bits } => code.store(at, *to, *bits),
            Action::Move { to, from } => code.move_value(at, *to, *from),
            Action::BranchIf { test, when, target } => {
                code.branch_if_holding(*test, *when, *target);
            }
            Action::Return { from } => {
                code.load_immediate_32(X1, u32::from(*from));
                code.store_32(X1, CONTEXT, AT_OFFSET);
                code.load_immediate_64(X0, RETURNED);
                code.jump(Label::Shared(Shared::Finish));
            }
            Action::Nothing => {
                if !starts_statement {
                    code.check_epoch(at);
                }
            }
        }
    }
    // Past the last instruction, which always finishes the unit: brk #0.
    starts.push(code.hot.len());
    code.word(0xD420_0000);
    code.place_starts();
    code.place_veneers();

    let hot_words = code.hot.len();
    let absolute = |place: Place| {
        if place.cold {
            hot_words + place.index
        } else {
            place.index
        }
    };
    let mut words = std::mem::take(&mut code.hot);
    words.extend_from_slice(&code.cold);
    let mut links = Vec::new();
    for (at, label, kind) in &code.fixups {
        let at = absolute(*at);
        let target = match *label {
            Label::Instruction(index) => starts[index],
            Label::Local(number) => absolute(code.locals[number].expect("a label is placed")),
            Label::Shared(to) => {
                links.push(Link {
                    at: at * 4,
                    to,
                    short: matches!(kind, BranchKind::Short),
                });
                continue;
            }
        };
        words[at] |= branch_offset(target as i64 - at as i64, *kind)?;
    }

    let table_offset = words.len() * 4;
    let mut bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    for start in &starts[..steps.len()] {
        let offset = (start * 4) as i64 - table_offset as i64;
        bytes.extend_from_slice(&(offset as i32).to_le_bytes());
    }
    Some(Emitted {
        bytes,
        table_offset,
        links,
    })
}

/// The offset field of a branch `words` words away, or None when that is
/// too far for its encoding.
fn branch_offset(words: i64, kind: BranchKind) -> Option<u32> {
    let bits = match kind {
        BranchKind::Unconditional => 26,
        BranchKind::Short => 19,
    };
    if !(-(1 << (bits - 1))..1 << (bits - 1)).contains(&words) {
        return None;
    }
    Some(match kind {
        BranchKind::Unconditional => (words as u32) & 0x03FF_FFFF,
        BranchKind::Short => ((words as u32) & 0x7_FFFF) << 5,
    })
}

/// Fill in the branch `link`, at address `from` in `bytes`, to go to `to`.
pub(super) fn link_branch(bytes: &mut [u8], link: Link, from: usize, to: usize) -> Option<()> {
    let kind = if link.short {
        BranchKind::Short
    } else {
        BranchKind::Unconditional
    };
    let field = branch_offset((to as i64 - from as i64) / 4, kind)?;
    let word = &mut bytes[link.at..link.at + 4];
    let held = u32::from_le_bytes(word.try_into().expect("a word is four bytes"));
    word.copy_from_slice(&(held | field).to_le_bytes());
    Some(())
}

/// Lay out the code every unit shares, with a trampoline for each of
/// `callees`.
pub(super) fn emit_shared(callees: &[Callee]) -> SharedLayout {
    let mut code = Emitter::default();
    let entry = code.hot.len();
    code.save_frame();
    let set_up = code.local();
    // bl setup, then on to the instruction in the context
    code.branch(0x9400_0000, set_up, BranchKind::Unconditional);
    let dispatch_jump = code.local();
    code.jump(dispatch_jump);

    let setup = code.hot.len();
    code.place(set_up);
    // mov x29, sp
    code.word(0x9100_0000 | SP << 5 | FRAME);
    // stp x19, x20, [sp, #16] ... stp x27, x28, [sp, #80]
    for first in SAVED_PAIRS {
        let offset = first - CONTEXT + 2;
        code.word(0xA900_0000 | offset << 15 | (first + 1) << 10 | SP << 5 | first);
    }
    code.move_register(CONTEXT, X0);
    code.load_64(STEP, X0, STEP_OFFSET);
    code.load_64(REGISTERS, X0, REGISTERS_OFFSET);
    code.load_64(RUN_INSTRUCTION, X0, RUN_INSTRUCTION_OFFSET);
    code.load_64(EPOCH, X0, EPOCH_OFFSET);
    code.load_64(COMPILED_IN, X0, COMPILED_IN_OFFSET);
    code.load_64(SIGNAL_PENDING, X0, SIGNAL_PENDING_OFFSET);
    code.load_64(COLLECTION_DUE, X0, COLLECTION_DUE_OFFSET);
    code.load_64(STATEMENTS_RUN, X0, STATEMENTS_RUN_OFFSET);
    // ret
    code.word(0xD65F_03C0);

    let dispatch = code.hot.len();
    code.compare_immediate(X0, super::JUMP);
    let finish = code.local();
    code.branch_if(NOT_EQUAL, finish);
    code.place(dispatch_jump);
    code.load_32(X9, CONTEXT, AT_OFFSET);
    code.load_64(X10, CONTEXT, TABLE_OFFSET);
    // ldrsw x11, [x10, x9, lsl #2]
    code.word(0xB8A0_7800 | X9 << 16 | X10 << 5 | X11);
    // add x11, x10, x11
    code.word(0x8B00_0000 | X11 << 16 | X10 << 5 | X11);
    // br x11
    code.word(0xD61F_0000 | X11 << 5);

    let deoptimize = code.hot.len();
    code.store_32(X1, CONTEXT, AT_OFFSET);
    code.load_immediate_64(X0, DEOPTIMIZE);

    code.place(finish);
    let finish_at = code.hot.len();
    // ldp x27, x28, [sp, #80] ... ldp x19, x20, [sp, #16]
    for first in SAVED_PAIRS.into_iter().rev() {
        let offset = first - CONTEXT + 2;
        code.word(0xA940_0000 | offset << 15 | (first + 1) << 10 | SP << 5 | first);
    }
    // ldp x29, x30, [sp], #FRAME_BYTES; ret
    code.word(0xA8C0_0000 | (FRAME_BYTES / 8) << 15 | LINK << 10 | SP << 5 | FRAME);
    code.word(0xD65F_03C0);

    let mut calls = Vec::with_capacity(callees.len());
    for callee in callees {
        calls.push((*callee, code.hot.len() * 4));
        code.move_register(X0, CONTEXT);
        let held = match callee {
            Callee::Step => STEP,
            Callee::RunInstruction => RUN_INSTRUCTION,
            Callee::Helper(helper) => {
                code.load_immediate_64(X16, helper.function() as usize as u64);
                X16
            }
        };
        // br <held>
        code.word(0xD61F_0000 | held << 5);
    }

    for (at, label, kind) in std::mem::take(&mut code.fixups) {
        let Label::Local(number) = label else {
            unreachable!("the shared code branches only within itself");
        };
        let target = code.locals[number].expect("a label is placed").index;
        code.hot[at.index] |=
            branch_offset(target as i64 - at.index as i64, kind).expect("the shared code is short");
    }
    SharedLayout {
        bytes: code
            .hot
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect(),
        entry: entry * 4,
        setup: setup * 4,
        dispatch: dispatch * 4,
        finish: finish_at * 4,
        deoptimize: deoptimize * 4,
        calls,
    }
}
