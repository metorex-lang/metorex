# Rex, the built-in spec framework

Goal: ship an RSpec clone inside Metorex itself, under the name Rex. A spec file runs with no gem, no `Gemfile`, and no `require`. `metorex spec` is a subcommand of the language, the way `describe` is part of the language.

This work starts after [MSPEC_ROADMAP.md](MSPEC_ROADMAP.md) is complete. That order matters: mspec drives `ruby/spec` through the same metaprogramming surface RSpec needs (`instance_exec`, `define_method`, `method_missing`, singleton classes, `prepend`, `Method#parameters`, `caller`, `at_exit`, `Thread`, `StringIO`, `Dir`, `ENV`). By the time the mspec roadmap closes, those are proven against the official suite, so this roadmap never has to stop and build a language feature first.

## Compatibility target

RSpec 3.13: the three libraries a user sees as one, `rspec-core`, `rspec-expectations`, and `rspec-mocks`. `rspec-rails` is out of scope. `rspec-support` has no public surface and is absorbed.

The acceptance test for the whole effort is a third-party suite written against real RSpec producing the same pass and fail counts and the same documentation-format output under `metorex spec`.

## The name

The framework is `Rex`, from the tail of METOREX (Meta Object Runtime EXecution). `RSpec` is a constant alias for the same module object, defined at load, so `RSpec.describe`, `RSpec::Matchers`, and `RSpec::Expectations::ExpectationNotMetError` all resolve and an existing suite runs unchanged. One implementation, two names.

```ruby
Rex.describe Stack do
  let(:stack) { Stack.new }

  it "pushes" do
    stack.push 1
    expect(stack.size).to eq 1
  end
end

Rex.configure do |config|
  config.order = :random
end
```

The internal split mirrors the three gems, so the alias lands on identical paths:

- `Rex::Core` for `ExampleGroup`, `Example`, `World`, `Configuration`, and the formatters
- `Rex::Expectations` for `expect`, the matcher protocol, and the failure class
- `Rex::Mocks` for doubles and message expectations
- `Rex::Matchers` for the matcher library and `Matchers.define`

Where RSpec's own name is longer than it needs to be, the Rex path is the short one and RSpec's is the alias. `Rex::ExpectationNotMet` is the class, and `Rex::Expectations::ExpectationNotMetError` points at it.

`require "rspec"` and `require "rspec/autorun"` succeed and load nothing. File conventions stay RSpec's, so `spec/`, `*_spec.rb`, and `spec/spec_helper.rb` need no moves in an existing project, and the runner reads both `.rspec` and `.rex` for options.

A `rex` gem exists (Metasploit's Ruby Exploitation library), as does a `rex` lexer generator. Metorex loads no gems, so neither can claim the constant.

## Correct first, then fast

The clone is built correct and then made fast, in that order. Every optimization needs a test that catches a behavior change, and that test is rspec-core's own suite running green, which is the last item of the compatibility proof. Building the fast path first leaves nothing to say when it lies.

Speed comes from a native fast path guarded by a redefinition check, deoptimizing to the Ruby definition the moment a suite changes anything observable. This is what MRI does for `Integer#+`. Compatibility is defined by behavior through the public API, not by the language a method is written in, so `Rex::Matchers#eq` stays a real Ruby method either way and the native path is an optimization of calling it.

The rule the speed work is held to: if a suite can tell the difference by any means other than a clock, the optimization is wrong.

Parallel execution is the exception to the ordering. It changes no observable behavior of a single example, so it stays where it is in the plan rather than waiting for the proof.

## Where the code lives

Three layers, chosen per feature by what the layer above cannot do:

- **`src/vm/stdlib/rex/`** holds the DSL, the matchers, the reporters, and the runner, written in Ruby and embedded in the binary the way the bundled standard library is. A matcher is then a real user-defined class, so a spec can subclass it, alias it, or redefine it, and `Rex::Matchers` opens the way a user's own module does. It loads on `metorex spec` and on `require "rspec"`, so a script that runs no spec pays nothing for it.
- **Rust native primitives** cover what Ruby in the prelude cannot reach: the stub and restore transaction on the method table, source extraction for an expression's AST, the LCS diff, monotonic timing, the seeded RNG, terminal width and color detection, and the example-status persistence file.
- **`src/main.rs` and `src/spec/`** recognize `metorex spec` and hand its arguments to `Rex::Core::Runner`, which does the argument parsing, `.rspec` files, file and line filtering, load ordering, and the exit code in Ruby, as RSpec's own runner does.

## What a built-in runner does that a gem cannot

- **Source-aware failure messages.** Metorex exposes the AST as runtime objects, so `expect(user.age).to eq(42)` can report `expected user.age to eq 42, got 41` with the real expression text rather than `expected 42, got 41`.
- **Stubbing as a transaction.** `allow(obj).to receive(:foo)` defines the stub on the object's singleton class and keeps the method it replaced, and the example's end puts that method back. Restoration is exact, it cannot leak across examples, and it needs no `alias_method` shuffling or `method_missing` shim.
- **Verifying doubles with real signatures.** `instance_double` reads `Method#parameters` from the VM, so a keyword argument typo fails when the double is set up.
- **Backtrace filtering by frame ownership.** A framework frame is marked when it is pushed, so filtering is exact rather than a regex over file paths.
- **`let` resolution in a real scope.** A per-example scope in the VM holds the memoized values, so a misspelled `let` name raises a NameError naming the helpers declared near it.
- **Coverage with no second tool.** The VM already walks lines, so `metorex spec --coverage` reports line and branch coverage with no instrumentation pass.

## 1. Runner skeleton and the CLI

- [x] 1.1. `metorex spec [files-or-directories]` subcommand, defaulting to `spec/`
- [x] 1.2. File discovery: `**/*_spec.rb` under the given paths, sorted for a deterministic default load order
- [x] 1.3. `spec/spec_helper.rb` loaded when present, before any spec file
- [x] 1.4. `.rspec` and `.rspec-local` option files, plus the `SPEC_OPTS` environment variable, merged in RSpec's precedence order
- [x] 1.5. `--require`, `--default-path`, `--pattern`, `--exclude-pattern`
- [x] 1.6. Location filters: `path/to/file_spec.rb:42`, repeatable, and `--example` for a name substring
- [x] 1.7. Exit status: 0 for a clean run, 1 for any failure, and `--failure-exit-code`
- [x] 1.8. `Rex::Core::World` singleton holding registered groups, filters, and the reporter
- [x] 1.9. Examples under `tests/_examples/rex/runner/` wired into `tests/rex_example_tests.rs`, each compared with RSpec's output for the same fixture as written by `scripts/rex_expected.sh`

## 2. Example group DSL

- [x] 2.1. `Rex.describe` and top-level `describe`, with a String, a Class, or a Module as the subject argument
- [x] 2.2. Group aliases: `context`, `xdescribe`, `xcontext`, `fdescribe`, `fcontext`, and `Rex.context`
- [x] 2.3. Nested groups to any depth, each a subclass of its parent group class
- [x] 2.4. `it`, `specify`, `example`, and their `x` and `f` forms
- [x] 2.5. Description inference for `it` with no description, read from the matcher
- [x] 2.6. `pending` and `skip`, as a method inside an example and as an argument on the group or the example
- [x] 2.7. A pending example that passes reported as a failure, with RSpec's wording
- [x] 2.8. `described_class` and the full `description` string built by walking the group chain
- [x] 2.9. `Rex::Core::ExampleGroup` as a real class, so `self` inside an example is an instance of it
- [x] 2.10. Examples under `tests/_examples/rex/dsl/`

## 3. Hooks, helpers, and memoized values

- [x] 3.1. `before` and `after` with `:each` and `:example`, running outside-in and inside-out
- [x] 3.2. `before` and `after` with `:all` and `:context`
- [x] 3.3. `before(:suite)` and `after(:suite)` through `Rex.configure`
- [x] 3.4. `around(:each)` with a yielded `Example` answering `run` and `call`
- [x] 3.5. `let` and `let!`, memoized per example, redefinable in a nested group
- [x] 3.6. `subject` and named `subject(:name)`, plus the implicit subject built from the described class
- [x] 3.7. `is_expected` and `should` on the implicit subject, for one-liner examples
- [x] 3.8. Helper modules through `config.include`, `config.extend`, and `config.prepend`, each with metadata filtering
- [x] 3.9. A NameError for an undefined `let` that names the helpers declared nearby
- [x] 3.10. Hook ordering across a group chain proved by an example that appends to an array
- [x] 3.11. Examples under `tests/_examples/rex/hooks/`

## 4. The expectation protocol

- [x] 4.1. `expect(value)` and `expect { }` answering an `ExpectationTarget`
- [x] 4.2. `to`, `not_to`, and `to_not`
- [x] 4.3. The matcher protocol: `matches?`, `does_not_match?`, `failure_message`, `failure_message_when_negated`, `description`, `supports_block_expectations?`
- [x] 4.4. `Rex::ExpectationNotMet` raised on failure, carrying the message and the caller location, with `Rex::Expectations::ExpectationNotMetError` aliased to it
- [x] 4.5. `expect` reachable inside an example with no receiver, and inside a custom matcher block
- [x] 4.6. `Rex::Matchers.define` for custom matchers, with `match`, `match_when_negated`, `failure_message`, `description`, and the `expected` and `actual` readers
- [x] 4.7. Matcher DSL argument access: `expected`, `actual`, `expected_as_array`
- [x] 4.8. The `should` and `should_not` syntax on every object, off by default and turned on by `config.expect_with`
- [x] 4.9. `config.expect_with(:rspec) { |c| c.syntax = ... }` honored by both syntaxes
- [x] 4.10. Examples under `tests/_examples/rex/expectations/`

## 5. Built-in matchers

- [x] 5.1. Equality: `eq`, `eql`, `equal`, `be`
- [x] 5.2. Comparison: `be >`, `be >=`, `be <`, `be <=`, `be_between(...).inclusive`, `be_within(...).of(...)`
- [x] 5.3. Type and identity: `be_a`, `be_an`, `be_kind_of`, `be_instance_of`, `respond_to(...).with(n).arguments`, `.with_keywords`
- [x] 5.4. Truthiness: `be_truthy`, `be_falsey`, `be_nil`
- [x] 5.5. Predicate matchers through `method_missing`: `be_empty`, `be_valid`, `have_key`, and the question-mark fallback
- [x] 5.6. Collections: `include`, `contain_exactly`, `match_array`, `start_with`, `end_with`, `all`
- [x] 5.7. `match` against a Regexp, a String, or a structure of nested matchers
- [x] 5.8. Errors: `raise_error` with a class, a message String, a Regexp, a block, and the bare form with its warning
- [x] 5.9. `throw_symbol` and `have_attributes`
- [x] 5.10. `change { }` with `by`, `from`, `to`, `by_at_least`, `by_at_most`
- [x] 5.11. `output(...).to_stdout`, `.to_stderr`, and the `_from_any_process` forms
- [x] 5.12. `yield_control`, `yield_with_args`, `yield_with_no_args`, `yield_successive_args`
- [x] 5.13. `satisfy`, the operator matchers, and the compound combinators `and` and `or`
- [x] 5.14. `aggregate_failures`, as a block method and as example metadata, reporting every failure in one message
- [x] 5.15. Matcher composition: any matcher accepted where a value is accepted, as in `include(a_string_matching(/x/))`
- [x] 5.16. Aliased matchers: `a_string_matching`, `an_instance_of`, and the rest of RSpec's alias table
- [x] 5.17. Examples under `tests/_examples/rex/matchers/`, one file per matcher family

## 6. Doubles and message expectations

- [x] 6.1. A stub transaction: define the stub on the object's singleton class, keep the method it replaces, and put that method back when the example ends
- [x] 6.2. `double`, `double("name")`, `double(name: value)`, and the strict-double error for an unstubbed message
- [x] 6.3. `allow(obj).to receive(:msg)`, with `and_return`, `and_raise`, `and_throw`, `and_yield`, `and_call_original`
- [x] 6.4. Several return values from one `and_return`, consumed in order
- [x] 6.5. `expect(obj).to receive(:msg)`, verified when the example ends, with the unfulfilled-expectation message
- [x] 6.6. Receive counts: `once`, `twice`, `exactly(n).times`, `at_least`, `at_most`, `never`
- [x] 6.7. Argument constraints: `with(...)`, `anything`, `any_args`, `no_args`, `hash_including`, `array_including`, `instance_of`, `kind_of`, and any matcher
- [x] 6.8. `receive_messages` and `receive_message_chain`
- [x] 6.9. Spies: `spy`, `have_received`, and `allow(obj).to receive(:msg)` followed by `expect(obj).to have_received(:msg)`
- [x] 6.10. Partial doubles on a real object, with the original method restored exactly
- [x] 6.11. Verifying doubles: `instance_double`, `class_double`, `object_double`, checking existence, arity, and keywords through `Method#parameters`
- [x] 6.12. `verify_partial_doubles` configuration, on by default
- [x] 6.13. `allow_any_instance_of` and `expect_any_instance_of`
- [x] 6.14. `config.mock_with` and the `should_receive` syntax behind it
- [x] 6.15. A leakage spec: a file whose second example asserts the first example's stub is gone
- [x] 6.16. Examples under `tests/_examples/rex/mocks/`

## 7. Metadata, filtering, and shared code

- [x] 7.1. Metadata on groups and examples, as a Hash and as bare Symbols, inherited by nested groups
- [x] 7.2. Built-in metadata keys: `described_class`, `full_description`, `file_path`, `line_number`, `location`, `scoped_id`
- [x] 7.3. `config.filter_run_when_matching :focus`, with `fit` and `fdescribe` applying it
- [x] 7.4. `--tag` and `--tag ~name`, `config.filter_run_including`, `config.filter_run_excluding`
- [x] 7.5. Conditional filters, where a filter value is a Proc taking the metadata
- [x] 7.6. `shared_examples`, `shared_examples_for`, `shared_context`, each with a name, a Symbol, or a Module
- [x] 7.7. `it_behaves_like`, `include_examples`, `it_should_behave_like`, with block arguments and a customization block
- [x] 7.8. `config.include_context` with metadata, and `config.shared_context_metadata_behavior`
- [x] 7.9. Examples under `tests/_examples/rex/metadata/`

## 8. Ordering, seeds, and run persistence

- [x] 8.1. `--order defined`, `--order random`, `--order rand:SEED`, and `config.order`
- [x] 8.2. Seeded ordering by RSpec's Jenkins hash of the seed and each example id, so one seed gives RSpec's order
- [x] 8.3. `config.register_ordering` for a named custom ordering, plus `:global`
- [x] 8.4. The example status persistence file, `config.example_status_persistence_file_path`
- [x] 8.5. `--only-failures` and `--next-failure`, reading that file
- [x] 8.6. `--fail-fast` and `--fail-fast=N`
- [x] 8.7. `--dry-run`, which builds every group and runs no example body
- [x] 8.8. `--bisect`, narrowing an ordering-dependent failure to the smallest reproducing set
- [x] 8.9. Examples under `tests/_examples/rex/ordering/`

## 9. Reporting and formatters

- [x] 9.1. The formatter protocol and the notification objects RSpec hands it
- [x] 9.2. `progress` formatter, the default: a dot per example, `F` for a failure, `*` for pending
- [x] 9.3. `documentation` formatter, indented by group nesting
- [x] 9.4. The failure block: description, message, the failing source line, and the filtered backtrace
- [x] 9.5. Color detection from the terminal, `--color`, `--no-color`, `--force-color`
- [x] 9.6. The summary line, the failed-examples rerun list, and the seed line
- [x] 9.7. `json` formatter, matching RSpec's document shape
- [x] 9.8. `html` formatter
- [x] 9.9. `--profile [N]`, ranked by monotonic example duration
- [x] 9.10. `--format` and `--out`, repeatable, so one run writes several formats
- [x] 9.11. `config.add_formatter` and a user-defined formatter class
- [x] 9.12. Output captured to a StringIO and asserted in `tests/_examples/rex/formatters/`

## 10. Diffs and source-aware messages

- [x] 10.1. An LCS diff native primitive, so no `diff-lcs` gem is involved, pairing lines as diff-lcs does
- [x] 10.2. Unified diff output for two Strings that differ, colorized
- [x] 10.3. Object diffs through `pretty_print`, for Hashes, Arrays, and Structs
- [x] 10.4. `config.diff_enabled`, `--no-diff`, and `config.diff_line_limit`, the size past which a diff is left out
- [x] 10.5. Source text of the expression handed to `expect`, read natively through the lexer
- [x] 10.6. Failure messages that name the expression: `expected user.age to eq 42, got 41`
- [x] 10.7. `config.source_expressions`, off by default so failures read as RSpec writes them, so a suite can turn the expression text on
- [x] 10.8. The failing statement printed whole, as RSpec prints it, and `config.failure_context_lines` for the lines around it
- [x] 10.9. Examples under `tests/_examples/rex/diffs/`

## 11. Backtraces and error handling

- [x] 11.1. Rex's own frames told apart by the `<metorex>/` path Metorex gives a library it loads from the binary, which no file on disk can have, rather than by a pattern a user's path could also match
- [x] 11.2. `config.backtrace_exclusion_patterns` and `config.backtrace_inclusion_patterns` layered on top
- [x] 11.3. `--backtrace` for the unfiltered form
- [x] 11.4. An error raised in `before(:all)` failing every example in the group, with RSpec's wording
- [x] 11.5. An error raised in `after` reported beside the example's own result
- [x] 11.6. An error outside any example reported as `An error occurred while loading <file>`, with the run continuing
- [x] 11.7. `config.raise_errors_for_deprecations!` and the deprecation stream
- [x] 11.8. Interrupt handling: a signal ends the run and prints the summary for what ran
- [x] 11.9. Examples under `tests/_examples/rex/errors/`

## 12. Configuration surface

- [x] 12.1. `Rex.configure` and the `Configuration` object, with every setting reachable as a reader and a writer
- [x] 12.2. `config.disable_monkey_patching!`
- [x] 12.3. `config.expose_dsl_globally` and the main-object DSL
- [x] 12.4. `config.define_derived_metadata`
- [x] 12.5. `config.when_first_matching_example_defined`
- [x] 12.6. `config.around`, `config.before`, `config.after` with a scope and metadata
- [x] 12.7. `config.default_formatter`, `config.silence_filter_announcements`, `config.warnings`
- [x] 12.8. `Rex::Core::Pending` and the remaining legacy settings an existing `spec_helper.rb` may set
- [x] 12.9. `RSpec` aliased to `Rex` at load, and `config.rspec_constant` to leave it undefined
- [x] 12.10. Examples under `tests/_examples/rex/config/`

## 13. Parallel execution

- [x] 13.1. Group-level work distribution across forked worker processes, since Metorex runs threads one at a time, with a recorder in each worker standing in for the reporter
- [x] 13.2. `--jobs N` and `--jobs auto` for the machine's core count, defaulting to 1 so `before(:suite)`, fail-fast, and interrupts behave as in RSpec unless a suite asks
- [x] 13.3. Output serialized per group, so a parallel documentation run reads the same as a serial one
- [x] 13.4. Shared-state detection: `--verify-parallel` warns when a parallel run and a serial run disagree under one seed
- [x] 13.5. `--jobs 1` as the exact equivalent of a serial run, proved by identical output bytes
- [x] 13.6. Examples under `tests/_examples/rex/parallel/`

## 14. Coverage

- [x] 14.1. `metorex spec --coverage`, reading the line table the VM already walks
- [x] 14.2. Branch coverage counted where the VM chooses an arm of an `if`, `unless`, `case`, `while`, or `&.`, since Metorex walks the syntax tree rather than bytecode
- [x] 14.3. `--coverage-format` for a summary, an LCOV file, and a JSON file
- [x] 14.4. `--coverage-threshold N`, failing the run below it
- [x] 14.5. Filters for which files count, defaulting to everything loaded that is not a spec file
- [x] 14.6. Examples under `tests/_examples/rex/coverage/`

## 15. Compatibility proof

- [x] 15.1. Port the `tests/_examples/` assertions that already read as specs into `spec/` files run by `metorex spec`, each also passing under MRI's rspec: the interpreter fixes the Rex work needed, 43 examples
- [x] 15.2. Run `rspec-expectations`' own spec suite unchanged, and record the pass count: 2231 of 2455, 17 pending
- [x] 15.3. Run `rspec-mocks`' own spec suite unchanged, and record the pass count: 1376 of 1806, 9 pending
- [x] 15.4. Run `rspec-core`'s own spec suite unchanged, and record the pass count: 2038 of 2258, 19 pending
- [x] 15.5. Pick a third-party library with an RSpec suite and no C extensions, run it, and record the result: diff-lcs, 322 of 322
- [x] 15.6. Byte-compare `documentation` formatter output against real RSpec for a fixture suite
- [x] 15.7. A `scripts/run_rspec_suite.sh` driver shaped like `scripts/run_ruby_spec.sh`, with a commented list advanced one line at a time
- [x] 15.8. README section describing the built-in runner, its flags, and what it does that the gem cannot

## 16. Speed

Begins after the compatibility proof passes. Each item lands with its guard and with a spec that redefines the thing it optimizes and proves the deopt fires.

- [x] 16.1. A benchmark fixture suite of a few thousand trivial examples, plus a recorded baseline checked into the repo
- [x] 16.2. `scripts/bench_rex.sh`, reporting boot time, examples per second, and peak memory against that baseline
- [x] 16.3. CI runs the compatibility suites twice, fast paths on and off, and byte-compares the output
- [x] 16.4. `--no-fast-paths` and `config.fast_paths`, so any divergence can be bisected by a user
- [x] 16.5. A method-table redefinition counter the fast paths read, shared with the VM's inline caches
- [x] 16.6. Prelude snapshot, or a split prelude whose spec half loads on `metorex spec` and on the first `describe`, whichever measures better: split, since a plain script boots in 0.04 s and loading Rex adds 0.05 s, of which parsing is 0.03 s. Naming `RSpec` loads Rex and runs the examples as the program ends
- [x] 16.7. Hook chains flattened into one ordered list when the group is defined, rather than walked per example
- [x] 16.8. `let` compiled to a method plus a memo slot in the example scope, with `__memoized` kept as a compatible view
- [x] 16.9. Metadata populated natively, still a mutable Hash the suite can write to
- [x] 16.10. Example group instances allocated natively, deoptimizing for a group that defines `initialize`
- [x] 16.11. Native `eq`, `be`, `be_truthy`, `be_nil`, `include`, and `match`, each behind its guard
- [x] 16.12. Native double teardown, so a mock-heavy suite pays the undo record rather than a global space walk
- [x] 16.13. Profile one third-party suite before and after, and record both numbers in the README
- [x] 16.14. Examples under `tests/_examples/rex/performance/`, one per guard, each redefining the optimized method and asserting the Ruby path runs

## Open questions

- **`--bisect` transport.** Settled: each round runs in a process forked after the requires and `spec_helper` load and before any spec file does, as RSpec's default fork runner does, so no round inherits another's state.
- **Monkey patching.** RSpec's top-level `describe` is a method on `main`. Since this ships in the language, `disable_monkey_patching!` could be the default and `Rex.describe` the only form. That breaks existing suites, so the plan keeps RSpec's default and leaves the setting to the user.
- **Load cost.** Settled: Rex is an embedded library rather than part of the prelude, so it loads on `metorex spec` and on `require "rspec"`, and a script that runs no spec does not build it.
