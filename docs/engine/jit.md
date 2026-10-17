# The JIT

## Decision

- The code generator is written by hand, one emitter for arm64 and one for
  x86-64, in `src/vm/jit/arm64.rs` and `src/vm/jit/x86_64.rs`. Each lays
  out machine code as words or bytes with the branches filled in once the
  targets are placed.
- A unit's machine code calls the interpreter's `step` for each
  instruction, where the interpreter's loop would dispatch on it with
  `match`, and branches to the next instruction's code itself. A `Jump`
  becomes a branch. Any other jump goes through a table of where each
  instruction's code starts, which `step` names by setting the
  instruction to go to.
- Integer `+`, `-`, and the six comparisons run in the machine code when the
  operands the interpreter saw there were all Integers. Their machine code
  checks the operands' tag bits, applies the operation to the tagged
  values, and writes the answer to the register.
- A unit is compiled once it has run `METOREX_JIT_CALL_THRESHOLD` times,
  30 unless set. The JIT is on unless `METOREX_JIT=0`.

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
- The emitters written by hand are 644 lines for both processors, and add
  no dependency.

## What the machine code assumes

Before each statement and before each Integer operation, the machine code
compares a thread's epoch with the epoch its unit was compiled in. These
move the epoch on:

| Event | What depends on it |
| :-- | :-- |
| A method defined or removed, a module included, prepended, or extended | The Integer operations, which stand for the core library's operators |
| A refinement activated | The Integer operations. After the first activation no Integer operation is compiled into machine code again |
| A constant set or removed, or an autoload registered | Nothing the machine code holds. A constant is read through its cache, as in the interpreter |
| A TracePoint or `set_trace_func` enabled | Nothing the machine code holds. `step` fires line events, as the interpreter does |
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

Machine code has no unwind tables, so no panic unwinds through it. `step`
catches one, keeps it, and returns, and the run resumes the panic once the
machine code has returned.

Code is written into chunks of 4 MB or more. On macOS a chunk is mapped
with `MAP_JIT`, and the thread writing code makes it writable with
`pthread_jit_write_protect_np` and executable again after. On Linux a chunk
is made writable and executable again with `mprotect`. A unit's code is
freed when the unit is, and its region holds later code of its size or
smaller.

## Speed

Recorded on 2026-10-17 on an Apple M5 Max with a release build, the fib
benchmark, `fib(32)` once, timed three times each way:

```sh
METOREX_JIT=0 WARMUP_ITRS=0 MIN_BENCH_ITRS=1 MIN_BENCH_TIME=0 target/release/metorex \
  -I benchmarks/ruby-bench/harness benchmarks/ruby-bench/benchmarks/fib.rb
```

| | Wall time |
| :-- | --: |
| `METOREX_JIT=0` | 5.38 to 5.76 s |
| The JIT on | 5.62 to 5.63 s |

Most of `fib`'s instructions are calls and returns, which the machine code
hands to `step`, so it runs them at the interpreter's speed.

## Tests

`tests/integration/cli_tests/jit.rs` runs the examples in `tests/_examples/jit`
under the interpreter and with every unit compiled on its first run, one for
each event in the table above, and checks that both print the same. The
`jit` stage in CI and the "JIT on every unit" steps of `./test.sh` run
`cargo test` and the Ruby spec suite with `METOREX_JIT_CALL_THRESHOLD=1`.
`METOREX_COUNTERS=1` reports the units compiled to machine code and the runs
of machine code that returned to the interpreter.
