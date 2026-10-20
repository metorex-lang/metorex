# Roadmap

Metorex runs every program by walking its syntax tree (`src/vm/statement`, `src/vm/eval`, and the modules they call). A compiler (`src/compiler`) and a stack machine (`src/bytecode/vm`, the `BytecodeVm` struct) exist, but nothing runs on them: `BytecodeVm` keeps its own globals and frames and has none of `VirtualMachine`'s method tables, native methods, exceptions, threads, fibers, `eval`, bindings, TracePoint, or coverage, and the compiler covers part of the syntax tree.

This roadmap moves execution onto bytecode that `VirtualMachine` itself runs, so the core methods, the C API, Rex, and the libraries keep working as they are. The syntax tree stays: it is what `parse`, `Method#body`, and the parser produce, and it is what gets compiled.

After that, it moves Metorex to Ruby 4.0.7, matches RSpec's own specs, carries the bundled gems, runs the suites of the most downloaded gems, builds the features beyond Ruby, and lets programs read and write their syntax tree, with JIT speed last.

Every item keeps `cargo test` and `scripts/run_ruby_spec.sh` passing, on both engines until the tree walker was removed.

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
- [x] Test harness on both engines: `scripts/run_ruby_spec.sh` and the example runners take the engine from the environment, and `test.sh` and the Jenkinsfile run `cargo test` and ruby/spec under `METOREX_ENGINE=bytecode` as a step of their own, with the engine report kept as a build artifact
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
- [x] Code compiled at runtime: `eval`, `instance_eval`, `class_eval`, `module_eval` with strings, `define_method` with a block, `Binding#eval`, and `RubyVM::InstructionSequence.compile` and `#eval`
- [x] Fibers and threads: a fiber or thread runs its block's bytecode, and suspending saves the instruction position and value stack so the block resumes where it stopped
- [x] Tracing and coverage from instructions: TracePoint `:line`, `:call`, `:return`, `:b_call`, `:b_return`, `:class`, `:end`, `:raise`, `:c_call`, and `:c_return` events, `set_trace_func`, line and branch coverage for Rex's `--coverage` and the `Coverage` module, and `caller`, `caller_locations`, and backtraces naming the same lines as the tree walker did
  - Acceptance also: Rex's coverage report for `tests/_examples/rex/site/ledger` is unchanged

## Removing the tree walker

- [x] No fallback: the engine report for a full `cargo test` and ruby/spec run under `METOREX_ENGINE=bytecode` lists no tree walker units
- [x] Bytecode is the default engine
  - Acceptance: `cargo test` and ruby/spec pass with `METOREX_ENGINE` unset, and CI runs the tree walker step only as the second engine
- [x] Remove the tree walker: the statement and expression evaluation in `src/vm/statement`, `src/vm/eval`, and the modules only they call, the engine switch, and its CI step. `parse`, `Method#body`, and the other syntax tree readers stay
  - Acceptance: `cargo test`, ruby/spec, clippy, and coverage at its current floor pass, and no code path evaluates a `Statement` or `Expression` directly
- [x] `RubyVM::InstructionSequence#disasm` and `#to_a` list the bytecode instructions, and `RubyVM.stat` counts what the machine keeps
  - Acceptance: the instruction sequence examples show instructions, and the README and metorex.org's architecture and differences pages say Metorex compiles to bytecode

## Speed in the interpreter

These start once the tree walker is removed.

- [x] Inline caches for method lookup at each call site, monomorphic and then polymorphic, invalidated when a method is defined or removed, a module is included or prepended, or a refinement is activated
- [x] Constant caches at each constant reference, invalidated when a constant is set or autoloaded
- [x] Instance variable shapes, so an instance variable read or write is an index into the object's slots
- [x] Specialized instructions for Integer and Float arithmetic and comparison, Array and Hash indexing, and `send` of a literal name, each falling back to a method call when the method is redefined
- [x] Block calls: `yield` and the common Enumerable and Integer iterators run a block without building a Proc
- [x] Allocation: removing the allocations the profiler shows on hot paths
- [x] Startup: the prelude and the libraries loaded at startup are compiled once at build time and loaded from the binary rather than parsed on each start

## Speed with a JIT

- [x] Baseline JIT: hot methods and blocks compiled to machine code for x86-64 and arm64, with the choice between a code generator library and hand-written emitters recorded in `docs/engine/jit.md`
  - Acceptance: `cargo test` and ruby/spec pass with the JIT compiling every unit on its first call
- [x] Deoptimization: JIT code returns to the interpreter at the current instruction when an assumption breaks: a method redefined, a constant set, a shape changed, TracePoint or `set_trace_func` enabled, or `binding` taken
  - Acceptance: a test for each of those causes runs the same output under the interpreter and with the JIT forced on
- [x] Type specialization in JIT code from the types the inline caches observed, with guards that deoptimize
- [x] JIT on by default once it passes ruby/spec

## Benchmarks

Each is run with `ruby benchmarks/compare.rb` on one machine, and the README's speed section records the table.

- [x] Cycle collection: `cycles` with a peak memory within 10% of `no_cycles`
- [x] Bytecode baseline: the table on bytecode beside CRuby 4.0.6 with and without `--yjit`
- [x] Speed in the interpreter: the table after each of its items, against the bytecode baseline, by time and peak memory
- [x] Startup: `empty` starts no slower than CRuby's

## Ruby 4.0.7

- [ ] Vendored libraries come from the release itself: `scripts/vendor_gems.rb` and `scripts/vendor_prism.rb` take each gem at the version the release lists in its default gem specifications and `gems/bundled_gems`, not the newest one installed. Today the copies are prism 1.9.0, pp 0.6.4, irb 1.18.0, and reline 0.7.0, while 4.0.6 ships prism 1.8.1 and pp 0.6.3
  - Acceptance: running the vendor scripts under 4.0.6 writes the versions 4.0.6 ships, and each header names them
- [ ] A script copies uri, net-http, net-protocol, optparse, erb, and fileutils from the release, which are copied by hand today
  - Acceptance: running it under 4.0.6 leaves those files unchanged
- [ ] Metorex is Ruby 4.0.7: the version constants in `src/lib.rs`, the encoding, core method name, native arity, and parser tables, RubyGems, the vendored gems and prism, and `ruby/spec` and `ruby/mspec` at commits from 4.0.7's release, with the tests and docs that name 4.0.6 updated
  - Acceptance: `metorex -v` reports 4.0.7, every generated file names 4.0.7 in its header, and `cargo test` and `scripts/run_ruby_spec.sh` pass
- [ ] CI and the container run MRI 4.0.7, where `.github/workflows/ci.yml` and `docker/Dockerfile` pin 4.0.1 today
  - Acceptance: both name 4.0.7 and the CI run that installs it passes

## RSpec parity

Rex runs RSpec 3.13.6's own specs with `scripts/run_rspec_suite.sh`. Parity is the same examples passing and pending under Metorex as under MRI 4.0.7, file by file.

- [ ] RSpec vendored: the rspec repository at `rspec-core-v3.13.6` is a submodule, as `ruby/spec` is, and `scripts/run_rspec_suite.sh` reads it instead of downloading a tarball
  - Acceptance: the script runs with no network access
- [ ] Results by file: the script writes each file's example, failure, and pending counts under Metorex and under MRI to a checked-in file, and prints the files whose counts differ
  - Acceptance: the file lists every spec file the script runs, and the README's table is built from it
- [ ] rspec-expectations: every file's counts match MRI's (2231 of 2455 examples pass today)
- [ ] rspec-mocks: every file's counts match MRI's (1376 of 1806 pass today)
- [ ] rspec-core: every file's counts match MRI's (2038 of 2258 pass today)
- [ ] CI fails when any file's counts differ from MRI's

## Standard library

Ruby 4.0.7's bundled gems that do not load under Metorex today without MRI's gem directory: minitest, test-unit, power_assert, rake, rexml, rss, net-imap, net-pop, net-smtp, rdoc, rbs, typeprof, repl_type_completor, and debug. Bundler and readline do not load at all.

- [ ] Each of those bundled gems is carried in the binary, as csv and logger are, and loads with no gem directory set
  - Acceptance: a `tests/_examples/stdlib` file requires each and uses one of its methods
- [ ] The bundled gems' own test suites pass under Metorex: minitest, test-unit, power_assert, rake, rexml, rss, and the net-imap, net-pop, and net-smtp suites that need no network
  - Acceptance: a script like `scripts/run_rspec_suite.sh` runs each suite under Metorex and MRI and the counts match
- [ ] readline loads and answers as reline, as it does in MRI 4.0.7
- [ ] Bundler: `bundle install` and `bundle exec` run under Metorex and install and run a Gemfile's gems
  - Acceptance: a fixture Gemfile with a pure Ruby gem installs from a local gem server and `bundle exec` runs a script that requires it
## Popular gems

The 100 most downloaded gems on rubygems.org (`rubygems.org/stats`, read on 2026-10-20). Each gem's source repository is a submodule under `gems/`, pinned to its latest release tag, since a packaged `.gem` usually leaves its specs out. A gem's suite passes when every file's example, failure, and pending counts match MRI 4.0.7's, as RSpec's do. Bundler, rake, minitest, rexml, rspec and its four parts, and diff-lcs are covered above. `rubygems-update` is RubyGems itself, which Metorex carries.

- [ ] Gem suite runner: `scripts/run_gem_suites.sh` runs each submodule's suite with the command a table in the script names for it (`rspec`, `rake test`, or a test file list), under Metorex and MRI, and writes each file's counts to a checked-in results file. Each gem's test dependencies come from a lockfile checked in beside the table, installed by Bundler into a directory under `target/`
  - Acceptance: the script runs one gem's suite under both and the results file lists its files
- [ ] A `gems` stage in the Jenkinsfile, after `rspec-suites`, on a release build in a pod of its own, with one parallel branch per group below so a failing group shows by name. Each branch runs `scripts/run_gem_suites.sh` for its group under Metorex and compares the counts with the results file, whose MRI counts are recorded once per gem version rather than run on each build. A file whose Metorex counts fall below the recorded ones fails the branch, and one that rises prints the line to update. GitHub CI does not run the gem suites
  - Acceptance: a Jenkins run shows the `gems` stage with a branch per group, and a branch fails when a recorded count is raised by hand above what the run produces
- [ ] The gems in the list that ship with Ruby and that Metorex carries: json, racc, base64, and bigdecimal
- [ ] Rack and HTTP: rack, rack-test, rack-protection, faraday, faraday-net_http, multipart-post, excon, httpclient, http-cookie, domain_name, connection_pool, and websocket-extensions
- [ ] Text, data, and time: i18n, tzinfo, multi_json, multi_xml, builder, mime-types, mime-types-data, mini_mime, mail, addressable, public_suffix, jwt, dotenv, netrc, and rubyzip
- [ ] Command line and developer tools: thor, rainbow, ruby-progressbar, unicode-display_width, parallel, method_source, coderay, pry, simplecov, docile, and mini_portile2
- [ ] Ruby parsing and linting: ast, parser, regexp_parser, rubocop-ast, and rubocop
- [ ] Templates, loading, and concurrency: tilt, erubi, zeitwerk, sprockets, concurrent-ruby, and thread_safe
- [ ] Cloud clients: the aws-sdk-ruby repository's aws-sdk-core, aws-sigv4, aws-partitions, aws-eventstream, aws-sdk-s3, and aws-sdk-kms, with jmespath, googleauth, and signet
- [ ] redis, against a `redis-server` the suite starts on a port of its own, run in the Jenkins branch's pod as a second container beside the build
- [ ] Native extensions: nokogiri, ffi, nio4r, puma, websocket-driver, msgpack, unf_ext, and rb-fsevent build against Metorex's C API, and their suites and those of the pure Ruby gems that need them (loofah, crass, unf) pass
- [ ] Rails: the rails repository's activesupport, activemodel, activerecord on SQLite, actionpack, actionview, actionmailer, activejob, actioncable, activestorage, railties, and rails, with globalid, rails-dom-testing, rails-html-sanitizer, and sprockets-rails
- [ ] The ruby-bench benchmarks that need gems run under `benchmarks/compare.rb`
  - Acceptance: the table has a row for each, with output matching CRuby's

## metorex.org core reference

- [ ] The core reference marks a method not implemented only when ruby/spec runs its spec under 4.0.7. A method whose spec is guarded for a later Ruby, such as `String#bit_count` and `Method#source_range` under `ruby_version_is "4.1"`, and one removed from Ruby, such as `File.exists?` and `Kernel#taint`, are listed apart or left out, and the Time page no longer lists `Encoding.default_internal`
  - Acceptance: every method the reference marks not implemented has a spec that runs under 4.0.7 and fails under Metorex

## Beyond Ruby

The features the README lists that Metorex does not have yet. Each starts with a design recorded in `docs/language/<feature>.md`: the syntax or API, how it sits beside the Ruby it extends, and what ruby/spec and RSpec parity require of it.

- [ ] Traits
- [ ] Optional static typing and type inference
- [ ] Channels, async and await, and atomics
- [ ] `Deque`, `PriorityQueue`, and `TreeMap`
- [ ] MessagePack serialization
- [ ] A language server
- [ ] A debugger
- [ ] A linter and formatter

## Code as objects

A Metorex program can read its own code today. `Method#body`, `Proc#statements`, and `parse` return syntax tree nodes as hashes built by `src/vm/native_methods/ast_methods.rs`. It cannot write code back, and the hashes leave out much of the tree:

- The serializer handles 22 of the 41 expression types and 10 of the 30 statement types. The rest come out as `{"type" => "Unknown"}`.
- `InterpolatedString` leaves out its parts, `MethodDef` leaves out its body, `MethodCall` leaves out its trailing block, and lambda and method parameters keep only their names.
- No node carries its source position.

This section gives programs node objects in place of hashes. Each object wraps the parser's own `Expression`, `Statement`, or nested structure, so nothing is copied into a second format and nothing can be left out. Nodes are immutable. A program reads them, builds new ones with typed constructors or `copy`, walks and rewrites trees, runs them, and installs them as methods. Code built from nodes behaves the same as the code would if it had been written in a file.

Every Ruby method keeps its Ruby behavior. The new operations get Metorex-only names, as `get_source` and `parse` have.

### Design decisions

These come first because every class, reader, and constructor is written against them.

- [ ] Class layout: the namespace (`Metorex::AST` or another), one class per node type and per nested structure (`Parameter`, each `MatchPattern` variant, `HashPatternRest`, `MatchCase`, `ExprMatchCase`, `RescueClause`, `ElsifBranch`, `InterpolationPart`), the parent classes (`Node`, `Expression`, `Statement`, `Pattern`), and reader names taken from the Rust field names. Where an expression and a statement share a name (`If`, `Unless`), the decision records how the two classes are told apart
  - Acceptance: `docs/meta/classes.md` lists every class with its parent, readers, and the Rust type it wraps
- [ ] Field values: what a reader returns for each Rust field type. Operators are Symbols (`:+`, `:<=>`), names are Symbols or Strings, lists are frozen Arrays, an absent optional field is `nil`, and a position is an `AST::Position` with `line`, `column`, and `offset`
  - Acceptance: the mapping for every Rust field type is in `docs/meta/classes.md`
- [ ] Equality and identity: whether `==` compares positions, what `hash` and `eql?` do so nodes work as Hash keys, and whether reading the same child twice returns the same object
  - Acceptance: recorded in `docs/meta/classes.md` with the reason for each choice
- [ ] Positions of built nodes: what a node built by a constructor reports in a backtrace, `__LINE__`, `__FILE__`, `Method#source_location`, TracePoint events, and coverage
  - Acceptance: `docs/meta/positions.md` records the file name and line each of those shows for a built node, with the reason for the choice

### Groundwork

- [ ] Node values: a heap value kind that holds an `Rc` of a parser node or nested structure, with its class looked up from the node's variant. Child readers wrap the child when it is read, with no copy of the Rust node
  - Acceptance: a test parses a method, reads its body through the wrapper, and checks the wrapped `Rc` points at the method's own body. `ObjectSpace.each_object` and `GC.start` handle the new kind
- [ ] One table for every node type: the classes, readers, and constructors come from a single list of variants and fields, generated or derived from the syntax tree types, so a node type or field added to the parser does not compile until it has a class and a reader
  - Acceptance: a compile-fail test shows the build failing when the parser has a variant or field the table lacks
- [ ] Constructors check their arguments: `AST::BinaryOp.new(op: :+, left: a, right: b)` raises `TypeError` naming the class, the keyword, and the expected type when an argument is wrong, and `ArgumentError` for a missing or unknown keyword
  - Acceptance: tests pass a wrong type, a missing keyword, and an unknown keyword to one class of each parent, and check each message
- [ ] `copy`: `node.copy(**fields)` returns a new node with those fields replaced, checked as a constructor checks them
  - Acceptance: tests check `node.copy` with no arguments equals `node`, and a replaced field reads back as the new value

### Node classes

Each item adds the classes, readers, and constructors for its node types. The acceptance for each is a test that parses `tests/_examples/meta/nodes_<group>.rb`, a file that uses every node type in the group, rebuilds every node through its constructor from the values its readers return, and compares the rebuilt node with the parser's node, positions included. The example prints one node of each type.

- [ ] Literals, variables, and names: `IntLiteral`, `FloatLiteral`, `BigIntLiteral`, `StringLiteral`, `InterpolatedString` with its parts, `RegexLiteral`, `BoolLiteral`, `NilLiteral`, `Symbol`, `Identifier`, `InstanceVariable`, `ClassVariable`, `GlobalVariable`, `MagicFile`, `MagicLine`, `MagicDir`, `SelfExpr`, `TopLevelConstant`, `ScopeResolution`
- [ ] Calls, operators, and collections: `Call`, `MethodCall` with its trailing block, `Super`, `Yield`, `Splat`, `KeywordSplat`, `BlockArg`, `Lambda` with its parameters, `BinaryOp`, `UnaryOp`, `Index`, `Array`, `Dictionary`, `Range`, `Grouped`, `Defined`, and `Parameter`
- [ ] Control flow: the `If`, `Unless`, `Case`, `BeginRescue`, and `PatternTest` expressions, and the `If`, `Unless`, `While`, `DoWhile`, `For`, `Return`, `Break`, `Continue`, `Redo`, `Retry`, `Raise`, `Begin`, `BeginBlock`, and `Block` statements, with `ElsifBranch` and `RescueClause`
- [ ] Pattern matching: `Match` and `CaseIn`, with `MatchCase`, `ExprMatchCase`, every `MatchPattern` variant, and `HashPatternRest`
- [ ] Definitions and assignment: `FunctionDef` and `MethodDef` with body and parameters, `ClassDef`, `ModuleDef`, `SingletonClass`, `Include`, `Extend`, `Alias`, `AttrReader`, `AttrWriter`, `AttrAccessor`, `Assignment`, `MultipleAssignment`, `DeclareLocals`, and the `Expression` statement
- [ ] Every file: the rebuild test passes on every `.rb` file under `tests/_examples`

### Getting nodes

- [ ] `AST.parse(source)` returns an array of statement nodes, and raises `SyntaxError` with the parser's message and position for invalid source
  - Acceptance: `tests/_examples/meta/parse.rb` parses a method definition and prints its name and parameters, and parses invalid source and prints the error
- [ ] `Method#ast` and `Proc#ast` return the `MethodDef` node of a method and the `Lambda` node of a Proc, wrapping the bodies they run
  - Acceptance: `tests/_examples/meta/method_ast.rb` prints `Invoice#total`'s operator, parameters, and position, and does the same for a block
- [ ] `quote { ... }` returns the statements of the block as nodes without running it
  - Acceptance: `tests/_examples/meta/quote.rb` quotes `a + b` with neither `a` nor `b` defined and prints the node
- [ ] `node.source` returns the source text the node was parsed from, or `nil` for a built node
  - Acceptance: `tests/_examples/meta/source.rb` prints the source of a method body and of one argument of a call in it

### Walking and rewriting

- [ ] `children` returns a node's child nodes in source order, and `each_node` yields the node and every node below it, depth first
  - Acceptance: `tests/_examples/meta/walk.rb` counts the `MethodCall` nodes in a parsed file and prints their method names
- [ ] `AST::Visitor`: a subclass defines `visit_binary_op` and the rest, one method per node class, and the default for each visits the children
  - Acceptance: `tests/_examples/meta/visitor.rb` collects every local variable a method reads
- [ ] `transform { |node| ... }` rebuilds the tree bottom up, using the block's return value in place of each node, and returns the new tree
  - Acceptance: `tests/_examples/meta/transform.rb` replaces every `IntLiteral` 5 with 10 in a method body and prints both trees' source
- [ ] Pattern matching: `deconstruct_keys` and `deconstruct` on every node class, so `case node in AST::BinaryOp[op: :+, right: AST::IntLiteral[value: 5]]` matches
  - Acceptance: `tests/_examples/meta/match.rb` finds every `x + 5` in a method body with a `case`/`in`

### Running and installing

- [ ] `node.evaluate(binding = caller's binding)` runs a statement node or an array of them and returns the last value, compiled through `compile_body` as `eval` is. Locals the nodes assign are visible to later code in the same binding
  - Acceptance: `tests/_examples/meta/evaluate.rb` builds a `BinaryOp` tree with constructors and prints its value, evaluates a `Lambda` node and calls the Proc, evaluates a `ClassDef` node and calls a method on the class, and shows the backtrace of a built `Raise` node with the position `docs/meta/positions.md` records
- [ ] `Module#define_method_from(method_def_node)` installs the method through the path `def` uses, so the method caches, compiled units, and JIT code for the old body are dropped with it
  - Acceptance: `tests/_examples/meta/install.rb` reads `Invoice#total`'s tree, changes `+ 5` to `+ 10`, installs it, and prints the new total. A test calls a method in a loop until its body is compiled to machine code, installs a new body, and checks the next call returns the new body's value
- [ ] `node.to_source` prints a tree as Ruby source, for display
  - Acceptance: for every `.rb` file under `tests/_examples`, `AST.parse(tree.to_source)` equals the tree with positions left out of the comparison

### Compatibility

- [ ] `node.to_h` returns the node as a hash with every field, and `Method#body`, `Proc#statements`, and `parse` return `to_h` hashes, so the keys programs read today (`"type"`, `"op"`, `"left"`, `"right"`, `"name"`, `"value"`, and the rest) keep their meaning. The hand-written serializer in `ast_methods.rs` is removed
  - Acceptance: `tests/_examples/site/meta.rb`, `meta_block.rb`, and `meta_parse.rb` print the same output as before, and `ast_methods.rs` no longer exists

### Documentation

- [ ] metorex.org: the "Code as objects" page documents the node classes, reading, building, walking, rewriting, pattern matching, running, and installing, with a reference page listing every class and its readers. The home page says a program can read and write its syntax tree at runtime. Each sample is a `tests/_examples/site/meta_*.rb` file with its output recorded by the examples runner
  - Acceptance: `blogin build` and `bin/check-links.raku` pass, and each sample on the page matches its recorded output

## JIT speed

Each is run with `ruby benchmarks/compare.rb` on one machine, and the README's speed section records the table.

- [ ] JIT: faster than the interpreter on every benchmark, with the table showing Metorex beside CRuby with `--yjit` on each
