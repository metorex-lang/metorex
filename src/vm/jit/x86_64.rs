//! Machine code for x86-64, under the System V calling convention. The
//! context is kept in rbx, `step` in r12, and the registers' start in r13,
//! which a call preserves.

use super::{
    AT_OFFSET, COMPILED_IN_OFFSET, Comparison, DEOPTIMIZE, EPOCH_OFFSET, Emitted, FALSE_BITS,
    IntegerOperation, LARGEST_INLINE_WORD, REGISTERS_OFFSET, STEP_OFFSET, Source, Step,
    TABLE_OFFSET, TRUE_BITS,
};

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

/// The second opcode byte of a conditional jump with a 32-bit
/// displacement, after `0x0F`.
const JUMP_IF_EQUAL: u8 = 0x84;
const JUMP_IF_NOT_EQUAL: u8 = 0x85;
const JUMP_IF_OVERFLOW: u8 = 0x80;
const JUMP_IF_ABOVE: u8 = 0x87;

/// The registers named in a ModRM byte.
const RAX: u8 = 0;
const RCX: u8 = 1;
const RDX: u8 = 2;
const RSI: u8 = 6;

struct Emitter {
    bytes: Vec<u8>,
    /// Each 32-bit displacement to fill in: where it sits, and its label.
    /// A displacement counts from the end of the four bytes it is.
    fixups: Vec<(usize, Label)>,
    locals: Vec<usize>,
}

impl Emitter {
    fn bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    /// The opcode bytes of a branch, then the displacement to `label`.
    fn branch(&mut self, opcode: &[u8], label: Label) {
        self.bytes(opcode);
        self.fixups.push((self.bytes.len(), label));
        self.bytes(&[0; 4]);
    }

    fn jump(&mut self, label: Label) {
        self.branch(&[0xE9], label);
    }

    fn jump_if(&mut self, condition: u8, label: Label) {
        self.branch(&[0x0F, condition], label);
    }

    fn here(&self) -> usize {
        self.bytes.len()
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

    /// `mov <register>, [r13 + offset]`.
    fn load_register(&mut self, to: u8, offset: u32) {
        self.bytes(&[0x49, 0x8B, 0x85 | to << 3]);
        self.bytes(&offset.to_le_bytes());
    }

    /// `mov [r13 + offset], rax`.
    fn store_rax(&mut self, offset: u32) {
        self.bytes(&[0x49, 0x89, 0x85]);
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

    /// Go back to the interpreter at instruction `at`.
    fn deoptimize(&mut self, at: usize) {
        // mov dword [rbx + AT_OFFSET], at
        self.bytes(&[0xC7, 0x43, AT_OFFSET as u8]);
        self.bytes(&(at as u32).to_le_bytes());
        // mov eax, DEOPTIMIZE
        self.bytes(&[0xB8]);
        self.bytes(&(DEOPTIMIZE as u32).to_le_bytes());
        self.jump(Label::Finish);
    }

    /// Go back to the interpreter at instruction `at` if the epoch has
    /// moved on since the unit was compiled.
    fn check_epoch(&mut self, at: usize) {
        let holds = self.local();
        // mov rax, [rbx + EPOCH_OFFSET]; mov rax, [rax]
        self.bytes(&[0x48, 0x8B, 0x43, EPOCH_OFFSET as u8, 0x48, 0x8B, 0x00]);
        // cmp rax, [rbx + COMPILED_IN_OFFSET]
        self.bytes(&[0x48, 0x3B, 0x43, COMPILED_IN_OFFSET as u8]);
        self.jump_if(JUMP_IF_EQUAL, holds);
        self.deoptimize(at);
        self.place(holds);
    }

    /// Call `step` for instruction `at` and go on as it says.
    fn call_step(&mut self, at: usize) {
        // mov rdi, rbx
        self.bytes(&[0x48, 0x89, 0xDF]);
        // mov esi, at
        self.bytes(&[0xBE]);
        self.bytes(&(at as u32).to_le_bytes());
        // call r12
        self.bytes(&[0x41, 0xFF, 0xD4]);
        // test rax, rax
        self.bytes(&[0x48, 0x85, 0xC0]);
        self.jump_if(JUMP_IF_NOT_EQUAL, Label::Dispatch);
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
        self.load_source(RCX, left);
        self.load_source(RDX, right);
        // mov rax, rcx; and rax, rdx; test al, 1; jnz integers
        self.bytes(&[0x48, 0x89, 0xC8, 0x48, 0x21, 0xD0, 0xA8, 0x01]);
        self.jump_if(JUMP_IF_NOT_EQUAL, integers);
        self.deoptimize(at);
        self.place(integers);
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
        // A register holding a heap value is released by `step`.
        let destination = u32::from(to) * 8;
        self.load_register(RSI, destination);
        // test sil, 7; jnz store
        self.bytes(&[0x40, 0xF6, 0xC6, 0x07]);
        self.jump_if(JUMP_IF_NOT_EQUAL, store);
        // cmp rsi, LARGEST_INLINE_WORD; ja slow
        self.bytes(&[0x48, 0x83, 0xFE, LARGEST_INLINE_WORD as u8]);
        self.jump_if(JUMP_IF_ABOVE, slow);
        self.place(store);
        self.store_rax(destination);
        self.jump(Label::Instruction(at + 1));
        self.place(slow);
        self.call_step(at);
    }
}

/// Lay out the machine code for a unit whose instructions run as `steps`.
pub(super) fn emit(steps: &[Step]) -> Emitted {
    let mut code = Emitter {
        bytes: Vec::new(),
        fixups: Vec::new(),
        locals: Vec::new(),
    };
    // push rbp; mov rbp, rsp; push rbx; push r12; push r13; push r14
    code.bytes(&[
        0x55, 0x48, 0x89, 0xE5, 0x53, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56,
    ]);
    // mov rbx, rdi
    code.bytes(&[0x48, 0x89, 0xFB]);
    // mov r12, [rdi + STEP_OFFSET]
    code.bytes(&[0x4C, 0x8B, 0x67, STEP_OFFSET as u8]);
    // mov r13, [rdi + REGISTERS_OFFSET]
    code.bytes(&[0x4C, 0x8B, 0x6F, REGISTERS_OFFSET as u8]);
    // mov eax, [rdi + AT_OFFSET]
    code.bytes(&[0x8B, 0x47, AT_OFFSET as u8]);
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
    // Past the last instruction, which always finishes the unit: ud2.
    starts.push(code.here());
    code.bytes(&[0x0F, 0x0B]);

    let dispatch = code.here();
    // cmp rax, JUMP
    code.bytes(&[0x48, 0x83, 0xF8, super::JUMP as u8]);
    code.jump_if(JUMP_IF_NOT_EQUAL, Label::Finish);
    // mov eax, [rbx + AT_OFFSET]
    code.bytes(&[0x8B, 0x43, AT_OFFSET as u8]);
    let dispatch_jump = code.here();
    // mov rcx, [rbx + TABLE_OFFSET]
    code.bytes(&[0x48, 0x8B, 0x4B, TABLE_OFFSET as u8]);
    // movsxd rdx, dword [rcx + rax * 4]
    code.bytes(&[0x48, 0x63, 0x14, 0x81]);
    // add rdx, rcx
    code.bytes(&[0x48, 0x01, 0xCA]);
    // jmp rdx
    code.bytes(&[0xFF, 0xE2]);

    let finish = code.here();
    // pop r14; pop r13; pop r12; pop rbx; pop rbp; ret
    code.bytes(&[0x41, 0x5E, 0x41, 0x5D, 0x41, 0x5C, 0x5B, 0x5D, 0xC3]);

    for (at, label) in std::mem::take(&mut code.fixups) {
        let target = match label {
            Label::Instruction(index) => starts[index],
            Label::Dispatch => dispatch,
            Label::DispatchJump => dispatch_jump,
            Label::Finish => finish,
            Label::Local(number) => code.locals[number],
        };
        let displacement = target as i64 - (at as i64 + 4);
        code.bytes[at..at + 4].copy_from_slice(&(displacement as i32).to_le_bytes());
    }

    let table_offset = code.bytes.len().next_multiple_of(4);
    code.bytes.resize(table_offset, 0xCC);
    for start in &starts[..steps.len()] {
        let offset = *start as i64 - table_offset as i64;
        code.bytes(&(offset as i32).to_le_bytes());
    }
    Emitted {
        bytes: code.bytes,
        table_offset,
    }
}
