# Roadmap: running programs on bytecode

Metorex runs every program by walking its syntax tree (`src/vm/statement`, `src/vm/eval`, and the modules they call). A compiler (`src/compiler`) and a stack machine (`src/bytecode/vm`, the `BytecodeVm` struct) exist, but nothing runs on them: `BytecodeVm` keeps its own globals and frames and has none of `VirtualMachine`'s method tables, native methods, exceptions, threads, fibers, `eval`, bindings, TracePoint, or coverage, and the compiler covers part of the syntax tree.

This roadmap moves execution onto bytecode that `VirtualMachine` itself runs, so the core methods, the C API, Rex, and the libraries keep working as they are. The syntax tree stays: it is what `parse`, `Method#body`, and the parser produce, and it is what gets compiled.

Every item keeps `cargo test` and `scripts/run_ruby_spec.sh` passing on both engines until the tree walker is removed.

## Design decisions

These come first because every instruction, frame, and native method is written against them, and changing one after the compiler is built means rewriting what was built on it. Each decision is made with a prototype measured on the benchmarks below, and the decision, the alternatives, and the numbers are written in `docs/engine/<topic>.md`.

- [x] Benchmarks that decide: the ruby-bench suite's (formerly yjit-bench) benchmarks that run under Metorex today, beside the five in `benchmarks/workloads` and the Rex suite, compared against CRuby 4.0.6 with and without `--yjit`. `benchmarks/compare.rb` reports each with startup time and peak memory
  - Acceptance: one command prints the table for all three engines, and the README's speed section shows it
- [x] Profiling: a documented way to profile a run with a sampling profiler on macOS and Linux, and `METOREX_COUNTERS=1` printing per-instruction and per-call-path counts and allocations at exit
  - Acceptance: a profile of one benchmark is recorded in `docs/engine/profiling.md` with the commands that produced it
- [x] Value representation: an 8-byte value with Integers that fit, Floats, Symbols, nil, true, and false held inline and everything else a pointer, against the current `Object` enum. Strings are part of it: a String held as bytes with an encoding, against the current Rust `String` that holds a binary string's bytes 0x80 to 0xFF as two-byte characters, where `setbyte`, `bytesize`, `bytesplice`, and `pack(buffer:)` take time in proportion to the string's length
  - Acceptance: the prototype's numbers on the benchmarks, protoboeuf-encode's under each String layout, and the C API's `VALUE` conversion cost are recorded, and the chosen representation is the one the instruction set uses
- [x] Memory management: reference counting as now, or a tracing collector, chosen with the C API's raw pointers, `ObjectSpace`, `WeakMap`, finalizers, and `GC` behavior in ruby/spec weighed in
  - Acceptance: the decision records how each of those keeps passing ruby/spec under it
- [x] Frames and calls: frames in one contiguous value stack, arguments passed in place with no allocation per call, and fibers holding their own segment of frames, with a native stack only for the native methods running in them
  - Acceptance: a prototype call path measured on `method_calls` and `block_calls`
- [x] Instruction format and dispatch: stack or register machine, operand encoding, and how dispatch is written in Rust, measured on a prototype loop
  - Acceptance: the chosen format is the one the Groundwork instruction set item builds

## Groundwork

- [x] Values in 8 bytes: `Object` becomes the 8-byte value `docs/engine/value-representation.md` describes. Code that matches on a value matches on `view()`, an enum of its parts borrowed from it, and the variants' constructors become functions
  - Acceptance: a test checks `std::mem::size_of::<Object>()` is 8, and `cargo test` and ruby/spec pass
- [x] Strings as bytes: a String holds its bytes and an encoding tag, and every String method reads the bytes through the encoding
  - Acceptance: `cargo test` and ruby/spec pass, a test counts the bytes `getbyte`, `setbyte`, `bytesize`, and `byteslice` read and finds the same count on a 1 KB and a 1 MB binary string
- [x] Cycle collection, as `docs/engine/memory-management.md` describes: every heap object listed, with its counts in its header, and trial deletion run when the list has doubled and on `GC.start`. `ObjectSpace.each_object` and `count_objects` read the list, `WeakMap` and `WeakKeyMap` hold weak references, a finalizer runs after its object is freed, and `GC.count` and `GC.stat` report the collector's numbers
  - Acceptance: `cargo test` and ruby/spec pass, and tests show an object in a cycle freed by `GC.start` with its finalizer run, a `WeakMap` entry gone once its key is freed, and `each_object(String)` finding a String
- [x] C extensions are handed values: a `VALUE` is the value's bits, the handle table in `src/vm/capi/handles.rs` is removed, and an object handed to C is pinned until a collection cannot reach it from the native stack, a `dmark` function, or a registered address
  - Acceptance: every C extension example and ruby/spec's C API specs pass, no file under `src/` names the handle table, and a test shows an object only a C extension's struct holds surviving a `GC.start` and freed by one after the struct lets it go
- [x] Engine switch: `METOREX_ENGINE=tree|bytecode`, read once at startup, with `tree` the default
  - Acceptance: a test runs one example under each value and both print its output, and an unknown value ends the run with a message naming the two allowed
- [x] Compilation unit and fallback: a method body, block body, class or module body, or file top level compiles on its first run and the result is kept on the `Method`, `BlockStatement`, or loaded file. A unit holding a node the compiler does not handle yet records that node type and runs on the tree walker
  - Acceptance: under `METOREX_ENGINE=bytecode` with an empty compiler, `cargo test` and ruby/spec pass, every unit falls back, and the fallback is visible in the report below
- [x] Engine report: `METOREX_ENGINE_REPORT=path` writes, at exit, the count of units run on bytecode and on the tree walker, and the node types that caused each fallback with their counts
  - Acceptance: a test runs a file with one compiled method and one that falls back and checks both counts and the named node type
- [ ] Test harness on both engines: `scripts/run_ruby_spec.sh` and the example runners take the engine from the environment, and `test.sh` and the Jenkinsfile run `cargo test` and ruby/spec under `METOREX_ENGINE=bytecode` as a step of their own, with the engine report kept as a build artifact
  - Acceptance: the bytecode step runs in CI and fails the build when a test fails
- [x] Instruction set for `VirtualMachine`: 4-byte register instructions with constant operands, dispatched by `match` in one loop, as `docs/engine/instruction-format.md` describes, over the value representation the design decisions chose. Replace `BytecodeVm`'s opcodes with instructions that call into `VirtualMachine` for method lookup, invocation, constants, instance variables, and exceptions. Each instruction carries a source position in a side table for backtraces, `error_highlight`, and line events. `src/bytecode/vm` and the `BytecodeVm` struct are removed once nothing uses them, and the parts of `src/compiler` and `src/bytecode/disassembler.rs` that fit are kept
  - Acceptance: `tests/bytecode` covers each instruction through `VirtualMachine`, no file under `src/` names `BytecodeVm`, and `METOREX_COUNTERS=1` counts each instruction executed beside the node kinds the tree walker evaluates
- [x] Locals in slots: each unit numbers its locals and keeps their names, so `binding`, `local_variables`, `eval` with a binding, and `Binding#local_variable_get` and `set` read and write the same slots. A block reaches its outer locals through shared cells, as `captured_vars` does now
  - Acceptance: the binding, eval, and closure examples pass with their units compiled, shown by the engine report naming no fallback for them
- [x] Disassembly: `METOREX_DISASM=1` prints each unit's instructions with their source lines as it compiles them, to standard error
  - Acceptance: a test compiles a three-line method and checks the listing

## The language on bytecode

Each item compiles a group of node types. Its acceptance is the same in every case: the node types it names no longer appear in the engine report's fallback list for a full `cargo test` and ruby/spec run under `METOREX_ENGINE=bytecode`, and both pass.

- [x] Literals and names: `IntLiteral`, `BigIntLiteral`, `FloatLiteral`, `StringLiteral`, `Symbol`, `BoolLiteral`, `NilLiteral`, `Array`, `Dictionary`, `Range`, `RegexLiteral`, `InterpolatedString`, `Text`, `SelfExpr`, `MagicFile`, `MagicLine`, `MagicDir`, `Identifier`, `InstanceVariable`, `ClassVariable`, `GlobalVariable`, `ScopeResolution`, `TopLevelConstant`, `Grouped`
- [x] Assignment: `Assignment` for each target kind, the operator assignments, `MultipleAssignment` with splats and nesting, `DeclareLocals`, and `Index` reads and writes
- [x] Operators and calls: `BinaryOp`, `UnaryOp`, `Call`, `MethodCall` with positional, splat, keyword, keyword splat, and block arguments (`Splat`, `KeywordSplat`, `BlockArg`), safe navigation, `PatternTest`, and `Defined`
- [x] Branches and loops: `If`, `Unless` as statements and expressions, `Case`, `While`, `DoWhile`, `For`, `Break`, `Continue`, `Redo`, and `&&`, `||`, and `!` short-circuiting
- [x] Blocks and procs: `Lambda`, blocks passed to calls, `Yield`, `Block`, numbered parameters and `it`, and `break`, `next`, and `return` from a block, including a `return` that leaves the method that wrote the block through several yielding methods
- [x] Methods, classes, and modules: `FunctionDef`, `MethodDef` with every parameter kind, `ClassDef`, `ModuleDef`, `SingletonClass`, `Include`, `Extend`, `Alias`, `AttrReader`, `AttrWriter`, `AttrAccessor`, `Super` with and without arguments, and `Return`
- [x] Exceptions: `Begin`, `BeginRescue`, `Raise`, `Retry`, `ensure`, method-level `rescue`, `throw` and `catch`, and the `ControlFlow` exits that pass through an `ensure`
- [x] Pattern matching: `CaseIn` and `Match`, with array, find, hash, alternative, pin, and guard patterns, and `deconstruct` and `deconstruct_keys` caching as MRI does
- [x] `BeginBlock` and `END`, `__END__` and `DATA`, and `-n` and `-p` loops around the main program
- [ ] Code compiled at runtime: `eval`, `instance_eval`, `class_eval`, `module_eval` with strings, `define_method` with a block, `Binding#eval`, and `RubyVM::InstructionSequence.compile` and `#eval`
- [ ] Fibers and threads: a fiber or thread runs its block's bytecode, and suspending saves the instruction position and value stack so the block resumes where it stopped
- [ ] Tracing and coverage from instructions: TracePoint `:line`, `:call`, `:return`, `:b_call`, `:b_return`, `:class`, `:end`, `:raise`, `:c_call`, and `:c_return` events, `set_trace_func`, line and branch coverage for Rex's `--coverage` and the `Coverage` module, and `caller`, `caller_locations`, and backtraces naming the same lines as the tree walker did
  - Acceptance also: Rex's coverage report for `tests/_examples/rex/site/ledger` is unchanged

## Removing the tree walker

- [ ] No fallback: the engine report for a full `cargo test` and ruby/spec run under `METOREX_ENGINE=bytecode` lists no tree walker units
- [ ] Bytecode is the default engine
  - Acceptance: `cargo test` and ruby/spec pass with `METOREX_ENGINE` unset, and CI runs the tree walker step only as the second engine
- [ ] Remove the tree walker: the statement and expression evaluation in `src/vm/statement`, `src/vm/eval`, and the modules only they call, the engine switch, and its CI step. `parse`, `Method#body`, and the other syntax tree readers stay
  - Acceptance: `cargo test`, ruby/spec, clippy, and coverage at its current floor pass, and no code path evaluates a `Statement` or `Expression` directly
- [ ] `RubyVM::InstructionSequence#disasm` and `#to_a` list the bytecode instructions, and `RubyVM.stat` counts what the machine keeps
  - Acceptance: the instruction sequence examples show instructions, and the README and metorex.org's architecture and differences pages say Metorex compiles to bytecode

## Speed in the interpreter

These start once the tree walker is removed.

- [ ] Inline caches for method lookup at each call site, monomorphic and then polymorphic, invalidated when a method is defined or removed, a module is included or prepended, or a refinement is activated
- [ ] Constant caches at each constant reference, invalidated when a constant is set or autoloaded
- [ ] Instance variable shapes, so an instance variable read or write is an index into the object's slots
- [ ] Specialized instructions for Integer and Float arithmetic and comparison, Array and Hash indexing, and `send` of a literal name, each falling back to a method call when the method is redefined
- [ ] Block calls: `yield` and the common Enumerable and Integer iterators run a block without building a Proc
- [ ] Allocation: removing the allocations the profiler shows on hot paths
- [ ] Startup: the prelude and the libraries loaded at startup are compiled once at build time and loaded from the binary rather than parsed on each start

## Speed with a JIT

- [ ] Baseline JIT: hot methods and blocks compiled to machine code for x86-64 and arm64, with the choice between a code generator library and hand-written emitters recorded in `docs/engine/jit.md`
  - Acceptance: `cargo test` and ruby/spec pass with the JIT compiling every unit on its first call
- [ ] Deoptimization: JIT code returns to the interpreter at the current instruction when an assumption breaks: a method redefined, a constant set, a shape changed, TracePoint or `set_trace_func` enabled, or `binding` taken
  - Acceptance: a test for each of those causes runs the same output under the interpreter and with the JIT forced on
- [ ] Type specialization in JIT code from the types the inline caches observed, with guards that deoptimize
- [ ] JIT on by default once it passes ruby/spec

## Benchmarks

Each is run with `ruby benchmarks/compare.rb` on one machine, and the README's speed section records the table.

- [ ] Cycle collection: `cycles` with a peak memory within 10% of `no_cycles`
- [ ] Bytecode baseline: the table on bytecode beside CRuby 4.0.6 with and without `--yjit`
- [ ] Speed in the interpreter: the table after each of its items, against the bytecode baseline, by time and peak memory
- [ ] Startup: `empty` starts no slower than CRuby's
- [ ] JIT: faster than the interpreter on every benchmark, with the table showing Metorex beside CRuby with `--yjit` on each
