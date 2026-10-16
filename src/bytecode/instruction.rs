//! The instructions `VirtualMachine` runs: register instructions, 4 bytes
//! each, as `docs/engine/instruction-format.md` describes.
//!
//! An operand named `left` or `right` below 128 names a register, and one at
//! 128 or above names the unit's constant at the operand less 128. Every
//! other 8-bit operand names a register. A 16-bit operand names a constant,
//! a name, a call site, or a jump target, by its index in the unit. An
//! `Extend` before an instruction supplies the high bits of its operands.

/// One instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// Supplies the high bits of the next instruction's operands: `high` is
    /// put above each 8-bit operand's bits, and `wide` above the 16-bit
    /// operand's.
    Extend {
        high: u8,
        wide: u16,
    },
    Move {
        to: u8,
        from: u8,
    },
    LoadConstant {
        to: u8,
        constant: u16,
    },
    LoadInteger {
        to: u8,
        value: i16,
    },
    LoadNil {
        to: u8,
    },
    LoadTrue {
        to: u8,
    },
    LoadFalse {
        to: u8,
    },
    LoadSelf {
        to: u8,
    },
    Add {
        to: u8,
        left: u8,
        right: u8,
    },
    Subtract {
        to: u8,
        left: u8,
        right: u8,
    },
    Multiply {
        to: u8,
        left: u8,
        right: u8,
    },
    Divide {
        to: u8,
        left: u8,
        right: u8,
    },
    Modulo {
        to: u8,
        left: u8,
        right: u8,
    },
    Equal {
        to: u8,
        left: u8,
        right: u8,
    },
    NotEqual {
        to: u8,
        left: u8,
        right: u8,
    },
    Less {
        to: u8,
        left: u8,
        right: u8,
    },
    LessEqual {
        to: u8,
        left: u8,
        right: u8,
    },
    Greater {
        to: u8,
        left: u8,
        right: u8,
    },
    GreaterEqual {
        to: u8,
        left: u8,
        right: u8,
    },
    Negate {
        to: u8,
        from: u8,
    },
    Not {
        to: u8,
        from: u8,
    },
    Jump {
        target: u16,
    },
    JumpIfTrue {
        test: u8,
        target: u16,
    },
    JumpIfFalse {
        test: u8,
        target: u16,
    },
    /// Call the method the call site names on the receiver in `base`, with
    /// the arguments in the registers after it, and put the answer in
    /// `base`.
    Call {
        base: u8,
        site: u16,
    },
    GetInstanceVariable {
        to: u8,
        name: u16,
    },
    SetInstanceVariable {
        from: u8,
        name: u16,
    },
    GetConstant {
        to: u8,
        name: u16,
    },
    SetConstant {
        from: u8,
        name: u16,
    },
    GetGlobal {
        to: u8,
        name: u16,
    },
    SetGlobal {
        from: u8,
        name: u16,
    },
    /// An Array of the `count` registers from `first`.
    NewArray {
        to: u8,
        first: u8,
        count: u8,
    },
    /// Raise what `from` holds, as `raise` does with one argument.
    Raise {
        from: u8,
    },
    /// End the unit, answering what `from` holds.
    Return {
        from: u8,
    },
}

impl Instruction {
    /// The instruction's name, as the counters and the disassembly write it.
    pub fn name(&self) -> &'static str {
        match self {
            Instruction::Extend { .. } => "Extend",
            Instruction::Move { .. } => "Move",
            Instruction::LoadConstant { .. } => "LoadConstant",
            Instruction::LoadInteger { .. } => "LoadInteger",
            Instruction::LoadNil { .. } => "LoadNil",
            Instruction::LoadTrue { .. } => "LoadTrue",
            Instruction::LoadFalse { .. } => "LoadFalse",
            Instruction::LoadSelf { .. } => "LoadSelf",
            Instruction::Add { .. } => "Add",
            Instruction::Subtract { .. } => "Subtract",
            Instruction::Multiply { .. } => "Multiply",
            Instruction::Divide { .. } => "Divide",
            Instruction::Modulo { .. } => "Modulo",
            Instruction::Equal { .. } => "Equal",
            Instruction::NotEqual { .. } => "NotEqual",
            Instruction::Less { .. } => "Less",
            Instruction::LessEqual { .. } => "LessEqual",
            Instruction::Greater { .. } => "Greater",
            Instruction::GreaterEqual { .. } => "GreaterEqual",
            Instruction::Negate { .. } => "Negate",
            Instruction::Not { .. } => "Not",
            Instruction::Jump { .. } => "Jump",
            Instruction::JumpIfTrue { .. } => "JumpIfTrue",
            Instruction::JumpIfFalse { .. } => "JumpIfFalse",
            Instruction::Call { .. } => "Call",
            Instruction::GetInstanceVariable { .. } => "GetInstanceVariable",
            Instruction::SetInstanceVariable { .. } => "SetInstanceVariable",
            Instruction::GetConstant { .. } => "GetConstant",
            Instruction::SetConstant { .. } => "SetConstant",
            Instruction::GetGlobal { .. } => "GetGlobal",
            Instruction::SetGlobal { .. } => "SetGlobal",
            Instruction::NewArray { .. } => "NewArray",
            Instruction::Raise { .. } => "Raise",
            Instruction::Return { .. } => "Return",
        }
    }
}

/// Where an operand of an arithmetic or comparison instruction reads from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Register(usize),
    Constant(usize),
}

/// The first operand that names a constant rather than a register.
pub const FIRST_CONSTANT_OPERAND: u8 = 128;

impl Operand {
    /// Read an arithmetic operand, with `high` from an `Extend` before it.
    pub fn read(operand: u8, high: usize) -> Operand {
        let low = usize::from(operand & (FIRST_CONSTANT_OPERAND - 1));
        let index = high * usize::from(FIRST_CONSTANT_OPERAND) + low;
        if operand >= FIRST_CONSTANT_OPERAND {
            Operand::Constant(index)
        } else {
            Operand::Register(index)
        }
    }
}
