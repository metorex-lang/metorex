//! What a run with `METOREX_MEMORY_REPORT=1` writes to standard error at
//! exit: what was resident when the most was, from samples of the resident
//! set taken as the program allocates. A sample counts the heap blocks in
//! use by what allocated them and by whether startup or the program did,
//! the pages malloc holds beyond those blocks, the machine code the JIT
//! wrote, the pages of the files mapped, and the stack. The rows add up to
//! compare with the peak resident set size. The report also lists the
//! objects alive by class.
//!
//! Each heap allocation carries a header naming what allocated it, so its
//! bytes come off the same row when it is freed. The header is only added
//! when the report was asked for, which the allocator reads from the
//! environment the first time it is called. Sampling runs inside the
//! allocator, so it makes no allocation of its own.

pub use crate::memory_category::{Category, attribute};
use std::alloc::{GlobalAlloc, Layout, System};
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU8, AtomicU64, AtomicUsize, Ordering};

const CATEGORIES: usize = 4;
const CATEGORY_NAMES: [&str; CATEGORIES] = [
    "everything else on the heap",
    "the syntax tree",
    "compiled units and their tables",
    "the JIT compiling",
];
/// The phases a block is counted under: startup, then the program.
const PHASES: usize = 2;
const ROWS: usize = CATEGORIES * PHASES;

const MODE_UNKNOWN: u8 = 0;
const MODE_OFF: u8 = 1;
const MODE_ON: u8 = 2;
static MODE: AtomicU8 = AtomicU8::new(MODE_UNKNOWN);

const HEADER_BYTES: usize = 16;

/// How many bytes are allocated between samples of the resident set.
const SAMPLE_STEP: u64 = 256 * 1024;
/// How far the resident set grows past the last sample kept before the
/// next is kept.
const KEPT_SAMPLE_STEP: i64 = 512 * 1024;
/// How far, in tenths, the resident set grows past the last walk of the
/// objects before the next, and the least it grows by.
const WALK_GROWTH_TENTHS: i64 = 11;
const WALK_LEAST_GROWTH: i64 = 1024 * 1024;

#[allow(clippy::declare_interior_mutable_const)]
const ZERO: AtomicI64 = AtomicI64::new(0);
static LIVE: [AtomicI64; ROWS] = [ZERO; ROWS];
static LIVE_HEADERS: AtomicI64 = AtomicI64::new(0);
static LAST_WALK: AtomicI64 = AtomicI64::new(0);
static WALK_WANTED: AtomicBool = AtomicBool::new(false);
/// How many startup steps are running now.
static STARTUP_DEPTH: AtomicUsize = AtomicUsize::new(0);
static MACHINE_CODE_BYTES: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED: AtomicU64 = AtomicU64::new(0);
static NEXT_SAMPLE: AtomicU64 = AtomicU64::new(0);

/// What the sample with the most resident counted.
static KEPT: Kept = Kept {
    resident: AtomicI64::new(0),
    rows: [ZERO; ROWS],
    headers: AtomicI64::new(0),
    malloc_resident: AtomicI64::new(0),
    machine_code: AtomicI64::new(0),
    files: AtomicI64::new(0),
    stack: AtomicI64::new(0),
};

struct Kept {
    resident: AtomicI64,
    rows: [AtomicI64; ROWS],
    headers: AtomicI64,
    malloc_resident: AtomicI64,
    machine_code: AtomicI64,
    files: AtomicI64,
    stack: AtomicI64,
}

thread_local! {
    /// Whether this thread is taking a sample, so one does not start
    /// another.
    static SAMPLING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The objects alive at the last walk, and the resident set then.
struct Walk {
    resident: i64,
    rows: Vec<ClassRow>,
}

/// The objects of one class a walk found.
pub struct ClassRow {
    pub class: String,
    pub count: u64,
    pub bytes: u64,
}

static LAST_WALK_FOUND: Mutex<Option<Walk>> = Mutex::new(None);

/// Whether this run writes the report.
#[inline]
pub fn enabled() -> bool {
    match MODE.load(Ordering::Relaxed) {
        MODE_ON => true,
        MODE_OFF => false,
        _ => decide_mode(),
    }
}

fn decide_mode() -> bool {
    // SAFETY: getenv reads the environment without allocating, and the name
    // is a terminated literal.
    let wanted = unsafe {
        let held = libc::getenv(c"METOREX_MEMORY_REPORT".as_ptr());
        !held.is_null() && *held != 0 && !(*held == b'0' as libc::c_char && *held.add(1) == 0)
    };
    MODE.store(if wanted { MODE_ON } else { MODE_OFF }, Ordering::Relaxed);
    wanted
}

/// Counts what is allocated under startup until it is dropped.
pub struct Starting;

/// Count what is allocated under startup, loading the core library and the
/// libraries a default start loads, until the answer is dropped.
pub fn starting() -> Starting {
    STARTUP_DEPTH.fetch_add(1, Ordering::Relaxed);
    Starting
}

impl Drop for Starting {
    fn drop(&mut self) {
        STARTUP_DEPTH.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Record that the JIT has handed out `used` bytes of its space.
pub fn machine_code_space_used(used: usize) {
    MACHINE_CODE_BYTES.fetch_max(used, Ordering::Relaxed);
}

fn header_bytes(layout: Layout) -> usize {
    layout.align().max(HEADER_BYTES)
}

#[cfg(target_vendor = "apple")]
fn usable_bytes(base: *mut u8) -> usize {
    // SAFETY: `base` is a block the system allocator handed out.
    unsafe { libc::malloc_size(base.cast()) }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn usable_bytes(base: *mut u8) -> usize {
    // SAFETY: `base` is a block the system allocator handed out.
    unsafe { libc::malloc_usable_size(base.cast()) }
}

#[cfg(not(any(target_vendor = "apple", all(target_os = "linux", target_env = "gnu"))))]
fn usable_bytes(_base: *mut u8) -> usize {
    0
}

/// The row this thread's allocations are counted under now.
fn current_row() -> usize {
    crate::memory_category::current() as usize * PHASES
        + usize::from(STARTUP_DEPTH.load(Ordering::Relaxed) == 0)
}

/// Write the header for a block whose usable size is `usable`, and count it
/// under `row`.
///
/// # Safety
///
/// `user` is `header` bytes past the start of a block at least that long.
unsafe fn take_in(user: *mut u8, header: usize, usable: usize, row: usize) {
    let held = usable.saturating_sub(header) as i64;
    // SAFETY: the header's 16 bytes sit just before `user`, inside the block.
    unsafe {
        let at = user.sub(HEADER_BYTES);
        at.add(4).write(row as u8);
        at.add(8).cast::<u64>().write_unaligned(usable as u64);
    }
    LIVE[row].fetch_add(held, Ordering::Relaxed);
    LIVE_HEADERS.fetch_add(header as i64, Ordering::Relaxed);
    let allocated = ALLOCATED.fetch_add(usable as u64, Ordering::Relaxed) + usable as u64;
    if allocated >= NEXT_SAMPLE.load(Ordering::Relaxed) {
        NEXT_SAMPLE.store(allocated + SAMPLE_STEP, Ordering::Relaxed);
        sample();
    }
}

/// Read the resident set, and when it is the most yet by a step, keep what
/// each row holds now.
fn sample() {
    if SAMPLING
        .try_with(|sampling| sampling.replace(true))
        .unwrap_or(true)
    {
        return;
    }
    let (resident, files) = resident_and_file_bytes();
    if resident >= KEPT.resident.load(Ordering::Relaxed) + KEPT_SAMPLE_STEP {
        KEPT.resident.store(resident, Ordering::Relaxed);
        for (kept, live) in KEPT.rows.iter().zip(LIVE.iter()) {
            kept.store(live.load(Ordering::Relaxed), Ordering::Relaxed);
        }
        KEPT.headers
            .store(LIVE_HEADERS.load(Ordering::Relaxed), Ordering::Relaxed);
        KEPT.malloc_resident
            .store(malloc_resident_bytes(), Ordering::Relaxed);
        KEPT.machine_code.store(
            MACHINE_CODE_BYTES
                .load(Ordering::Relaxed)
                .next_multiple_of(page_bytes()) as i64,
            Ordering::Relaxed,
        );
        KEPT.files.store(files, Ordering::Relaxed);
        KEPT.stack
            .store(this_thread_stack_resident_bytes(), Ordering::Relaxed);
        let walked = LAST_WALK.load(Ordering::Relaxed);
        if resident >= (walked * WALK_GROWTH_TENTHS / 10).max(walked + WALK_LEAST_GROWTH) {
            WALK_WANTED.store(true, Ordering::Relaxed);
        }
    }
    let _ = SAMPLING.try_with(|sampling| sampling.set(false));
}

/// Take a block's bytes off its row, answering where the block starts.
///
/// # Safety
///
/// `user` is a block this report handed out with `layout`.
unsafe fn give_back(user: *mut u8, layout: Layout) -> (*mut u8, usize) {
    let header = header_bytes(layout);
    // The caller's pointer is only good for the bytes it asked for, so the
    // block's start comes from the address `alloc` exposed.
    let base = std::ptr::with_exposed_provenance_mut::<u8>(user.addr() - header);
    // SAFETY: the block was handed out `header` bytes past its start, with
    // the header just before `user`.
    unsafe {
        let at = base.add(header - HEADER_BYTES);
        let row = at.add(4).read() as usize;
        let usable = at.add(8).cast::<u64>().read_unaligned() as i64;
        LIVE[row].fetch_sub(usable - header as i64, Ordering::Relaxed);
        LIVE_HEADERS.fetch_sub(header as i64, Ordering::Relaxed);
        (base, row)
    }
}

/// Allocate a block with a header before it.
///
/// # Safety
///
/// As `GlobalAlloc::alloc`.
pub unsafe fn alloc(layout: Layout, zeroed: bool) -> *mut u8 {
    let header = header_bytes(layout);
    // SAFETY: the size grows by at most the alignment, which a valid layout
    // leaves room for.
    let widened =
        unsafe { Layout::from_size_align_unchecked(layout.size() + header, layout.align()) };
    // SAFETY: the widened layout is non-zero in size.
    let base = unsafe {
        if zeroed {
            System.alloc_zeroed(widened)
        } else {
            System.alloc(widened)
        }
    };
    if base.is_null() {
        return base;
    }
    base.expose_provenance();
    let usable = usable_bytes(base).max(widened.size());
    // SAFETY: the block is `header` plus the size asked for long.
    unsafe {
        let user = base.add(header);
        take_in(user, header, usable, current_row());
        user
    }
}

/// Free a block `alloc` handed out.
///
/// # Safety
///
/// As `GlobalAlloc::dealloc`.
pub unsafe fn dealloc(user: *mut u8, layout: Layout) {
    let header = header_bytes(layout);
    // SAFETY: the block was handed out by `alloc` with this layout.
    unsafe {
        let (base, _) = give_back(user, layout);
        System.dealloc(
            base,
            Layout::from_size_align_unchecked(layout.size() + header, layout.align()),
        );
    }
}

/// Resize a block `alloc` handed out, keeping its row.
///
/// # Safety
///
/// As `GlobalAlloc::realloc`.
pub unsafe fn realloc(user: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    let header = header_bytes(layout);
    // SAFETY: the block was handed out by `alloc` with this layout, and its
    // header is copied with it.
    unsafe {
        let (base, row) = give_back(user, layout);
        let moved = System.realloc(
            base,
            Layout::from_size_align_unchecked(layout.size() + header, layout.align()),
            new_size + header,
        );
        let kept = if moved.is_null() { base } else { moved };
        kept.expose_provenance();
        let usable = usable_bytes(kept).max(if moved.is_null() {
            layout.size() + header
        } else {
            new_size + header
        });
        take_in(kept.add(header), header, usable, row);
        if moved.is_null() {
            return moved;
        }
        moved.add(header)
    }
}

/// Whether the resident set has grown enough since the last walk of the
/// objects for another.
#[inline]
pub fn walk_wanted() -> bool {
    WALK_WANTED.load(Ordering::Relaxed)
}

/// Keep what a walk of the objects found.
pub fn record_walk(rows: Vec<ClassRow>) {
    let resident = LAST_WALK.load(Ordering::Relaxed);
    *LAST_WALK_FOUND
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Walk { resident, rows });
}

/// Note that a walk is starting, so what it allocates does not ask for
/// another.
pub fn walk_starting() {
    WALK_WANTED.store(false, Ordering::Relaxed);
    LAST_WALK.store(KEPT.resident.load(Ordering::Relaxed), Ordering::Relaxed);
}

fn page_bytes() -> usize {
    // SAFETY: sysconf reads a setting of the system.
    usize::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).unwrap_or(4096)
}

/// How many bytes of `length` from `start` are resident.
fn resident_bytes(start: usize, length: usize) -> usize {
    let page = page_bytes();
    let first = start / page * page;
    let end = (start + length).next_multiple_of(page);
    let pages = (end - first) / page;
    if pages == 0 {
        return 0;
    }
    let mut flags = [0u8; 1024];
    let mut resident = 0;
    let mut at = first;
    while at < end {
        let length = (end - at).min(flags.len() * page);
        // SAFETY: the range is page aligned and `flags` holds a byte for
        // each of its pages.
        if unsafe { libc::mincore(at as _, length, flags.as_mut_ptr().cast()) } != 0 {
            return resident;
        }
        resident += flags[..length / page]
            .iter()
            .filter(|flag| **flag & 1 != 0)
            .count()
            * page;
        at += length;
    }
    resident
}

/// How many bytes of this thread's stack are resident.
#[cfg(target_vendor = "apple")]
fn this_thread_stack_resident_bytes() -> i64 {
    // SAFETY: both read the calling thread's own stack bounds.
    let (top, size) = unsafe {
        let thread = libc::pthread_self();
        (
            libc::pthread_get_stackaddr_np(thread) as usize,
            libc::pthread_get_stacksize_np(thread),
        )
    };
    resident_bytes(top - size, size) as i64
}

#[cfg(target_os = "linux")]
fn this_thread_stack_resident_bytes() -> i64 {
    // SAFETY: the attributes are read into a struct this frame owns and
    // destroyed after.
    unsafe {
        let mut attributes: libc::pthread_attr_t = std::mem::zeroed();
        if libc::pthread_getattr_np(libc::pthread_self(), &mut attributes) != 0 {
            return 0;
        }
        let mut start = std::ptr::null_mut();
        let mut size = 0;
        libc::pthread_attr_getstack(&attributes, &mut start, &mut size);
        libc::pthread_attr_destroy(&mut attributes);
        resident_bytes(start as usize, size) as i64
    }
}

#[cfg(not(any(target_vendor = "apple", target_os = "linux")))]
fn this_thread_stack_resident_bytes() -> i64 {
    0
}

/// What `task_info` answers for `TASK_VM_INFO`, as far as its first
/// revision.
#[cfg(target_vendor = "apple")]
#[repr(C)]
#[derive(Default)]
struct TaskVmInfo {
    virtual_size: u64,
    region_count: i32,
    page_size: i32,
    resident_size: u64,
    resident_size_peak: u64,
    device: u64,
    device_peak: u64,
    internal: u64,
    internal_peak: u64,
    external: u64,
    external_peak: u64,
    reusable: u64,
    reusable_peak: u64,
    purgeable_volatile_pmap: u64,
    purgeable_volatile_resident: u64,
    purgeable_volatile_virtual: u64,
    compressed: u64,
    compressed_peak: u64,
    compressed_lifetime: u64,
    phys_footprint: u64,
}

#[cfg(target_vendor = "apple")]
const TASK_VM_INFO: u32 = 22;

#[cfg(target_vendor = "apple")]
unsafe extern "C" {
    /// The port naming this task, which `mach_task_self()` reads.
    static mach_task_self_: libc::mach_port_t;
}

/// The resident set, and the part of it that maps files: the binary and
/// the system libraries.
#[cfg(target_vendor = "apple")]
fn resident_and_file_bytes() -> (i64, i64) {
    let mut info = TaskVmInfo::default();
    let mut count = (std::mem::size_of::<TaskVmInfo>() / 4) as u32;
    // SAFETY: task_info writes at most `count` words into `info`.
    let read = unsafe {
        libc::task_info(
            mach_task_self_,
            TASK_VM_INFO,
            (&mut info as *mut TaskVmInfo).cast(),
            &mut count,
        )
    };
    if read != 0 {
        return (0, 0);
    }
    (info.resident_size as i64, info.external as i64)
}

/// What `mach_vm_region_recurse` answers of a region, packed as the header
/// packs it.
#[cfg(target_vendor = "apple")]
#[repr(C, packed(4))]
#[derive(Default)]
struct RegionInfo {
    protection: i32,
    max_protection: i32,
    inheritance: u32,
    offset: u64,
    user_tag: u32,
    pages_resident: u32,
    pages_shared_now_private: u32,
    pages_swapped_out: u32,
    pages_dirtied: u32,
    ref_count: u32,
    shadow_depth: u16,
    external_pager: u8,
    share_mode: u8,
    is_submap: i32,
    behavior: i32,
    object_id: u32,
    user_wired_count: u16,
    pages_reusable: u32,
    object_id_full: u64,
}

#[cfg(target_vendor = "apple")]
unsafe extern "C" {
    fn mach_vm_region_recurse(
        task: libc::mach_port_t,
        address: *mut u64,
        size: *mut u64,
        depth: *mut u32,
        info: *mut RegionInfo,
        count: *mut u32,
    ) -> i32;
}

/// The tags malloc gives the regions it maps: `VM_MEMORY_MALLOC` through
/// `VM_MEMORY_MALLOC_HUGE`, `VM_MEMORY_REALLOC` through
/// `VM_MEMORY_MALLOC_LARGE_REUSED`, `VM_MEMORY_MALLOC_NANO`, and
/// `VM_MEMORY_MALLOC_MEDIUM`.
#[cfg(target_vendor = "apple")]
const MALLOC_REGION_TAGS: [u32; 10] = [1, 2, 3, 4, 6, 7, 8, 9, 11, 12];

/// The resident bytes of the regions malloc maps.
#[cfg(target_vendor = "apple")]
fn malloc_resident_bytes() -> i64 {
    let page = page_bytes() as i64;
    let mut address = 0u64;
    let mut total = 0;
    loop {
        let mut size = 0u64;
        let mut depth = 0u32;
        let mut info = RegionInfo::default();
        let mut count = (std::mem::size_of::<RegionInfo>() / 4) as u32;
        // SAFETY: the call writes one region's address, size, and at most
        // `count` words of what it says of it.
        let read = unsafe {
            mach_vm_region_recurse(
                mach_task_self_,
                &mut address,
                &mut size,
                &mut depth,
                &mut info,
                &mut count,
            )
        };
        if read != 0 {
            return total;
        }
        let tag = info.user_tag;
        if info.is_submap == 0 && MALLOC_REGION_TAGS.contains(&tag) {
            total += i64::from(info.pages_resident) * page;
        }
        address += size;
    }
}

/// The resident set, and the part of it that maps files, from
/// `/proc/self/statm`, read with no allocation.
#[cfg(target_os = "linux")]
fn resident_and_file_bytes() -> (i64, i64) {
    let mut held = [0u8; 256];
    // SAFETY: the path is a terminated literal, and the read fills at most
    // the buffer.
    let read = unsafe {
        let file = libc::open(c"/proc/self/statm".as_ptr(), libc::O_RDONLY);
        if file < 0 {
            return (0, 0);
        }
        let read = libc::read(file, held.as_mut_ptr().cast(), held.len());
        libc::close(file);
        read
    };
    let Ok(read) = usize::try_from(read) else {
        return (0, 0);
    };
    let mut fields = held[..read]
        .split(|byte| byte.is_ascii_whitespace())
        .map(|field| {
            field.iter().fold(0i64, |sum, digit| {
                sum * 10 + i64::from(digit.wrapping_sub(b'0'))
            })
        });
    let page = page_bytes() as i64;
    let _size = fields.next();
    let resident = fields.next().unwrap_or(0);
    let shared = fields.next().unwrap_or(0);
    (resident * page, shared * page)
}

/// The bytes glibc's malloc holds: its arenas and the blocks it mapped on
/// their own.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn malloc_resident_bytes() -> i64 {
    // SAFETY: mallinfo2 reads malloc's own counts.
    let info = unsafe { libc::mallinfo2() };
    (info.arena + info.hblkhd) as i64
}

#[cfg(all(target_os = "linux", not(target_env = "gnu")))]
fn malloc_resident_bytes() -> i64 {
    0
}

#[cfg(not(any(target_vendor = "apple", target_os = "linux")))]
fn resident_and_file_bytes() -> (i64, i64) {
    (0, 0)
}

#[cfg(not(any(target_vendor = "apple", target_os = "linux")))]
fn malloc_resident_bytes() -> i64 {
    0
}

fn peak_resident_bytes() -> u64 {
    // SAFETY: getrusage fills the struct this frame owns.
    let usage = unsafe {
        let mut usage: libc::rusage = std::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut usage);
        usage
    };
    let reported = usage.ru_maxrss as u64;
    if cfg!(target_vendor = "apple") {
        reported
    } else {
        reported * 1024
    }
}

/// Write the report at exit when `METOREX_MEMORY_REPORT` asks for it.
pub fn enable_from_environment() {
    if !enabled() {
        return;
    }
    extern "C" fn report_at_exit() {
        report(&mut std::io::stderr());
    }
    // SAFETY: the handler is a plain function that lives as long as the
    // process does.
    unsafe {
        libc::atexit(report_at_exit);
    }
}

fn megabytes(bytes: i64) -> String {
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

/// Write the report.
pub fn report(out: &mut dyn Write) {
    if !SAMPLING.try_with(|sampling| sampling.get()).unwrap_or(true) {
        sample();
    }
    let peak_resident = peak_resident_bytes() as i64;
    let rows: Vec<i64> = KEPT
        .rows
        .iter()
        .map(|row| row.load(Ordering::Relaxed))
        .collect();
    let headers = KEPT.headers.load(Ordering::Relaxed);
    let heap = rows.iter().sum::<i64>() + headers;
    let malloc_beyond = KEPT.malloc_resident.load(Ordering::Relaxed) - heap;
    let mut text = format!(
        "metorex memory report, process {}\n\npeak resident set size: {}\nthe sample with the most resident: {}\n\n",
        std::process::id(),
        megabytes(peak_resident),
        megabytes(KEPT.resident.load(Ordering::Relaxed)),
    );
    text.push_str(&format!(
        "heap blocks in use at that sample, {}:\n{:>12}  {:>12}  what allocated them\n",
        megabytes(heap),
        "startup",
        "program"
    ));
    for (category, name) in CATEGORY_NAMES.iter().enumerate() {
        text.push_str(&format!(
            "{:>12}  {:>12}  {name}\n",
            megabytes(rows[category * PHASES]),
            megabytes(rows[category * PHASES + 1]),
        ));
    }
    text.push_str(&format!(
        "{:>12}  {:>12}  this report's headers\n",
        megabytes(headers),
        ""
    ));
    text.push_str("\nbeside the heap blocks at that sample:\n");
    let beside = [
        (
            malloc_beyond,
            "malloc's resident pages holding no block in use: blocks freed, its own records, and C libraries' blocks",
        ),
        (
            KEPT.machine_code.load(Ordering::Relaxed),
            "the JIT's machine code",
        ),
        (
            KEPT.files.load(Ordering::Relaxed),
            "the resident pages of the binary and the system libraries",
        ),
        (
            KEPT.stack.load(Ordering::Relaxed),
            "the stack's resident pages",
        ),
    ];
    for (bytes, name) in beside {
        text.push_str(&format!("{:>12}  {name}\n", megabytes(bytes)));
    }
    let accounted = heap + beside.iter().map(|(bytes, _)| bytes).sum::<i64>();
    text.push_str(&format!(
        "\naccounted for: {}, {:.0}% of the peak resident set size\n",
        megabytes(accounted),
        accounted as f64 * 100.0 / peak_resident.max(1) as f64
    ));
    if let Some(walk) = LAST_WALK_FOUND
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
    {
        let mut rows: Vec<&ClassRow> = walk.rows.iter().collect();
        rows.sort_by(|left, right| {
            right
                .bytes
                .cmp(&left.bytes)
                .then(left.class.cmp(&right.class))
        });
        let bytes: u64 = rows.iter().map(|row| row.bytes).sum();
        text.push_str(&format!(
            "\nobjects alive at the sample of {}, {} by their sizes and capacities:\n{:>12}  {:>12}  class\n",
            megabytes(walk.resident),
            megabytes(bytes as i64),
            "objects",
            "bytes"
        ));
        for row in rows.into_iter().take(OBJECT_CLASSES_REPORTED) {
            text.push_str(&format!(
                "{:>12}  {:>12}  {}\n",
                row.count,
                megabytes(row.bytes as i64),
                row.class
            ));
        }
    }
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
}

/// How many classes the objects table lists, the most bytes first.
const OBJECT_CLASSES_REPORTED: usize = 30;
