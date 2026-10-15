//! What a run with `METOREX_COUNTERS=1` counts, and the report it prints to
//! standard error at exit: each kind of statement and expression evaluated,
//! each call by the method it was made from and the method it reached with
//! the heap allocations made while it ran, and every heap allocation.

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static COUNTS: Mutex<Option<Counts>> = Mutex::new(None);

/// How many call paths the report lists, the most frequent first.
const CALL_PATHS_REPORTED: usize = 100;

#[derive(Default)]
struct Counts {
    statements: HashMap<&'static str, u64>,
    expressions: HashMap<&'static str, u64>,
    calls: HashMap<String, CallCount>,
}

#[derive(Default)]
struct CallCount {
    calls: u64,
    allocations: u64,
}

/// The system allocator, counting each allocation while counters are on.
pub struct CountingAllocator;

// SAFETY: every call is handed to the system allocator unchanged, and the
// counting touches nothing but atomics, so it neither allocates nor blocks.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_allocation(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count_allocation(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, held: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count_allocation(new_size);
        unsafe { System.realloc(held, layout, new_size) }
    }

    unsafe fn dealloc(&self, held: *mut u8, layout: Layout) {
        unsafe { System.dealloc(held, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn count_allocation(size: usize) {
    if ENABLED.load(Ordering::Relaxed) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    }
}

/// Whether this run counts.
#[inline]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Start counting when `METOREX_COUNTERS` is set to anything but `0` or
/// nothing, and print the report when the process exits.
pub fn enable_from_environment() {
    let wanted =
        std::env::var("METOREX_COUNTERS").is_ok_and(|held| !held.is_empty() && held != "0");
    if !wanted {
        return;
    }
    *COUNTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Counts::default());
    ENABLED.store(true, Ordering::Relaxed);
    extern "C" fn report_at_exit() {
        report(&mut std::io::stderr());
    }
    // SAFETY: the handler is a plain function that lives as long as the
    // process does.
    unsafe {
        libc::atexit(report_at_exit);
    }
}

fn with_counts(change: impl FnOnce(&mut Counts)) {
    let mut held = COUNTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(counts) = held.as_mut() {
        change(counts);
    }
}

pub fn count_statement(kind: &'static str) {
    with_counts(|counts| *counts.statements.entry(kind).or_default() += 1);
}

pub fn count_expression(kind: &'static str) {
    with_counts(|counts| *counts.expressions.entry(kind).or_default() += 1);
}

/// How many heap allocations this run has made so far.
pub fn allocations_so_far() -> u64 {
    ALLOCATIONS.load(Ordering::Relaxed)
}

/// Count a call from `caller` to `callee` that made `allocations` heap
/// allocations, counting those the calls it made in turn made.
pub fn count_call(caller: &str, callee: &str, allocations: u64) {
    with_counts(|counts| {
        let held = counts
            .calls
            .entry(format!("{caller} -> {callee}"))
            .or_default();
        held.calls += 1;
        held.allocations += allocations;
    });
}

/// Write the report: the totals, then each table, the largest count first.
pub fn report(out: &mut dyn Write) {
    let Some(counts) = COUNTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
    else {
        return;
    };
    ENABLED.store(false, Ordering::Relaxed);
    let mut text = String::new();
    text.push_str(&format!(
        "metorex counters, process {}\n",
        std::process::id()
    ));
    text.push_str(&format!(
        "heap allocations: {} ({} bytes)\n",
        ALLOCATIONS.load(Ordering::Relaxed),
        ALLOCATED_BYTES.load(Ordering::Relaxed)
    ));
    for (title, table) in [
        ("statements", &counts.statements),
        ("expressions", &counts.expressions),
    ] {
        let total: u64 = table.values().sum();
        text.push_str(&format!("\n{title} evaluated: {total}\n"));
        let mut rows: Vec<_> = table.iter().collect();
        rows.sort_by(|left, right| right.1.cmp(left.1).then(left.0.cmp(right.0)));
        for (kind, count) in rows {
            text.push_str(&format!("{count:>14}  {kind}\n"));
        }
    }
    let total: u64 = counts.calls.values().map(|held| held.calls).sum();
    text.push_str(&format!(
        "\ncalls: {total}, the {CALL_PATHS_REPORTED} most frequent paths, with the heap allocations made during them\n"
    ));
    text.push_str(&format!(
        "{:>14}  {:>14}  caller -> callee\n",
        "calls", "allocations"
    ));
    let mut rows: Vec<_> = counts.calls.iter().collect();
    rows.sort_by(|left, right| right.1.calls.cmp(&left.1.calls).then(left.0.cmp(right.0)));
    for (path, held) in rows.into_iter().take(CALL_PATHS_REPORTED) {
        text.push_str(&format!(
            "{:>14}  {:>14}  {path}\n",
            held.calls, held.allocations
        ));
    }
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
}
