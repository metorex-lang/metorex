//! Machine code for x86-64, under the System V calling convention. A call
//! preserves rbx, r12, r13, r14, and r15, which hold the context, `step`,
//! the registers' start, `run_instruction`, and where the epoch is.
//!
//! What runs on the usual path is laid out first, instruction by
//! instruction, and what runs when a check fails is laid out after it, so
//! the usual path stays short. Starting, finishing, deciding where to go
//! after a call, and calling into Rust are in code every unit shares, which
//! a call reaches through a trampoline that loads the context and jumps to
//! the function.

use super::{
    AT_OFFSET, Action, COLLECTION_DUE_OFFSET, COMPILED_IN_OFFSET, Callee, Comparison, DEOPTIMIZE,
    EPOCH_OFFSET, Emitted, FALSE_BITS, Helper, Holding, IntegerOperation, LARGEST_INLINE_WORD,
    Link, NIL_BITS, Planned, REGISTERS_OFFSET, RETURNED, RUN_INSTRUCTION_OFFSET,
    SIGNAL_PENDING_OFFSET, STATEMENT_INDEX_OFFSET, STATEMENTS_BETWEEN_CLOCK_READS,
    STATEMENTS_RUN_OFFSET, STEP_OFFSET, Shared, SharedLayout, Source, Start, TABLE_OFFSET,
    TRUE_BITS,
};

/// Where a branch goes once its target's place is known.
#[derive(Clone, Copy)]
enum Label {
    Instruction(usize),
    Shared(Shared),
    /// A place inside one instruction's code, by the number `local` gave.
    Local(usize),
}

/// The second opcode byte of a conditional jump with a 32-bit
/// displacement, after `0x0F`.
const JUMP_IF_EQUAL: u8 = 0x84;
const JUMP_IF_NOT_EQUAL: u8 = 0x85;
const JUMP_IF_OVERFLOW: u8 = 0x80;
const JUMP_IF_ABOVE: u8 = 0x87;
const JUMP_IF_NOT_SIGN: u8 = 0x89;

/// The registers named in a ModRM byte.
const RAX: u8 = 0;
const RCX: u8 = 1;
const RDX: u8 = 2;
const RSI: u8 = 6;

/// The most registers a statement start empties one by one rather than in
/// a loop.
const MOST_REGISTERS_UNROLLED: u32 = 2;

/// A byte's place: in the usual path, or laid out after it.
#[derive(Clone, Copy)]
struct Place {
    cold: bool,
    index: usize,
}

#[derive(Default)]
struct Emitter {
    hot: Vec<u8>,
    cold: Vec<u8>,
    writing_cold: bool,
    /// Each 32-bit displacement to fill in: where it sits, and its label.
    /// A displacement counts from the end of the four bytes it is.
    fixups: Vec<(Place, Label)>,
    locals: Vec<Option<Place>>,
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

    fn bytes(&mut self, bytes: &[u8]) {
        if self.writing_cold {
            self.cold.extend_from_slice(bytes);
        } else {
            self.hot.extend_from_slice(bytes);
        }
    }

    /// The opcode bytes of a branch, then the displacement to `label`.
    fn branch(&mut self, opcode: &[u8], label: Label) {
        self.bytes(opcode);
        self.fixups.push((self.here(), label));
        self.bytes(&[0; 4]);
    }

    fn jump(&mut self, label: Label) {
        self.branch(&[0xE9], label);
    }

    fn jump_if(&mut self, condition: u8, label: Label) {
        self.branch(&[0x0F, condition], label);
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

    /// `mov <register>, [r13 + offset]`.
    fn load_register(&mut self, to: u8, offset: u32) {
        self.bytes(&[0x49, 0x8B, 0x85 | to << 3]);
        self.bytes(&offset.to_le_bytes());
    }

    /// `mov [r13 + offset], <register>`.
    fn store_register(&mut self, from: u8, offset: u32) {
        self.bytes(&[0x49, 0x89, 0x85 | from << 3]);
        self.bytes(&offset.to_le_bytes());
    }

    /// `mov <register>, imm64`.
    fn load_immediate(&mut self, to: u8, value: u64) {
        self.bytes(&[0x48, 0xB8 | to]);
        self.bytes(&value.to_le_bytes());
    }

    fn load_source(&mut self, to: u8, source: Source) {
        match source {
            Source::Register(index) => self.load_register(to, u32::from(index) * 8),
            Source::Bits(bits) => self.load_immediate(to, bits),
        }
    }

    /// Call `callee` for instruction `at` through its trampoline, and go on
    /// past the call when it answers NEXT.
    fn call(&mut self, at: usize, callee: Callee) {
        // mov esi, at
        self.bytes(&[0xBE]);
        self.bytes(&(at as u32).to_le_bytes());
        self.branch(&[0xE8], Label::Shared(Shared::Call(callee)));
        self.next_unless_answered();
    }

    /// `test rax, rax; jnz dispatch`.
    fn next_unless_answered(&mut self) {
        self.bytes(&[0x48, 0x85, 0xC0]);
        self.jump_if(JUMP_IF_NOT_EQUAL, Label::Shared(Shared::Dispatch));
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
            // mov esi, at
            code.bytes(&[0xBE]);
            code.bytes(&(at as u32).to_le_bytes());
            code.jump(Label::Shared(Shared::Deoptimize));
        })
    }

    /// Go back to the interpreter at instruction `at` if the epoch has
    /// moved on since the unit was compiled.
    fn check_epoch(&mut self, at: usize) {
        let moved = self.deoptimization(at);
        // mov rax, [r15]; cmp rax, [rbx + COMPILED_IN_OFFSET]
        self.bytes(&[0x49, 0x8B, 0x07, 0x48, 0x3B, 0x43, COMPILED_IN_OFFSET as u8]);
        self.jump_if(JUMP_IF_NOT_EQUAL, moved);
    }

    /// Jump to `slow` unless `value`, one of rax, rcx, rdx, and rsi, holds
    /// an inline value.
    fn unless_inline(&mut self, value: u8, slow: Label) {
        let inline = self.local();
        // test <value>b, 7; jnz inline
        self.bytes(&[0x40, 0xF6, 0xC0 | value, 0x07]);
        self.jump_if(JUMP_IF_NOT_EQUAL, inline);
        // cmp <value>, LARGEST_INLINE_WORD; ja slow
        self.bytes(&[0x48, 0x83, 0xF8 | value, LARGEST_INLINE_WORD as u8]);
        self.jump_if(JUMP_IF_ABOVE, slow);
        self.place(inline);
    }

    /// rcx = the address of register `register`, and edx = `count`.
    fn registers_from(&mut self, register: u32, count: u32) {
        // lea rcx, [r13 + register * 8]
        self.bytes(&[0x49, 0x8D, 0x8D]);
        self.bytes(&(register * 8).to_le_bytes());
        // mov edx, count
        self.bytes(&[0xBA]);
        self.bytes(&count.to_le_bytes());
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
        // mov esi, at; call subroutine
        self.bytes(&[0xBE]);
        self.bytes(&(at as u32).to_le_bytes());
        self.branch(&[0xE8], subroutine);
        self.next_unless_answered();
        if let Some(index) = index {
            // mov qword [rbx + STATEMENT_INDEX_OFFSET], index
            self.bytes(&[0x48, 0xC7, 0x43, STATEMENT_INDEX_OFFSET as u8]);
            self.bytes(&index.to_le_bytes());
        }
    }

    /// Lay out each statement start subroutine `start_inline` called. One
    /// is called with the instruction's place in esi, and answers in rax as
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
            let moved = self.local();
            self.place(label);
            // mov rax, [r15]; cmp rax, [rbx + COMPILED_IN_OFFSET]; jne moved
            self.bytes(&[0x49, 0x8B, 0x07, 0x48, 0x3B, 0x43, COMPILED_IN_OFFSET as u8]);
            self.jump_if(JUMP_IF_NOT_EQUAL, moved);
            let emptied = count.saturating_sub(live);
            let looped = emptied > MOST_REGISTERS_UNROLLED;
            if looped {
                let check = self.local();
                self.registers_from(live, emptied);
                self.place(check);
                // mov rax, [rcx]
                self.bytes(&[0x48, 0x8B, 0x01]);
                self.unless_inline(RAX, slow);
                // add rcx, 8; dec edx; jnz check
                self.bytes(&[0x48, 0x83, 0xC1, 0x08, 0xFF, 0xCA]);
                self.jump_if(JUMP_IF_NOT_EQUAL, check);
            } else {
                for register in live..count {
                    self.load_register(RAX, register * 8);
                    self.unless_inline(RAX, slow);
                }
            }
            if checks {
                // mov rax, [rbx + SIGNAL_PENDING_OFFSET]; mov cl, [rax];
                // mov rax, [rbx + COLLECTION_DUE_OFFSET]; or cl, [rax]; jnz slow
                self.bytes(&[0x48, 0x8B, 0x43, SIGNAL_PENDING_OFFSET as u8, 0x8A, 0x08]);
                self.bytes(&[0x48, 0x8B, 0x43, COLLECTION_DUE_OFFSET as u8, 0x0A, 0x08]);
                self.jump_if(JUMP_IF_NOT_EQUAL, slow);
                // mov rax, [rbx + STATEMENTS_RUN_OFFSET]; mov rcx, [rax];
                // add rcx, 1; test cl, STATEMENTS_BETWEEN_CLOCK_READS - 1; jz slow
                self.bytes(&[0x48, 0x8B, 0x43, STATEMENTS_RUN_OFFSET as u8]);
                self.bytes(&[0x48, 0x8B, 0x08, 0x48, 0x83, 0xC1, 0x01]);
                self.bytes(&[0xF6, 0xC1, (STATEMENTS_BETWEEN_CLOCK_READS - 1) as u8]);
                self.jump_if(JUMP_IF_EQUAL, slow);
                // mov [rax], rcx
                self.bytes(&[0x48, 0x89, 0x08]);
            }
            if looped {
                let empty = self.local();
                self.registers_from(live, emptied);
                self.load_immediate(RAX, NIL_BITS);
                self.place(empty);
                // mov [rcx], rax; add rcx, 8; dec edx; jnz empty
                self.bytes(&[0x48, 0x89, 0x01, 0x48, 0x83, 0xC1, 0x08, 0xFF, 0xCA]);
                self.jump_if(JUMP_IF_NOT_EQUAL, empty);
            } else if live < count {
                self.load_immediate(RAX, NIL_BITS);
                for register in live..count {
                    self.store_register(RAX, register * 8);
                }
            }
            // xor eax, eax; ret
            self.bytes(&[0x31, 0xC0, 0xC3]);
            self.place(slow);
            // sub rsp, 8; call start_statement; add rsp, 8; ret
            self.bytes(&[0x48, 0x83, 0xEC, 0x08]);
            self.branch(
                &[0xE8],
                Label::Shared(Shared::Call(Callee::Helper(Helper::StartStatement))),
            );
            self.bytes(&[0x48, 0x83, 0xC4, 0x08, 0xC3]);
            self.place(moved);
            // add rsp, 8, dropping this call's return address; jmp deoptimize
            self.bytes(&[0x48, 0x83, 0xC4, 0x08]);
            self.jump(Label::Shared(Shared::Deoptimize));
        }
    }

    fn store(&mut self, at: usize, to: u8, bits: u64) {
        let slow = self.cold_call(at, Callee::RunInstruction);
        let destination = u32::from(to) * 8;
        self.load_register(RSI, destination);
        self.unless_inline(RSI, slow);
        self.load_immediate(RAX, bits);
        self.store_register(RAX, destination);
    }

    fn move_value(&mut self, at: usize, to: u8, from: u8) {
        let slow = self.cold_call(at, Callee::RunInstruction);
        let destination = u32::from(to) * 8;
        self.load_register(RAX, u32::from(from) * 8);
        self.unless_inline(RAX, slow);
        self.load_register(RSI, destination);
        self.unless_inline(RSI, slow);
        self.store_register(RAX, destination);
    }

    fn branch_if_holding(&mut self, test: u8, when: Holding, target: usize) {
        self.load_register(RAX, u32::from(test) * 8);
        // cmp rax, NIL_BITS
        let compare_nil = [0x48, 0x83, 0xF8, NIL_BITS as u8];
        // test rax, rax
        let test_false = [0x48, 0x85, 0xC0];
        match when {
            Holding::Nil => {
                self.bytes(&compare_nil);
                self.jump_if(JUMP_IF_EQUAL, Label::Instruction(target));
            }
            Holding::Falsy => {
                self.bytes(&test_false);
                self.jump_if(JUMP_IF_EQUAL, Label::Instruction(target));
                self.bytes(&compare_nil);
                self.jump_if(JUMP_IF_EQUAL, Label::Instruction(target));
            }
            Holding::Truthy => {
                let falsy = self.local();
                self.bytes(&test_false);
                self.jump_if(JUMP_IF_EQUAL, falsy);
                self.bytes(&compare_nil);
                self.jump_if(JUMP_IF_NOT_EQUAL, Label::Instruction(target));
                self.place(falsy);
            }
        }
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
        self.load_source(RCX, left);
        self.load_source(RDX, right);
        // mov rax, rcx; and rax, rdx; test al, 1; jz not_integers
        self.bytes(&[0x48, 0x89, 0xC8, 0x48, 0x21, 0xD0, 0xA8, 0x01]);
        self.jump_if(JUMP_IF_EQUAL, not_integers);
        match operation {
            IntegerOperation::Add => {
                // lea rsi, [rdx - 1]; mov rax, rcx; add rax, rsi
                self.bytes(&[0x48, 0x8D, 0x72, 0xFF, 0x48, 0x89, 0xC8, 0x48, 0x01, 0xF0]);
                self.jump_if(JUMP_IF_OVERFLOW, slow);
            }
            IntegerOperation::Subtract => {
                // mov rax, rcx; sub rax, rdx
                self.bytes(&[0x48, 0x89, 0xC8, 0x48, 0x29, 0xD0]);
                self.jump_if(JUMP_IF_OVERFLOW, slow);
                // add rax, 1
                self.bytes(&[0x48, 0x83, 0xC0, 0x01]);
            }
            IntegerOperation::Multiply => {
                // lea rax, [rcx - 1]; mov rsi, rdx; sar rsi, 1; imul rax, rsi
                self.bytes(&[0x48, 0x8D, 0x41, 0xFF, 0x48, 0x89, 0xD6, 0x48, 0xD1, 0xFE]);
                self.bytes(&[0x48, 0x0F, 0xAF, 0xC6]);
                self.jump_if(JUMP_IF_OVERFLOW, slow);
                // or rax, 1
                self.bytes(&[0x48, 0x83, 0xC8, 0x01]);
            }
            IntegerOperation::Modulo => {
                let settled = self.local();
                // mov rax, rcx; sar rax, 1; mov rsi, rdx; sar rsi, 1;
                // test rsi, rsi; jz slow
                self.bytes(&[
                    0x48, 0x89, 0xC8, 0x48, 0xD1, 0xF8, 0x48, 0x89, 0xD6, 0x48, 0xD1, 0xFE,
                ]);
                self.bytes(&[0x48, 0x85, 0xF6]);
                self.jump_if(JUMP_IF_EQUAL, slow);
                // cqo; idiv rsi; test rdx, rdx; jz settled
                self.bytes(&[0x48, 0x99, 0x48, 0xF7, 0xFE, 0x48, 0x85, 0xD2]);
                self.jump_if(JUMP_IF_EQUAL, settled);
                // mov rax, rdx; xor rax, rsi; jns settled
                self.bytes(&[0x48, 0x89, 0xD0, 0x48, 0x31, 0xF0]);
                self.jump_if(JUMP_IF_NOT_SIGN, settled);
                // add rdx, rsi
                self.bytes(&[0x48, 0x01, 0xF2]);
                self.place(settled);
                // lea rax, [rdx + rdx + 1]
                self.bytes(&[0x48, 0x8D, 0x44, 0x12, 0x01]);
            }
            IntegerOperation::Compare(comparison) => {
                let condition = match comparison {
                    Comparison::Equal => 0x44,
                    Comparison::NotEqual => 0x45,
                    Comparison::Less => 0x4C,
                    Comparison::LessEqual => 0x4E,
                    Comparison::Greater => 0x4F,
                    Comparison::GreaterEqual => 0x4D,
                };
                self.load_immediate(RAX, FALSE_BITS);
                self.load_immediate(RSI, TRUE_BITS);
                // cmp rcx, rdx; cmov<condition> rax, rsi
                self.bytes(&[0x48, 0x39, 0xD1, 0x48, 0x0F, condition, 0xC6]);
            }
        }
        // A register holding a heap value is released by `run_instruction`.
        let destination = u32::from(to) * 8;
        self.load_register(RSI, destination);
        self.unless_inline(RSI, slow);
        self.store_register(RAX, destination);
    }
}

/// Lay out the machine code for a unit whose instructions run as `steps`.
pub(super) fn emit(steps: &[Planned]) -> Option<Emitted> {
    let mut code = Emitter::default();
    // A run from the first instruction starts here: push rbp; mov rbp, rsp;
    // then the rest in the shared code.
    code.bytes(&[0x55, 0x48, 0x89, 0xE5]);
    code.branch(&[0xE8], Label::Shared(Shared::Setup));

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
                // mov dword [rbx + AT_OFFSET], from; mov eax, RETURNED
                code.bytes(&[0xC7, 0x43, AT_OFFSET as u8]);
                code.bytes(&u32::from(*from).to_le_bytes());
                code.bytes(&[0xB8]);
                code.bytes(&(RETURNED as u32).to_le_bytes());
                code.jump(Label::Shared(Shared::Finish));
            }
            Action::Nothing => {
                if !starts_statement {
                    code.check_epoch(at);
                }
            }
        }
    }
    // Past the last instruction, which always finishes the unit: ud2.
    starts.push(code.hot.len());
    code.bytes(&[0x0F, 0x0B]);
    code.place_starts();

    let hot_bytes = code.hot.len();
    let absolute = |place: Place| {
        if place.cold {
            hot_bytes + place.index
        } else {
            place.index
        }
    };
    let mut bytes = std::mem::take(&mut code.hot);
    bytes.extend_from_slice(&code.cold);
    let mut links = Vec::new();
    for (at, label) in &code.fixups {
        let at = absolute(*at);
        let target = match *label {
            Label::Instruction(index) => starts[index],
            Label::Local(number) => absolute(code.locals[number].expect("a label is placed")),
            Label::Shared(to) => {
                links.push(Link {
                    at,
                    to,
                    short: false,
                });
                continue;
            }
        };
        let displacement = i32::try_from(target as i64 - (at as i64 + 4)).ok()?;
        bytes[at..at + 4].copy_from_slice(&displacement.to_le_bytes());
    }

    let table_offset = bytes.len().next_multiple_of(4);
    bytes.resize(table_offset, 0xCC);
    for start in &starts[..steps.len()] {
        let offset = *start as i64 - table_offset as i64;
        bytes.extend_from_slice(&(offset as i32).to_le_bytes());
    }
    Some(Emitted {
        bytes,
        table_offset,
        links,
    })
}

/// Fill in the displacement `link`, at address `from` in `bytes`, to go to
/// `to`.
pub(super) fn link_branch(bytes: &mut [u8], link: Link, from: usize, to: usize) -> Option<()> {
    assert!(
        !link.short,
        "every branch to the shared code has a 32-bit displacement"
    );
    let displacement = i32::try_from(to as i64 - (from as i64 + 4)).ok()?;
    bytes[link.at..link.at + 4].copy_from_slice(&displacement.to_le_bytes());
    Some(())
}

/// Lay out the code every unit shares, with a trampoline for each of
/// `callees`.
pub(super) fn emit_shared(callees: &[Callee]) -> SharedLayout {
    let mut code = Emitter::default();
    let entry = code.hot.len();
    // push rbp; mov rbp, rsp; call setup; then on to the instruction in the
    // context
    code.bytes(&[0x55, 0x48, 0x89, 0xE5]);
    let set_up = code.local();
    code.branch(&[0xE8], set_up);
    let dispatch_jump = code.local();
    code.jump(dispatch_jump);

    let setup = code.hot.len();
    code.place(set_up);
    // pop rax, the address to go back to; push rbx; push r12; push r13;
    // push r14; push r15; sub rsp, 8
    code.bytes(&[
        0x58, 0x53, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57, 0x48, 0x83, 0xEC, 0x08,
    ]);
    // mov rbx, rdi
    code.bytes(&[0x48, 0x89, 0xFB]);
    // mov r12, [rdi + STEP_OFFSET]
    code.bytes(&[0x4C, 0x8B, 0x67, STEP_OFFSET as u8]);
    // mov r13, [rdi + REGISTERS_OFFSET]
    code.bytes(&[0x4C, 0x8B, 0x6F, REGISTERS_OFFSET as u8]);
    // mov r14, [rdi + RUN_INSTRUCTION_OFFSET]
    code.bytes(&[0x4C, 0x8B, 0x77, RUN_INSTRUCTION_OFFSET as u8]);
    // mov r15, [rdi + EPOCH_OFFSET]
    code.bytes(&[0x4C, 0x8B, 0x7F, EPOCH_OFFSET as u8]);
    // jmp rax
    code.bytes(&[0xFF, 0xE0]);

    let dispatch = code.hot.len();
    // cmp rax, JUMP
    code.bytes(&[0x48, 0x83, 0xF8, super::JUMP as u8]);
    let finish = code.local();
    code.jump_if(JUMP_IF_NOT_EQUAL, finish);
    code.place(dispatch_jump);
    // mov eax, [rbx + AT_OFFSET]; mov rcx, [rbx + TABLE_OFFSET]
    code.bytes(&[
        0x8B,
        0x43,
        AT_OFFSET as u8,
        0x48,
        0x8B,
        0x4B,
        TABLE_OFFSET as u8,
    ]);
    // movsxd rdx, dword [rcx + rax * 4]; add rdx, rcx; jmp rdx
    code.bytes(&[0x48, 0x63, 0x14, 0x81, 0x48, 0x01, 0xCA, 0xFF, 0xE2]);

    let deoptimize = code.hot.len();
    // mov [rbx + AT_OFFSET], esi; mov eax, DEOPTIMIZE
    code.bytes(&[0x89, 0x73, AT_OFFSET as u8, 0xB8]);
    code.bytes(&(DEOPTIMIZE as u32).to_le_bytes());

    code.place(finish);
    let finish_at = code.hot.len();
    // add rsp, 8; pop r15; pop r14; pop r13; pop r12; pop rbx; pop rbp; ret
    code.bytes(&[
        0x48, 0x83, 0xC4, 0x08, 0x41, 0x5F, 0x41, 0x5E, 0x41, 0x5D, 0x41, 0x5C, 0x5B, 0x5D, 0xC3,
    ]);

    let mut calls = Vec::with_capacity(callees.len());
    for callee in callees {
        calls.push((*callee, code.hot.len()));
        // mov rdi, rbx
        code.bytes(&[0x48, 0x89, 0xDF]);
        match callee {
            // jmp r12
            Callee::Step => code.bytes(&[0x41, 0xFF, 0xE4]),
            // jmp r14
            Callee::RunInstruction => code.bytes(&[0x41, 0xFF, 0xE6]),
            Callee::Helper(helper) => {
                code.load_immediate(RAX, helper.function() as usize as u64);
                // jmp rax
                code.bytes(&[0xFF, 0xE0]);
            }
        }
    }

    for (at, label) in std::mem::take(&mut code.fixups) {
        let Label::Local(number) = label else {
            unreachable!("the shared code branches only within itself");
        };
        let target = code.locals[number].expect("a label is placed").index;
        let displacement = (target as i64 - (at.index as i64 + 4)) as i32;
        code.hot[at.index..at.index + 4].copy_from_slice(&displacement.to_le_bytes());
    }
    SharedLayout {
        bytes: code.hot,
        entry,
        setup,
        dispatch,
        finish: finish_at,
        deoptimize,
        calls,
    }
}
