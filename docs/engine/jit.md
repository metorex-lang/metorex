# The JIT

## Decision

- The code generator is written by hand, one emitter for arm64 and one for
  x86-64, in `src/vm/jit/arm64.rs` and `src/vm/jit/x86_64.rs`. Each lays
  out machine code as words or bytes with the branches filled in once the
  targets are placed.
- A unit's machine code runs these instructions itself:
  - loading an inline value: `LoadInteger`, `LoadNil`, `LoadTrue`,
    `LoadFalse`, `LoadConstant` of an inline constant, and `LoadSymbol` of
    the Symbol the literal made when it last ran, while the source encoding
    is the one it was made under
  - `Move` between registers that hold inline values
  - `Jump`, `JumpIfTrue`, `JumpIfFalse`, and `JumpIfNil`
  - `Return`
  - `CoverBranch` while coverage is not measured
  - Integer `+`, `-`, `*`, `%`, and the six comparisons, when the operands
    the interpreter saw there were all Integers
- `GetLocal`, `SetLocal`, `Unhoist`, `GetIdentifier`, `CallResolved`,
  `Checkpoint`, an instance variable read from the shape its site saw, and
  an instance variable write each call a Rust function for that instruction alone. Every other
  instruction calls `run_instruction`, which runs it as the interpreter
  does. A heap value in the way of an inline one, such as a register to
  overwrite that holds one, leaves the instruction to `run_instruction`
  too.
- A `GetIdentifier` of a name that may call a method on `self`, one
  starting with a lowercase letter or `_` other than `self`, calls a Rust
  function that keeps the method the name called last. While the name is
  read in a method's own scope, which reads through nothing but the root
  scope, neither scope binds it, the root scope's names and the method
  state are as they were, and `self` is an instance with no singleton
  class or methods of its own while no refinement is active, the next read
  calls that method without the interpreter's scope and method lookups.
  The scope may be a block's too, with `self` read from the root scope
  when the block binds none. A name bound to the method a `def` made, as a
  `def` at the top level binds its name and a block captures it, is called
  on `self` while the binding holds that method and the method state is as
  it was when the class the `def` defined it in, or Object, held it under
  that name. Any other read is the interpreter's, and a site whose reads
  turn out otherwise four times, as one that reads a local does, leaves
  every later read to it.
- A `Call` written with a receiver, rather than sent with `send`, calls a
  Rust function that keeps, for the class of the values it was made on,
  whether the interpreter's lookup would end at the value's native method:
  the name is no private method of every object, Enumerable does not stand
  in for it, and no method of the program's answers it. While the receiver
  is a nil, true, false, Integer, Float, Symbol, String, Array, or Hash
  with no singleton class, no refinement is active, and the method state
  is as it was, the call goes straight to the native method. One that has
  no answer, and every other call, goes through the interpreter's lookup.
  A site made on four values that are none of these is compiled as the
  interpreter's call from the next run of its unit on, so a unit with
  nothing else machine code runs itself is left to the interpreter.
- A `SetInstanceVariable` keeps the shape it found `self` with, the shape
  the write left it with, and the slot it wrote. While `self` is an
  instance that is not frozen and has that first shape, and the name does
  not start with `__`, the value goes into the slot, or into a new last
  slot as the object moves to the second shape. Every other write is the
  interpreter's.
- A `StoreTarget` of `receiver.name = value` keeps, for the receiver's
  class and shape under the method state, the slot that `name=` writes
  when it is a method writing one instance variable from its one
  parameter, as `attr_writer` and `Struct` define them, an explicit
  receiver may call it, and no native method answers `name=` for the
  class. While the receiver is an instance of that class and shape that
  is not frozen, nothing traces calls, and a variable named with `__` is
  one the instance already lists, the value goes into the slot without a
  call.
- A `Call` with no arguments or block on an instance, whose call site's
  cache holds a method answering one instance variable, as `attr_reader`
  and `Struct` define them, for which no native method answers, reads the
  variable without a call while nothing traces calls, and counts the
  cache's hit as the call would. A bare name whose method is such a reader
  reads the variable the same way.
- Two or more instructions in a row that would each call into Rust are
  one call, which runs them in turn. A jump to one after the first enters
  that call from there.
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
- Each unit has its own threshold of runs before it is compiled: 30 times
  its instructions over those of them machine code runs itself, counting a
  `GetIdentifier` of a name that may call a method, a `Call` written with
  a receiver, an instance variable write, and a store through a writer,
  so a unit of Integer operations and branches is compiled after
  30 runs and one of mostly calls into Rust waits longer. A unit whose
  machine code would run none of its instructions itself, reads no name that
  may call a method, makes no call written with a receiver, writes no
  instance variable, and stores through no writer, only calls
  into Rust and its `Return`, is left to the interpreter, since entering
  machine code and leaving it again takes longer than the interpreter takes
  to dispatch them.
- A unit whose machine code returned to the interpreter waits twice as
  many runs as before to be compiled again.
- A loop going back counts as a run, so a loop in a unit run once is
  compiled, and goes on in machine code from where it goes back to.
- `METOREX_JIT_CALL_THRESHOLD` sets one threshold for every unit in place
  of each unit's own, and with it a unit is compiled after that many runs
  again after returning to the interpreter. The JIT is on unless
  `METOREX_JIT=0`.

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
| A method named `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, or `>=` defined, removed, aliased, or given a visibility, or a module answering one of them included, prepended, or extended | The Integer operations, which stand for the core library's operators |
| A refinement activated | The Integer operations. After the first activation no Integer operation is compiled into machine code again |
| A TracePoint or `set_trace_func` enabled | The statement starts, which fire no line event |
| `Coverage.start` | The statement starts and the skipped `CoverBranch` instructions, which count nothing |
| `Timeout.timeout` opening a limit | The statement starts, which check no limit |

Other method changes, a constant set or removed, an autoload registered, and
a `binding` taken leave the epoch where it is. A method call is made through
its call site's cache and a constant read through its own, as in the
interpreter, and locals live in cells a binding shares, never in machine
registers.

The Integer operations also check their operands: an operand that is not an
inline Integer returns the run to the interpreter. An instance variable read
whose site has seen one shape checks that `self` has it, and returns the run
to the interpreter when it does not.

A check that fails returns from the machine code with the instruction the
interpreter resumes at. The unit's machine code is dropped, and the unit is
compiled again once it has run twice its threshold, from what the
interpreter has seen by then. A site that returned to the
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

The README's speed table, recorded on 2026-10-18 on an Apple M5 Max with
`ruby benchmarks/compare.rb`, times Metorex with the JIT and with
`METOREX_JIT=0`. Its Metorex columns:

| Benchmark | `METOREX_JIT=0` | The JIT on | JIT ÷ interpreter |
| :-- | --: | --: | --: |
| empty | 26.0 ms | 26.3 ms | 1.01 |
| block_calls | 247.0 ms | 223.2 ms | 0.90 |
| hash_access | 276.8 ms | 268.9 ms | 0.97 |
| method_calls | 222.0 ms | 211.6 ms | 0.95 |
| object_allocation | 593.9 ms | 579.7 ms | 0.98 |
| string_building | 201.4 ms | 194.9 ms | 0.97 |
| cycles | 587.0 ms | 599.3 ms | 1.02 |
| no_cycles | 344.0 ms | 347.7 ms | 1.01 |
| 30k_ifelse | 5.84 s | 6.20 s | 1.06 |
| 30k_methods | 3.40 s | 3.53 s | 1.04 |
| 30k_variables | 18.07 s | 16.34 s | 0.90 |
| attr_accessor | 5.48 s | 5.22 s | 0.95 |
| binarytrees | 6.76 s | 6.51 s | 0.96 |
| block_methods | 31.27 s | 27.48 s | 0.88 |
| blurhash | 4.09 s | 3.85 s | 0.94 |
| cfunc_itself | 761.5 ms | 749.5 ms | 0.98 |
| etanni | 2.45 s | 2.42 s | 0.99 |
| fannkuchredux | 3.98 s | 3.34 s | 0.84 |
| fib | 4.98 s | 4.54 s | 0.91 |
| getivar | 550.0 ms | 230.7 ms | 0.42 |
| getivar-module | 1.97 s | 1.42 s | 0.72 |
| keyword_args | 10.14 s | 9.99 s | 0.98 |
| knucleotide | 328.1 ms | 308.0 ms | 0.94 |
| loops-times | 13.14 s | 12.17 s | 0.93 |
| matmul | 5.74 s | 5.01 s | 0.87 |
| nbody | 1.55 s | 1.44 s | 0.93 |
| nqueens | 2.33 s | 1.98 s | 0.85 |
| object-new | 1.01 s | 912.8 ms | 0.91 |
| object-new-initialize | 2.49 s | 2.38 s | 0.96 |
| object-new-no-escape | 5.55 s | 5.39 s | 0.97 |
| protoboeuf | 3.11 s | 2.84 s | 0.91 |
| protoboeuf-encode | 22.22 s | 21.95 s | 0.99 |
| respond_to | 4.50 s | 4.50 s | 1.00 |
| ruby-json | 18.47 s | 18.06 s | 0.98 |
| ruby-xor | 2.12 s | 1.96 s | 0.92 |
| send_bmethod | 6.38 s | 6.32 s | 0.99 |
| send_cfunc_block | 8.87 s | 9.01 s | 1.02 |
| send_rubyfunc_block | 5.42 s | 5.33 s | 0.98 |
| send_rubyfunc_inline | 10.87 s | 8.66 s | 0.80 |
| setivar | 337.8 ms | 198.7 ms | 0.59 |
| setivar_object | 491.2 ms | 349.7 ms | 0.71 |
| setivar_young | 494.7 ms | 399.0 ms | 0.81 |
| splay | 2.07 s | 2.05 s | 0.99 |
| str_concat | 1.11 s | 990.9 ms | 0.89 |
| struct-new-no-escape | 2.37 s | 2.22 s | 0.93 |
| structaref | 3.71 s | 3.29 s | 0.89 |
| structaset | 8.47 s | 8.31 s | 0.98 |
| sudoku | 6.02 s | 5.12 s | 0.85 |
| throw | 407.7 ms | 417.9 ms | 1.02 |
| rex_suite | 4.81 s | 4.82 s | 1.00 |

The JIT is slower on 30k_ifelse and 30k_methods, which run thousands of
methods of a few instructions, each calling the next 30 deep. Where most of
the time is spent in calls, which the machine code hands to the
interpreter, the two take about as long. Another program was running on
the machine during the run, so a difference of 2% or less is within what
two runs of the same build differ by.

## Tests

`tests/integration/cli_tests/jit.rs` runs the examples in `tests/_examples/jit`
under the interpreter and with every unit compiled on its first run, one for
each event in the table above, and checks that both print the same. The
`jit` stage in CI and the "JIT on every unit" steps of `./test.sh` run
`cargo test` and the Ruby spec suite with `METOREX_JIT_CALL_THRESHOLD=1`.
`METOREX_COUNTERS=1` reports the units compiled to machine code and the runs
of machine code that returned to the interpreter.
