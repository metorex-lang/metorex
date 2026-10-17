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
use crate::bytecode::unit::Unit;
use crate::error::MetorexError;
use crate::object::Object;
use crate::object::shape::Shape;
use crate::vm::bytecode_runner::{Next, UnitRun};
use crate::vm::{ControlFlow, VirtualMachine};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

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

/// What an arithmetic instruction's operands have been, as `observe` sees
/// them.
const SEEN_INTEGERS: u8 = 1;
const SEEN_OTHERS: u8 = 2;

thread_local! {
    /// The epoch every check of the machine code's assumptions compares
    /// with the one its unit was compiled in.
    static EPOCH: Cell<u64> = const { Cell::new(0) };
    /// Whether a refinement has been activated, after which no Integer
    /// operator is taken to be the core library's.
    static REFINED: Cell<bool> = const { Cell::new(false) };
}

/// Move the epoch on, so machine code compiled before goes back to the
/// interpreter at its next check.
pub(crate) fn assumptions_changed() {
    EPOCH.with(|epoch| epoch.set(epoch.get() + 1));
}

/// Note that a refinement was activated.
pub(crate) fn refinement_activated() {
    REFINED.with(|refined| refined.set(true));
    assumptions_changed();
}

fn epoch() -> u64 {
    EPOCH.with(Cell::get)
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
    table_offset: usize,
    epoch: u64,
    /// Set once a check found an assumption broken, so the unit is
    /// compiled again rather than run through this code.
    broken: Cell<bool>,
    /// For each instance variable read run from its shape, the shape and
    /// slot, by instruction.
    slot_reads: Vec<Option<SlotRead>>,
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
}

/// What the emitters are told about each instruction.
enum Step {
    /// Call `step` and go on as it says, after checking the epoch when
    /// `checked`.
    Call { checked: bool },
    /// Branch to the instruction given, as a `Jump` does.
    Branch(usize),
    /// Check the epoch, apply `operation` to the Integers `left` and
    /// `right`, and write the answer to register `to`. An operand that is
    /// not an Integer goes back to the interpreter. An answer past the
    /// range of an inline Integer, or a register holding a heap value,
    /// leaves the instruction to `step`.
    Integer {
        operation: IntegerOperation,
        to: u8,
        left: Source,
        right: Source,
    },
}

#[derive(Clone, Copy)]
enum IntegerOperation {
    Add,
    Subtract,
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

/// The bits of `true` and `false`.
const TRUE_BITS: u64 = 0x14;
const FALSE_BITS: u64 = 0x00;

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
    let emitted = emit(&steps)?;
    let region = memory::Region::holding(&emitted.bytes)?;
    crate::vm::counters::count_unit_compiled_to_machine_code();
    Some(Rc::new(MachineCode {
        region,
        table_offset: emitted.table_offset,
        epoch: compiled_in,
        broken: Cell::new(false),
        slot_reads,
    }))
}

/// How each instruction of `unit` runs in machine code, and which
/// instance variable reads `step` answers from the shape their site saw.
/// An instruction that follows an `Extend` is left to `step`.
fn plan(vm: &VirtualMachine, unit: &Unit) -> (Vec<Step>, Vec<Option<SlotRead>>) {
    let integers = !REFINED.with(Cell::get) && vm.integer_operators_in_force();
    let seen = unit.jit.seen.borrow();
    let mut extended = false;
    let mut slot_reads = vec![None; unit.instructions.len()];
    let steps = unit
        .instructions
        .iter()
        .enumerate()
        .map(|(at, instruction)| {
            let after_extend = std::mem::replace(
                &mut extended,
                matches!(instruction, Instruction::Extend { .. }),
            );
            let starts_statement = unit.statement_starts[at].is_some();
            if after_extend || starts_statement {
                return Step::Call {
                    checked: starts_statement,
                };
            }
            if let Instruction::Jump { target } = instruction {
                return Step::Branch(usize::from(*target));
            }
            let seen_others = seen.get(at).copied().unwrap_or(0) & SEEN_OTHERS != 0;
            if let Instruction::GetInstanceVariable { to, name } = instruction
                && !seen_others
                && let Some((shape, slot)) = unit.slot_caches[usize::from(*name)].seen()
            {
                slot_reads[at] = Some(SlotRead {
                    shape,
                    slot,
                    to: usize::from(*to),
                });
            }
            if integers
                && !seen_others
                && let Some((operation, to)) = integer_operation(*instruction)
                && let Some((_, left, right)) = arithmetic_operands(*instruction)
                && let (Some(left), Some(right)) = (source(unit, left), source(unit, right))
            {
                return Step::Integer {
                    operation,
                    to,
                    left,
                    right,
                };
            }
            Step::Call { checked: false }
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
fn emit(steps: &[Step]) -> Option<Emitted> {
    Some(arm64::emit(steps))
}

#[cfg(target_arch = "x86_64")]
fn emit(steps: &[Step]) -> Option<Emitted> {
    Some(x86_64::emit(steps))
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn emit(_steps: &[Step]) -> Option<Emitted> {
    None
}

/// What the machine code and `step` share while a unit runs. The machine
/// code reads the fields up to `compiled_in` at the offsets below.
#[repr(C)]
struct Context {
    step: extern "C" fn(*mut Context, u32) -> u64,
    at: u32,
    table: *const u8,
    registers: *mut Object,
    epoch: *const u64,
    compiled_in: u64,
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
    let mut context = Context {
        step,
        at: start as u32,
        table: code.region.start().wrapping_add(code.table_offset),
        registers,
        epoch: EPOCH.with(Cell::as_ptr),
        compiled_in: code.epoch,
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
    let entry: extern "C" fn(*mut Context) -> u64 =
        unsafe { std::mem::transmute(code.region.start()) };
    let ended = entry(&mut context);
    if let Some(payload) = context.panic {
        std::panic::resume_unwind(payload);
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

/// Run the instruction at `at` for the machine code. A panic is kept in the
/// context rather than unwinding through the machine code, which has no
/// unwind tables, and `run` resumes it once the code has returned.
extern "C" fn step(context: *mut Context, at: u32) -> u64 {
    // SAFETY: the machine code hands back the context `run` gave it, which
    // lives until the code returns.
    let context = unsafe { &mut *context };
    // SAFETY: `run` made these pointers from references it holds and does
    // not use while the code runs.
    let (vm, run, code) = unsafe { (&mut *context.vm, &mut *context.run, &*context.code) };
    let at = at as usize;
    if let Some(read) = code.slot_reads[at] {
        match vm.read_slot_of_shape(read.shape, read.slot) {
            Some(value) => {
                run.set_register(read.to, value);
                return NEXT;
            }
            None => {
                // The site is left to `step` from the next compilation on.
                mark_seen(run.unit(), at, SEEN_OTHERS);
                context.at = at as u32;
                return DEOPTIMIZE;
            }
        }
    }
    let stepped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| vm.step(run, at)));
    match stepped {
        Ok(Ok(Next::Step)) => NEXT,
        Ok(Ok(Next::JumpTo(target))) => {
            context.at = target as u32;
            JUMP
        }
        Ok(Ok(Next::Finish(flow))) => {
            context.outcome = Some(Ok(flow));
            FINISH
        }
        Ok(Err(error)) => {
            context.outcome = Some(Err(error));
            FINISH
        }
        Err(payload) => {
            context.panic = Some(payload);
            FINISH
        }
    }
}
