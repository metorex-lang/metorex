//! The JIT. A unit run often enough is compiled to machine code that calls
//! the interpreter's `step` for each instruction in turn, where the
//! interpreter's loop would dispatch on it, and branches to the next
//! instruction itself. A `Jump` becomes a branch, and every other jump goes
//! through a table of where each instruction's code starts.
//!
//! Integer arithmetic and comparisons whose operands have only been
//! Integers run in the machine code itself. That rests on assumptions, and
//! the machine code checks them before such an instruction and before each
//! statement: that the operands are Integers, and that nothing has moved the
//! epoch on since the unit was compiled. A method defined or removed, a
//! constant set, a refinement activated, a TracePoint or `set_trace_func`
//! enabled, or a `binding` taken moves it on. An instance variable read
//! checks the shape of `self` against the one its site saw. A check that
//! fails returns the run to the interpreter at that instruction, and the
//! unit is compiled again once it has run often enough again.
//!
//! It is on unless `METOREX_JIT=0`, and `METOREX_JIT_CALL_THRESHOLD` sets
//! how many runs of a unit come before it is compiled.

mod memory;

#[cfg(target_arch = "aarch64")]
mod arm64;
#[cfg(target_arch = "x86_64")]
mod x86_64;

use crate::bytecode::instruction::{Instruction, Operand};
use crate::bytecode::unit::{StartKind, Unit};
use crate::error::MetorexError;
use crate::object::Object;
use crate::object::shape::Shape;
use crate::vm::bytecode_runner::{Next, UnitRun};
use crate::vm::{ControlFlow, VirtualMachine};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// How many runs of a unit come before it is compiled, unless
/// `METOREX_JIT_CALL_THRESHOLD` says otherwise.
const DEFAULT_CALL_THRESHOLD: u32 = 30;

/// What `step` and the machine code's checks hand back: go on to the next
/// instruction, return, jump to the instruction in `Context::at`, or go
/// back to the interpreter at the instruction in `Context::at`.
const NEXT: u64 = 0;
const FINISH: u64 = 1;
const JUMP: u64 = 2;
const DEOPTIMIZE: u64 = 3;
/// The unit returned the value in the register `Context::at` names.
const RETURNED: u64 = 4;

/// What an arithmetic instruction's operands have been, as `observe` sees
/// them.
const SEEN_INTEGERS: u8 = 1;
const SEEN_OTHERS: u8 = 2;

/// The epoch every check of the machine code's assumptions compares with
/// the one its unit was compiled in. It is shared by every thread, so a
/// change on one thread sends the others' machine code back to the
/// interpreter too, which costs a compilation and is never wrong.
static EPOCH: AtomicU64 = AtomicU64::new(0);

/// Whether a refinement has been activated on any thread, after which no
/// Integer operator is taken to be the core library's.
static REFINED: AtomicBool = AtomicBool::new(false);

/// Move the epoch on, so machine code compiled before goes back to the
/// interpreter at its next check.
pub(crate) fn assumptions_changed() {
    EPOCH.fetch_add(1, Ordering::Relaxed);
}

/// Note that a refinement was activated.
pub(crate) fn refinement_activated() {
    REFINED.store(true, Ordering::Relaxed);
    assumptions_changed();
}

fn epoch() -> u64 {
    EPOCH.load(Ordering::Relaxed)
}

struct Settings {
    enabled: bool,
    call_threshold: u32,
}

fn settings() -> &'static Settings {
    static SETTINGS: std::sync::OnceLock<Settings> = std::sync::OnceLock::new();
    SETTINGS.get_or_init(|| Settings {
        enabled: std::env::var("METOREX_JIT").map_or(true, |value| value != "0"),
        call_threshold: std::env::var("METOREX_JIT_CALL_THRESHOLD")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_CALL_THRESHOLD)
            .max(1),
    })
}

/// Whether the JIT is on, so the interpreter records what it compiles from.
pub(crate) fn enabled() -> bool {
    settings().enabled
}

/// Where a unit keeps how often it has run, what its arithmetic
/// instructions' operands have been, and the machine code it was compiled
/// to.
#[derive(Default)]
pub struct JitSlot {
    runs: Cell<u32>,
    seen: RefCell<Vec<u8>>,
    code: RefCell<Option<Rc<MachineCode>>>,
    /// Set when there is no emitter for this processor or no memory for
    /// the code.
    refused: Cell<bool>,
}

impl std::fmt::Debug for JitSlot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JitSlot")
            .field("runs", &self.runs.get())
            .field("compiled", &self.code.borrow().is_some())
            .finish()
    }
}

/// Record what the operands of the arithmetic instruction at `at` are,
/// which decides whether the machine code applies it to Integers itself.
pub(crate) fn observe(
    unit: &Unit,
    at: usize,
    instruction: Instruction,
    registers: &[Object],
    high: usize,
) {
    let Some((_, left, right)) = arithmetic_operands(instruction) else {
        return;
    };
    let value = |operand: u8| match Operand::read(operand, high) {
        Operand::Register(index) => registers[index].bits(),
        Operand::Constant(index) => unit.constants[index].bits(),
    };
    let seen = if value(left) & value(right) & 1 == 1 {
        SEEN_INTEGERS
    } else {
        SEEN_OTHERS
    };
    mark_seen(unit, at, seen);
}

fn mark_seen(unit: &Unit, at: usize, seen: u8) {
    let mut held = unit.jit.seen.borrow_mut();
    if held.len() < unit.instructions.len() {
        held.resize(unit.instructions.len(), 0);
    }
    held[at] |= seen;
}

/// A unit's machine code and what it was compiled under.
pub(crate) struct MachineCode {
    region: memory::Region,
    /// Where the shared code that starts running a unit at any instruction
    /// is. A run from the first instruction starts at the region's start.
    entry: usize,
    table_offset: usize,
    epoch: u64,
    /// Set once a check found an assumption broken, so the unit is
    /// compiled again rather than run through this code.
    broken: Cell<bool>,
    /// For each instance variable read run from its shape, the shape and
    /// slot, by instruction.
    slot_reads: Vec<Option<SlotRead>>,
    /// The flag for a due collection of the thread the code was compiled
    /// on, which is the thread it runs on, since the memory it is in
    /// belongs to that thread.
    collection_due: *const u8,
}

#[derive(Clone, Copy)]
struct SlotRead {
    shape: *const Shape,
    slot: usize,
    to: usize,
}

/// The machine code `unit` runs as, once it has run often enough to be
/// compiled.
pub(crate) fn code_for(vm: &VirtualMachine, unit: &Unit) -> Option<Rc<MachineCode>> {
    let settings = settings();
    if !settings.enabled {
        return None;
    }
    let held = unit.jit.code.borrow().clone();
    if let Some(code) = held {
        if !code.broken.get() && code.epoch == epoch() {
            return Some(code);
        }
        *unit.jit.code.borrow_mut() = None;
        unit.jit.runs.set(0);
    }
    if unit.jit.refused.get() {
        return None;
    }
    let runs = unit.jit.runs.get() + 1;
    unit.jit.runs.set(runs);
    if runs < settings.call_threshold {
        return None;
    }
    let compiled = compile(vm, unit);
    match &compiled {
        Some(code) => *unit.jit.code.borrow_mut() = Some(Rc::clone(code)),
        None => unit.jit.refused.set(true),
    }
    compiled
}

/// Machine code laid out by an emitter: its bytes and where its table
/// starts.
struct Emitted {
    bytes: Vec<u8>,
    table_offset: usize,
    /// The branches to the shared code, filled in once the unit's place
    /// is known.
    links: Vec<Link>,
}

/// A branch in a unit's code to the code every unit shares.
#[derive(Clone, Copy)]
struct Link {
    /// Where the branch is in the unit's bytes.
    at: usize,
    to: Shared,
    /// Whether the branch is one whose offset is shorter, on processors
    /// that have one.
    short: bool,
}

/// The code every unit on a thread shares: where machine code starts,
/// decides where to go after a call, finishes, goes back to the
/// interpreter, and calls into Rust.
#[derive(Clone, Copy, PartialEq)]
enum Shared {
    /// Save what a run of machine code saves and load what it keeps in
    /// registers, for a unit's own start, which saved the frame.
    Setup,
    Dispatch,
    Finish,
    /// Go back to the interpreter at the instruction in the second
    /// argument register.
    Deoptimize,
    Call(Callee),
}

/// What machine code calls for an instruction it does not run itself.
#[derive(Clone, Copy, PartialEq)]
enum Callee {
    Step,
    RunInstruction,
    Helper(Helper),
}

/// The functions machine code calls for one kind of instruction each.
#[derive(Clone, Copy, PartialEq)]
enum Helper {
    StartStatement,
    GetLocal,
    SetLocal,
    Unhoist,
    GetIdentifier,
    Checkpoint,
    ReadSlot,
}

impl Helper {
    const ALL: [Helper; 7] = [
        Helper::StartStatement,
        Helper::GetLocal,
        Helper::SetLocal,
        Helper::Unhoist,
        Helper::GetIdentifier,
        Helper::Checkpoint,
        Helper::ReadSlot,
    ];

    fn function(self) -> Function {
        match self {
            Helper::StartStatement => start_statement,
            Helper::GetLocal => get_local,
            Helper::SetLocal => set_local,
            Helper::Unhoist => unhoist,
            Helper::GetIdentifier => get_identifier,
            Helper::Checkpoint => checkpoint,
            Helper::ReadSlot => read_slot,
        }
    }
}

/// Where each part of the shared code starts, from the start of its bytes.
struct SharedLayout {
    bytes: Vec<u8>,
    entry: usize,
    setup: usize,
    dispatch: usize,
    finish: usize,
    deoptimize: usize,
    calls: Vec<(Callee, usize)>,
}

/// The shared code written into this thread's space.
struct SharedCode {
    region: memory::Region,
    layout: SharedLayout,
}

impl SharedCode {
    fn address(&self, to: Shared) -> usize {
        let offset = match to {
            Shared::Setup => self.layout.setup,
            Shared::Dispatch => self.layout.dispatch,
            Shared::Finish => self.layout.finish,
            Shared::Deoptimize => self.layout.deoptimize,
            Shared::Call(callee) => {
                self.layout
                    .calls
                    .iter()
                    .find(|(held, _)| *held == callee)
                    .expect("the shared code has a trampoline for every callee")
                    .1
            }
        };
        self.region.start() as usize + offset
    }
}

thread_local! {
    /// This thread's shared code, once a unit has been compiled, or None
    /// when there was no memory for it.
    static SHARED: std::cell::OnceCell<Option<SharedCode>> = const { std::cell::OnceCell::new() };
}

/// The callees the shared code has a trampoline for.
fn callees() -> Vec<Callee> {
    [Callee::Step, Callee::RunInstruction]
        .into_iter()
        .chain(Helper::ALL.into_iter().map(Callee::Helper))
        .collect()
}

fn shared_code() -> Option<SharedCode> {
    let layout = emit_shared(&callees())?;
    let region = memory::Region::reserve(layout.bytes.len())?;
    region.write(&layout.bytes);
    Some(SharedCode { region, layout })
}

/// What the emitters are told about each instruction: whether it starts a
/// statement, and how it runs.
struct Planned {
    start: Start,
    action: Action,
}

/// How the statement an instruction starts is started, after the epoch is
/// checked.
#[derive(Clone, Copy)]
enum Start {
    /// The instruction starts no statement.
    None,
    /// By the machine code, while no line is traced, no coverage measured,
    /// and no `Timeout.timeout` limit open: registers `live` up to `count`
    /// emptied, the statement's place in its body handed to the context
    /// when it has one, and when `checks`, the signal and collection flags
    /// tested and the statement counted. A register holding a heap value, a
    /// flag set, or a count that is due for reading the clock leaves the
    /// start to `start_statement`.
    Inline {
        live: u32,
        count: u32,
        index: Option<u32>,
        checks: bool,
    },
    /// By `start_statement`.
    Call,
}

impl Start {
    fn starts(self) -> bool {
        !matches!(self, Start::None)
    }
}

enum Action {
    /// Call `step`, which starts the statement too.
    Step,
    /// Call the function given, which takes the context and the
    /// instruction's place as `step` does.
    Call(Callee),
    /// Branch to the instruction given, as a `Jump` does.
    Branch(usize),
    /// Check the epoch, apply `operation` to the Integers `left` and
    /// `right`, and write the answer to register `to`. An operand that is
    /// not an Integer goes back to the interpreter. An answer past the
    /// range of an inline Integer, or a register holding a heap value,
    /// leaves the instruction to `run_instruction`.
    Integer {
        operation: IntegerOperation,
        to: u8,
        left: Source,
        right: Source,
    },
    /// Write the inline value `bits` to register `to`, leaving a register
    /// holding a heap value to `run_instruction`.
    Store { to: u8, bits: u64 },
    /// Copy register `from` to register `to` when both hold inline values,
    /// and leave the rest to `run_instruction`.
    Move { to: u8, from: u8 },
    /// Branch to `target` when register `test` holds a value of the kind
    /// `when` names.
    BranchIf {
        test: u8,
        when: Holding,
        target: usize,
    },
    /// Finish the unit with the value in register `from`.
    Return { from: u8 },
    /// Check the epoch and go on, for a branch count while coverage is
    /// off.
    Nothing,
}

/// What a conditional branch tests its register for.
#[derive(Clone, Copy)]
enum Holding {
    Truthy,
    Falsy,
    Nil,
}

/// What the machine code calls for an instruction it does not run itself.
type Function = extern "C" fn(*mut Context, u32) -> u64;

#[derive(Clone, Copy)]
enum IntegerOperation {
    Add,
    Subtract,
    Multiply,
    /// Ruby's `%`, which takes the sign of the divisor. A divisor of 0 is
    /// left to `run_instruction`, which raises.
    Modulo,
    Compare(Comparison),
}

#[derive(Clone, Copy)]
enum Comparison {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// Where an Integer operand comes from: a register, or a constant whose
/// bits are known when the unit is compiled.
#[derive(Clone, Copy)]
enum Source {
    Register(u8),
    Bits(u64),
}

/// The bits a heap value never has, below which a register holds an
/// inline value the machine code may overwrite without a release: nil and
/// false. A value with any of its low three bits set is inline too.
const LARGEST_INLINE_WORD: u64 = 0x08;

/// The bits of `true`, `false`, and `nil`.
const TRUE_BITS: u64 = 0x14;
const FALSE_BITS: u64 = 0x00;
const NIL_BITS: u64 = 0x08;

/// Whether `bits` are an inline value, which needs no reference counted.
fn inline(bits: u64) -> bool {
    bits & 7 != 0 || bits <= LARGEST_INLINE_WORD
}

fn arithmetic_operands(instruction: Instruction) -> Option<(Instruction, u8, u8)> {
    match instruction {
        Instruction::Add { left, right, .. }
        | Instruction::Subtract { left, right, .. }
        | Instruction::Multiply { left, right, .. }
        | Instruction::Divide { left, right, .. }
        | Instruction::Modulo { left, right, .. }
        | Instruction::Equal { left, right, .. }
        | Instruction::NotEqual { left, right, .. }
        | Instruction::Less { left, right, .. }
        | Instruction::LessEqual { left, right, .. }
        | Instruction::Greater { left, right, .. }
        | Instruction::GreaterEqual { left, right, .. } => Some((instruction, left, right)),
        _ => None,
    }
}

/// The Integer operation an instruction applies itself in machine code,
/// with its destination register.
fn integer_operation(instruction: Instruction) -> Option<(IntegerOperation, u8)> {
    Some(match instruction {
        Instruction::Add { to, .. } => (IntegerOperation::Add, to),
        Instruction::Subtract { to, .. } => (IntegerOperation::Subtract, to),
        Instruction::Multiply { to, .. } => (IntegerOperation::Multiply, to),
        Instruction::Modulo { to, .. } => (IntegerOperation::Modulo, to),
        Instruction::Equal { to, .. } => (IntegerOperation::Compare(Comparison::Equal), to),
        Instruction::NotEqual { to, .. } => (IntegerOperation::Compare(Comparison::NotEqual), to),
        Instruction::Less { to, .. } => (IntegerOperation::Compare(Comparison::Less), to),
        Instruction::LessEqual { to, .. } => (IntegerOperation::Compare(Comparison::LessEqual), to),
        Instruction::Greater { to, .. } => (IntegerOperation::Compare(Comparison::Greater), to),
        Instruction::GreaterEqual { to, .. } => {
            (IntegerOperation::Compare(Comparison::GreaterEqual), to)
        }
        _ => return None,
    })
}

fn compile(vm: &VirtualMachine, unit: &Unit) -> Option<Rc<MachineCode>> {
    let compiled_in = epoch();
    let (steps, slot_reads) = plan(vm, unit);
    let mut emitted = emit(&steps)?;
    let region = memory::Region::reserve(emitted.bytes.len())?;
    let entry = SHARED.with(|shared| {
        let shared = shared.get_or_init(shared_code).as_ref()?;
        for link in &emitted.links {
            let from = region.start() as usize + link.at;
            link_branch(&mut emitted.bytes, *link, from, shared.address(link.to))?;
        }
        Some(shared.region.start() as usize + shared.layout.entry)
    })?;
    region.write(&emitted.bytes);
    crate::vm::counters::count_unit_compiled_to_machine_code(emitted.bytes.len());
    Some(Rc::new(MachineCode {
        region,
        entry,
        table_offset: emitted.table_offset,
        epoch: compiled_in,
        broken: Cell::new(false),
        slot_reads,
        collection_due: crate::object::live::due_flag(),
    }))
}

/// How each instruction of `unit` runs in machine code, and which
/// instance variable reads are answered from the shape their site saw.
/// An `Extend`, and the instruction after it, are left to `step`.
fn plan(vm: &VirtualMachine, unit: &Unit) -> (Vec<Planned>, Vec<Option<SlotRead>>) {
    let integers = !REFINED.load(Ordering::Relaxed) && vm.integer_operators_in_force();
    let counting_branches = vm.coverage_running();
    let inline_starts =
        !counting_branches && vm.tracepoints.is_empty() && vm.timeout_limits.is_empty();
    let seen = unit.jit.seen.borrow();
    let mut extended = false;
    let mut slot_reads = vec![None; unit.instructions.len()];
    let steps = unit
        .instructions
        .iter()
        .enumerate()
        .map(|(at, instruction)| {
            let start = match unit.statement_starts[at] {
                None => Start::None,
                Some(statement)
                    if inline_starts
                        && statement
                            .index
                            .is_none_or(|index| i32::try_from(index).is_ok()) =>
                {
                    Start::Inline {
                        live: statement.live as u32,
                        count: unit.register_count as u32,
                        index: statement.index.map(|index| index as u32),
                        checks: match statement.kind {
                            StartKind::Statement { .. } => true,
                            StartKind::BodyStatement { runs } => runs,
                            StartKind::Terminal { .. } | StartKind::InBranch { .. } => false,
                        },
                    }
                }
                Some(_) => Start::Call,
            };
            let starts_statement = start.starts();
            let after_extend = std::mem::replace(
                &mut extended,
                matches!(instruction, Instruction::Extend { .. }),
            );
            if after_extend || extended {
                return Planned {
                    start,
                    action: Action::Step,
                };
            }
            let seen_others = seen.get(at).copied().unwrap_or(0) & SEEN_OTHERS != 0;
            let action = match *instruction {
                Instruction::Jump { target } => Action::Branch(usize::from(target)),
                Instruction::Return { from } => Action::Return { from },
                Instruction::JumpIfTrue { test, target } => Action::BranchIf {
                    test,
                    when: Holding::Truthy,
                    target: usize::from(target),
                },
                Instruction::JumpIfFalse { test, target } => Action::BranchIf {
                    test,
                    when: Holding::Falsy,
                    target: usize::from(target),
                },
                Instruction::JumpIfNil { test, target } => Action::BranchIf {
                    test,
                    when: Holding::Nil,
                    target: usize::from(target),
                },
                Instruction::LoadInteger { to, value } => Action::Store {
                    to,
                    bits: Object::Int(i64::from(value)).bits(),
                },
                Instruction::LoadNil { to } => Action::Store { to, bits: NIL_BITS },
                Instruction::LoadTrue { to } => Action::Store {
                    to,
                    bits: TRUE_BITS,
                },
                Instruction::LoadFalse { to } => Action::Store {
                    to,
                    bits: FALSE_BITS,
                },
                Instruction::LoadConstant { to, constant }
                    if inline(unit.constants[usize::from(constant)].bits()) =>
                {
                    Action::Store {
                        to,
                        bits: unit.constants[usize::from(constant)].bits(),
                    }
                }
                Instruction::Move { to, from } => Action::Move { to, from },
                Instruction::CoverBranch { .. } if !counting_branches => Action::Nothing,
                Instruction::GetLocal { .. } => Action::Call(Callee::Helper(Helper::GetLocal)),
                Instruction::SetLocal { .. } => Action::Call(Callee::Helper(Helper::SetLocal)),
                Instruction::Unhoist { .. } => Action::Call(Callee::Helper(Helper::Unhoist)),
                Instruction::GetIdentifier { .. } => {
                    Action::Call(Callee::Helper(Helper::GetIdentifier))
                }
                Instruction::Checkpoint => Action::Call(Callee::Helper(Helper::Checkpoint)),
                Instruction::GetInstanceVariable { to, name }
                    if !seen_others
                        && let Some((shape, slot)) = unit.slot_caches[usize::from(name)].seen() =>
                {
                    slot_reads[at] = Some(SlotRead {
                        shape,
                        slot,
                        to: usize::from(to),
                    });
                    Action::Call(Callee::Helper(Helper::ReadSlot))
                }
                _ => match integer_operation(*instruction) {
                    Some((operation, to))
                        if integers
                            && !seen_others
                            && let Some((_, left, right)) = arithmetic_operands(*instruction)
                            && let (Some(left), Some(right)) =
                                (source(unit, left), source(unit, right)) =>
                    {
                        Action::Integer {
                            operation,
                            to,
                            left,
                            right,
                        }
                    }
                    _ if starts_statement => Action::Step,
                    _ => Action::Call(Callee::RunInstruction),
                },
            };
            Planned { start, action }
        })
        .collect();
    (steps, slot_reads)
}

/// Where an operand comes from, when it is a register or an Integer
/// constant.
fn source(unit: &Unit, operand: u8) -> Option<Source> {
    match Operand::read(operand, 0) {
        Operand::Register(index) => Some(Source::Register(index as u8)),
        Operand::Constant(index) => {
            let bits = unit.constants[index].bits();
            (bits & 1 == 1).then_some(Source::Bits(bits))
        }
    }
}

#[cfg(target_arch = "aarch64")]
use arm64 as processor;
#[cfg(target_arch = "x86_64")]
use x86_64 as processor;

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
fn emit(steps: &[Planned]) -> Option<Emitted> {
    processor::emit(steps)
}

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
fn emit_shared(callees: &[Callee]) -> Option<SharedLayout> {
    Some(processor::emit_shared(callees))
}

/// Fill in the branch `link` at address `from` in `bytes` to go to `to`,
/// or None when it is too far for its encoding.
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
fn link_branch(bytes: &mut [u8], link: Link, from: usize, to: usize) -> Option<()> {
    processor::link_branch(bytes, link, from, to)
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn emit(_steps: &[Planned]) -> Option<Emitted> {
    None
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn emit_shared(_callees: &[Callee]) -> Option<SharedLayout> {
    None
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn link_branch(_bytes: &mut [u8], _link: Link, _from: usize, _to: usize) -> Option<()> {
    None
}

/// What the machine code and `step` share while a unit runs. The machine
/// code reads the fields up to `run_instruction` at the offsets below.
#[repr(C)]
struct Context {
    step: Function,
    at: u32,
    table: *const u8,
    registers: *mut Object,
    epoch: *const u64,
    compiled_in: u64,
    run_instruction: Function,
    statement_index: u64,
    signal_pending: *const u8,
    collection_due: *const u8,
    statements_run: *mut u64,
    vm: *mut VirtualMachine,
    run: *mut UnitRun<'static>,
    code: *const MachineCode,
    outcome: Option<Result<ControlFlow, MetorexError>>,
    panic: Option<Box<dyn Any + Send>>,
}

const STEP_OFFSET: u32 = 0;
const AT_OFFSET: u32 = 8;
const TABLE_OFFSET: u32 = 16;
const REGISTERS_OFFSET: u32 = 24;
const EPOCH_OFFSET: u32 = 32;
const COMPILED_IN_OFFSET: u32 = 40;
const RUN_INSTRUCTION_OFFSET: u32 = 48;
const STATEMENT_INDEX_OFFSET: u32 = 56;
const SIGNAL_PENDING_OFFSET: u32 = 64;
const COLLECTION_DUE_OFFSET: u32 = 72;
const STATEMENTS_RUN_OFFSET: u32 = 80;

/// The statements counted between readings of the clock, a power of two.
const STATEMENTS_BETWEEN_CLOCK_READS: u64 =
    crate::vm::statement::running::STATEMENTS_BETWEEN_CLOCK_READS;
const _: () = assert!(STATEMENTS_BETWEEN_CLOCK_READS.is_power_of_two());

/// How a run through machine code ended.
pub(crate) enum Ran {
    Finished(Result<ControlFlow, MetorexError>),
    /// A check failed before the instruction given, which the interpreter
    /// runs from.
    Deoptimized(usize),
}

/// Run `run` through `code` from the instruction at `start`.
pub(crate) fn run(
    vm: &mut VirtualMachine,
    code: &MachineCode,
    run: &mut UnitRun,
    start: usize,
) -> Ran {
    let registers = run.registers_pointer();
    let vm: *mut VirtualMachine = vm;
    let mut context = Context {
        step,
        at: start as u32,
        table: code.region.start().wrapping_add(code.table_offset),
        registers,
        epoch: EPOCH.as_ptr().cast_const(),
        compiled_in: code.epoch,
        run_instruction,
        statement_index: 0,
        signal_pending: crate::vm::signals::pending_flag(),
        collection_due: code.collection_due,
        // SAFETY: `vm` is valid, and the field is written only through
        // this pointer while the code runs.
        statements_run: unsafe { std::ptr::addr_of_mut!((*vm).statements_run) },
        vm,
        run: (run as *mut UnitRun).cast(),
        code,
        outcome: None,
        panic: None,
    };
    // SAFETY: the region holds code an emitter laid out for the C calling
    // convention, taking the context and answering FINISH once `outcome`
    // or `panic` is set, or DEOPTIMIZE with `at` set. Nothing else touches
    // the machine or the run while it runs, since both are reached only
    // through the context.
    let entry_address = if start == 0 {
        code.region.start()
    } else {
        code.entry as *const u8
    };
    let entry = unsafe {
        std::mem::transmute::<*const u8, extern "C" fn(*mut Context) -> u64>(entry_address)
    };
    let context_pointer: *mut Context = &mut context;
    // SAFETY: `vm` is valid, no reference to it is held across these
    // calls, and the context outlives the code.
    unsafe {
        (*vm).hand_statement_to(std::ptr::addr_of_mut!((*context_pointer).statement_index));
    }
    let ended = entry(context_pointer);
    // SAFETY: as above.
    unsafe { (*vm).take_statement_back() };
    if let Some(payload) = context.panic {
        std::panic::resume_unwind(payload);
    }
    if ended == RETURNED {
        return Ran::Finished(Ok(ControlFlow::Value(run.register(context.at as usize))));
    }
    if ended == DEOPTIMIZE {
        crate::vm::counters::count_deoptimization();
        code.broken.set(true);
        return Ran::Deoptimized(context.at as usize);
    }
    Ran::Finished(
        context
            .outcome
            .expect("the machine code returns once step has set the outcome"),
    )
}

/// Run `body` over the machine and the run for the instruction at `at`,
/// and tell the machine code how to go on. A panic is kept in the context
/// rather than unwinding through the machine code, which has no unwind
/// tables, and `run` resumes it once the code has returned.
fn hand_back(
    context: *mut Context,
    at: u32,
    body: impl FnOnce(
        &mut VirtualMachine,
        &mut UnitRun,
        &MachineCode,
        usize,
    ) -> Result<Next, MetorexError>,
) -> u64 {
    // SAFETY: the machine code hands back the context `run` gave it, which
    // lives until the code returns.
    let context = unsafe { &mut *context };
    // SAFETY: `run` made these pointers from references it holds and does
    // not use while the code runs.
    let (vm, run, code) = unsafe { (&mut *context.vm, &mut *context.run, &*context.code) };
    let at_slot = &mut context.at;
    let outcome = &mut context.outcome;
    let stepped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match body(vm, run, code, at as usize) {
            Ok(Next::Step) => NEXT,
            Ok(Next::JumpTo(target)) => {
                *at_slot = target as u32;
                JUMP
            }
            Ok(Next::Finish(flow)) => {
                *outcome = Some(Ok(flow));
                FINISH
            }
            Err(error) => {
                *outcome = Some(Err(error));
                FINISH
            }
        }
    }));
    match stepped {
        Ok(answer) => answer,
        Err(payload) => {
            context.panic = Some(payload);
            FINISH
        }
    }
}

/// Run the instruction at `at` as the interpreter does, with the statement
/// it starts.
extern "C" fn step(context: *mut Context, at: u32) -> u64 {
    hand_back(context, at, |vm, run, _, at| vm.step(run, at, false))
}

/// Run the instruction at `at` without the statement it starts.
extern "C" fn run_instruction(context: *mut Context, at: u32) -> u64 {
    hand_back(context, at, |vm, run, _, at| vm.step_instruction(run, at))
}

/// Start the statement the instruction at `at` starts.
extern "C" fn start_statement(context: *mut Context, at: u32) -> u64 {
    hand_back(context, at, |vm, run, _, at| vm.step_statement(run, at))
}

extern "C" fn get_local(context: *mut Context, at: u32) -> u64 {
    hand_back(context, at, |vm, run, _, at| vm.step_get_local(run, at))
}

extern "C" fn set_local(context: *mut Context, at: u32) -> u64 {
    hand_back(context, at, |vm, run, _, at| vm.step_set_local(run, at))
}

extern "C" fn unhoist(context: *mut Context, at: u32) -> u64 {
    hand_back(context, at, |vm, run, _, at| vm.step_unhoist(run, at))
}

extern "C" fn get_identifier(context: *mut Context, at: u32) -> u64 {
    hand_back(context, at, |vm, run, _, at| {
        vm.step_get_identifier(run, at)
    })
}

extern "C" fn checkpoint(context: *mut Context, at: u32) -> u64 {
    hand_back(context, at, |vm, run, _, at| vm.step_checkpoint(run, at))
}

/// Read an instance variable from the slot its site saw, when `self` still
/// has the shape the site saw. Otherwise the site is left to `step` from
/// the next compilation on, and the run goes back to the interpreter.
extern "C" fn read_slot(context: *mut Context, at: u32) -> u64 {
    let deoptimize = std::cell::Cell::new(false);
    let answer = hand_back(context, at, |vm, run, code, at| {
        let read = code.slot_reads[at].expect("a slot read is planned with its shape");
        match vm.read_slot_of_shape(read.shape, read.slot) {
            Some(value) => run.set_register(read.to, value),
            None => {
                mark_seen(run.unit(), at, SEEN_OTHERS);
                deoptimize.set(true);
            }
        }
        Ok(Next::Step)
    });
    if deoptimize.get() {
        // SAFETY: as in `hand_back`.
        unsafe { (*context).at = at };
        return DEOPTIMIZE;
    }
    answer
}
