// The allocator prism's C code calls, named by build/prism/prism_xallocator.h.
// While prism checks a whole file its blocks come from mapped chunks that
// are unmapped when the check ends. Blocks freed one by one would leave
// prism's whole tree resident, in pages the rest of the run does not reuse.

use std::os::raw::c_void;
use std::sync::Mutex;

/// The fewest bytes a chunk maps.
const CHUNK_BYTES: usize = 4 << 20;

/// Each block starts this far into what it takes, after its length, which
/// keeps the block as aligned as malloc's.
const HEADER_BYTES: usize = 16;

/// The chunks of a check running now, or None.
static CHECKING: Mutex<Option<Arena>> = Mutex::new(None);

thread_local! {
    /// Whether the check `CHECKING` holds the chunks of runs on this thread,
    /// where prism's blocks come from them. Prism on any other thread takes
    /// its blocks from malloc.
    static CHECKING_HERE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The arena of the check running on this thread, read with `read`.
fn with_arena<R>(read: impl FnOnce(Option<&mut Arena>) -> R) -> R {
    if !CHECKING_HERE.get() {
        return read(None);
    }
    read(CHECKING.lock().expect("the arena lock").as_mut())
}

struct Arena {
    chunks: Vec<(usize, usize)>,
    /// Where the next block goes in the last chunk, and the bytes left
    /// there.
    next: usize,
    left: usize,
}

// SAFETY: the arena holds addresses of mappings it alone owns.
unsafe impl Send for Arena {}

impl Arena {
    fn allocate(&mut self, size: usize) -> *mut c_void {
        let taken = (size + HEADER_BYTES).next_multiple_of(HEADER_BYTES);
        if taken > self.left {
            let length = taken.max(CHUNK_BYTES);
            // SAFETY: a fresh private anonymous mapping, owned by the arena
            // until it unmaps it.
            let mapped = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    length,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_PRIVATE | libc::MAP_ANON,
                    -1,
                    0,
                )
            };
            if mapped == libc::MAP_FAILED {
                return std::ptr::null_mut();
            }
            self.chunks.push((mapped as usize, length));
            self.next = mapped as usize;
            self.left = length;
        }
        let block = self.next;
        self.next += taken;
        self.left -= taken;
        // SAFETY: the header lies inside the chunk the block was cut from.
        unsafe { (block as *mut usize).write(size) };
        (block + HEADER_BYTES) as *mut c_void
    }

    fn holds(&self, pointer: *mut c_void) -> bool {
        let address = pointer as usize;
        self.chunks
            .iter()
            .any(|&(start, length)| (start..start + length).contains(&address))
    }
}

/// Run `check` with prism's blocks taken from chunks unmapped after it, or
/// from malloc while another thread's check holds the arena.
pub(super) fn in_an_arena<R>(check: impl FnOnce() -> R) -> R {
    {
        let mut checking = CHECKING.lock().expect("the arena lock");
        if checking.is_some() {
            drop(checking);
            return check();
        }
        *checking = Some(Arena {
            chunks: Vec::new(),
            next: 0,
            left: 0,
        });
    }
    CHECKING_HERE.set(true);
    let checked = check();
    CHECKING_HERE.set(false);
    let arena = CHECKING
        .lock()
        .expect("the arena lock")
        .take()
        .expect("the check's arena");
    for (start, length) in arena.chunks {
        // SAFETY: prism keeps none of its blocks once the check returns.
        unsafe { libc::munmap(start as *mut c_void, length) };
    }
    checked
}

#[unsafe(no_mangle)]
extern "C" fn metorex_prism_malloc(size: usize) -> *mut c_void {
    with_arena(|arena| match arena {
        Some(arena) => arena.allocate(size),
        // SAFETY: malloc takes any size.
        None => unsafe { libc::malloc(size) },
    })
}

#[unsafe(no_mangle)]
extern "C" fn metorex_prism_calloc(count: usize, size: usize) -> *mut c_void {
    with_arena(|arena| match arena {
        // A chunk is mapped zeroed and no block in it is used twice.
        Some(arena) => match count.checked_mul(size) {
            Some(bytes) => arena.allocate(bytes),
            None => std::ptr::null_mut(),
        },
        // SAFETY: calloc takes any count and size.
        None => unsafe { libc::calloc(count, size) },
    })
}

#[unsafe(no_mangle)]
extern "C" fn metorex_prism_realloc(pointer: *mut c_void, size: usize) -> *mut c_void {
    with_arena(|arena| match arena {
        Some(arena) if pointer.is_null() => arena.allocate(size),
        Some(arena) if arena.holds(pointer) => {
            // SAFETY: an arena block's length sits just before it.
            let held = unsafe { ((pointer as usize - HEADER_BYTES) as *const usize).read() };
            let moved = arena.allocate(size);
            if !moved.is_null() {
                // SAFETY: both blocks hold at least the bytes copied.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        pointer.cast::<u8>(),
                        moved.cast::<u8>(),
                        held.min(size),
                    )
                };
            }
            moved
        }
        // SAFETY: a block malloc made, which realloc takes back.
        _ => unsafe { libc::realloc(pointer, size) },
    })
}

#[unsafe(no_mangle)]
extern "C" fn metorex_prism_free(pointer: *mut c_void) {
    if with_arena(|arena| arena.is_some_and(|arena| arena.holds(pointer))) {
        return;
    }
    // SAFETY: a block malloc made, or null.
    unsafe { libc::free(pointer) }
}
