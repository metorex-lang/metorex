//! Memory machine code runs from. Each thread reserves one space for its
//! code, so a unit's code and the code every unit shares are close enough
//! for a direct branch from one to the other. A region freed goes on a list
//! the next code of its size or smaller is written into.
//!
//! On macOS the space is mapped with `MAP_JIT`, and a thread makes it
//! writable while it writes and executable again after. Elsewhere the pages
//! written are made writable and then executable again with `mprotect`.
//! Only the thread that runs the program writes or runs machine code.

use std::cell::RefCell;

/// The space each thread reserves for its code. A direct branch on arm64
/// reaches 128 MB either way.
const SPACE_BYTES: usize = 128 * 1024 * 1024;

/// The alignment of each region.
const REGION_ALIGNMENT: usize = 16;

thread_local! {
    static HEAP: RefCell<CodeHeap> = RefCell::new(CodeHeap::default());
}

#[derive(Default)]
struct CodeHeap {
    /// Where the space starts, once it is mapped, or None when mapping it
    /// failed.
    space: Option<Option<*mut u8>>,
    /// How much of the space has been handed out.
    used: usize,
    /// Regions freed, each where it starts and how long it is.
    freed: Vec<(*mut u8, usize)>,
}

/// Memory in the space that machine code is written into.
pub(super) struct Region {
    start: *mut u8,
    length: usize,
}

impl Region {
    /// A region of at least `length` bytes, or None when the space is full
    /// or could not be mapped.
    pub(super) fn reserve(length: usize) -> Option<Region> {
        let length = length.max(1).next_multiple_of(REGION_ALIGNMENT);
        let start = HEAP.with(|heap| heap.borrow_mut().take(length))?;
        Some(Region { start, length })
    }

    /// Copy `bytes` to the start of the region and make them ready to run.
    pub(super) fn write(&self, bytes: &[u8]) {
        assert!(
            bytes.len() <= self.length,
            "code is no longer than its region"
        );
        // SAFETY: the region is `length` bytes of the space that nothing
        // else holds, and no machine code runs from it yet.
        unsafe { write_code(self.start, bytes) };
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
        let space = (*self.space.get_or_insert_with(|| map_space(SPACE_BYTES)))?;
        if SPACE_BYTES - self.used < length {
            return None;
        }
        // SAFETY: `used` plus `length` is within the space.
        let start = unsafe { space.add(self.used) };
        self.used += length;
        Some(start)
    }
}

#[cfg(not(target_os = "macos"))]
fn page_size() -> usize {
    // SAFETY: sysconf reads a setting of the system.
    let size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    usize::try_from(size).unwrap_or(16 * 1024)
}

#[cfg(target_os = "macos")]
fn map_space(size: usize) -> Option<*mut u8> {
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
fn map_space(size: usize) -> Option<*mut u8> {
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
/// `start` is the start of at least `bytes.len()` bytes of the space,
/// which no machine code is running from.
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
/// `start` is the start of at least `bytes.len()` bytes of the space,
/// which no machine code is running from.
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
