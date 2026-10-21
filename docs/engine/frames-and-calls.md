# Frames and calls

## Decision

- Each thread and each fiber has a segment of value stack, a fixed block of
  values that never moves, and a list of frame records beside it. A thread's
  segment is 1 MB and a fiber's 128 KB, CRuby 4.0's `thread_vm_stack_size`
  and `fiber_vm_stack_size`. A call that would pass the end of the segment
  raises SystemStackError.
- A call pushes the receiver and the arguments, then a frame record: the
  unit, the instruction to resume the caller at, where the callee's locals
  start, self, the block handler, and the environment. The arguments become
  the callee's first locals where they lie, its other locals are pushed as
  nil, and a return cuts the stack back to below the receiver and pushes the
  answer. A call from bytecode to bytecode stays in the same run of the
  interpreter loop, so a Ruby call nests no Rust call.
- An optional parameter's default runs from an entry point chosen by the
  number of arguments. Keyword arguments are pushed as values, with the call
  site's keyword names kept beside the instruction, and the callee moves
  them into its keyword locals. A splat is spread onto the stack. An Array
  is made only for a `*` parameter and a Hash only for a `**` parameter.
- A block passed to a call is a block handler in the callee's frame record:
  the block's unit, self, and the environment it was written in. `yield`
  pushes a frame for the block as a call does. A Proc is made only when the
  block is kept, by a `&block` parameter, `proc`, `lambda`, or `Proc.new`.
- A frame reads and writes its locals through a pointer to its first local,
  which points into the stack segment. A unit that writes a block keeps its
  locals in an environment it makes when it starts, one allocation, which
  its blocks, the Procs made from them, and their bindings share. `binding`
  called in a unit with no environment copies the frame's locals into a new
  one and points the frame at it. It is reached through `send(:binding)` in
  `core/binding/fixtures/classes.rb`, so the compiler cannot see every call
  of it.
- A native method reads its arguments from the caller's part of the stack.
  A block it calls runs in a new run of the interpreter loop on the same
  stack, which returns when the block's frame does.
- A fiber has its own segment and frame list, and a native stack of 512 KB,
  CRuby 4.0's `fiber_machine_stack_size`, for the native methods running in
  it. The interpreter checks the native stack's room when a run starts and
  raises SystemStackError when it is short.

## Why a fiber keeps a native stack

`Enumerator#next` runs the enumerator's `each` on a fiber, in
`src/vm/prelude/enumerator.rs`, and the block it passes calls `Fiber.yield`.
`Array#each` is native, so the fiber suspends with `Array#each`'s Rust frames
between its start and the `Fiber.yield`, and those frames have to be kept
until it resumes. A fiber with no native stack of its own would need every
native method that calls a block to be written in Ruby or to keep its state
where the interpreter can resume it.

The native stack shrinks from 32 MB, `FIBER_STACK_BYTES` in
`src/vm/fibers/state.rs`, because Ruby calls no longer nest Rust calls: it
holds native methods and the runs of the interpreter they start, and the
Ruby frames are in the segment.

## Calls now

The tree walker nests Rust calls for each Ruby call, and `invoke_method`
takes its arguments as a `Vec<Object>`. From `METOREX_COUNTERS=1` on the two
workloads, and their times less the 0.04 s `benchmarks/startup/empty.rb`
takes, on 2026-10-15:

| Workload | Calls | Allocations per call | Time per call |
| :-- | --: | --: | --: |
| method_calls | 400,000 | 43 | 1.03 µs |
| block_calls | 400,001 | 49 | 1.25 µs |

The time per call includes walking the syntax tree of each body.

## The prototype

`prototypes/values/src/calls.rs` runs the two workloads in the prototype's
instructions over the 8-byte tagged value, in two frame layouts:

- **Heap frames**: each call gathers its arguments into a vector, and makes
  a frame on the heap holding a vector of locals, with an operand stack of
  its own. A block reaches its writer's locals through the writer's frame,
  which it keeps alive.
- **Contiguous stack**: the decision above. Each unit that writes a block
  makes an environment when it starts.

In both, `Integer#times` is native and calls its block from Rust, and
`Proc#call` pushes the Proc's frame. `method_calls` runs 10,000,000 steps and
`block_calls` 2,000,000, past which its total no longer fits in an Integer
held inline.

```sh
cargo run --release --manifest-path prototypes/values/Cargo.toml
```

Recorded on 2026-10-15 on arm64, with rustc
1.96.0. Each time is the fastest of five runs.

| Workload | Calls | Heap frames | Contiguous stack | Heap ÷ stacked | Heap frames, per call | Contiguous stack, per call |
| :-- | --: | --: | --: | --: | --: | --: |
| method_calls | 20000000 | 1403.7 ms | 367.8 ms | 3.82× | 70.2 ns | 18.4 ns |
| block_calls | 4000001 | 320.2 ms | 102.6 ms | 3.12× | 80.0 ns | 25.6 ns |

| Workload | Heap frames, allocations per call | Contiguous stack, allocations per call |
| :-- | --: | --: |
| method_calls | 4.00 | 0.00 |
| block_calls | 4.00 | 0.00 |

### Fibers

`prototypes/values/src/fibers.rs` makes 100,000 fibers, resumes each to a
`suspend` and again to its end, and drops it.

| A fiber made, suspended once, and finished | Per fiber |
| :-- | --: |
| Native stack of 32 MB (now) | 15169 ns |
| Native stack of 512 KB | 1957 ns |
| A value stack segment of 128 KB, made and dropped | 71 ns |

A resume and the suspension it runs to take 2.2 ns.

## Alternatives

- **A frame per call on the heap**, the heap frames column: four
  allocations per call, and 3.12 to 3.82 times the time per call.
- **Fibers with no native stack**: every native method that calls a block
  would have to be resumable, as above.
- **Moving a frame's locals to the heap only when a Proc or a binding is
  made**, instead of making an environment when a unit that writes a block
  starts. It saves that allocation when the block is only yielded to, and
  every block frame running at the time reads the locals through the stack,
  so each of those frames has to be pointed at the moved locals.
