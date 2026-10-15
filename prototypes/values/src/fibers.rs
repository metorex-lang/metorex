//! What a fiber costs to make and to switch to, on a native stack of the
//! size Metorex gives one now and of the size CRuby gives one, and what its
//! own segment of the value stack adds.

use corosensei::stack::DefaultStack;
use corosensei::{Coroutine, CoroutineResult};
use std::hint::black_box;

/// The native stack Metorex gives a fiber now, `FIBER_STACK_BYTES`.
pub const METOREX_STACK_BYTES: usize = 32 * 1024 * 1024;
/// CRuby 4.0's `fiber_machine_stack_size`.
pub const CRUBY_STACK_BYTES: usize = 512 * 1024;
/// CRuby 4.0's `fiber_vm_stack_size`, in 8-byte values.
pub const SEGMENT_VALUES: usize = 128 * 1024 / 8;

/// Make `count` fibers on native stacks of `stack_bytes`, resume each to a
/// suspension and again to its end, and answer the sum of what they handed
/// back.
pub fn make_and_finish(count: usize, stack_bytes: usize) -> u64 {
    let mut sum = 0u64;
    for index in 0..count as u64 {
        let stack = DefaultStack::new(stack_bytes).expect("fiber stack");
        let mut fiber = Coroutine::with_stack(stack, move |yielder, first: u64| {
            let resumed = yielder.suspend(first + index);
            resumed + 1
        });
        if let CoroutineResult::Yield(handed) = fiber.resume(1) {
            sum += handed;
        }
        if let CoroutineResult::Return(answered) = fiber.resume(2) {
            sum += answered;
        }
    }
    sum
}

/// Make and drop `count` value stack segments.
pub fn make_segments(count: usize) -> usize {
    let mut made = 0;
    for _ in 0..count {
        let segment: Vec<u64> = Vec::with_capacity(SEGMENT_VALUES);
        made += black_box(&segment).capacity();
    }
    made
}

/// Resume one fiber `count` times, each running to its next suspension.
pub fn switches(count: usize) -> u64 {
    let stack = DefaultStack::new(CRUBY_STACK_BYTES).expect("fiber stack");
    let mut fiber = Coroutine::with_stack(stack, |yielder, mut handed: u64| {
        loop {
            handed = yielder.suspend(handed + 1);
        }
    });
    let mut sum = 0u64;
    for step in 0..count as u64 {
        let CoroutineResult::Yield(handed) = fiber.resume(step);
        sum += handed;
    }
    sum
}
