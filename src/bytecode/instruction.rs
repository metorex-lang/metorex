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
    /// Apply the operator `BINARY_OPERATORS` holds at `op` to the register
    /// `first` and the one after it.
    BinaryOperator {
        to: u8,
        first: u8,
        op: u8,
    },
    Negate {
        to: u8,
        from: u8,
    },
    Not {
        to: u8,
        from: u8,
    },
    /// Unary `+`.
    Positive {
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
    JumpIfNil {
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
    /// Begin an argument list, for a call with `*`, `**`, or `&` among its
    /// arguments.
    StartArguments,
    /// Add `from` to the argument list.
    PushArgument {
        from: u8,
    },
    /// Add what `*from` spreads to the argument list.
    SplatArgument {
        from: u8,
    },
    /// Add the keywords `**from` passes to the argument list.
    KeywordSplatArgument {
        from: u8,
    },
    /// Take `&from` as the block of the call the argument list is for.
    BlockArgument {
        from: u8,
    },
    /// End the argument list, putting its arguments in an Array.
    ArgumentsToArray {
        to: u8,
    },
    /// Decide how the call the site names, written without a receiver, is
    /// made, before its arguments are evaluated.
    ResolveCall {
        site: u16,
    },
    /// Decide how `namespace::name(arguments)` is made: a method call on
    /// the namespace, or a call of the constant it holds.
    ResolveScopedCall {
        namespace: u8,
        site: u16,
    },
    /// Make the call the last `ResolveCall` or `ResolveScopedCall` decided,
    /// with the arguments after `base`, and put the answer in `base`.
    CallResolved {
        base: u8,
        site: u16,
    },
    /// Whether the name is bound to a Kernel function, which `name[...]`
    /// calls with the subscripts in an Array.
    IsNativeName {
        to: u8,
        name: u16,
    },
    /// Call the Kernel function the site's name is bound to with the
    /// arguments after `base` gathered in an Array.
    CallNativeName {
        base: u8,
        site: u16,
    },
    /// Count the branch `&.` takes for coverage, and jump when `test` is nil.
    SafeNavigation {
        test: u8,
        target: u16,
    },
    /// A Proc of the unit's block literal the operand numbers.
    MakeBlock {
        to: u8,
        literal: u16,
    },
    /// Append what `*from` spreads to the Array in `array`.
    SplatInto {
        array: u8,
        from: u8,
    },
    /// `collection[*subscripts]`, the subscripts the Array in `subscripts`
    /// holds.
    IndexSpread {
        to: u8,
        collection: u8,
        subscripts: u8,
    },
    /// Match the value in `register` against the unit's pattern the operand
    /// numbers, binding what it captures, and put the answer in `register`.
    MatchPattern {
        register: u8,
        pattern: u16,
    },
    /// Bind the local the slot numbers to nil when no scope binds it yet,
    /// as a `for` loop does for the names it binds.
    DeclareLocal {
        slot: u16,
    },
    /// Say that the assignment the local the slot numbers was reserved for
    /// has begun, so a block written in its value closes over the local.
    Unhoist {
        slot: u16,
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
    /// `collection[key]`.
    Index {
        to: u8,
        collection: u8,
        key: u8,
    },
    /// Write `from` through the unit's assignment target the operand
    /// numbers.
    StoreTarget {
        from: u8,
        target: u16,
    },
    /// Read what the unit's assignment target the operand numbers holds now,
    /// for an operator assignment.
    ReadTarget {
        to: u8,
        target: u16,
    },
    /// Spread `from` across the targets of the unit's multiple assignment
    /// the operand numbers.
    AssignMultiple {
        from: u8,
        shape: u16,
    },
    /// What `defined?` answers for the unit's expression the operand
    /// numbers.
    IsDefined {
        to: u8,
        expression: u16,
    },
    /// End the unit with `break`, carrying what `from` holds, for a `break`
    /// written outside any loop of the unit.
    ExitBreak {
        from: u8,
    },
    /// Raise LocalJumpError when the call the running block was attached
    /// to has returned, which is what a `break` from the block does then,
    /// before it leaves the `begin` it is written in.
    BreakCheck {
        from: u8,
    },
    /// End the unit with `next`, carrying what `from` holds.
    ExitNext {
        from: u8,
    },
    /// End the unit with `redo`.
    ExitRedo,
    /// End the unit with `retry`, written outside the rescue bodies of the
    /// unit's own `begin`s.
    ExitRetry,
    /// Put what `$!` names in `to`.
    SaveCurrentException {
        to: u8,
    },
    /// Make `$!` name what `from` holds again, as a `begin` that handled an
    /// exception leaves it.
    RestoreCurrentException {
        from: u8,
    },
    /// Whether the unit's rescue clause the operand numbers takes the
    /// exception in `register`, put in the register after it. The values of
    /// the clause's splatted classes are in the registers after that.
    RescueMatches {
        register: u8,
        clause: u16,
    },
    /// What a rescue clause does as it takes the exception in `from`: a
    /// `rescue` event for a trace, and the name it is given bound.
    EnterRescue {
        from: u8,
        clause: u16,
    },
    /// Raise again the error kept in the slot the operand numbers.
    Reraise {
        slot: u16,
    },
    /// `raise`, written as a statement, with the exception, or what makes
    /// one, in `from` when `written` is 1, and `$!` raised again otherwise.
    RaiseStatement {
        from: u8,
        written: u8,
    },
    /// End the unit with `return`, carrying what `from` holds. `written`
    /// is 1 when a value was written after `return`, which the top level
    /// warns is ignored.
    ExitReturn {
        from: u8,
        written: u8,
    },
    /// The block the running method was called with, which `yield` calls,
    /// raising LocalJumpError when there is none.
    FindBlock {
        to: u8,
    },
    /// Call the block in `base` with the arguments the site lays out, as
    /// `yield` does, and put the answer in `base`.
    CallBlock {
        base: u8,
        site: u16,
    },
    /// Open a scope, as a `Block` statement runs in one.
    PushScope,
    /// Whether the `BEGIN` block written at the instruction's position runs
    /// now: it runs the first time its code unit does and not again.
    BeginBlockOnce {
        to: u8,
    },
    /// Run the unit's class or module body statement the operand numbers by
    /// the body's own rules, with `to` holding the value the body answers.
    BodyStatement {
        to: u8,
        statement: u16,
    },
    /// Bind the constant the name names, as a class or module body binds
    /// one, to the value in `from`.
    BodyConstant {
        from: u8,
        name: u16,
    },
    /// `attr_reader`, `attr_writer`, or `attr_accessor`, by `kind` 0, 1,
    /// or 2, in a class body, with the names in the Array in `names`.
    BodyAttributes {
        to: u8,
        names: u8,
        kind: u8,
    },
    /// `@name = from` in a class body.
    BodyInstanceVariable {
        from: u8,
        name: u16,
    },
    /// `@@name = from` in a class body.
    BodyClassVariable {
        from: u8,
        name: u16,
    },
    /// `alias_method` in a class body, with the new name in `first` and the
    /// old in the register after it.
    BodyAliasMethod {
        first: u8,
    },
    /// `define_method(arguments) { block }` in a class body, with the block
    /// in `block` and the `count` arguments in the registers after it.
    BodyDefineMethod {
        to: u8,
        block: u8,
        count: u8,
    },
    /// The accessors an `attr_accessor` in a `Block` of a class body
    /// defines, for the names in the Array in `names`.
    BodyBlockAccessors {
        names: u8,
    },
    /// When the value in `register` is a method, call it with no arguments
    /// and put what it answers there, as a bare name in a class body does.
    InvokeIfMethod {
        register: u8,
    },
    /// Run the unit's body statement the operand numbers as a statement of a
    /// `Block` in a module body, with the attribute names an `attr_*` there
    /// was given in the Array in `names`.
    ModuleBlockStatement {
        names: u8,
        statement: u16,
    },
    /// Make the unit's definition the operand numbers, putting what it
    /// answers in `to`.
    Define {
        to: u8,
        definition: u16,
    },
    /// Add what `*from` spreads to the attribute names in the Array in
    /// `array`, through `to_a`, as `attr_reader *names` does.
    AttributeSplat {
        array: u8,
        from: u8,
    },
    /// Mark the block in `from`, written on a `super` call, as attached to
    /// the call being made.
    AttachSuperBlock {
        from: u8,
    },
    /// Before the arguments of a `super` written with `&block` are
    /// evaluated, no block stands. `written` is 1 when a block is written on
    /// the call too.
    ClearSuperBlock {
        written: u8,
    },
    /// Call the parent's method of the running method's name, as `super`
    /// does, with the arguments and block the site lays out after `base`,
    /// and put the answer in `base`.
    CallSuper {
        base: u8,
        site: u16,
    },
    /// Begin a loop, during which a `break` ends the loop rather than
    /// returning from the call a surrounding block was attached to.
    EnterLoop,
    /// End the loop `EnterLoop` began.
    LeaveLoop,
    /// A place the thread's turn can be handed over, as each pass through a
    /// loop is.
    Checkpoint,
    /// Count a run of arm `arm` of the branch written at the instruction's
    /// position, for coverage.
    CoverBranch {
        arm: u16,
    },
    /// Count the instruction's line, for coverage.
    CoverLine,
    /// Test the unit's `case` arm the operand numbers against the value in
    /// `subject`, putting whether it matched in the register after it. A
    /// match opens a scope holding what the pattern bound.
    CaseArm {
        subject: u8,
        arm: u16,
    },
    /// Warn about the `when` clauses of the unit's list the operand numbers
    /// that are written the same as one before them.
    WarnDuplicatedWhen {
        list: u16,
    },
    /// Test the unit's `when` arm the operand numbers against the value in
    /// `subject`, putting whether it matched in the register after it. A
    /// match binds what the pattern bound.
    WhenArm {
        subject: u8,
        arm: u16,
    },
    /// Test the unit's `in` pattern the operand numbers against the value in
    /// `subject`, putting whether it matched in the register after it, and
    /// binding the names the pattern reached.
    InArm {
        subject: u8,
        arm: u16,
    },
    /// Record that an `in` clause's guard did not let the match stand.
    GuardFailed,
    /// Make the value in `from` the one a `case`/`in` asks for its elements
    /// once for the whole `case`.
    PushDeconstructed {
        from: u8,
    },
    /// Let go of the value `PushDeconstructed` held.
    PopDeconstructed,
    /// Raise NoMatchingPatternError for the value in `subject`, with what
    /// the one pattern found when `single` is 1.
    NoMatchingPattern {
        subject: u8,
        single: u8,
    },
    /// Close the scope a matching `CaseArm` opened.
    PopScope,
    /// Drop what a jump out of the middle of a call leaves behind: `lists`
    /// argument lists, `routes` call routes, and `scopes` scopes.
    Unwind {
        lists: u8,
        routes: u8,
        scopes: u8,
    },
    /// Whether the unit's flip-flop the operand numbers is on.
    FlipFlopState {
        to: u8,
        flip: u16,
    },
    /// Turn the unit's flip-flop the operand numbers on, or off when `on`
    /// is 0.
    FlipFlopSet {
        on: u8,
        flip: u16,
    },
    /// Whether the line the last read left off at, `$.`, is the Integer
    /// constant the operand numbers.
    LineIs {
        to: u8,
        constant: u16,
    },
    /// The line the last read answered, `$_`.
    LastLine {
        to: u8,
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
            Instruction::BinaryOperator { .. } => "BinaryOperator",
            Instruction::Not { .. } => "Not",
            Instruction::Positive { .. } => "Positive",
            Instruction::Jump { .. } => "Jump",
            Instruction::JumpIfTrue { .. } => "JumpIfTrue",
            Instruction::JumpIfFalse { .. } => "JumpIfFalse",
            Instruction::JumpIfNil { .. } => "JumpIfNil",
            Instruction::Call { .. } => "Call",
            Instruction::StartArguments => "StartArguments",
            Instruction::PushArgument { .. } => "PushArgument",
            Instruction::SplatArgument { .. } => "SplatArgument",
            Instruction::KeywordSplatArgument { .. } => "KeywordSplatArgument",
            Instruction::BlockArgument { .. } => "BlockArgument",
            Instruction::ArgumentsToArray { .. } => "ArgumentsToArray",
            Instruction::ResolveCall { .. } => "ResolveCall",
            Instruction::ResolveScopedCall { .. } => "ResolveScopedCall",
            Instruction::CallResolved { .. } => "CallResolved",
            Instruction::IsNativeName { .. } => "IsNativeName",
            Instruction::CallNativeName { .. } => "CallNativeName",
            Instruction::SafeNavigation { .. } => "SafeNavigation",
            Instruction::MakeBlock { .. } => "MakeBlock",
            Instruction::SplatInto { .. } => "SplatInto",
            Instruction::IndexSpread { .. } => "IndexSpread",
            Instruction::MatchPattern { .. } => "MatchPattern",
            Instruction::GetLocal { .. } => "GetLocal",
            Instruction::Unhoist { .. } => "Unhoist",
            Instruction::DeclareLocal { .. } => "DeclareLocal",
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
            Instruction::ExitBreak { .. } => "ExitBreak",
            Instruction::ExitNext { .. } => "ExitNext",
            Instruction::ExitRedo => "ExitRedo",
            Instruction::BreakCheck { .. } => "BreakCheck",
            Instruction::ExitRetry => "ExitRetry",
            Instruction::SaveCurrentException { .. } => "SaveCurrentException",
            Instruction::RestoreCurrentException { .. } => "RestoreCurrentException",
            Instruction::RescueMatches { .. } => "RescueMatches",
            Instruction::EnterRescue { .. } => "EnterRescue",
            Instruction::Reraise { .. } => "Reraise",
            Instruction::RaiseStatement { .. } => "RaiseStatement",
            Instruction::ExitReturn { .. } => "ExitReturn",
            Instruction::FindBlock { .. } => "FindBlock",
            Instruction::CallBlock { .. } => "CallBlock",
            Instruction::PushScope => "PushScope",
            Instruction::BeginBlockOnce { .. } => "BeginBlockOnce",
            Instruction::Define { .. } => "Define",
            Instruction::BodyStatement { .. } => "BodyStatement",
            Instruction::BodyConstant { .. } => "BodyConstant",
            Instruction::BodyAttributes { .. } => "BodyAttributes",
            Instruction::BodyInstanceVariable { .. } => "BodyInstanceVariable",
            Instruction::BodyClassVariable { .. } => "BodyClassVariable",
            Instruction::BodyAliasMethod { .. } => "BodyAliasMethod",
            Instruction::BodyDefineMethod { .. } => "BodyDefineMethod",
            Instruction::InvokeIfMethod { .. } => "InvokeIfMethod",
            Instruction::BodyBlockAccessors { .. } => "BodyBlockAccessors",
            Instruction::ModuleBlockStatement { .. } => "ModuleBlockStatement",
            Instruction::AttributeSplat { .. } => "AttributeSplat",
            Instruction::AttachSuperBlock { .. } => "AttachSuperBlock",
            Instruction::ClearSuperBlock { .. } => "ClearSuperBlock",
            Instruction::CallSuper { .. } => "CallSuper",
            Instruction::EnterLoop => "EnterLoop",
            Instruction::LeaveLoop => "LeaveLoop",
            Instruction::Checkpoint => "Checkpoint",
            Instruction::CoverBranch { .. } => "CoverBranch",
            Instruction::CoverLine => "CoverLine",
            Instruction::CaseArm { .. } => "CaseArm",
            Instruction::PopScope => "PopScope",
            Instruction::WarnDuplicatedWhen { .. } => "WarnDuplicatedWhen",
            Instruction::WhenArm { .. } => "WhenArm",
            Instruction::InArm { .. } => "InArm",
            Instruction::GuardFailed => "GuardFailed",
            Instruction::PushDeconstructed { .. } => "PushDeconstructed",
            Instruction::PopDeconstructed => "PopDeconstructed",
            Instruction::NoMatchingPattern { .. } => "NoMatchingPattern",
            Instruction::Unwind { .. } => "Unwind",
            Instruction::FlipFlopState { .. } => "FlipFlopState",
            Instruction::FlipFlopSet { .. } => "FlipFlopSet",
            Instruction::LineIs { .. } => "LineIs",
            Instruction::LastLine { .. } => "LastLine",
            Instruction::Index { .. } => "Index",
            Instruction::StoreTarget { .. } => "StoreTarget",
            Instruction::ReadTarget { .. } => "ReadTarget",
            Instruction::AssignMultiple { .. } => "AssignMultiple",
            Instruction::IsDefined { .. } => "IsDefined",
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

/// The operators `BinaryOperator` applies, by its `op` operand.
pub const BINARY_OPERATORS: [crate::ast::BinaryOp; 22] = {
    use crate::ast::BinaryOp::*;
    [
        Add,
        Subtract,
        Multiply,
        Divide,
        Modulo,
        Power,
        Equal,
        CaseEqual,
        NotEqual,
        Less,
        Greater,
        LessEqual,
        GreaterEqual,
        Spaceship,
        BitwiseAnd,
        BitwiseOr,
        Xor,
        AddAssign,
        SubtractAssign,
        MultiplyAssign,
        DivideAssign,
        Assign,
    ]
};

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
