# The JIT

## Decision

- The code generator is written by hand, one emitter for arm64 and one for
  x86-64, in `src/vm/jit/arm64.rs` and `src/vm/jit/x86_64.rs`. Each lays
  out machine code as words or bytes with the branches filled in once the
  targets are placed.
- A unit's machine code runs these instructions itself:
  - loading an inline value: `LoadInteger`, `LoadNil`, `LoadTrue`,
    `LoadFalse`, and `LoadConstant` of an inline constant
  - `Move` between registers that hold inline values
  - `Jump`, `JumpIfTrue`, `JumpIfFalse`, and `JumpIfNil`
  - `Return`
  - `CoverBranch` while coverage is not measured
  - Integer `+`, `-`, `*`, `%`, and the six comparisons, when the operands
    the interpreter saw there were all Integers
- `GetLocal`, `SetLocal`, `Unhoist`, `GetIdentifier`, `Checkpoint`, and an
  instance variable read from the shape its site saw each call a Rust
  function for that instruction alone. Every other instruction calls
  `run_instruction`, which runs it as the interpreter does. A heap value
  in the way of an inline one, such as a register to overwrite that holds
  one, leaves the instruction to `run_instruction` too.
- A statement starts in a subroutine at the end of its unit's code, one
  for each set of registers to empty. It empties them and tests the
  signal and collection flags, and counts the statement for the turn
  between threads, which is all the interpreter does at a statement while
  nothing traces lines, measures coverage, or holds a `Timeout.timeout`
  limit. When a register holds a heap value, a flag is set, or the count
  is due for reading the clock, it calls the interpreter's start of a
  statement instead. Otherwise each statement starts through `step`.
- Starting a run, finishing it, going to an instruction by its place, and
  the trampolines that call into Rust are code every unit on a thread
  shares, written once, so a unit's code is its instructions.
- A unit is compiled once it has run `METOREX_JIT_CALL_THRESHOLD` times, 30
  unless set. A loop going back counts as a run, so a loop in a unit run
  once is compiled, and goes on in machine code from where it goes back
  to. The JIT is on unless `METOREX_JIT=0`.

## Why not a code generator library

The choice was between Cranelift, `dynasm-rs`, and emitters written by
hand.

- Cranelift compiles its own intermediate form, with a register allocator,
  for both processors. Code that is mostly calls into the interpreter gains
  nothing from its optimizer, and its crates would be compiled into every
  build.
- `dynasm-rs` assembles code written in assembly syntax inside Rust macros.
  It covers both processors, but the code each instruction needs is a few
  fixed sequences, which a function per sequence writes as directly.
- The emitters written by hand are 1,528 lines for both processors, and
  add no dependency.

## What the machine code assumes

Before each statement, each Integer operation, and each `CoverBranch` it
skips, the machine code compares the epoch with the epoch its unit was
compiled in. The epoch is shared by every thread, so moving it on one sends
every thread's machine code back to the interpreter, which costs a
compilation and is never wrong. These move the epoch on:

| Event | What depends on it |
| :-- | :-- |
| A method defined or removed, a module included, prepended, or extended | The Integer operations, which stand for the core library's operators |
| A refinement activated | The Integer operations. After the first activation no Integer operation is compiled into machine code again |
| A constant set or removed, or an autoload registered | Nothing the machine code holds. A constant is read through its cache, as in the interpreter |
| A TracePoint or `set_trace_func` enabled | The statement starts, which fire no line event |
| `Coverage.start` | The statement starts and the skipped `CoverBranch` instructions, which count nothing |
| `Timeout.timeout` opening a limit | The statement starts, which check no limit |
| A `binding` taken | Nothing the machine code holds. Locals live in cells the binding shares, never in machine registers |

The Integer operations also check their operands: an operand that is not an
inline Integer returns the run to the interpreter. An instance variable read
whose site has seen one shape checks that `self` has it, and returns the run
to the interpreter when it does not.

A check that fails returns from the machine code with the instruction the
interpreter resumes at. The unit's machine code is dropped, and the unit is
compiled again once it has run the threshold's number of times again, from
what the interpreter has seen by then. A site that returned to the
interpreter because of what it saw is left to `step` from then on.

## Unwinding and memory

Machine code has no unwind tables, so no panic unwinds through it. Each
function machine code calls catches one, keeps it, and returns, and the run
resumes the panic once the machine code has returned.

Each thread reserves 128 MB for its code, which a direct branch on arm64
reaches from end to end, so every unit branches straight to the shared
code. On macOS the space is mapped with `MAP_JIT`, and the thread writing
code makes it writable with `pthread_jit_write_protect_np` and executable
again after. On Linux the pages written are made writable and executable
again with `mprotect`. A unit's code is freed when the unit is, and its
region holds later code of its size or smaller. A unit whose code does not
fit is left to the interpreter.

## Speed

Recorded on 2026-10-17 on an Apple M5 Max with a release build. Each time
is the fastest timed iteration of three runs of a ruby-bench benchmark
under its harness, each run one warmup iteration and two timed ones, as
`benchmarks/compare.rb` runs Metorex:

```sh
METOREX_JIT=0 WARMUP_ITRS=1 MIN_BENCH_ITRS=2 MIN_BENCH_TIME=0 ../../target/release/metorex \
  -I harness benchmarks/fib.rb
```

run from `benchmarks/ruby-bench`. sudoku and throw were not timed.

| Benchmark | `METOREX_JIT=0` | The JIT on | JIT ÷ interpreter |
| :-- | --: | --: | --: |
| 30k_ifelse | 5.87 s | 6.11 s | 1.04 |
| 30k_methods | 3.38 s | 3.52 s | 1.04 |
| 30k_variables | 16.96 s | 15.23 s | 0.90 |
| attr_accessor | 5.46 s | 5.19 s | 0.95 |
| binarytrees | 6.73 s | 6.49 s | 0.97 |
| block_methods | 29.77 s | 27.88 s | 0.94 |
| blurhash | 4.00 s | 3.83 s | 0.96 |
| cfunc_itself | 754 ms | 715 ms | 0.95 |
| etanni | 2.40 s | 2.38 s | 0.99 |
| fannkuchredux | 3.88 s | 3.30 s | 0.85 |
| fib | 4.95 s | 4.67 s | 0.94 |
| getivar | 548 ms | 232 ms | 0.42 |
| getivar-module | 1.93 s | 1.42 s | 0.74 |
| keyword_args | 10.16 s | 10.07 s | 0.99 |
| knucleotide | 301 ms | 295 ms | 0.98 |
| loops-times | 12.69 s | 12.34 s | 0.97 |
| matmul | 5.62 s | 5.00 s | 0.89 |
| nbody | 1.51 s | 1.43 s | 0.95 |
| nqueens | 2.25 s | 1.93 s | 0.86 |
| object-new | 958 ms | 893 ms | 0.93 |
| object-new-initialize | 2.46 s | 2.32 s | 0.94 |
| object-new-no-escape | 5.46 s | 5.38 s | 0.98 |
| protoboeuf | 3.10 s | 2.83 s | 0.91 |
| protoboeuf-encode | 22.56 s | 21.89 s | 0.97 |
| respond_to | 4.38 s | 4.35 s | 0.99 |
| ruby-json | 17.84 s | 17.70 s | 0.99 |
| ruby-xor | 2.02 s | 1.92 s | 0.95 |
| send_bmethod | 6.14 s | 6.10 s | 0.99 |
| send_cfunc_block | 8.66 s | 8.46 s | 0.98 |
| send_rubyfunc_block | 5.15 s | 5.16 s | 1.00 |
| send_rubyfunc_inline | 10.40 s | 8.49 s | 0.82 |
| setivar | 324 ms | 202 ms | 0.62 |
| setivar_object | 474 ms | 338 ms | 0.71 |
| setivar_young | 473 ms | 331 ms | 0.70 |
| splay | 2.00 s | 1.96 s | 0.98 |
| str_concat | 1.05 s | 928 ms | 0.88 |
| struct-new-no-escape | 2.28 s | 2.17 s | 0.95 |
| structaref | 3.64 s | 3.36 s | 0.92 |
| structaset | 8.38 s | 8.31 s | 0.99 |

30k_ifelse and 30k_methods each run thousands of methods of a few
instructions, every one run few enough times that its machine code is
not in the processor's caches when it runs, and the interpreter runs them
faster. Where most of the time is spent in calls, which the machine code
hands to the interpreter, the two take about as long.

## Tests

`tests/integration/cli_tests/jit.rs` runs the examples in `tests/_examples/jit`
under the interpreter and with every unit compiled on its first run, one for
each event in the table above, and checks that both print the same. The
`jit` stage in CI and the "JIT on every unit" steps of `./test.sh` run
`cargo test` and the Ruby spec suite with `METOREX_JIT_CALL_THRESHOLD=1`.
`METOREX_COUNTERS=1` reports the units compiled to machine code and the runs
of machine code that returned to the interpreter.
