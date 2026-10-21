# Profiling

Two tools say where a run spends its time. A sampling profiler records which
function each thread is in every millisecond. `METOREX_COUNTERS=1` counts what
the interpreter evaluates, the calls it makes, and the heap allocations they
make.

## A sampling profile

`scripts/profile.sh REPORT [metorex arguments...]` builds the `profiling`
profile, a release build with line tables, runs `target/profiling/metorex` with
the arguments given, and writes the profiler's report to `REPORT`.

- On macOS the profiler is `sample`, which comes with the system. The report
  is the call tree of each thread, then a list headed "Sort by top of stack" of
  the functions samples were taken in, the most frequent first.
- On Linux the profiler is `perf`. The script runs
  `perf record --call-graph dwarf` and writes `perf report --stdio --no-children`
  to `REPORT`, with the recording beside it as `REPORT.data`.

Metorex runs the program on a thread of its own. The main thread waits for it
in `__ulock_wait` on macOS, so that entry in the top-of-stack list is time the
program spent, not a function it ran.

## Counters

With `METOREX_COUNTERS` set to anything but `0` or nothing, Metorex prints to
standard error at exit:

- the heap allocations the run made, and their bytes
- the blocks a block literal built. A literal run again in the scope where it
  ran last, while nothing holds the block it made there, hands out that block
  again, so a block written in a loop is built once
- the units the JIT compiled to machine code, the bytes of machine code
  they compiled to, and the runs of machine code that returned to the
  interpreter. An instruction the machine code runs itself, such as an
  Integer operation or a jump, is not counted among the instructions
  executed
- each library parsed from its source, with its name. The core library and
  the libraries a default start loads are parsed when Metorex is built, and
  the binary holds what the parse produced, so a default start lists none
- each kind of instruction executed, with its count
- the 100 most frequent call paths, each written `caller -> callee`, with the
  number of calls and the heap allocations made while those calls ran. A
  method of a class or module itself is written `Name.method`, and a method
  answered in Rust ends in `(native)`. The allocations of a call include those
  of the calls it made, so a recursive method counts its inner calls' again.

A process started with `fork` prints its own report when it exits. `exit!`
prints none.

## The fib benchmark

Recorded on 2026-10-15 on arm64, with Metorex
0.1.0 built by rustc 1.96.0. `fib` calls `fib(32)` once per iteration, 7,049,155
calls of a method that holds an `if`, a comparison, two subtractions, and an
addition.

```sh
WARMUP_ITRS=0 MIN_BENCH_ITRS=1 MIN_BENCH_TIME=0 \
  scripts/profile.sh fib.sample \
  -I benchmarks/ruby-bench/harness benchmarks/ruby-bench/benchmarks/fib.rb

cargo build --release
METOREX_COUNTERS=1 WARMUP_ITRS=0 MIN_BENCH_ITRS=1 MIN_BENCH_TIME=0 \
  target/release/metorex \
  -I benchmarks/ruby-bench/harness benchmarks/ruby-bench/benchmarks/fib.rb
```

The iteration took 10.9 seconds under `sample`. CRuby takes 101 ms without
YJIT and 17 ms with it.

### Where the samples were taken

The program's thread was sampled 7,676 times. Of those in the top-of-stack
list:

| Where | Samples | Share |
| :-- | --: | --: |
| The system allocator: `malloc`, `free`, and the zone functions they call | 2,502 | 33% |
| `memmove`, `memcpy`, and `memcmp` | 739 | 10% |
| Dropping and cloning `Object` values and `String`s | 273 | 4% |
| Everything else, spread over the interpreter's functions | 4,129 | 53% |

The interpreter functions sampled most often:

| Function | Samples |
| :-- | --: |
| `VirtualMachine::invoke_method_unguarded` | 289 |
| `Class::find_own_method` | 190 |
| `VirtualMachine::evaluate_expression_inner` | 186 |
| SipHash, hashing a name for `Scope::define` and for a class's set of private method names | 145 |
| `VirtualMachine::evaluate_expression` | 142 |
| `Scope::get` | 134 |
| `IndexMap::get` | 127 |
| `ast::scope_locals::walk_expression` | 124 |
| `VirtualMachine::evaluate_binary_operation` | 121 |
| `VirtualMachine::method_is_restricted` | 94 |
| `VirtualMachine::execute_statement` | 81 |
| `Class::find_method_with_owner` | 74 |
| `Scope::define` | 66 |

### What the counters reported

```
heap allocations: 303961489 (10122752478 bytes)
statements evaluated: 14104637
expressions evaluated: 63481040
calls: 7055138
```

| Count | Kind |
| --: | :-- |
| 17,636,370 | `Identifier` expressions |
| 17,628,775 | `BinaryOp` expressions |
| 14,102,098 | `IntLiteral` expressions |
| 7,050,586 | `If` statements |
| 7,050,314 | `Call` expressions |
| 7,050,205 | `SelfExpr` expressions |
| 7,049,248 | `Return` statements |

Each call of `fib` makes 43 heap allocations, 304 million over the run, and
10 GB were allocated in all. Each call also evaluates 9 expressions and 2
statements, looks `fib` up again by name in `find_own_method`, checks
whether it is private, and defines `n` in a new scope, which is where the
hashing in the table above comes from.

### What this says for the design decisions

- Allocation is the largest single cost. The value representation, frame, and
  call decisions in the roadmap are measured against 43 allocations per call.
  Under CRuby, `GC.stat(:total_allocated_objects)` grows by 1 across
  `fib(25)`, which makes 242,785 calls.
- A call looks its method up by name in a hashed table each time. An inline
  cache at the call site, in the instruction set, removes that.
- `walk_expression` is `run_body_statements` calling `collect_assigned_locals`,
  which walks a method's body on each call to find the locals it assigns.
  Numbering locals into slots when a unit compiles does that once.

## Calls on bytecode

Recorded on 2026-10-17 with the counters, which count the same on any build.
Each figure is the difference between two runs of one program at two sizes,
divided by the extra calls, so startup is left out.

| Program | Heap allocations per call |
| :-- | --: |
| `tests/_examples/cli_flags/fib_calls.rb`, `fib` calling itself | 18 |
| `tests/_examples/cli_flags/calls_in_a_loop.rb`, `identity(value)` | 9 |
| `tests/_examples/cli_flags/calls_in_a_loop.rb`, `bump(by)` writing an instance variable | 13 |

The fib benchmark's 43 per call above came from the tree walker. On
bytecode, what a call still allocates is mostly its scope: a `String` and a
shared cell for each name it binds, `self` and `block_given?` among them,
and a copy of the arguments for a bare `super`. A popped scope nothing holds
is kept for the next call, so its table is not allocated again, and a block
written in a loop is built once rather than on each pass.

```sh
PASSES=1000 METOREX_COUNTERS=1 target/debug/metorex tests/_examples/cli_flags/calls_in_a_loop.rb
PASSES=2000 METOREX_COUNTERS=1 target/debug/metorex tests/_examples/cli_flags/calls_in_a_loop.rb
```

## Startup

Recorded on 2026-10-17 on the machine above with a release build.

`build.rs` parses the core library (`src/vm/prelude`) and the libraries a
default start loads with the crate's own lexer and parser, and the binary
holds the statements and the symbol names each parse recorded. A default
start then parses only the program. The class and module bodies the core
library runs still compile to instructions when they first run, which takes
1.8 ms of a start. A unit shares the bodies of the methods, blocks, classes,
and modules written in it rather than copying them, which took that from
7.8 ms.

| `metorex -e 1` | Wall time | Heap allocations |
| :-- | --: | --: |
| Parsing the core library and RubyGems at each start | 60 ms | 770,859 |
| Reading them parsed from the binary | 20 ms | 303,474 |

`ruby benchmarks/compare.rb --only=empty` times an empty script at 25.8 ms
under Metorex, 27.3 ms under CRuby 4.0.6, and 27.2 ms under CRuby with
`--yjit`.
