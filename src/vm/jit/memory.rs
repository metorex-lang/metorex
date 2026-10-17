//! Memory machine code runs from. Code is written into chunks mapped for
//! it, and a region freed goes on a list the next code of its size or
//! smaller is written into.
//!
//! On macOS the chunks are mapped with `MAP_JIT`, and a thread makes them
//! writable while it writes and executable again after. Elsewhere a chunk
//! is made writable and then executable again with `mprotect`. Only the
//! thread that runs the program writes or runs machine code.

use std::cell::RefCell;

/// The least a chunk maps.
const CHUNK_BYTES: usize = 4 * 1024 * 1024;

/// The alignment of each region.
const REGION_ALIGNMENT: usize = 16;

thread_local! {
    static HEAP: RefCell<CodeHeap> = RefCell::new(CodeHeap::default());
}

#[derive(Default)]
struct CodeHeap {
    /// The chunk being filled: where its unused part starts and how long
    /// that part is.
    unused: Option<(*mut u8, usize)>,
    /// Regions freed, each where it starts and how long it is.
    freed: Vec<(*mut u8, usize)>,
}

/// Machine code written into executable memory.
pub(super) struct Region {
    start: *mut u8,
    length: usize,
}

impl Region {
    /// A region holding `bytes`, made executable, or None when no memory
    /// could be mapped for it.
    pub(super) fn holding(bytes: &[u8]) -> Option<Region> {
        let length = bytes.len().next_multiple_of(REGION_ALIGNMENT);
        let start = HEAP.with(|heap| heap.borrow_mut().take(length))?;
        // SAFETY: `take` handed out `length` bytes of a chunk nothing else
        // holds, and `bytes` is no longer than that.
        unsafe { write_code(start, bytes) };
        Some(Region { start, length })
    }

    pub(super) fn start(&self) -> *const u8 {
        self.start
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        // A region dropped as the thread ends finds the heap gone with it,
        // and the memory goes when the process does.
        let _ = HEAP.try_with(|heap| heap.borrow_mut().freed.push((self.start, self.length)));
    }
}

impl CodeHeap {
    fn take(&mut self, length: usize) -> Option<*mut u8> {
        if let Some(at) = self.freed.iter().position(|(_, held)| *held >= length) {
            let (start, held) = self.freed.swap_remove(at);
            if held > length {
                // SAFETY: the rest of the freed region is still inside it.
                self.freed
                    .push((unsafe { start.add(length) }, held - length));
            }
            return Some(start);
        }
        let fits = self.unused.is_some_and(|(_, left)| left >= length);
        if !fits {
            let size = length.max(CHUNK_BYTES).next_multiple_of(page_size());
            self.unused = Some((map_chunk(size)?, size));
        }
        let (start, left) = self.unused.expect("a chunk was mapped above");
        // SAFETY: the chunk has `left` bytes from `start`, and `length` of
        // them are taken.
        self.unused = Some((unsafe { start.add(length) }, left - length));
        Some(start)
    }
}

fn page_size() -> usize {
    // SAFETY: sysconf reads a setting of the system.
    let size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    usize::try_from(size).unwrap_or(16 * 1024)
}

#[cfg(target_os = "macos")]
fn map_chunk(size: usize) -> Option<*mut u8> {
    // SAFETY: an anonymous mapping touches no memory already in use.
    let mapped = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_JIT,
            -1,
            0,
        )
    };
    (mapped != libc::MAP_FAILED).then_some(mapped.cast())
}

#[cfg(not(target_os = "macos"))]
fn map_chunk(size: usize) -> Option<*mut u8> {
    // SAFETY: an anonymous mapping touches no memory already in use.
    let mapped = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_EXEC,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    (mapped != libc::MAP_FAILED).then_some(mapped.cast())
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn sys_icache_invalidate(start: *mut libc::c_void, length: libc::size_t);
}

/// Copy `bytes` to `start` and make them ready to run.
///
/// # Safety
///
/// `start` is the start of at least `bytes.len()` bytes of a chunk, which
/// no machine code is running from.
#[cfg(target_os = "macos")]
unsafe fn write_code(start: *mut u8, bytes: &[u8]) {
    unsafe {
        libc::pthread_jit_write_protect_np(0);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), start, bytes.len());
        libc::pthread_jit_write_protect_np(1);
        sys_icache_invalidate(start.cast(), bytes.len());
    }
}

#[cfg(all(not(target_os = "macos"), target_arch = "aarch64"))]
unsafe extern "C" {
    fn __clear_cache(start: *mut libc::c_char, end: *mut libc::c_char);
}

/// Copy `bytes` to `start` and make them ready to run.
///
/// # Safety
///
/// `start` is the start of at least `bytes.len()` bytes of a chunk, which
/// no machine code is running from.
#[cfg(not(target_os = "macos"))]
unsafe fn write_code(start: *mut u8, bytes: &[u8]) {
    let page = page_size();
    let first = (start as usize) & !(page - 1);
    let end = (start as usize + bytes.len()).next_multiple_of(page);
    let pages = first as *mut libc::c_void;
    unsafe {
        libc::mprotect(pages, end - first, libc::PROT_READ | libc::PROT_WRITE);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), start, bytes.len());
        libc::mprotect(pages, end - first, libc::PROT_READ | libc::PROT_EXEC);
        #[cfg(target_arch = "aarch64")]
        __clear_cache(start.cast(), start.add(bytes.len()).cast());
    }
}
