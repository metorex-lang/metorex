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
    /// The Symbol the name names, in the encoding of the source it was
    /// written in.
    LoadSymbol {
        to: u8,
        name: u16,
    },
    /// A new String of the literal text the name holds.
    NewString {
        to: u8,
        text: u16,
    },
    /// The Regexp of the unit's pattern the operand numbers.
    LoadRegex {
        to: u8,
        pattern: u16,
    },
    /// `__FILE__`.
    LoadFile {
        to: u8,
    },
    /// `__dir__`.
    LoadDirectory {
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
    /// Read the local the slot numbers.
    GetLocal {
        to: u8,
        slot: u16,
    },
    /// Write the local the slot numbers.
    SetLocal {
        from: u8,
        slot: u16,
    },
    /// Read a name the unit does not assign: a local of the scope around
    /// it, or a method called on self with no arguments.
    GetIdentifier {
        to: u8,
        name: u16,
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
    /// `::Name`.
    GetTopLevelConstant {
        to: u8,
        name: u16,
    },
    /// The constant the name names in the class or module the register
    /// holds, put in the same register.
    GetScopedConstant {
        namespace: u8,
        name: u16,
    },
    GetClassVariable {
        to: u8,
        name: u16,
    },
    SetClassVariable {
        from: u8,
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
    /// Append the `count` registers from `first` to the Array in `array`.
    AppendArray {
        array: u8,
        first: u8,
        count: u8,
    },
    /// A Hash of the registers the unit's Hash shape the operand numbers
    /// lays out.
    NewHash {
        to: u8,
        shape: u16,
    },
    /// Add the entries of the shape the operand numbers to the Hash literal
    /// being built in `hash`.
    AddToHash {
        hash: u8,
        shape: u16,
    },
    /// A Range from the register `first` to the one after it. `flags` holds
    /// `RANGE_EXCLUSIVE` and `RANGE_LITERAL_ENDS`.
    NewRange {
        to: u8,
        first: u8,
        flags: u8,
    },
    /// The text the value in `from` puts in an interpolation's hole.
    ToText {
        to: u8,
        from: u8,
    },
    /// A String of the pieces the unit's interpolation the operand numbers
    /// lays out.
    Interpolate {
        to: u8,
        shape: u16,
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
            Instruction::LoadSymbol { .. } => "LoadSymbol",
            Instruction::NewString { .. } => "NewString",
            Instruction::LoadRegex { .. } => "LoadRegex",
            Instruction::LoadFile { .. } => "LoadFile",
            Instruction::LoadDirectory { .. } => "LoadDirectory",
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
            Instruction::GetLocal { .. } => "GetLocal",
            Instruction::SetLocal { .. } => "SetLocal",
            Instruction::GetIdentifier { .. } => "GetIdentifier",
            Instruction::GetInstanceVariable { .. } => "GetInstanceVariable",
            Instruction::SetInstanceVariable { .. } => "SetInstanceVariable",
            Instruction::GetConstant { .. } => "GetConstant",
            Instruction::GetTopLevelConstant { .. } => "GetTopLevelConstant",
            Instruction::GetScopedConstant { .. } => "GetScopedConstant",
            Instruction::GetClassVariable { .. } => "GetClassVariable",
            Instruction::SetClassVariable { .. } => "SetClassVariable",
            Instruction::SetConstant { .. } => "SetConstant",
            Instruction::GetGlobal { .. } => "GetGlobal",
            Instruction::SetGlobal { .. } => "SetGlobal",
            Instruction::NewArray { .. } => "NewArray",
            Instruction::AppendArray { .. } => "AppendArray",
            Instruction::AddToHash { .. } => "AddToHash",
            Instruction::NewHash { .. } => "NewHash",
            Instruction::NewRange { .. } => "NewRange",
            Instruction::ToText { .. } => "ToText",
            Instruction::Interpolate { .. } => "Interpolate",
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

/// `NewRange` builds a Range that leaves out its end.
pub const RANGE_EXCLUSIVE: u8 = 1;
/// `NewRange` builds a Range written with literal ends, which is one object
/// however many times it runs.
pub const RANGE_LITERAL_ENDS: u8 = 2;

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
