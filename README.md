# Metorex Programming Language

**METOREX** (**M**eta **O**bject **R**untime **E**xecution) is a programming language that combines the expressiveness of Ruby with the performance and safety of Rust. It features a unique **Code-as-Object** meta-programming system that exposes the AST as first-class runtime objects, enabling powerful DSL construction and runtime code manipulation.

✅ &nbsp;Passes the [ruby/spec](https://github.com/ruby/spec) suite: core, library, language, command_line, optional, and security.

🙂 &nbsp;[PRs](https://github.com/metorex-lang/metorex/pulls) and [new](https://github.com/metorex-lang/metorex/issues/new) [issues](https://github.com/metorex-lang/metorex/issues) are welcome.

###

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/metorex-lang/metorex/blob/main/LICENSE) [![CI](https://github.com/metorex-lang/metorex/workflows/CI/badge.svg)](https://github.com/metorex-lang/metorex/actions) [![codecov](https://codecov.io/gh/metorex-lang/metorex/graph/badge.svg)](https://codecov.io/gh/metorex-lang/metorex)

## Project Status

Metorex compiles each method, block, class body, and file to bytecode the first time it runs, and its interpreter runs those instructions.

## Key Features

### Core Language Features
- **Exception Handling**: Full begin/rescue/ensure with exception hierarchies and stack traces
- **Pattern Matching**: Powerful pattern matching with destructuring and guards
- **Built-in Testing**: Integrated test framework with assertions and test discovery
- **Traits/Interfaces**: Flexible polymorphism through trait system
- **Optional Type System**: Gradual typing with type inference for performance and safety
- **Advanced Collections**: Set, Deque, PriorityQueue, TreeMap, and immutable structures
- **Struct**: `Struct.new` builds member classes with accessors, `[]`/`[]=`, `to_a`, `to_h`, `each`, `each_pair`, `dig`, `values_at`, and value equality, with `keyword_init:` and a class-body block
- **Struct members and Enumerable**: a member accessor wins over Struct's own `length`, `size`, `members`, and `to_a`, member values stay out of `instance_variables`, `each`/`each_pair`/`select`/`filter` answer an Enumerator without a block, and Struct mixes in Enumerable
- **Struct pattern matching and conversion**: `deconstruct_keys` takes member names, positions, and `to_int` keys, `to_h` accepts a block that returns `[key, value]` pairs, `values_at` takes Ranges, `keyword_init?` reports how the class was built, and writers raise FrozenError on a frozen struct
- **Enumerable**: a class that defines `each` and includes Enumerable answers `to_a`, `entries`, `map`, `select`, `filter`, `find_all`, `reject`, `partition`, `group_by`, `sort`, `sort_by`, `min`, `max`, `minmax`, `min_by`, `max_by`, `minmax_by`, `include?`, `member?`, `take`, `first`, `each_with_index`, `each_entry`, `reverse_each`, `filter_map`, `compact`, `inject`, and the rest of the walk
- **Enumerable argument forms**: `min(n)` and `max(n)` answer the n smallest or largest, on an Array too, and a range of Integers counts them from the end asked for without walking, so `(0...2**64).max(2)` answers at once. `min_by(n)` and `max_by(n)` do the same by the block's value, `find` and `detect` take an ifnone callable and answer an Enumerator without a block, and `take` coerces its count through `to_int`
- **Enumerable enumerators**: a method called without its block answers an Enumerator whose `size` is the receiver's own, and an `each` that yields several values at once hands the block one packed array
- **Non-local return**: `return` inside a block leaves the method that wrote the block, however many yielding methods it travels through first
- **Splatted super arguments**: `super(*list)` spreads the array across the parent's parameters, and mixes with plain arguments as `super(0, *rest)`
- **Enumerable predicates**: `all?`, `any?`, `none?`, `one?`, `count`, and `find_index` take a pattern matched with `===`, which stands in for a block and warns that the block went unused, and more than one argument is an ArgumentError
- **Enumerable batching and counting**: `each_slice` and `each_cons` coerce their width through `to_int` and report the batch count on the Enumerator they answer, `tally` counts into a Hash given as an argument, `zip` reaches an argument through `to_ary` or `to_enum`, and `flat_map` flattens one level through `to_ary`
- **Enumerable counts and sums**: `take`, `drop`, and `first` coerce a count through `to_int` and raise TypeError, ArgumentError, or RangeError for one they cannot use, `find` and `detect` take an ifnone callable, and `sum` adds floats with Kahan-Babuska compensation
- **break through a walk**: `break` in a block leaves the method the block was handed, so `take_while { break :stopped }` answers `:stopped` even when the walk runs through several yielding methods
- **NilClass, TrueClass, and FalseClass**: `nil`, `true`, and `false` answer to their own classes, so `NilClass === nil` and `true.is_a?(TrueClass)` hold, and each class lists its own methods, such as `nil.method(:to_a).owner`, which is NilClass
- **Optional parameters before a splat**: `def each(arg = :default, *rest)` binds the default when the call does not reach the parameter
- **Enumerator#each with a block**: walking with a block runs the method the Enumerator was cut from and answers what that method answers
- **Enumerable#inject and #reduce**: fold through a block or through an operator named by a Symbol, a String, or an object answering `to_str`, with the starting value optional, a warning when a block goes unused beside an operator, and an ArgumentError when neither is given
- **inject over a growing Array**: the walk reads one element at a time, so a block that appends to the array reaches what it added
- **Enumerable#grep and #grep_v**: the elements a pattern matches with `===`, or the ones it does not, passed through the block when one is given
- **Enumerator::Lazy**: `lazy` answers a walk that applies `map`, `select`, `filter`, `find_all`, `reject`, `filter_map`, `flat_map`, `collect_concat`, `compact`, `take`, `take_while`, `drop`, `drop_while`, `grep`, `grep_v`, `uniq`, `with_index`, and `zip` one element at a time, so a source with no end still answers `first` and `force`. A step hands on what the source yields as one value, packing several into an Array and none into nil, and a `zip` starts its other sources over on each walk. `Enumerator.produce` reports an endless size or the one its `size:` keyword names
- **Lazy sizes**: a step that keeps every element reports the source's size, `take` and `drop` report what the count leaves, and a step that decides what to keep reports nil
- **Lazy grouping**: `chunk`, `chunk_while`, `slice_when`, `slice_before`, and `slice_after` cut runs as the walk goes, and the same methods on Enumerable answer an Enumerator over the runs
- **Enumerator::Chain**: `chain` and `+` build a walk over several collections in order, reporting the sum of their sizes and answering nil or Infinity for the first part of such a size
- **Array pickings**: `combination`, `permutation`, `repeated_combination`, and `repeated_permutation` yield each group of a given length and answer the array itself, or an Enumerator reporting how many groups there are
- **Array#product**: the rows drawn one element from each list, reaching an argument through `to_ary` and refusing a number of rows it will not walk
- **Method#<< and #>>**: a Method composes with any callable the way a Proc does, `>>` running the method first and `<<` running the argument first
- **Array#bsearch and #bsearch_index**: halve a sorted array rather than walking it, reading true and false from the block or a number saying which way to go
- **Set#classify and #divide**: group the elements under what a block answers, where a block of two parameters instead groups the elements it relates
- **Set#flatten and #flatten!**: open up every set held inside, raising ArgumentError for a set that reaches itself
- **Hash#to_proc**: a lambda of one parameter that reads a key out of the hash, so `[:a, :b].map(&hash)` looks each one up
- **Enumerator#with_object**: walks with a second value handed to the block each time, answering that value when the walk is done
- **A lazy walk pulled one at a time**: `next`, `peek`, and `rewind` run the walk no further than what was asked for, and `String#each_char` without a block answers an Enumerator
- **Data**: `Data.define` builds a value class whose members are read-only, with `new` and `[]` taking positional or keyword arguments, `with` for a copy carrying changes, plus `members`, `to_h`, `deconstruct`, `deconstruct_keys`, `==`, `eql?`, `hash`, and an `inspect` that shows a value reaching itself as the class alone. A keyword may name a member by Symbol, by String, or by anything answering `to_str`, and a block handed to `new` reaches an `initialize` that takes one
- **Method#parameters and #arity**: each parameter reports its kind (`:req`, `:opt`, `:rest`, `:keyreq`, `:key`, `:keyrest`, `:block`), and the arity counts the required ones, turning negative where a call may pass more or fewer
- **Proc#parameters and #arity**: a lambda counts the way a method does, while a proc reports its positional parameters as optional and only a splat leaves its count open
- **Method and UnboundMethod rendering**: `inspect` and `to_s` name the receiver's class, the module the method came from when that differs, the parameters, and the file and line
- **Method#super_method**: the next definition of the name above the module the method was found in, following an alias back to what it was cut from
- **UnboundMethod#bind_call**: binds and calls in one step, without an intermediate Method object
- **A class defining `self.new`**: builds its instances that way rather than through allocate-and-initialize
- **`super` with keyword arguments**: the argument list reads the same as any other call's, so keywords, splats, and a block argument all reach the parent method
- **`super` with a block**: `super { ... }` and `super(a) do ... end` both hand the parent method a block of their own, in place of the one the caller supplied. A bare `super` still forwards the enclosing method's arguments
- **`def !` and `def ~`**: an object may name what the operator written in front of it does, and `!obj` reaches the method rather than answering whether the object is false
- **A superclass named from the top level**: `class C < ::Parent` starts the name at the top level rather than inside the module being opened, whether it names a class or builds one, as `class Pair < ::Struct.new(:left, :right)` does
- **A constant bound inside a conditional**: a class assigned to a constant inside an `if` in a module body still reports the namespaced name
- **Time**: a point in time held as a whole number of seconds since the epoch plus an exact fraction, with `now`, `at`, `new`, `utc`/`gm`, and `local`/`mktime` building one
- **Time calendar fields**: `year`, `month`/`mon`, `day`/`mday`, `hour`, `min`, `sec`, `wday`, `yday`, `zone`, `utc_offset`, `dst?`, and the `monday?` through `sunday?` questions, read in UTC, in a fixed offset, or in the zone the operating system holds
- **Time subseconds**: `subsec`, `usec`, and `nsec` keep the fraction exact, so `Time.at(Rational(3, 2)).subsec` answers `(1/2)` and `Time.at(10.75).subsec` answers `(3/4)`
- **Time rendering**: `to_s`, `inspect`, `asctime`/`ctime`, `strftime` (including `%L`, `%N`, `%z`, and `%:z`), `iso8601`/`xmlschema`, and `to_a`
- **Time arithmetic**: `+` and `-` shift by an exact number of seconds, subtracting two times answers a Float, and `<=>`, `==`, `eql?`, `hash`, `round`, `floor`, and `ceil` all read the exact value
- **Time zones**: `utc`/`gmtime`, `getutc`/`getgm`, `localtime`, and `getlocal` move between UTC, a fixed offset given as seconds or as `"+05:00"`, and the local zone
- **ENV writes reach the process environment**: setting `ENV["TZ"]` changes what a local time reads, since the C library sees the same environment
- **A control-flow form read for its value**: `case`, `if`, `unless`, `while`, `until`, and `begin` may be chained onto, so `case x when 1 then :a end.to_s` reads the way it does in Ruby
- **`then` before a newline**: `if cond then` may hold its body on the following line
- **An empty pair of parentheses**: `()` answers nil, which `[0, (), 2]` and `{() => ()}` rely on
- **A group of statements**: `(a; b; c)` answers the last one
- **Percent literals**: `%W` and `%I` fill in their `#{}` parts, and `%q` reads none
- **A chain with the dot leading the line**: newlines and comments may sit between a call and the dot that continues it
- **Safe navigation**: `a&.b` answers nil for a nil receiver without running the method
- **Quoted symbols in `alias`**: `alias :'new' :'old'` names either method that way
- **Libraries metorex carries**: `require` finds `expect`, `fiber`, `open-uri`, `base64`, `shellwords`, `abbrev`, `singleton`, `observer`, `securerandom`, `digest`, `delegate`, `weakref`, `logger`, `monitor`, `tmpdir`, `tempfile`, `bigdecimal`, `zlib`, `erb`, `open3`, `syslog`, `openssl`, `cgi`, `socket`, `net/http`, `net/ftp`, `English`, `io/nonblock`, `yaml`, `rbconfig`, `rbconfig/sizeof`, `resolv`, `optparse`, `rubygems`, `mkmf`, `random/formatter`, `json`, `objspace`, `objspace/trace`, `ripper`, `irb/color`, `did_you_mean`, `error_highlight`, `syntax_suggest`, `prism`, `nkf`, `kconv`, `racc`, `mutex_m`, `resolv-replace`, `rinda`, `irb`, `reline`, `readline`, `minitest`, `test/unit`, `power_assert`, `rake`, `rexml`, `rss`, `net/imap`, `net/pop`, `net/smtp`, `rdoc`, `rbs`, `typeprof`, `repl_type_completor`, `debug`, `bundler`, `io/console`, and `stringio` without a directory on the load path
- **did_you_mean**: the report of a NameError, NoMethodError, KeyError, LoadError or NoMatchingPatternKeyError ends with what the misspelled name may have meant, as MRI's does, from the locals, methods, constants, Hash keys and loadable features that were there. `DidYouMean` loads at startup and `--disable=did_you_mean` leaves it out. `detailed_message(did_you_mean: false)` leaves the suggestion off, `#corrections` lists them, and `DidYouMean::SpellChecker`, `TreeSpellChecker` and `Formatter` are there to call. A NameError answers the local variables in scope where it was raised, and a NoMethodError whether the call named no receiver or `self`
- **Percent literals with any delimiter**: `%!text!`, `%@text@`, and `%_text_` all read as strings, they fill in their `#{}` parts, and `%x(...)` runs its text as a command. A `%` that follows a value still divides
- **`not` with parentheses**: `not(x)` takes what the parentheses hold, so a call may be chained onto the answer
- **A group with a trailing modifier**: `(123 if true)` and `(count += 1 until done)` answer what the modifier left
- **error_highlight**: the report of a NameError, NoMethodError, TypeError or ArgumentError shows the line of the program it was raised on with `^` under the call, name or operator that failed, as MRI's does. An ArgumentError from a call to a method the program wrote shows both the call and the method's `def` line. `ErrorHighlight.spot` answers the columns and the snippet, `detailed_message(highlight: false)` carries the same lines, and `--disable=error_highlight` leaves them out. Code run by `eval` has no snippet
- **syntax_suggest**: the report of a program that does not parse, and the `detailed_message` of a SyntaxError raised by `load` or `require`, open with the lines around a missing or extra `end`, as MRI's do. This is Ruby's syntax_suggest, which reads the file through Prism as MRI's does. `--disable=syntax_suggest` and `--disable=gems` leave it out, and `detailed_message(syntax_suggest: false)` leaves it off one message
- **prism**: `require "prism"` loads Prism 1.8.1, the version MRI 4.0.7 ships. `Prism.parse`, `parse_file`, `parse_stream`, `lex`, `lex_file`, `parse_lex`, `parse_comments`, `dump`, `parse_success?` and the rest answer Prism's own node, token, location and diagnostic objects, and its visitors, `Prism::Pattern`, `Prism::Dispatcher`, `Prism.lex_compat` and `Prism::Translation::Ripper` work on them. build.rs compiles Prism's C parser from `vendor/prism` into the binary, and its Ruby library runs over the parser's serialized results. `scripts/vendor_prism.rb` copies it in from the source tree of a Ruby release
- **Bundled gems**: `require` finds MRI's `csv`, `drb` with its `drbunix:` transport, `nkf` and `kconv`, `racc` with its runtime parser `racc/parser` and its grammar compiler, `mutex_m`, `resolv-replace`, `rinda`, `minitest`, `test-unit`, `power_assert`, `rake`, `rexml`, `rss`, `net-imap`, `net-pop`, `net-smtp`, `rdoc`, `rbs`, `typeprof`, `repl_type_completor`, `debug`, and `readline`, which answers as `Reline`, copied in by `scripts/vendor_gems.rb` from the source tree of a Ruby release at the versions it ships, with no gem directory set. The C extensions of nkf, rbs, and debug are compiled into the binary and start when their gems require them. rbs's signatures of Ruby's core and standard library are carried too, and written once to `metorex-<uid>/rbs-<version>` under the temporary directory, which rbs reads them from. `resolv` is MRI's own, with its DNS resolver, `Resolv::IPv4` and `Resolv::IPv6`
- **irb**: `binding.irb` and `IRB.start` run MRI's irb 1.16.0. On a terminal it reads through reline 0.6.3, which completes names on Tab and keeps history, and on a pipe it reads standard input, both as MRI's do. io/console works on a terminal: `raw`, `noecho`, `echo=`, `winsize`, `console_mode`, `getch`, `getpass`, `cursor`, `goto` and the rest of io-console 0.8.2, the version MRI 4.0 carries
- **pp**: `pp` and `pretty_inspect` are MRI's, loaded the first time `pp` is called, so a value wider than 80 columns is broken across lines by nesting
- **Ruby 4.0's additions**: `Array#rfind` answers the last element its block accepts, or what the `if_none` callable answers, and `Method#box` answers nil while namespaces are off, as MRI's do. Time, Thread, Fiber, the thread queues and mutexes, and Process::Status descend from Object, so their ancestors end in Kernel and BasicObject
- **Pathname**: a core class, as in Ruby 4.0. Naming it loads MRI's own `pathname_builtin.rb`, and `require "pathname"` adds `find`, `rmtree` and `Pathname.mktmpdir`
- **`Object#methods`**: leaves out the private methods, which are not the ones an object answers to from outside. `methods`, `public_methods`, `private_methods` and `instance_methods` of a built-in value or class list the methods Ruby's core gives it, by the visibility the nearest class defining each one gives it, alongside the methods the program wrote, so `[1].methods` lists `first` and did_you_mean suggests it for `[1].frist`
- **Private methods of every object**: Kernel's functions, such as `puts` and `Integer`, and BasicObject's `initialize`, `method_missing` and `singleton_method_added` hooks are private. A call written with a receiver other than `self` raises NoMethodError, or reaches the receiver's own `method_missing`, `send` reaches them, and `respond_to?(name, true)` answers true. A class that makes one public with `public :puts` answers it. `iterator?` answers what `block_given?` does and warns that it is deprecated under `-W:deprecated`
- **Code named after where it was written**: `__FILE__` inside `eval` reports `(eval at <file>:<line>)`
- **`Signal.trap` takes anything that answers to `call`** when the signal arrives, whether it could when it was written down or not, and a signal set aside with nil is left alone. A signal nothing was written for reads as `SYSTEM_DEFAULT`, the name may be written long or short as a Symbol, a String or a number, and `EXIT` names what to run as the program ends, ahead of what `at_exit` left. A signal the interpreter keeps for itself, a number naming none, and a value of the wrong kind are each refused in their own words. A write where nothing is left to read ends the program by the signal when the program leaves that signal to the operating system
- **A program that ends takes its threads with it**: each thread unwinds where it waits, so the `ensure` clauses the fiber it is on sits inside run, while a fiber it left suspended stays that way and its own do not. An exception reaching the top level reports each cause under it with the cause's own backtrace
- **A command's two streams as one**: `IO.popen(argv, err: [:child, :out])` gives the command one stream to write both of its own into, keeping them interleaved as they were written, and the command still runs without a shell, so one ended by a signal reports that signal. A signal a program sends itself arrives inside the call that sent it, which is what a report of it names
- **A constant reads through the scopes it was written in**: the open scopes first, innermost out, then the ancestors of the innermost one, and an autoload registered on a scope fires there rather than further out. `instance_eval` handed a String reads the receiver's singleton class and class ahead of the caller's scopes, sees the caller's locals, and takes the file and line it is counted from, or reads as `(eval at file:line)` with none. A block keeps the scopes it was written in, which is where a class variable it names belongs
- **`sleep` and a fiber scheduler**: a length may be named by anything that says how to divide itself, waiting backwards is refused, and a thread that sleeps for a length lets the other threads run until it passes or a wakeup cuts it short, answering the whole seconds it slept. `Fiber.set_scheduler` gives the running thread a scheduler, which a fiber that is not blocking hands its waits to: `kernel_sleep` for a sleep, `block` and `unblock` for a Mutex another fiber holds, an empty Queue and a thread still running, `io_wait` for a read with nothing ready, and `process_wait` and `timeout_after` where the scheduler answers them. `Fiber.schedule` asks the scheduler for a fiber, and a thread closes its scheduler when it is given another one and when it ends, the main thread before the `at_exit` blocks run. A scheduler that does not answer for `block`, `unblock`, `kernel_sleep` and `io_wait` is refused, and one without `fiber_interrupt` is warned about
- **`require` searches the load path**: a name written from the working directory or from the root is that one file and is not searched for, a name already listed as loaded is read without looking, a name answered once from the load path stays answered while its file is listed, and a file built for the machine rather than written in Ruby is refused with no path in the error. A file another thread is part-way through loading is waited for. What the interpreter carries itself (`complex`, `enumerator`, `fiber`, `pathname`, `rational`, `ruby2_keywords`, `set`, `thread`) is listed as loaded from the start
- **`require_relative` names a `.rb` file**: a path written without that ending has it added, whatever other ending it carries, and the path it expands to is what a LoadError reports and what `$LOADED_FEATURES` lists. It takes anything that says how to read itself as a path, resolves against the file the call was written in (through a symlink, the file it points at), and leaves the feature list as it stood when a load fails part-way through. `instance_eval` with a file name counts the code as written there
- **`raise` with a named cause**: `cause:` settles what an error follows. It is refused on its own, refused when it names something that is not an exception, refused when the chain would run back on itself, and left as it stands for an error raised a second time. An exception's message comes from what its own `initialize` hands to `super`, and one built with none says what class it is
- **A load wrapped in a module**: `load(path, true)` runs the file in a module of its own and `load(path, SomeModule)` in the one it is handed. What the file defines belongs to that module, a method written at its top level is private there, and the file runs against an object of its own that says it is main and carries the wrapping module ahead of whatever main was extended with
- **Circular require**: a file that asks for itself while it is still running gets nothing back, and a verbose run says `loading in progress, circular require considered harmful`
- **A thread that ends the program**: `exit` in a thread carries SystemExit to the main thread rather than dying of it, so nothing is written about a thread that failed
- **A name loaded once under threads**: a thread that asks for a constant another thread is part-way through autoloading waits for that load to finish, so it reads a module that is built rather than one that is half built. A method that ran threads still returns from itself afterwards, since the frame a `return` belongs to travels with the thread that opened it
- **`eval` in the scope around it**: a class it opens belongs to the scope the eval was written in, a `return` in the code returns from the lambda or method around it and is a LocalJumpError with neither, the block the call was written with is not handed on to what the code calls, a name the code only assigns to on a line that never runs is still a local of the binding, and a magic `# encoding:` comment says how to read source handed over as bytes. `Kernel.eval` runs the code against Kernel, and `method(:eval)` hands out the method a bare call reaches
- **StringIO**: a string read and written the way a file is. Reading with `getc`, `getbyte`, `readchar`, `readbyte`, `gets`, `readline`, `readlines`, `each_line`, `each_char`, `each_byte`, `each_codepoint`, and `ungetc`. Writing with `write`, `print`, `printf`, `puts`, `putc`, and `<<`, padding with NUL when the position sits past the end. Moving with `pos`, `seek`, `rewind`, and `lineno`. Reshaping with `truncate`, `string=`, and `reopen`. Modes decide which sides are open, `close_read` and `close_write` close them apart, and `inspect` shows the class and address alone. `ungetc` puts characters back over the bytes before the cursor, adding what does not fit to the start
- **`String#index` and `#rindex`**: where a substring or pattern first or last sits, counted in characters, with an optional offset
- **Octal escapes**: `"\000"` reads up to three digits as one character
- **An argument may assign**: `StringIO.new(text = "hello")` binds `text` and passes what it assigned
- **`io/console`**: `require` finds it, adding `getch` and `getpass` to StringIO
- **StringScanner**: `require 'strscan'` gives a cursor over a string. `scan`, `check`, `match?`, and `skip` match where the cursor stands, `scan_until`, `check_until`, `skip_until`, `exist?`, `search_full`, and `scan_full` search ahead, and `matched`, `pre_match`, `post_match`, `[]`, `captures`, `named_captures`, and `values_at` report what the last match found. `match?`, `skip`, `skip_until`, `exist?`, and `matched_size` count bytes. `pos`, `charpos`, `rest`, `eos?`, `bol?`, `reset`, `terminate`, and `unscan` move the cursor, and `getch`, `peek`, `peek_byte`, and `scan_byte` read one piece at a time
- **OpenStruct**: `require 'ostruct'` gives an object whose fields are decided as they are set, with `[]`, `[]=`, `delete_field`, `dig`, `each_pair`, `to_h`, `==`, `marshal_dump`, and an `inspect` that shows a field reaching back to the object as the class alone. A frozen one may be read but not written
- **Prime**: `require 'prime'` gives `Prime.prime?`, `Prime.each` (lazy without a block, so `next` and `rewind` walk it), `Prime.first`, `Prime.prime_division`, and `Prime.int_from_prime_division`, plus `Integer#prime?`, `Integer#prime_division`, `Integer.from_prime_division`, and `Integer.each_prime`
- **Matrix and Vector**: `require 'matrix'` gives rectangular arrays of numbers and the arithmetic over them. Matrices are built with `[]`, `rows`, `columns`, `build`, `diagonal`, `scalar`, `identity`, `zero`, `row_vector`, `column_vector`, and `empty`, and answer `row`, `column`, `[]`, `transpose`, `+`, `-`, `*`, `/`, `**`, `determinant`, `trace`, `rank`, `inverse`, `minor`, `first_minor`, `cofactor`, `collect`, `each`, `each_with_index`, and the shape questions from `square?` through `unitary?`. Vectors answer `+`, `-`, `*`, `inner_product`, `cross_product`, `magnitude`, `normalize`, `each2`, `covector`, and `angle_with`
- **ObjectSpace::WeakMap and ObjectSpace::WeakKeyMap**: two maps that hold their entries only as long as something else does, written here as ordinary maps since metorex frees an object when the last reference to it goes. The first is keyed by identity and answers `[]`, `[]=`, `delete`, `key?`, `member?`, `key`, `size`, `length`, `keys`, `values`, `each`, `each_pair`, `each_key`, and `each_value`, and includes Enumerable. The second is keyed by value, adds `getkey` and `clear`, and refuses a number, a symbol, or one of the three singletons as a key
- **ObjectSpace.memsize_of**: `require 'objspace'` reports how much an object holds, counted from its instance variables and its contents. A name ObjectSpace keeps no account of still answers nil, and a name the library adds now wins over that
- **Message digests**: `require 'digest'` gives `Digest::MD5`, `Digest::SHA1`, `Digest::SHA256`, `Digest::SHA384`, `Digest::SHA512`, `Digest::RMD160`, which `require 'digest/rmd160'` also loads, and `Digest::SHA2`, which picks one of SHA256, SHA384 and SHA512 by bit length. Each answers `digest`, `hexdigest`, `base64digest`, and the `!` forms of all three, both on the class and on an object that takes its message through `update` or `<<`. An object also answers `reset`, `new`, `file`, `length`, `size`, `digest_length`, `block_length`, `to_s`, `inspect`, and an `==` that compares against another digest or against the text of a hexdigest. `Digest.hexencode` names bytes in hex and `Digest.bubblebabble` names them in syllables
- **Naming the parts of a path**: `File.basename` takes the last part, with a named suffix or `".*"` taken off the end, `File.extname` answers the extension a name carries, and `File.split` gives the directory and the name as a pair. Each takes `to_path` or `to_str` from an object that answers one
- **Reading a directory**: `Dir.empty?` says whether one holds anything, `Dir.home` says where a user's files live, `Dir.foreach` and `Dir.each_child` walk the names one at a time or hand back an Enumerator, and `Dir.entries` and `Dir.children` answer them all at once. An `encoding:` keyword tags the names, and `Encoding.default_internal` decides when none is given. `Dir.delete`, `Dir.rmdir`, and `Dir.unlink` remove an empty directory, reporting `Errno::ENOTEMPTY`, `Errno::ENOTDIR`, or `Errno::ENOENT` for one they cannot
- **`File.binread` and `File.binwrite`**: the bytes a file holds, one to a character, so a file that is not text reads back and writes out unchanged
- **Character sets a String reads**: `count`, `delete`, `squeeze`, `tr`, and `tr_s` take the set syntax `tr` writes, so `a-z` is a range, a leading `^` means every character but those, and a descending range is an ArgumentError. `chop`, `delete_prefix`, `delete_suffix`, `partition`, `rpartition`, `intern`, `sum`, `casecmp`, `casecmp?`, `center`, `dedup`, `scrub`, `undump`, `grapheme_clusters`, and `each_grapheme_cluster` read and cut a string without changing it, `upto` walks from one string to another and steps by code point when both are a single character, and `String.try_convert` takes `to_str` from an object that answers one
- **Network addresses**: `require 'socket'` gives `Addrinfo`, which names one endpoint. `Addrinfo.tcp`, `.udp`, `.ip`, and `.unix` build one, `ip_address`, `ip_port`, `ip_unpack`, `afamily`, `pfamily`, `socktype`, and `protocol` report its parts, and `to_s`, `inspect`, `inspect_sockaddr`, and `to_sockaddr` write it back out. The predicates from `ipv4_loopback?` through `ipv6_v4mapped?` say what kind of address it is, and `ipv6_to_ipv4` unwraps the IPv4 address an IPv6 one carries. `Socket::Constants` names the families, socket kinds, and protocols, `sockaddr_in` and `sockaddr_un` write the structs the operating system takes, and `Socket.getaddrinfo` and `gethostname` ask the system
- **TCP sockets**: `TCPServer` listens on a name and a port, `#accept` takes the next connection made to it, and `TCPSocket` reaches one that something is listening on. Both answer `addr`, `close`, and `closed?`, and a connection answers `read`, `write`, `puts`, `peeraddr`, and `remote_address`. `open_timeout:` limits how long `TCPSocket.new` and `Socket.tcp` take to connect, raising `IO::TimeoutError` and `Errno::ETIMEDOUT` once it passes, and an IP socket's `inspect` names its descriptor, family, address and port, or that it is closed
- **IO::Buffer**: a run of bytes of a given size, over memory, a String or a mapped file, with `get_string`, `set_string`, `write` to a stream from an offset, and the rest. The first buffer a program makes warns that the interface is experimental, where `Warning[:experimental]` asks for that
- **Socket::AncillaryData**: a control message sent beside a socket's bytes, named by family, level and type as numbers or names, with `int`, `unix_rights`, `ip_pktinfo` and `ipv6_pktinfo` building and reading the common kinds and `cmsg_is?` testing which one a message is. The socket constants a platform carries alone, such as `SOCK_PACKET` and `UDP_CORK` on Linux, are defined only there
- **HTTP requests and responses**: `require 'net/http'` loads MRI 4.0.7's own net-http 0.9.1 and net-protocol 0.2.2, which metorex carries unchanged. A body is read to its `Content-Length` or through its chunks, a gzipped or deflated body arrives inflated, a connection is kept open between requests and opened again when the server closed it, and a request that fails on a connection the server dropped is sent again when it is safe to repeat. `net/http/status` gives `Net::HTTP::STATUS_CODES`
- **FTP sessions**: `require 'net/ftp'` gives `Net::FTP`. `new` and `open` build a session, taking `passive`, `debug_mode`, `resume`, `open_timeout`, `read_timeout`, `ssl_handshake_timeout`, and `port` as options or as the older positional arguments, and `Net::FTP.default_passive` sets what a session starts in. `connect` and `login` reach a server, `sendcmd`, `voidcmd`, `getresp`, and `voidresp` carry commands and replies, `pwd`, `getdir`, `chdir`, `mkdir`, `rmdir`, `delete`, `rename`, `size`, `mdtm`, `status`, `system`, `site`, `noop`, `help`, `abort`, and `quit` name the operations, and `close` and `closed?` end it. `Net::FTPError` and the `FTPReplyError`, `FTPTempError`, `FTPPermError`, and `FTPProtoError` under it name what the server refused
- **FTP transfers**: a data connection is opened for each transfer, in passive mode when the session asks for it and otherwise by listening on a port of the program's own and naming it with `PORT`. `retrlines` and `retrbinary` read what a command answers, `storlines` and `storbinary` write what a stream holds, `list`, `ls`, `dir`, and `nlst` name what a directory holds, `get`, `gettextfile`, and `getbinaryfile` read a file down, `put`, `puttextfile`, and `putbinaryfile` write one up, and `resume` picks a transfer up where it left off by appending. `mtime` reads back when a file was last written, and `acct` sends the account a server asks for
- **Sockets of every kind**: `Socket.new` takes a family and a kind and refuses a name neither is known by. `bind`, `listen`, `connect`, `accept`, `accept_nonblock`, and `sysaccept` work a socket of any family, answering a `Socket` and an `Addrinfo` where Ruby does, and reporting an address already in use, one the machine does not answer on, and a second bind or connect by the errors the operating system names. `Socket.pair` joins two ends in either family, `Socket.tcp` reaches a server, and `Socket.accept_loop`, `tcp_server_loop`, `unix_server_loop`, `udp_server_loop`, and `udp_server_loop_on` hand connection after connection to a block
- **Reading and writing a socket**: `read`, `recv`, `recvfrom`, and their `_nonblock` forms fill a buffer handed in, keeping the encoding it was tagged with, answer nothing at all once the other end has finished, and report that nothing is there yet as a wait rather than as an error. `send` and `sendmsg` carry the flags they are given, so out-of-band data travels ahead of what is queued and `MSG_PEEK` takes a look without taking the message. `close_read` and `close_write` shut one side and close the socket once both are shut, and `shutdown` takes its mode by number, symbol, string, or anything spelling itself out as one
- **Addresses and names**: `Socket.getaddrinfo`, `Addrinfo.getaddrinfo`, `Addrinfo.tcp`, `.udp` and `.ip` ask the system's resolver, answering its entries in the order it gives them, one for each kind of socket it pairs with an address, with the canonical name `AI_CANONNAME` asks for. An address written as digits is answered as MRI answers it, once each for TCP, UDP and a raw socket, an empty host or `<any>` reads as `0.0.0.0`, and a failed lookup raises `Socket::ResolutionError`. `Socket.getnameinfo`, `gethostbyname`, `gethostbyaddr`, `getservbyname`, and `getifaddrs` answer what the system knows, `BasicSocket#local_address` and `#remote_address` name both ends whatever kind of socket they belong to, and `Addrinfo#connect`, `#connect_from`, `#connect_to`, and `#family_addrinfo` reach one address from another of the same family
- **Sockets named by a path**: a UNIX socket carries each message on its own as well as over a connection, `recvfrom` says which path a datagram came from, and `send_io` and `recv_io` hand a stream of the program to the other end and take one back
- **Socket settings**: `Socket::Option` names the family, the level, and the option a setting belongs to, taking each as a symbol, a string, or a number and refusing an unknown one with a SocketError. `Socket::Option.int`, `.bool`, and `.linger` build one, `int`, `bool`, `linger`, `unpack`, and `data` read it back, and `inspect` renders a linger setting the way Ruby does. Every socket answers `binmode?`, `nonblock?`, `close_on_exec?`, and `autoclose?`, `UNIXSocket.socketpair` gives two ends already joined, and `Socket.udp_server_recv` hands each datagram to a block with a `Socket::UDPSource` saying where it came from
- **YAML**: `require 'yaml'` loads MRI's own Psych library, carried unchanged, over a YAML reader and writer of Metorex's own that follow libyaml 0.2.5: the reader hands Psych's parser the events libyaml does, with the marks and errors it reports, and the writer chooses each scalar's style and lays out each document as libyaml's emitter does. Psych's own test suite from Ruby 4.0.7 gives the same counts under Metorex as under MRI, which `scripts/run_psych_tests.sh` compares file by file
- **Build settings**: `require 'rbconfig'` gives `RbConfig::CONFIG` naming the version, the host, the extensions, and the tools the interpreter was built with, and `RbConfig::TOPDIR` is nil because metorex is one binary rather than an installed tree. `RbConfig.ruby` and `RbConfig::CONFIG["ruby_install_name"]` name the path and file the binary was started through, so metorex linked in as `ruby` starts other processes as `ruby`. `require 'rbconfig/sizeof'` adds `RbConfig::SIZEOF` and `RbConfig::LIMITS`
- **C extensions**: `require` of a `.bundle` or `.so` opens it and runs its `Init_<name>` function, and so does a `require` or `autoload` of its path written without the ending, once no `.rb` file of the name is on the load path. A name ending in `.so` or `.o` loads the extension built for the platform, so `require "debug/debug.so"` loads `debug.bundle` on macOS. An extension is compiled against the headers under `include/`, which `RbConfig::CONFIG["rubyhdrdir"]` names, and `require 'mkmf'` gives the `create_makefile`, `append_cflags` and `dir_config` an extconf.rb calls, building in the current directory from the sources under `$srcdir`, the extconf.rb's own directory, or the ones `$srcs` names, with `$INCFLAGS` on the compiler's command line and a directory in the extension's name, such as `debug/debug`, written to the Makefile as `target_prefix`, with `have_header`, `have_library`, `have_func`, `have_var`, `have_type`, `have_macro` and `have_const` compiling a test program and adding the `HAVE_` define or library the Makefile carries. `ruby -run -e mkmf` runs those checks from its options and writes the Makefile. The C API defines classes, modules, methods of any arity, attributes, constants, class variables, Structs, Data classes, and globals backed by C memory or C functions. It calls into Ruby with `rb_funcall`, `rb_call_super`, `rb_call_super_kw`, and `rb_block_call`, whose C function can yield to the block of the C method that called it, and `rb_iterate`, whose walk hands its block to the next method it calls, as `rb_each` does. `rb_call_super` passes the C method's block on and reaches `method_missing` when nothing is above it, and a keyword argument C passes is read through `to_hash`, reads an object's type tag with `TYPE` and `RB_TYPE_P`, reads arguments with `rb_scan_args` including keywords and the block, and yields with `rb_yield`, `rb_block_proc` and `rb_proc_new`. Strings are read and written through `RSTRING_PTR` and `StringValue`, with room to grow that `rb_str_set_len` and `rb_str_resize` work on, and Arrays through `RARRAY_PTR`, `RARRAY_AREF` and `RARRAY_ASET`, with what C writes carried back into the object when control leaves C. `RBASIC` reads an object's class and flags, and setting or clearing its FREEZE flag freezes or thaws the object. `rb_protect`, `rb_rescue`, `rb_rescue2` and `rb_ensure` catch what Ruby raises, throws or breaks out of through C, with `$!` and `rb_errinfo` naming the exception while a rescue or ensure function runs, and `rb_jump_tag` raises it again. `rb_catch` and `rb_throw` work alongside Ruby's `catch` and `throw`, `rb_set_end_proc` runs a C function with the `at_exit` blocks, and `ruby_vm_at_exit` runs one as the process ends. It allocates objects through the allocator a class has, `rb_get_alloc_func` handing back one C can call for a class without its own, converts objects with `rb_convert_type` and its `rb_check_` forms, and keeps an instance variable C names without an `@` where Ruby code cannot see it. An IO is reached through `GetOpenFile` and `rb_io_descriptor`, written to with `rb_io_write` and its relatives, and waited on with `rb_io_wait` and `rb_thread_fd_select`, which let the other threads run while the descriptor is not ready. Encodings are found by name or by their index in `Encoding.list`, and `rb_define_dummy_encoding` and `rb_enc_alias` add to what `Encoding.find` answers. The `rb_enc_` character functions read the characters a run of bytes holds in any encoding. A C extension adds a digest algorithm by subclassing Digest::Base and naming its init, update and finish functions through `rb_digest_make_metadata`. The debug inspector, `rb_debug_inspector_open`, hands C the `self`, class, binding and backtrace location of each frame on the call stack, the native methods running among them, such as the `Kernel#require` loading the extension, and `rb_debug_inspector_frame_depth` and `rb_debug_inspector_current_depth` count the frames from the outermost. The instruction sequence functions the debug gem declares for itself, `rb_iseqw_to_iseq`, `rb_iseqw_new`, `rb_iseq_realpath`, `rb_iseq_type`, `rb_iseq_parameters` and `rb_iseq_code_location`, answer for a `RubyVM::InstructionSequence`. `rb_objspace_each_objects` hands C a `T_IMEMO` object for each method, class, module and block written in the Ruby files loaded so far, read from them with metorex's own parser, and `rb_obj_memsize_of` answers what `ObjectSpace.memsize_of` does. `rb_raise`, `rb_sprintf` and `rb_warn` format with `PRIsVALUE`. It wraps C pointers in objects with `TypedData_Wrap_Struct` and `Data_Wrap_Struct`, checked by type with `TypedData_Get_Struct`, and makes a class's instances with the allocator `rb_define_alloc_func` gives it. `GC.start` frees a wrapped object that neither Ruby, the mark function of a wrapped object something reaches, nor an address given to `rb_gc_register_address` reaches, running its free function, and the program frees the rest as it ends. It makes threads around C functions and runs a C function without the interpreter through `rb_thread_call_without_gvl`, on an operating system thread of its own that waking the calling thread or a signal can interrupt. It also covers IDs and Symbols, Arrays, Hashes and their hash codes, Sets, Ranges, Fibers, Mutexes, TracePoints, Regexps, Times, `st_table`, Integers of any width, Floats, Rationals and Complex numbers with their conversions and coercion, exceptions and `rb_errinfo`, the `rb_c*`, `rb_m*` and `rb_e*` class globals, and `rb_marshal_dump` and `rb_marshal_load`. An error raised in C reaches Ruby as the exception it was
- **Fiddle**: `require 'fiddle'` opens shared libraries with `Fiddle::Handle` and finds the symbols in them, `Fiddle::Function` calls a C function with up to eight integer or pointer arguments and eight floating point ones, writing what the function puts in a String argument back into the String, and `Fiddle::Pointer` reads the byte, the run of bytes, or the C string at an address. The `TYPE_`, `SIZEOF_`, and `ALIGN_` constants name the C types
- **Name lookup**: `require 'resolv'` gives `Resolv` and `Resolv::Hosts`, reading a hosts file for `getaddress`, `getaddresses`, `getname`, and `getnames`, and raising `Resolv::ResolvError` for a name nothing answers to
- **Command-line options**: `require 'optparse'` gives MRI's own `OptionParser`, with `optparse/time`, `optparse/date`, `optparse/uri`, `optparse/shellwords`, `optparse/kwargs` and `optparse/ac`. `on` describes an option by its short and long names, taking `--[no-]name` for one that negates and `--name VALUE` for one that takes a value, and `parse`, `parse!`, `order`, and `order!` read them out of an argument list, storing what they find in the hash an `into:` keyword names
- **Syslog**: `require 'syslog'` gives the system log. `open`, `close`, `reopen`, and `opened?` work it, `ident`, `options`, `facility`, and `mask` report how it was opened, and `debug`, `info`, `notice`, `warning`, `err`, `crit`, `alert`, `emerg`, and `log` write at their severity. `Syslog::Constants` names the severities, facilities, and options, with `LOG_MASK` and `LOG_UPTO` building a mask. A message goes through the C library, and under `LOG_PERROR` it also reaches `$stderr`
- **OpenSSL ciphers**: `OpenSSL::Cipher` runs AES-128-CBC, AES-192-CBC, and AES-256-CBC, padding as PKCS#7 pads, with `encrypt`, `decrypt`, `key=`, `iv=`, `random_key`, `random_iv`, `padding=`, `update`, and `final`, and refuses any other algorithm with `OpenSSL::Cipher::CipherError`. `OpenSSL::SSL` names the verify modes OpenSSL has
- **TLS**: `OpenSSL::SSL::SSLSocket` and `SSLServer` speak TLS 1.3 over any IO, as a client or a server, with one cipher suite, TLS_AES_128_GCM_SHA256, keys agreed with X25519, and RSA certificates proved with rsa_pss_rsae_sha256. The engine in `src/tls` reads and writes no socket itself, and pauses once the server's chain arrives, so the client checks it through `OpenSSL::X509::Store` against the context's `ca_file`, `ca_path`, and `cert_store`, and its name with `verify_hostname` and `post_connection_check`, failing with OpenSSL's messages. `OpenSSL::X509::Certificate.new` reads PEM and DER, and `OpenSSL::PKey.read` reads RSA keys. MRI's `openssl/buffering.rb` gives the socket its IO methods. Sessions are not resumed, and client certificates are refused
- **OpenSSL digests and keys**: `require 'openssl'` gives `OpenSSL::Digest`, named the way OpenSSL names one and reaching the same algorithms `digest` carries; `OpenSSL::HMAC` for a digest keyed with a secret; `OpenSSL::KDF.pbkdf2_hmac` for a key derived from a password; `OpenSSL::Random` for bytes nobody can guess; and `OpenSSL.secure_compare` and `fixed_length_secure_compare`, which take the same time however two strings differ
- **A block shares the locals it closes over**: a block reads and writes the locals of the scope it was written in through one map every call shares, so calling it copies nothing. The program runs in a scope of its own beneath the core library's, so a method or a class body does not see a top-level local, while constants reach everywhere
- **A name is a local where its scope bound it**: `name [1]` indexes a local and passes an array to a method of that name otherwise, and the parser decides by the scope the name is written in. A `def`, `class` or `module` body sees no local bound outside it, a block's locals end with the block, and a pattern, a regexp with named groups matched by `=~`, and a `->` parameter each bind a local
- **Ripper**: `require 'ripper'` parses Ruby source into the events Ruby's own parser reports. `Ripper.lex` and `Ripper.tokenize` answer every token, whitespace and comments included, each with its position and the lexer state after it. `Ripper.sexp` and `Ripper.sexp_raw` answer the parse as nested arrays and nil for source that does not parse, and a subclass of `Ripper` receives each event through the `on_` method named for it. `Ripper::Filter` threads a value through the tokens, and `Ripper.slice` finds a run of tokens by their kinds. The scanner tracks local variables and the lexer states as Ruby's does, so `x -1` lexes as a subtraction once `x` is a local and as a call with a negative argument before
- **CGI escaping**: `require 'cgi/escape'` gives `CGI.escape` and `unescape` for a form value, `escapeURIComponent` and `unescapeURIComponent` for a piece of a URL, `escapeHTML` and `unescapeHTML` for the five characters a page reads as markup, and `escapeElement` and `unescapeElement` for the tags of named elements alone
- **Case equality of a class**: `Left === right` asks whether the right stands as one of the left whatever the right is, so `Module === String` is true, `Object === SomeClass` is true, and `String === String` is false
- **A Kernel function the program writes over**: a `def` of `p`, `puts`, or another Kernel name, in whichever module the program opens, is what a bare call reaches
- **JSON**: `require 'json'` reads and writes the interchange format. `JSON.parse` takes `symbolize_names` and `max_nesting`, `JSON.generate` and `#to_json` write any object, `JSON.pretty_generate` lays one out over several lines, and `JSON.load` reads from anything that answers `read`. A malformed document raises `JSON::ParserError`. A number is read by finding where it ends, so a document carrying a very long one is read in the time a short one takes
- **ObjectSpace.dump and .dump_all**: what an object is, written as JSON, to a String, to a temporary File, to `$stdout`, or to an IO given to it. Metorex has no heap to walk, so `dump_all` writes every object the running program can still reach. `require 'objspace/trace'` records where each object was made and has `p` name that place beside what it writes
- **Zlib**: `require 'zlib'` gives the two checksums a compressed stream keeps and the encoding underneath it. `Zlib.crc32`, `Zlib.adler32`, and `Zlib.crc_table` answer the values every implementation agrees on, and `inflate`, `deflate`, `gzip`, and `gunzip` read and write the streams. A stream any other zlib wrote reads back here, since the decoder handles stored, fixed, and dynamic blocks alike. What metorex writes is what zlib itself writes, byte for byte at the default level: the same hash chains and lazy matching choose the matches, the same heap order hands out the code lengths, and each block is written as whichever of a stored, a fixed, or a dynamic block is smallest. `Zlib::Deflate#set_dictionary` writes a stream against a dictionary, which its header says so in. `Zlib::Inflate`, `Deflate`, `GzipReader`, and `GzipWriter` carry the same over a stream, with `GzipReader` walking what it holds a line, a character, or a paragraph at a time
- **ERB**: `require 'erb'` gives templates with Ruby in them. `<%= %>` prints what it names, `<% %>` runs code, and `<%# %>` is a comment. A trim mode decides what a tag leaves behind: `>` drops the newline after a tag that ends a line, `<>` only when the tag opens the line too, `-` trims where `<%-` and `-%>` say to, and `%` reads a line opening with `%` as a line of code. `result`, `result_with_hash`, and `run` render one, `src` shows the Ruby it compiled to, `def_method`, `def_class`, and `def_module` write it onto a class as a method, and `ERB::Util` adds `html_escape` and `url_encode` under their short names too
- **Open3**: `require 'open3'` runs a command and hands back what it wrote. `capture2` answers its output and status, `capture2e` joins its error stream onto the output, and `capture3` keeps the two apart. `popen2`, `popen2e`, `popen3`, and the `pipeline` family hand the streams over instead
- **BigDecimal**: `require 'bigdecimal'` gives decimal numbers with as many digits as they are given. A value is a sign, its significant digits, and the power of ten they sit against, which `split`, `exponent`, `precision`, and `sign` report. Arithmetic covers `+`, `-`, `*`, `/`, `div`, `quo`, `%`, `modulo`, `remainder`, `divmod`, and `**`, each carrying the digits asked for, and `add`, `sub`, and `mult` take a count of their own. `round`, `ceil`, `floor`, `truncate`, `fix`, and `frac` cut at a named place under any of the seven rounding rules, spelled either by name or by constant. `sqrt` takes a count of digits, `to_s` writes the number in exponent or full form with optional grouping, and `NaN` and the two infinities answer `nan?`, `infinite?`, and `finite?`. `BigMath` adds `PI`, `E`, and `sqrt`, and `bigdecimal/util` adds `to_d` to Integer, Float, String, Rational, and nil
- **Delegation**: `require 'delegate'` gives `Delegator`, `SimpleDelegator`, and `DelegateClass(Klass)`. A delegator passes on whatever the object behind it answers to, settles `==`, `!=`, `equal?`, and `eql?` against itself before asking, and reports the object's methods alongside its own. `__getobj__` and `__setobj__` say which object is standing behind it
- **Forwardable**: `require 'forwardable'` gives `def_delegator`, `def_delegators`, `delegate` and `instance_delegate`, which hand a call on to whatever an accessor names: an instance variable, a method, a global, or an expression written as a String. `SingleForwardable` does the same for one object, and a call forwarded to a private method reaches it with a warning naming the call
- **io/wait**: `require 'io/wait'` loads, and `IO#wait`, `wait_readable`, `wait_writable` and `wait_priority` wait on pipes, files and sockets alike, answering the stream when it is ready, nil when the time runs out, and the ready events for a mask of them
- **PTY**: `require 'pty'` gives `PTY.spawn`, which runs a command in a session of its own with a fresh pseudo-terminal as its terminal and answers the ends to read and type through with its process id, `PTY.open`, which answers a terminal pair, and `PTY.check`, which reports a child's status without waiting and raises `PTY::ChildExited` when asked to. `File.new` and `File.for_fd` over a descriptor keep the `path:` they are given
- **Continuations**: `require 'continuation'` warns that `callcc` is obsolete, as MRI does, and gives `Kernel#callcc`, whose `Continuation#call` and `[]` return from the block at once with the values given. A continuation called after its block has returned runs the statement its `callcc` was written in again, with `callcc` answering the values given, while the method or file holding that statement still runs. Called once that code has returned, it raises NotImplementedError. A native method between the two, such as `map`, is run again from its start rather than resumed part-way through
- **PStore**: `require 'pstore'` keeps a Hash of objects in a file written the way Marshal writes it, so MRI reads what metorex stores and the other way round. `transaction` reads the table, runs the block and writes the table back, `transaction(true)` only reads, and `commit` and `abort` leave early with or without the changes. `[]`, `[]=`, `fetch`, `delete`, `roots`, `root?` and their `keys` and `key?` names raise `PStore::Error` outside a transaction, and a change inside a read-only one raises it too
- **Benchmark**: `require 'benchmark'` gives `Benchmark.measure`, which answers a `Benchmark::Tms` of the user, system, children's and real time a block took, `realtime` and `ms`, which answer the clock time in seconds and milliseconds, and `bm` and `bmbm`, which print a report line for each block, `bmbm` after a rehearsal run. A `Tms` adds, subtracts, multiplies and divides, and writes itself through `format` with `%u`, `%y`, `%U`, `%Y`, `%t`, `%r` and `%n`
- **FileUtils**: `require 'fileutils'` gives every command MRI's does, from `mkdir_p`, `cp_r`, `ln_sr`, `mv` and `rm_rf` to `install`, `chmod_R` with symbolic modes such as `u+x,go=r`, `chown_R`, `touch`, `compare_file` and `uptodate?`. Each is a module function a class can include, prints the shell command it stands for when given `verbose: true`, and does nothing when given `noop: true`. `FileUtils::Verbose`, `NoWrite` and `DryRun` give every command those options, and `commands`, `options`, `options_of`, `have_option?` and `collect_method` answer which command takes which
- **`ruby -run -e`**: `require 'un'`, which `-run` names, gives the commands `cp`, `ln`, `mv`, `rm`, `mkdir`, `rmdir`, `install`, `chmod`, `touch`, `wait_writable`, `httpd`, `mkmf`, `colorize` and `help`, each reading its options with OptionParser and working through FileUtils, so `metorex -run -e cp -- -rv src dest` copies a tree and prints what it did. `httpd` needs the webrick gem, as it does under MRI. `colorize` writes a file with the terminal colors `IRB::Color.colorize_code` gives each token, and writes it unchanged when standard output is not a terminal
- **OptionParser reads options in order or permuted**: `order!` hands each argument that is not an option to its block, or stops at the first one without a block, `permute!` and `parse!` collect them, and short flags may be written together (`-rv`) with a value written attached (`-n5`)
- **TSort**: `require 'tsort'` orders a graph an object describes through `tsort_each_node` and `tsort_each_child`, or two callables describe, with `tsort`, `tsort_each`, `strongly_connected_components` and `each_strongly_connected_component`, and raises `TSort::Cyclic` for a cycle
- **Methods core classes answer natively are reported**: `respond_to?`, `defined?` and `method_defined?` answer for the methods Array, Hash, String, Symbol, the numbers, nil, true, false and the rest answer without a method table, so `[1].respond_to?(:each)` is true and `Class.new.method_defined?(:__send__)` is true, and a class below a core class can `alias` one of them. Every class answers Class's `allocate`, `superclass`, `subclasses`, `attached_object` and `new`, which `Class.instance_methods(false)` lists apart from the methods Class inherits from Module
- **WeakRef**: `require 'weakref'` gives a reference that delegates to an object and answers `weakref_alive?`. Metorex frees an object when the last reference to it goes, so the reference holds it through ObjectSpace's weak map
- **Logger**: `require 'logger'` gives a log with a severity on each message. `debug`, `info`, `warn`, `error`, `fatal`, and `unknown` write at their own level, `add` and `log` take one, and `level` decides which of them reach the device. A level may be named or numbered. `Logger::LogDevice` writes to an IO or to a file it opens by name, rotating that file when it is asked to keep more than one, and `Logger::Formatter` decides how a line reads
- **Monitor**: `require 'monitor'` gives `Monitor` and `MonitorMixin`, a lock a thread may take more than once. `enter`, `exit`, `try_enter`, `synchronize`, `mon_locked?`, and `mon_owned?` work it, `new_cond` makes a `MonitorMixin::ConditionVariable` to wait on, and leaving a monitor nobody is holding raises `ThreadError`
- **Tempfile**: `require 'tempfile'` gives a file with a name nobody else is using. `Tempfile.new`, `.open`, and `.create` make one, `path` names it, `close` and `close!` finish with it, and `unlink` removes it. `Dir.tmpdir` names the directory they go in
- **`File.rename` and `File.chmod`**: move a name onto another, and set the permissions on the names given
- **A Singleton is reached through `instance` alone**: both `new` and `allocate` are refused on a class that includes it
- **Etc**: `require 'etc'` reads the password and group databases and what the system reports about itself. `getpwuid`, `getpwnam`, `getpwent`, `setpwent`, `endpwent`, and `passwd` walk the accounts, `getgrgid`, `getgrnam`, `getgrent`, `setgrent`, `endgrent`, and `group` walk the groups, and both hand back `Etc::Passwd` and `Etc::Group` structs. `uname`, `nprocessors`, `sysconf`, `confstr`, `getlogin`, `sysconfdir`, and `systmpdir` answer the rest, with the `SC_` and `CS_PATH` constants the C library knows
- **Coverage**: `require 'coverage'` gives `supported?`, `running?`, `start`, `setup`, `result`, `peek_result`, and `line_stub`. A file loaded while measurement is on is counted line by line, with a count on every line that carries something to run and nil on every other one. `start` with `:all` or with a mode named outright reports each mode under its own name, where `methods` keys each definition by owner, name, and the span it was written across, and `eval` counts code handed to `eval` or `class_eval` into the file it names. `result` takes `stop:` and `clear:`, which decide whether measurement ends and whether the counts go back to zero
- **A `key => value` pair without parentheses**: `held.update "a" => 1` and `send @method, Object.new => "0"` gather their pairs into a Hash the same way a call written with parentheses does
- **An unknown pack directive raises ArgumentError**: `[1].pack('%')` reports `unknown pack directive '%' in '%'`
- **Process identity and scheduling**: `Process.getpgrp`, `setpgrp`, `getpgid`, `setpgid`, `getsid`, and `setsid` name the group and session a process belongs to, `getpriority` and `setpriority` read and write how much of the processor it is given, and `initgroups` and `groups=` set the supplementary groups it runs with. `Process.times` answers a `Process::Tms`, `setproctitle` renames the process in a listing, and `Process._fork` reports that forking is not carried here. `Process::GID`, `Process::UID`, and `Process::Sys` name the same ids Process itself does, and the WNOHANG, PRIO_, RLIMIT_, and RLIM_ constants carry the numbers the operating system names its settings by
- **Signals reach the operating system**: `Process.kill` sends the signal itself rather than only raising, so a process that kills itself ends with that signal and one sent to a process that is not there reports `Errno::ESRCH`. A `SignalException` nothing catches ends the program the way the signal would have, which is what a wait reports back as `Process::Status#termsig`
- **FileTest**: the questions File answers about a path, gathered in a module so they can be asked without naming File. `exist?`, `file?`, `directory?`, `readable?`, `readable_real?`, `writable?`, `writable_real?`, `executable?`, `executable_real?`, `symlink?`, `blockdev?`, `chardev?`, `pipe?`, `socket?`, `setuid?`, `setgid?`, `sticky?`, `owned?`, `grpowned?`, `identical?`, `zero?`, `empty?`, `size`, `size?`, `world_readable?`, and `world_writable?`
- **A path may be named by an object**: `File.exist?`, `file?`, `directory?`, `executable?`, `symlink?`, `size`, `size?`, and `readlink` ask an argument that is not a String for `to_path`, and then for `to_str`, before reaching the filesystem
- **File.birthtime**: answers when a file was made, the same time `File.stat(path).birthtime` reports
- **Process.groups**: the supplementary groups this process belongs to. `grpowned?` counts them, so a file whose group is any of them belongs to this process's group
- **`RUBY_PLATFORM` names an architecture and an operating system**: `arm64-darwin` rather than `macos`, which is the form Ruby reports and the form the spec suite's platform guards read
- **An arithmetic sequence**: `1.step(10, 3)` and `(1..10).step(3)` answer an `Enumerator::ArithmeticSequence` when they are handed no block, and `(1..10) % 3` writes the same thing. It answers `begin`, `end`, `step`, `exclude_end?`, `each`, `first`, `last`, `size`, `hash`, `==`, and an `inspect` that writes it back the way it was asked for. A sequence with no end reports `Float::INFINITY` as its size, and it is built only through those methods, so `new` raises NoMethodError and `allocate` raises TypeError. `Numeric#step` also names its end and its stride by keyword, as `1.step(to: 9, by: 4)`, and refuses one given both ways, an unknown keyword, and a stride that cannot be compared against zero. A range over anything but numbers answers a plain `Enumerator` instead, whose size is unknown: `("A".."G").step(2)` walks with `succ` and hands out every second value, `("A".."AAA").step("A")` adds the step with `+`, a step running against the range's direction reaches nothing, and a step over numbers that is neither `nil` nor a `Numeric` is read through `coerce` where the walk runs
- **Enumerator::Product**: the Cartesian product of several walks, yielding one array per combination in the order the walks were given. It answers `each`, `to_a`, `size`, `rewind`, `inspect`, and a private `initialize_copy` that refuses a frozen receiver, a different class, and an uninitialized argument. `size` is nil as soon as one walk cannot say how long it is
- **Enumerator::Yielder takes a block**: `Enumerator::Yielder.new { |value| ... }` hands everything to that block. `yield` passes its arguments through and answers what the block answers, `<<` passes one value and answers the yielder, and `to_proc` answers the block itself
- **Rewinding a walk rewinds its source**: `Enumerator#rewind` hands the rewind on to the object the walk was cut from when that object can be rewound
- **A missing method names its receiver the way Ruby does**: `undefined method 'each_entry' for an instance of Object`, and `for nil`, `for true`, `for class Foo`, or `for module Bar` when that is what it was called on
- **A class may answer `allocate` itself**: a `def self.allocate` is what the class hands back, and a subclass inherits it
- **An empty matrix keeps its shape**: `Matrix.columns([[], [], []])` is 0 by 3 and `Matrix.column_vector([])` is 0 by 1, `transpose` swaps the two sizes, `inspect` writes `Matrix.empty(0, 3)`, and two empty matrices of different widths are not equal. Multiplying by an empty matrix answers one as wide as the right side
- **Matrix#minor and Matrix#find_index**: `minor` reads a rectangle given as a start and a count on each side or as two ranges, counts backwards from a negative start, ignores a count past the edge, and answers nil for a start out of range or a negative count. `find_index` reads a value, a block, or one of `:diagonal`, `:off_diagonal`, `:lower`, `:strict_lower`, `:strict_upper`, and `:upper`, and answers an Enumerator when given neither a value nor a block
- **A size given to `Matrix.build` goes through `to_int`**, so anything that converts is accepted and anything else raises TypeError. `Vector#each2` and `Vector#collect2` pair with any sized and indexable object, including a plain Array, and `Vector#inner_product` takes the conjugate of its argument
- **A subscript may be written across several lines**: a newline after the opening bracket carries no meaning, so `Matrix[` may be followed by one row per line before the closing bracket
- **`private_class_method :new`**: a class method the runtime answers can be made private, so a class built only through its named builders refuses `new` and reports false from `respond_to?(:new)`
- **`Enumerator#to_a` collects without answering the collection**: the block it walks with answers nil, so a method that stops on the first truthy block result walks the whole way when it is asked for an Enumerator
- **Date**: `require 'date'` gives a calendar day held as the Julian Day Number it stands for, together with the day the Gregorian calendar takes over from the Julian one. Dates are built with `civil`, `jd`, `ordinal`, `commercial`, `today`, `parse`, `strptime`, `iso8601`, and `rfc3339`, and answer `year`, `month`, `day`, `yday`, `wday`, `cwyear`, `cweek`, `cwday`, `mjd`, `ajd`, `amjd`, `ld`, `leap?`, and a question per weekday. `+`, `-`, `>>`, `<<`, `next_day`, `prev_month`, `succ`, `upto`, `downto`, and `step` move about, `new_start`, `italy`, `england`, `julian`, and `gregorian` say which calendar a date is read on, and `strftime`, `to_s`, `inspect`, `asctime`, `iso8601`, and `rfc3339` write one out. `Date.valid_civil?`, `valid_ordinal?`, `valid_commercial?`, `valid_jd?`, `leap?`, `gregorian_leap?`, and `julian_leap?` answer without raising, and `Date::Infinity` stands for a calendar that never reforms
- **DateTime**: `require 'date'` also gives a calendar day carrying a time of day and an offset from UTC, held as an exact fraction of a day so arithmetic below a millisecond loses nothing. Built with `new`, `civil`, `jd`, `ordinal`, `commercial`, `now`, `parse`, `strptime`, `iso8601`, `rfc3339`, `rfc2822`, and `httpdate`, and answering `hour`, `min`, `sec`, `sec_fraction`, `offset`, `zone`, `new_offset`, `to_date`, `to_time`, and every Date method besides. An hour written as a negative counts back from the next day, and 24 names midnight on the day after
- **The time library**: `require 'time'` loads MRI's own `time.rb`, which adds `Time.parse`, `Time.strptime`, `Time.iso8601`, `Time.xmlschema`, `Time.rfc2822`, `Time.httpdate`, and `Time.zone_offset`, along with `Time#iso8601`, `Time#xmlschema`, `Time#rfc2822`, and `Time#httpdate`, over the fields `Date._parse` and `Date._strptime` read. `Time#to_datetime` and `Time#to_date` cross to the calendar the offset a Time carries puts it on
- **strftime writes the clock**: `%H`, `%I`, `%k`, `%l`, `%M`, `%S`, `%L`, `%N` with a digit count, `%P`, `%p`, `%s`, `%Q`, `%z`, `%:z`, and `%::z` all read the time and offset the receiver holds
- **String interpolation asks for `to_s`**: `"#{object}"` calls the object's own `to_s` rather than writing the default `#<Class:0x...>` form
- **`Float#to_r` is exact**: `0.00001001.to_r` is the fraction the float actually holds, not one truncated partway through
- **A Rational compares by value inside a collection**: a Hash or an Array holding `Rational(0, 1)` equals one holding `0`
- **URI**: `require 'uri'` loads MRI 4.0.7's own uri 1.1.1, which metorex carries unchanged: `URI.parse`, `URI.join`, the `encode_www_form` pairs, the `RFC2396_PARSER` and `RFC3986_PARSER`, and the `URI::HTTP` through `URI::WSS` classes, every one of which includes `URI` as MRI's do
- **Pathname operations**: a Pathname is a path as an object, with the naming operations kept apart from the ones that reach the filesystem. `+`, `/`, `join`, `parent`, `basename`, `dirname`, `extname`, `sub`, `sub_ext`, `cleanpath`, `relative_path_from`, `each_filename`, `descend`, and `ascend` work on the name alone, while `exist?`, `file?`, `directory?`, `empty?`, `size`, `read`, `children`, `glob`, and `realpath` ask the filesystem. `Pathname(path)` hands back a Pathname that arrives already made
- **File.size and Dir.entries**: `File.size(path)` and `File.size?(path)` answer how many bytes a file holds, and `Dir.entries(path)` and `Dir.children(path)` list what a directory holds
- **Dir.glob takes flags and a base**: `Dir.glob(pattern, File::FNM_DOTMATCH)` matches names that open with a dot, and `base:` reads the pattern against a directory and answers paths relative to it. The `File::FNM_*` constants are defined
- **An array of objects compares with their own equality**: `[held] == [other]` asks the objects, so two Pathnames naming the same path make their arrays equal
- **An open Dir**: `Dir.new` and `Dir.open` answer a directory that is walked one name at a time, with `read`, `each`, `each_child`, `pos`, `tell`, `seek`, `pos=`, `rewind`, `entries`, `children`, `path`, `to_path`, `close`, and `closed?`. Walking the whole of one leaves the position at the end, the path stands after it is closed, and every other method on a closed one raises IOError. `Dir` includes Enumerable
- **ENV answers a few things its own way**: `to_s` is "ENV", `rehash` is nil, `to_h` and `to_hash` hand back a copy so changing it leaves the environment alone, and `dup` and `clone` are refused the way Ruby refuses them
- **Encoding.find**: names an encoding, with "locale", "external", and "filesystem" answering what those settings hold and "internal" answering `Encoding.default_internal`. `String#force_encoding` answers the string it was given
- **`pretty_inspect`**: every object answers it once pp is loaded, by `require "pp"` or the first `pp`, as in Ruby
- **A symbol is quoted when its name needs it**: `:"a b"` and `:"1x"` are written with quotes, while `:@a`, `:$b`, `:foo?`, `:[]`, and the operator names stand bare
- **An escaped space keeps a percent-list word whole**: `%w[a\ b c]` is two words, not three
- **File::Stat**: `File.stat` and `File.lstat` answer a real File::Stat, reading what the operating system keeps about a file. `dev`, `ino`, `mode`, `nlink`, `uid`, `gid`, `rdev`, `size`, `blksize`, `blocks`, the major and minor pairs, `atime`, `mtime`, `ctime`, `birthtime`, `ftype`, `inspect`, `<=>`, and the questions from `file?` through `world_writable?`. `File` asks the same questions of a path with `ftype`, `zero?`, `size?`, `identical?`, `owned?`, `grpowned?`, `setuid?`, `setgid?`, `sticky?`, `blockdev?`, `chardev?`, `pipe?`, `socket?`, `world_readable?`, and `world_writable?`
- **Links**: `File.symlink`, `File.readlink`, and `File.link` make and read them, and `File.symlink?` answers what the filesystem says rather than always false
- **File.chown, File.lchown, and File.utime**: set who owns a file and the times it reports
- **Dir.entries and Dir.children**: list what a directory holds
- **Process.gid and Process.egid**: answer the group this process runs as
- **`method_missing` is handed what Ruby hands it**: the name arrives as a Symbol and the call's arguments arrive one by one, so `def method_missing(name, path)` reads the path as itself rather than as an array holding it
- **`Integer#to_s` takes a base**: `255.to_s(16)` is "ff"
- **The reported Ruby version is 4.0.7**: `RUBY_VERSION`, `RUBY_PATCHLEVEL` (0), `RUBY_RELEASE_DATE` and `-v` name the MRI release metorex follows, and the spec suite's version guards open for it. `METOREX_RUBY_VERSION` names another release instead, and the guards follow it, so `RUBY_VERSIONS="4.0.7 3.4.8 3.3.10" scripts/run_ruby_spec.sh` runs the whole queue once per release and reports each
- **A raise inside a Set block carries on as itself**: `Set[1].each { raise ArgumentError }` reaches a `rescue ArgumentError` rather than arriving as a RuntimeError
- **A Set decides equality itself**: `set == other` reaches Set's own `==`, so anything that answers `is_a?(Set)` is compared by the elements it holds
- **GetoptLong**: `require 'getoptlong'` reads command line options the way the GNU getopt_long function does. An option may be written in full or as one letter, one-letter options may be run together, a word may follow an equals sign or stand as the next word, a shortened name is taken when only one option starts with it, and `--` ends the options. `get`, `each`, `terminate`, `terminated?`, `ordering=`, `set_options`, `error_message`, and the `MissingArgument`, `InvalidOption`, `AmbiguousOption`, and `NeedlessArgument` errors are all there
- **IPAddr**: `require 'ipaddr'` gives an IPv4 or IPv6 address held as the number it stands for, with the mask that says how much of it names the network. It reads the short IPv6 form, a dotted quad written inside an IPv6 address, and a mask written as a length or as an address, and answers `to_s`, `to_string`, `inspect`, `family`, `prefix`, `reverse`, `ip6_arpa`, `ip6_int`, `native`, `ipv4_compat`, `ipv4_mapped`, `include?`, `mask`, and the bit operations `&`, `|`, `<<`, `>>`, and `~`
- **Find**: `require 'find'` walks every path beneath a directory, and `Find.prune` leaves the directory it was handed unwalked
- **Timeout**: `require 'timeout'` gives `Timeout.timeout`, which runs a block and answers what it answers. Once the limit has passed it raises `Timeout::Error`, or whatever class and message the call named, into the block wherever it stands, so a loop that never waits is stopped too. It raises once while the block's ensure clauses run
- **Random is the Mersenne Twister**: `Random.new(seed)` draws from MT19937 seeded the way Ruby seeds it, so a seed gives the same numbers here as it does there. `bytes`, `rand`, `random_number`, `seed`, `==`, `Random.new_seed`, `Random.urandom`, and `Random.srand` are all there, and `srand` settles what the seedless `rand` answers as well
- **A Range bounds a draw by its width**: `rand(first..last)` measures `last - first`, so any type that subtracts and adds bounds a draw of its own kind, and a pair of ends that cannot be subtracted raises ArgumentError
- **Encoding::Converter**: names the two encodings a conversion runs between, the steps it takes, and what stands in for a character the destination cannot spell. The steps are the fewest that reach the destination, Shift_JIS, EUC-JP, stateless-ISO-2022-JP and ISO-2022-JP stepping between each other without UTF-8, and `convert` carries text through each one, holding a character or an escape a piece stops part-way through for the next piece. A character a step cannot spell is refused naming that step, and the newline decorators sit in the convpath where Ruby puts them. `source_encoding`, `destination_encoding`, `convpath`, `Encoding::Converter.search_convpath`, `Encoding::Converter.asciicompat_encoding`, `replacement`, `inspect`, and the flag constants are defined
- **A write puts out the bytes the string is made of**: text outside ASCII goes onto a stream in the encoding the string carries rather than one byte per character, and a string holding bytes of its own is written as those bytes. A `gets` given a separator of more than one byte keeps whatever it read past the separator for the next read, so nothing is lost on a pipe, which cannot be wound back
- **A read with nothing to hand over yet waits**: a blocked read or an `IO.select` with no timeout hands control to the other threads rather than holding the interpreter, and a stream another thread closes while one waits on it raises IOError. `IO.pipe` takes the encodings a stream is opened with and builds two ends of its own class, and closing a stream a child is still writing to lets go of the reading end so the child is told to stop
- **A range written where a condition goes is a flip-flop**: it turns on when its first side holds and stays on until its second does, each one keeping its own state, and a number written as a side is compared against the line the last read left off at. The `-n` and `-p` loops read the files named on the command line, and `$.` counts the records they read
- **An `if` whose every branch jumps away answers nothing**, so assigning from one is refused where it is written, the way Ruby refuses a void value expression
- **The top level runs against an object**: `self` outside every class is the ordinary Object Ruby calls `main`, so `self.class` is Object and a `def self.name` written there lands on that object alone. Every Kernel function is a private method of every object, which is what lets `held.send(:require, path)` reach the one a bare `require` does while `held.respond_to?(:require)` stays false. Its singleton class carries `define_method`, `include`, `private`, `public`, `ruby2_keywords` and `using` as private methods, which `send` reaches and which run as their bare forms do
- **A backtrace names a block for where it was written**: a block body is a place of its own, reported as `block in <scope>` and `block (N levels) in <scope>` by how deep it sits, where the scope is the method, class body, or file it was written in rather than whatever it is later called from. The body of a required file is `<top (required)>`, a method written in `class << Name` is `Name.method`, and a thread that dies of an exception writes that report on `$stderr` when `report_on_exception` is true. A thread waiting in `Mutex#lock`, `sleep`, `Queue#pop` or `ConditionVariable#wait` answers a `backtrace` starting at that method, and a deadlock report lists the same. A backtrace, `caller`, `caller_locations` and a thread's `backtrace` list the native methods running between frames, such as `Array#each`, `Kernel#Integer` and `Integer#/`, inside threads, Ractors, `instance_eval` and `class_eval` blocks alike, and leave out the frames of core library code written in Ruby. Each location carries the column of its call, which `RubyVM::AbstractSyntaxTree.of` finds the call node by
- **Patterns are matched by an engine that gives ground**: metorex reads a pattern into the pieces it names and walks them over the subject, taking back what a piece read when something further on cannot carry on. That is what lookbehind, backreferences by number and by name, a name written on more than one group, `\\g` calls, atomic and possessive groups, `\\K`, `\\G`, `(?~)`, and conditional groups all need. Character classes read the POSIX names, `\\p{...}` properties, nesting, and `&&`, and `\\X` reads one character as a reader sees it. A pattern that opens a group it never closes, names a group with a digit, or spans a named run is refused where it is written
- **Bytes held back after a conversion stops**: a run the source encoding cannot read is cut at unit boundaries, so `primitive_errinfo` names the whole unit that failed and the whole unit after it, and `putback` hands those later bytes over for the next piece of text without changing what the error info reports
- **A number written in another base takes the `r` and `i` suffixes**: `0xffr` is `(255/1)` and `0b1110i` is `(0+14i)`, and a literal wider than a machine word keeps every digit, so `0.0174532925199432957r` is exactly the fraction it was written as
- **A range binds looser than every operator but the conditional**: `1..false || 10` is `1..10`, a range takes no range as an end, a relation takes no relation as an operand, and a `rescue` modifier reaches around a whole conditional rather than around one of its branches
- **A pattern can be given a time limit**: `Regexp.timeout=` names how long one match of any pattern may run, read back as a number of seconds, and `Regexp.new(source, timeout: seconds)` names a limit for that pattern alone, which `Regexp#timeout` answers. A match passing its limit raises `Regexp::TimeoutError`, and a limit that is not positive is refused
- **A pattern has to be sound before it is built**: `Regexp.new` refuses an unclosed or unopened group and a group name opening with a digit or a minus, naming each the way Ruby does, and `\c` takes the character written after it, so `/\c#{name}/` is the control-# escape rather than a substitution
- **An `alias` lands where a `def` would**: one written inside `instance_eval` or `instance_exec` names a method of that object alone, a value the program cannot hold one copy of refuses the second name, and a name written at the top level belongs to Object
- **A Range built by hand**: `Range.allocate` answers a range whose ends `initialize` writes, which is the only range that can be written to, since every one written as a literal is frozen the moment it is made
- **A source written in bytes keeps them**: a `# encoding: binary` magic comment makes a run of `\xNN` or `\NNN` escapes name those bytes one by one, while elsewhere bytes that spell a character in UTF-8 read back as that character
- **Dir.glob reads the whole pattern language**: `*` and `?` within one name, `[...]` for a set or a range, `{a,b}` for a list of patterns expanded left-most first, `**` for a walk down through the subdirectories, and a backslash for a character that would otherwise be special. `base:` reads the pattern against a directory and answers paths relative to it, `sort:` says whether the answer is ordered, and the `File::FNM_*` flags widen what a name may match
- **ISO-2022-JP, IBM720, and the rest of the named encodings**: `encode` writes the escape runs ISO-2022-JP needs and the bytes every other named encoding spells its characters with, so a run of text read back through the encoding it was written in is the text it started as
- **Symbol encodings**: `String#intern` and `#to_sym` keep the string's encoding, `Symbol#to_s` hands it back, and a string whose bytes its encoding cannot read names no symbol at all
- **Shift_JIS, Windows-31J, and MacJapanese**: `encode` rewrites the bytes rather than re-tagging them, so `"日本語".encode(Encoding::SHIFT_JIS)` holds the six bytes that encoding spells it with and reads back as the same text. The half-width katakana take a byte each, `length` counts characters rather than bytes, and a lead byte with nothing after it leaves the run unreadable
- **Unicode normalization**: `unicode_normalize`, `unicode_normalize!`, and `unicode_normalized?` answer the four forms Unicode names, reading the whole character database. A Hangul syllable comes apart and back together by counting, and text in an encoding that is not Unicode raises `Encoding::CompatibilityError`
- **A string standing for bytes keeps standing for them**: joining one to text, splicing one into an interpolated literal, and `-@` all carry the bytes through, and `force_encoding("binary")` spreads the text out one character to the byte so `bytesize` and `chop!` count the same things Ruby counts
- **`Rational#**`**: an exact answer where the exponent is a whole number, a Float where no fraction stands for it, and a complex number where the base is negative. Raising nothing to a negative power raises ZeroDivisionError, and an answer too wide to build raises ArgumentError
- **`Float#round` rounds the number as it is written**: `5.55.round(1)` is 5.6, `half:` names where a value exactly between two places goes, a precision may be a Float or a Bignum, and the exceptional values raise FloatDomainError or RangeError depending on the count asked for
- **`Array#flatten` asks each element for `to_ary`**: an object that says it answers one is asked, a `method_missing` that catches the call is reached, and an answer that is not an Array raises TypeError
- **`IO.read` and `File.read` read the options Ruby reads**: `mode:`, `open_args:`, `encoding:`, `external_encoding:`, a length and an offset, and a `BOM|` mode prefix that takes a byte-order mark off the front and reads the rest in the encoding it names. `IO#read` with a count answers a run of bytes, with nothing answers text in the stream's encoding, and fills a buffer either way
- **`String#split` in every shape Ruby reads**: whitespace, a run of text, an empty separator, and a pattern whose groups are kept alongside the pieces. A count caps the pieces, a negative one keeps the empty ones at the end, `$;` stands in when no pattern is given, and a block is handed each piece while the string itself is the answer
- **`String#scrub` and `String#undump`**: `scrub` puts aside every run of bytes the encoding cannot read, taking a replacement or a block, and `undump` reads back what `dump` wrote, including the `.force_encoding(...)` a string in an encoding that spells no ASCII carries
- **`pack` and `unpack` with 'P' and 'p'**: an address rather than the text itself, read back out of the string `pack` built or a copy of it. Quoted-printable breaks its lines the way Ruby breaks them, and base64 holds 45 bytes to the line unless a count says otherwise
- **Time zones**: `in:` and the zone argument take an offset, one of the military letters, or an object that converts between its own readings and UTC, and a class carrying `find_timezone` is asked for a name no offset reads as. `Time.new("2020-12-25T00:56:17+09:00")` reads a whole date written out, with a `precision:` for the digits after the point
- **strftime writes every directive Ruby writes**: the flags `-`, `_`, `0`, `^`, and `#`, a width, the `:` forms of `%z`, and `%v`, `%c`, `%x`, `%X`, `%D`, `%F`, `%R`, `%r`, `%T`, `%U`, `%W`, `%V`, `%G`, and `%g` alongside the clock directives
- **File.new takes a numeric mode**: `File::CREAT | File::TRUNC | File::WRONLY` says what a mode string would, a `flags:` keyword is ORed into it, a third argument names the permissions a file brought into being carries, and a descriptor may stand in place of a name
- **A Float switches to exponent form**: `to_s` and `inspect` write `1.0e+15` and `1.0e-05` once the decimal point sits past the fifteenth significant place or before the fourth place to its left, and the text they answer is US-ASCII
- **`Integer#pow`**: raises the number the way `**` does, and a second argument multiplies under that modulus so a large power stays small
- **`cycle` walks the elements over and over**: a count names the rounds, the first round hands out the elements as it reads them, `nil` or no argument walks without end, and the enumerator the block-less form answers reports the count it will hand out
- **Appending negotiates encodings**: `<<` and `concat` take the other string's encoding when the receiver is empty or holds nothing but ASCII, refuse a pair that cannot be read alongside each other, and read an Integer as a codepoint in the receiver's own encoding
- **`String#initialize`, `String#setbyte`, `String#crypt`**: a string given another takes its characters and encoding, one byte may be written whatever it spells, and `crypt` is the C library's one-way hash
- **A time built from its parts**: `Time.utc` and `Time.local` read a numeral spelled out as base ten, take the month in words or digits, coerce through `to_int` and `to_str`, refuse a part outside its range, and read ten arguments in the order the C library writes them, with the daylight-saving flag telling the two readings of a repeated hour apart
- **`File.dirname` takes a level**: a second argument says how many parts to drop, a run of separators at the front reads as the one root, and the answer keeps the path's own encoding
- **`File#flock`**: locks a whole file, with `LOCK_NB` answering false rather than waiting, and a waiting lock leaves the other threads running
- **`Process.detach`**: watches a child in a thread of its own, which answers the status the child exited with and carries the child's id in `[:pid]` and `#pid`
- **What the object space can say**: `GC.stat` reports the collector's readings as Integers, `ObjectSpace.memsize_of_all` counts what the program has built, and `ObjectSpace.reachable_objects_from` lists what one object refers to directly
- **`Zlib::Inflate` reads a stream a piece at a time**: text arrives through `<<` or `inflate`, and what each piece stands for is handed back as soon as it arrives, as zlib hands it back. A window of 16 or more reads gzip, and one of 32 or more reads gzip or zlib, told apart by the gzip header, which is what `Net::HTTP` inflates a body with. `inflate(text, buffer:)` writes into the buffer given. `nil` marks the end of the stream and anything after it is passed through, a block is handed the text in 16 KiB pieces, and a stream written against a dictionary raises `Zlib::NeedDict` until `set_dictionary` supplies it
- **Keywords spread and named together**: `f(**held, size: 9)` carries one set of keywords rather than a hash argument beside them, and a keyword the call leaves out is named in the message: `missing keyword: :salt`
- **`class << x` runs with the singleton class as self**: a bare `to_s` in a class or module body is that class's own rather than the top level's, and a `def self.name` method is owned by the singleton class
- **Byte counts are byte counts**: `bytesize`, `bytes`, and `getbyte` read a binary string one byte to a character, and `StringIO#pos` and `StringScanner#pos` count the bytes before the cursor rather than the characters
- **A path may name itself**: `File.directory?`, `File.symlink`, and the other predicates take anything answering `to_path` or `to_str`, and the predicates take a stream through `to_io`
- **ENV is looked up by text**: `has_key?`, `include?`, `member?`, `key?`, `key`, `delete`, `assoc`, `has_value?`, `value?`, and `rassoc` ask an argument for `to_str`. A name that reads as nothing is refused, and a value that reads as nothing matches nothing
- **A time in UTC says so**: a zone written as "UTC", "Z", or "-00:00" names UTC itself, `Time.now`, `Time.new`, and `Time.at` take it through `in:`, and `Time#zone` answers in US-ASCII
- **A String changes what it holds**: `<<`, `concat`, `replace`, `prepend`, `insert`, `clear`, `setbyte`, `slice!`, and the `!` forms of `sub`, `gsub`, `strip`, `chomp`, `chop`, `squeeze`, `delete`, `tr`, `tr_s`, `upcase`, `downcase`, `capitalize`, `swapcase`, `reverse`, `succ`, and `encode` all change the string every reference to it sees. A method whose name ends in `!` answers nil when it found nothing to change
- **`freeze` and `frozen?` mean what they say**: a String reports itself frozen only after it has been frozen, a change to a frozen one raises FrozenError, `+str` answers one that changes, and `-str` answers one that does not
- **RubyVM**: names MRI's instructions in `RubyVM::INSTRUCTION_NAMES`, its options in `RubyVM::OPTS` and its stack sizes in `RubyVM::DEFAULT_PARAMS`, answers `RubyVM.stat` with the counts of its constant caches and the object shapes it has made under MRI's keys, MRI's class variable state and shape cache size at zero since metorex keeps neither, beside the units of code it has compiled, the instructions they hold, and the calls its call sites' method caches answered and missed, and remembers `RubyVM.keep_script_lines`. `RubyVM::YJIT` and `RubyVM::ZJIT` answer that they are not running and cannot be turned on. `RubyVM.new` is undefined and `RubyVM.allocate` refused. `RubyVM::AbstractSyntaxTree.parse` and `parse_file` build MRI's node tree, with MRI's node types and line and column spans, for literals, variables, calls, assignments, control flow, method, class, module, block and lambda definitions with their parameters and locals, strings, symbols, regexps, word lists and heredocs, pattern matching, and `BEGIN` and `END` blocks. `keep_script_lines: true` lets a node answer its `source` and `script_lines`, `keep_tokens: true` lets it answer `tokens` and `all_tokens` named as MRI's parser names them, `of` answers the node a Proc, a Method or a backtrace location was written as, and `locations` answers `RubyVM::AbstractSyntaxTree::Location` spans for a node and for the keywords and operators MRI records for its type, such as an `if`'s `if`, `then` and `end`. A program it refuses raises SyntaxError with MRI's message, quoting the line at fault with a caret beneath it. `RubyVM::InstructionSequence` compiles a program with `compile` or `compile_file`, runs it with `eval`, and names the method or block a Proc or a Method was compiled into with `of`. `disasm` and `to_a` list the instructions Metorex compiles the code to, which are its own rather than YARV's
- **Ractor**: `Ractor.new` runs its block on a thread of its own with `self` as the new Ractor and the arguments it was given copied in, unless they are shareable. `value` and `join` wait for the block, and what it raised comes back as `Ractor::RemoteError` with the original as its `cause`. `Ractor.current`, `Ractor.main`, `Ractor.main?`, `Ractor.count`, `#name` and `#inspect` answer as MRI's do, and a thread made inside a Ractor belongs to it. A block given to `Ractor.new` may not name a local of the scope it was written in, and a Ractor other than the main one may not reach most globals, any class variable, or a constant holding an object that is not shareable. `Ractor.make_shareable` freezes an object and everything it refers to, or a deep copy of it with `copy: true`, and makes a Proc shareable when its `self` and the outer locals it reads are shareable and none of them is assigned more than once. `Ractor.shareable_proc` and `Ractor.shareable_lambda` make one with `self` set to nil or the `self:` given. A `Ractor::Port` receives what any Ractor sends it, and only the Ractor that made it receives from it. `Ractor#send` and `Ractor.receive` use each Ractor's default port, a value sent is copied unless it is shareable, one sent with `move: true` answers `Ractor::MovedError` to any method called on it afterwards, and `Ractor.select` answers the first port or finished Ractor with its value. `Ractor[]`, `Ractor[]=` and `Ractor.store_if_absent` keep values per Ractor
- **Threads run a step at a time**: `Thread.new` starts its block on a coroutine, a thread hands the turn over at the next statement or loop pass once it has held it for 100 ms, as MRI's timer marks a turn spent, so one that never waits still lets the others run, and a socket that has nothing to give hands every waiting thread a turn rather than waiting alone, as do `system`, a backtick command and `Process.wait` while the child runs. Every Kernel, IO, File, Socket and Process call that waits lets the other threads run meanwhile: reads and writes on pipes and sockets, `select`, `IO.copy_stream`, `flock`, opening a FIFO, accepting a connection and looking a host name up. `IO#write` waits for room on a full pipe and writes everything, `syswrite` waits and writes what fits, and `write_nonblock` never waits. Every thread runs on the one operating system thread the interpreter runs on, so `Thread#native_thread_id` answers that thread's id for the thread running now and nil for the rest, as MRI answers for a thread scheduled M:N while it is off a native thread, and the `RUBY_MN_THREADS` and `RUBY_MAX_CPU` settings are accepted. A server written in one thread answers a client written in another, which is what `Net::HTTP` talking to a `TCPServer` in the same program rests on
- **The locale names the default external encoding**: without `-E`, the text a program reads and writes is in the locale's encoding, US-ASCII under the C locale. A file name, `Dir.pwd`, `Dir.home` or an argument that US-ASCII cannot hold is tagged binary, as Ruby tags it
- **Hash keys are placed by `#hash`**: every key that answers `#hash`, an Array or a Range as well as an object with a `#hash` of its own, is placed by it, so a lookup compares a key with `eql?` only against the keys that share its hash
- **Installed gems**: with `GEM_HOME` and `GEM_PATH` naming a gem directory, such as MRI's, `require` activates the newest installed gem holding the file and its dependencies, and `gem "rake", "~> 13.3"` picks a version. RubyGems is MRI's own library, 4.0.20 as Ruby 4.0.7 ships it, with what metorex sets as the packager of its Ruby in `rubygems/defaults/operating_system.rb`. A gem with C extensions has them built against metorex's headers the first time it is activated: its extconf.rb runs under metorex in a directory under `~/.gem/metorex/extensions` named for a digest of those headers, with the build's output kept in `gem_make.out` beside it, and that directory goes on the load path ahead of the gem's `lib`. A gem metorex carries itself, such as prism, is used in place of an installed copy when another gem depends on it. `binding.break` stops the program at the debug gem's console
- **Bundler**: `require "bundler/setup"` reads the Gemfile and Gemfile.lock `BUNDLE_GEMFILE` names and puts the locked version of each gem on the load path, though a newer one is installed, with bundler from the gem path, such as bundler 4.0.19. `gem` of another version then raises Bundler's message
- **Method lookups are cached**: a class remembers the method it found for a name, and looks again only after a method is defined, removed or aliased, or a module is included or prepended, anywhere in the program
- **Socket reads wait for what they ask for**: `read(length)` on a TCP or Unix socket waits for that many bytes or for the other end to finish, and `read` with no length waits for the end, while `readpartial`, `recv` and `gets` answer what has arrived. `eof?` waits for data or the end and leaves what arrived to be read. `UNIXSocket#close_write` shuts the writing side down, so the other end reads to its end. A failed read or write is raised as the `Errno` the system names, such as `Errno::ECONNRESET`. A thread waiting to read a pipe or a socket ends its wait when it is killed
- **A deadlock is reported as MRI reports it**: when every thread waits on something only another thread can give it, through `Mutex#lock`, `Queue#pop`, a full `SizedQueue#push`, `ConditionVariable#wait`, `Thread.stop`, or `join` and `value` with no limit, the main thread raises `fatal` with `No live threads left. Deadlock?` and a listing of each thread. None of those waits has a time limit of its own, a thread killed while it waits stops waiting, and `Thread.stop` with no other thread raises ThreadError
- **A `do ... end` block belongs to the outermost command**, where a `{ }` block belongs to the call it sits next to, so `answer.reading_body socket, request.permitted? do ... end` hands its block to `reading_body`
- **A socket reads past a separator without losing what it took**: `TCPSocket#gets` reads until the separator it was given and keeps the rest for the next read
- **A name in a rescue clause is read where it was written**, so a method of a nested class reaches an exception class its enclosing module holds by its simple name. `__dir__` names the directory of the file the code was written in for the same reason
- **A Regexp subclass** carries the pattern it was built with and answers Regexp's methods through it, so `Class.new(Regexp).new("ab", Regexp::IGNORECASE)` is a pattern and an instance of that class at once. Methods the subclass writes answer ahead of Regexp's, `super` reaches the pattern, and `String#match` hands a match to the subclass's own `match`
- **Reading a socket without waiting**: `recv_nonblock` takes what has already arrived, fills a buffer handed to it, and reports that nothing is there yet either by raising or, with `exception: false`, by answering `:wait_readable`. A listener waiting in `accept` or `sysaccept` reports as asleep and is stopped where it waits
- **A copy holds its own characters**: `clone` and `dup` on a String give the copy its own text, so changing one leaves the other alone. A copy of a String subclass is one of the same class, carrying the instance variables and the characters, and `super` from such a subclass's `initialize` sets those characters the way String's own does
- **Chilled string literals**: a literal in a source with no `frozen_string_literal` magic comment is handed back with notice that a later release will freeze it. `+str` on one answers a mutable copy, `-str` and `freeze` answer the one frozen string every place writing the same literal shares, and changing it, taking its singleton class, or setting an instance variable on it warns once under `-W:deprecated`. A source that says `frozen_string_literal: false` outright gets literals with no such notice
- **Adjacent string literals**: two written next to each other are one string, which is how a long one is split across lines
- **Case mapping takes options**: `:ascii` leaves everything but the ASCII letters alone, `:turkic` maps the dotted and dotless `i` to their own pairs, `:lithuanian` maps the way the full rules do, and `:fold` on `downcase` turns a sharp s into two of them
- **`valid_encoding?` and `ascii_only?`**: a run of bytes tagged as text says whether those bytes spell characters there, and an encoding that spells even the ASCII letters in more than one byte holds nothing ASCII-only
- **`Integer#chr`**: a number under 128 names an ASCII character and one up to 255 names a byte of its own, and a wider one needs an encoding named: `0x3042.chr("UTF-8")`
- **StringIO reads the way IO does**: `read`, `sysread`, and `read_nonblock` fill a buffer handed to them and answer it, `gets` and `readline` set `$_` where `each_line` and `readlines` leave it alone, every line reader takes a separator that reads as a String, a limit that reads as an Integer, and `chomp:`, and `truncate` cuts the buffer itself
- **A Refinement names what it refines**: `refine` answers a Refinement, which reports `target` and `refined_class`
- **A collection carries instance variables**: `instance_variable_set` and `instance_variable_get` work on an Array, a Hash, or a Set, and a new collection built from one does not carry them
- **A lambda counts its arguments wherever it is called**: a lambda handed to `each` as a block is refused the same way a call with the wrong number of arguments is
- **A symbol's characters are ASCII when that is all they are**: `Symbol#encoding`, `#to_s`, `#id2name`, and `#name` answer US-ASCII for a name written in ASCII
- **`File.expand_path` reads a leading `~`**: it names the home directory, and `Kernel#system` takes `out:` and `err:` to say where the child's streams go, a leading Hash that adds to the child's environment or removes a name given nil, and a `[program, name]` pair that runs the program under the name its `$0` answers
- **A left shift is exact**: `1 << 200` keeps its value rather than dropping the high bits
- **The rest of the operator assignments**: `%=`, `**=`, `|=`, `&=`, `^=`, `<<=`, and `>>=` all parse
- **A percent list nests its delimiters**: `%w[a [ b ] c]` reads its brackets as words rather than ending at the first one
- **Setting an ENV name to nil removes it**: `ENV["X"] = nil` makes `ENV.include?("X")` false, the way Ruby's does. `ENV.delete`, `clear`, `delete_if` and the rest take a name out of the process environment too, so a child started afterwards does not see it
- **ARGV is always there**: a program reads or replaces it whether or not a caller handed one over
- **A method may be named for a keyword**: `def self.for` parses, and `URI.for` calls it
- **`Warning[:performance]` reports a replaced core method**: redefining one the interpreter would otherwise answer without a dispatch, such as `Integer#+` or `String#size`, prints `warning: Redefining 'Integer#+' disables interpreter and JIT optimizations`. The category is off by default
- **`srand` keeps the seed it was handed**: a seed wider than a machine word, such as `0x12345678901234567890`, is what the next `srand` answers back
- **`def` inside `instance_exec` defines on the receiver**: a method written at the top of the block belongs to that object alone, so another instance of the same class does not answer it
- **Interpolation carries bytes**: splicing a run of bytes into an ASCII literal, as in `"\x00 - #{[0xFF].pack('C')}"`, leaves the whole a run of bytes in that encoding rather than re-reading it as characters
- **A `tr`-style set must spell characters**: `delete`, `count`, `squeeze`, and `tr` refuse a set whose bytes are not characters in its own encoding, raising ArgumentError
- **One default may set several parameters**: `def foo(a = b = c = {})` gives all three the one value
- **Logger rotates on the calendar**: `Logger.new(path, 'daily')` moves the file aside when the day turns, naming it for the period it covered, and `shift_period_suffix:` says how that name is written
- **TracePoint sees compiled source**: a `:script_compiled` event fires for each `eval`, `TracePoint#eval_script` answers the source it compiled, and `TracePoint#instruction_sequence` the sequence it compiled to, with the methods, classes and blocks written in it as its children. A TracePoint switched on with `enable(target:)` aimed at an instruction sequence hears the code that sequence runs
- **TracePoint hears operators**: an operator or index a built-in value answers itself, such as `+`, `==`, `!`, or `[]`, fires `c_call` and `c_return`, and Integer's and Float's `-@` and Numeric's `+@` fire `call` and `return` from `<internal:numeric>`, the file MRI writes them in. A method written in Ruby fires its `return` while its frame is still on the stack, a method reached through an alias names the method as `method_id` and the alias as `callee_id`, and `disable` fires its `call` from `<internal:trace_point>`
- **An UnboundMethod cut from a core operator runs the original**: `Integer.instance_method(:+)` bound back to a number does the arithmetic even where the program has since replaced `Integer#+`
- **`retry` runs a begin body again**: written in a rescue body it goes back to the top of the begin, and written anywhere else it is refused as a SyntaxError while the source is read
- **A copy of a callable or a Binding is its own object**: `clone` and `dup` on a Method, an UnboundMethod, a Proc, or a Binding answer a separate object that carries the instance variables the original had, keeps the finalizers `clone` copies, and compares `==` to it where Ruby does. A Binding copy shares the cells it names and keeps its own list of them
- **`public_send` reaches only what a caller outside could write**: a private or protected name is refused with NoMethodError, including one declared on the object's own singleton class
- **A symbol proc is a lambda**: `:name.to_proc` counts its arguments strictly, reports `[[:req], [:rest]]` for `parameters`, and calls publicly
- **`Enumerator.new` names a size**: `Enumerator.new(100) { }` answers 100 for `size`, anything callable there is asked each time, and an uninitialized walk inspects as `#<Enumerator: uninitialized>`
- **`Enumerator::Generator` and `Enumerator::Chain`**: a Generator holds the block a walk runs and is Enumerable, and a Chain rewinds the walks it has run, last one first
- **IO reads a byte and a character at a time**: `getbyte`, `readbyte`, `getc`, `readchar`, `each_byte`, `each_char`, `each_codepoint`, and `ungetbyte` are all there, `advise` refuses what Ruby refuses, and `pread` reads at an offset without moving the cursor
- **A stream may be built over a descriptor**: `IO.sysopen` opens a name and hands back the number, `IO.new`, `IO.for_fd`, and `IO.open` stand over one, and the mode and the options say how the stream is used. A mode names the encodings after it (`"w:utf-8:iso-8859-1"`), `mode:`, `encoding:`, `external_encoding:`, `internal_encoding:`, `binmode:`, `textmode:`, `newline:`, and `autoclose:` say the same things as options, and asking for one thing twice is refused. A descriptor opened for reading cannot be written through
- **`Dir.for_fd`**: a Dir over a descriptor another Dir holds reads the same directory and has no path. A descriptor that is not valid raises `Errno::EBADF` and one that names no directory `Errno::ENOTDIR`, both named after `fdopendir`
- **`Numeric#coerce` compares the classes values answer to**: a value given a singleton class is not the same class as its sibling, so the pair is turned into Floats. A Numeric refuses a singleton method with MRI's message and takes the method away again
- **Lines are read up to a separator or a count**: `gets`, `readline`, `readlines`, `each_line`, and `IO.foreach` take a separator, a byte limit, and `chomp:`; an empty separator reads a paragraph, a `nil` separator reads what is left, and a count stops at a whole character rather than inside one. `gets` and `readline` set `$_` and `$.`, and walking the lines leaves them alone
- **A stream says where it stands**: `pos`, `pos=`, `seek`, `rewind`, `size`, and `truncate` work the descriptor, and an offset too large for the operating system to hold is refused
- **Reading straight through and reading what is there**: `sysread` refuses a stream whose lines have been read, `readpartial` and `read_nonblock` hand back as much as is waiting, `write_nonblock` says when a write would block, and `syswrite` says when a buffered write came first. Each of them clears the buffer it was handed before filling it, and the buffer keeps the encoding it was tagged with
- **`IO.copy_stream`, `IO.read`, and `IO.binread`**: the first writes everything one stream holds to another, and the other two take a count and an offset naming a run of bytes
- **`print` reads `$,` and `$\`**: the values are separated by one and followed by the other, and `print` with nothing to write writes the line last read. `puts` asks an object for `to_ary` before `to_s`, writes `[...]` for an array that reaches itself, writes nothing for an empty one, and ends its lines the way `newline:` asked for
- **ARGF walks the files it was handed**: `each_byte`, `each_char`, `each_codepoint`, and `each_line` read them as though they were one file, `read` fills a buffer handed to it, `readpartial` and `read_nonblock` read a file at a time, `set_encoding` says what the files are read as, and a stream over no names at all reads standard input. `-i` edits each file in place, keeping a backup under the extension written after the flag
- **A stream told not to sync holds what is written**: `sync = false` keeps writes until `flush` or `close`, which is where a broken pipe is reported
- **`Dir.chroot` and `Dir.mktmpdir`**: the first names a new root and reports the Errno a regular user gets, and the second makes a scratch directory of its own, dropping a separator in the name rather than following it
- **A multiple assignment may group its targets**: `(a, b), c = pair, held` takes the value apart a group at a time, however deeply the groups nest, a splat with no name after it keeps nothing, `m::A, m::B = :a, :b` names constants under a module, and a subscript naming more than one index is a target too
- **A condition may name what it reads**: `while line = stream.gets do ... end` reads the `do` as the start of the body rather than a block on the value, and `and` and `or` join the tests a modifier reads
- **`$!` names what is being handled**: a rescue clause inside a handler puts the outer exception back when it is done, so the handler that follows still names what it is handling
- **A curried callable is the library's own**: it names no arguments of its own, reporting `[[:rest]]` for `parameters`, and has no scope behind it, so `binding` is refused
- **A hash takes a callable as its default**: `default_proc=` asks anything else for `to_proc`, refuses what cannot be one, clears the value it had, and refuses a lambda that does not take the hash and the key
- **`unpack` and `unpack1` read from an offset**: `offset:` counts bytes, and an offset outside the string is refused
- **A pattern says which encoding it matches in**: `options` carries `FIXEDENCODING` for a pattern written with `u`, `e`, or `s`, `NOENCODING` for one written with `n`, and a Regexp made without a pattern is refused rather than treated as an empty one
- **A Struct writes its own `initialize`**: it is private, assigns the members it is given, and a struct class built from a subclass of Struct stands under that subclass so a method the subclass wrote is on the chain
- **`BigDecimal.mode` refuses what it was told to**: `EXCEPTION_NaN`, `EXCEPTION_INFINITY`, `EXCEPTION_OVERFLOW`, and `EXCEPTION_ZERODIVIDE` each raise FloatDomainError instead of answering the value, and an exponent past what BigDecimal carries stands for Infinity
- **A two by two matrix answers its eigenvalues**: `eigensystem` gives `eigenvalues` and `eigenvalue_matrix`, exact where the roots come out whole and a conjugate pair where they do not
- **`-l` chomps each line the loop reads**: it sets `$\` to `$/` and `$-l`, `BEGIN` runs before the loop so a separator it names is in force for the first record, and `-S` looks the script up in RUBYPATH and then in PATH
- **`eval` reports what it refuses as MRI does**: code `eval` cannot read raises a SyntaxError worded and laid out by prism, as MRI's is: `(eval at FILE:LINE):N: syntax error found`, each line at fault marked with `>`, up to two lines around it, and carets under the part at fault with prism's message. A local named `it` in the code around a block is what a bare `it` in the block reads, so `it = 0; proc { |x| it }` is no error
- **Programs Ruby's grammar refuses are refused**: a script, `-e` code or an `eval` string metorex could read but prism refuses, such as `1 2` or `def pair(left, left); end`, raises MRI's SyntaxError and never runs. A script's report opens with syntax_suggest's lines, and `-c` reports it as `ruby -c` does
- **Operators called by name raise what Ruby raises**: `public_send` of an operator a value has no rule for reaches the value's own methods, so a number asks the other side to coerce itself (`nil can't be coerced into Integer`), an Array or a String wants one of its own kind or an Integer (`no implicit conversion of nil into Array`), a failed comparison names both sides (`comparison of Float with :sym failed`), and a value without the operator, such as `=~` on an Integer or `<` on a Complex, raises NoMethodError. `!` answers for every object
- **Reports made before a program runs name the interpreter as it was started**: a missing program file, a switch RUBYOPT may not carry and the other refusals of the command line begin with the command as written, such as `./metorex` or `/usr/local/bin/metorex`, as MRI's do
- **RUBYLIB and RUBYOPT add to the load path**: the directories RUBYLIB names stand after everything a `-I` asked for, including a `-I` written inside RUBYOPT
- **`$0` names the program**: it is the path the command line spelled, or `-e` for code given there
- **`source_location` names where a method was written**: one built by `define_method`, one defined from another Method, and one read from a string handed to `eval` with a name of its own all report it
- **A call form reaches a method sharing a constant's name**: `URI(text)` calls the method where a bare `URI` names the module, and `Kernel::URI(text)` does the same through a module
- **String#scan**: walks every match of a pattern, answering the matched text or, when the pattern captures, the groups it took
- **String#gsub and #sub take a block**: the block is called with each match and answers what replaces it
- **String#gsub answers an Enumerator**: a pattern written on its own, with neither a replacement nor a block, walks the matches
- **A substitution negotiates the encoding**: the answer takes on the encoding of the first piece holding more than ASCII, and a piece that cannot be read alongside it raises Encoding::CompatibilityError
- **String#split takes a limit**: a positive one caps the fields, and trailing empty fields are dropped unless the limit is negative
- **String#each_line and #lines**: the separator defaults to what `$/` names, an empty one cuts the text into paragraphs, each running through the first two line ends in a row, `\n` or `\r\n`, `nil` leaves the string whole, `chomp:` takes the separator back off, and the separator is read through `to_str`
- **Range#reverse_each**: a range counting integers walks down from its end past any beginning, one walking with `succ` needs both ends, and a range that cannot be walked names the value it would have started from
- **Object#<=>**: two values with no order of their own compare as 0 when `==` calls them the same, which is what lets a range of them be built
- **Kernel#warn**: `uplevel:` names the caller of a `require` rather than the require itself, and `warn` called on the Warning module writes the message out rather than handing it back to `Warning.warn`
- **IBM437, ISO-8859-9 and macCyrillic**: `encode` converts between any of them and Unicode, byte for byte
- **EUC-JP**: `encode` carries text either way, `inspect` names a character the answer cannot show by the bytes it is spelled with, and a name written in it reaches `const_set` and friends as the characters it spells
- **ISO-2022-JP**: `Encoding::Converter` opens a run of two-byte characters with an escape and closes it on `finish`, and reads ISO-2022-JP back by following its escapes. A string in a dummy encoding inspects as its bytes
- **valid_encoding? reads the encoding's own shape**: a single-byte encoding takes any byte, one that pairs its bytes refuses a broken run, UTF-16 and UTF-32 refuse a lone surrogate half or a partial unit, and an encoding Ruby names without converting through it takes every run
- **Encoding names 91 encodings**, the single-byte IBM, Windows, mac and ISO-8859 families among them
- **Array#pack and String#unpack**: `U` writes a code point in UTF-8 and carries on past the range Unicode names, a count names how many bytes one uuencoded line holds, the answer is tagged UTF-8, US-ASCII or bytes by what the directives wrote, and bytes that spell no character are refused on the way back
- **RUBY_ENGINE reports `ruby`**: metorex runs Ruby, so a test suite reading the constant to decide which of an implementation's limits apply sees Ruby's own
- **$LOAD_PATH names where a library is looked for**: the `lib` directory under the prefix metorex was installed into, by version and then by platform, standing after everything `-I`, `RUBYOPT` and `RUBYLIB` named
- **Integer shifts read their count through `to_int`**, and a count too wide for a machine word shifts everything out or fills with the sign
- **Raising to a power**: an answer too wide to build is refused with `exponent is too large`, `0 ** -1` is a division by zero, a Rational exponent answers exactly where it names a whole number, and a negative base to a fractional power answers a Complex
- **Integer#chr**: a code is read as a code point in a Unicode encoding, as a byte in a single-byte one, and as the bytes themselves in one that spells characters with several, with EUC-JP, Shift_JIS, CESU-8 and the surrogate range each refused where the code names no character
- **Integer#div**: the pair a `coerce` answers is asked for `div` rather than for `/`
- **A range carries its own identity**: two ranges over the same values are not the same object, a range written with literal ends is built once, `dup` answers an unfrozen copy, and `clone` keeps the frozen state
- **Exception reports**: `detailed_message` names the class at the end of the first line of a multi-line message, a message of nil is no message, `full_message` hands its keywords on and falls back to the class name, and `Exception.to_tty?` says whether a report would be painted
- **raise takes three arguments**: a class, a message and a backtrace, with `cause:` alongside them
- **Time#<=>**: anything that is not a Time is asked to order itself against the time, and the answer is turned around
- **Method#parameters**: an anonymous `*`, `**` or `&` reports the mark it was written with, `**nil` reports `[:nokey]`, and a method the interpreter answers itself reports kinds without names
- **Proc subclasses**: `Class.new(Proc).new { }` stands for the block it was built with, its own `initialize` runs with the block already attached, and `Proc.new(&callable)` answers that same callable
- **A stream reads in the encoding it was told**: `readchar` and `getc` hand back whole characters of the external encoding, `read` carries them into the internal one, `File.open` takes `mode:`, and `set_encoding` remembers `invalid:` and `replace:` for every read. `set_encoding` refuses an internal encoding named alone, a name that is not ASCII or holds a NUL byte, an encoding that is not a superset of ASCII on a stream that reads without binmode, and a `newline:` decorator on a binmode stream. A stream reading BINARY carries its bytes into no internal encoding, and `newline: :crlf` and `:cr` convert every newline `write` and `puts` send
- **Source files in an encoding of their own**: a file whose magic comment names one is read through it, and a literal there carries the bytes that encoding spells it with
- **sprintf, format, printf and String#%**: every directive Ruby writes (`bBdiouxX`, `eEfgG`, `aA`, `cps`), the flags `-+ 0#`, width and precision written outright or as `*`, `N$` argument numbers, `%<name>` and `%{name}`, two's complement for negative numbers in a base of its own, and the conversions `to_int`, `to_i`, `Integer()`, `Float()`, `to_str`, `to_s` and `inspect`
- **String#to_i takes a base**: `"ff".to_i(16)` is 255
- **String#to_i reads a radix prefix and underscores**: `"+0d56".to_i` is 56, `"0xFAZ".to_i(0)` is 250, `"1_2_3asdf".to_i` is 123, and reading stops at the first character the base does not name. `Integer#to_s` takes the same base at any width
- **Array.try_convert and Hash.try_convert**: an object answering `to_ary` or `to_hash` converts, one answering neither is nil, and one answering with the wrong type raises TypeError
- **Array#sample takes a count and a generator**: `sample(n)` draws that many without replacement, and `random:` names an object asked for `rand`, refusing an answer outside the range with RangeError
- **Proc#>> and Proc#<<**: compose two callables, taking any object answering `call`, and the composition is a lambda when the callable reached first is one
- **`alias` reaches a native method**: `alias old_spaceship <=>` inside `class Integer` keeps the original reachable after a redefinition replaces it, the way `alias_method` already did
- **String#slice and #[] take one argument**: an Integer, a Range, another String, or a pattern
- **A quoted string may run across lines**: a newline inside `"..."` is content rather than the end of the literal
- **`next` ends a block with its value**: `values.map { |held| next 0 if held.nil?; held }` answers 0 for the nil ones
- **A symbol carries no space after its colon**: `flag ? held : "text"` is a ternary rather than a call passing `:"text"`
- **A method may be named for the right shift operator**: `def >>(count)` parses
- **`self.` reaches a private method**: `self.hidden` calls it the way a bare `hidden` does, rather than raising NoMethodError
- **The recursion guard counts per thread**: two virtual machines running side by side no longer add their nesting together
- **A splat spreads across a subscript**: `Held[*values]` passes the values one by one, the same way `Held.[](*values)` does
- **A block may follow a subscript**: `Ledger[3] { |index| index * 2 }` and `Ledger[3] do ... end` hand the block to `[]`, while a `do` after a `while` condition or a `for` collection opens the loop body
- **A semicolon ends a statement**: `counted = 0; until counted == 2 do counted += 1 end` runs the loop rather than reading `until` as a modifier on the assignment
- **A trailing comma destructures anyway**: `first, = pair` takes the first element the way a fuller target list would
- **An assignment reads as an operand**: `count.should == held += 1`, `4 < held += 1`, and `ready && held = 1` each read the assignment as the right operand, the way Ruby reads them
- **A method name is not a local variable**: `take ['a']` passes the array, where before the name being defined was mistaken for a variable and the brackets read as a subscript
- **A `when` reaches a constant through an included module**: `when Socket::SOCK_DGRAM` matches, where the constant lives on a module the namespace includes
- **`alias` renames a global**: `alias $ERROR_INFO $!` gives one global a second name that reads and writes the first
- **A splat spreads a range**: `[*"a".."z"]` reads the whole range and spreads the values it covers, where before the splat bound to the first value alone
- **A bitwise operator carries a line on**: `held = one |` followed by the next line reads as one expression, the way it does for `+`
- **A number can carry no singleton method**: `def one.name` on an Integer, a Float, or a Symbol raises TypeError, since the same object turns up wherever that value does
- **`Float#round` answers a whole number**: with no argument, or with a count of digits at or left of the point, it answers an Integer the way Ruby's does
- **Two hashes compare by what they hold**: a value that is an object of the program's own is asked with `==` rather than compared as data, which is what `Array#==` already did
- **A splat destructures a multiple assignment**: `first, *rest = values` gives `rest` everything the named targets leave, and the splat may sit anywhere in the list, so `*leading, last`, `head, *middle, tail`, and a bare `*everything` all read the same way. A splat with nothing left for it takes an empty Array
- **`super` from a constructor with nothing above it**: an `initialize` whose ancestors define none reaches Object's, which takes no arguments and answers nil
- **A method may be named for the power operator**: `def **(count)` parses
- **An unknown name reaches `method_missing`**: an attribute assignment with no setter behind it, and `send` of a name with no method behind it, both go there
- **Singleton definition on an expression**: `def (@matcher = Object.new).===(other)` assigns first and defines the method on what the variable holds
- **`__FILE__`**: names the file the code was written in, so a method or block from a required file reports that file rather than the one being run
- **MatchData**: a match answers one, with `[]` by index or group name, `captures`, `named_captures`, `names`, `values_at`, `begin`, `end`, `offset`, `match`, `match_length`, `pre_match`, `post_match`, `to_a`, `deconstruct`, `deconstruct_keys`, and equality by subject, pattern, and positions
- **Named groups**: once a pattern names any group the unnamed ones stop capturing, which is what puts `captures` and `inspect` on the named ones alone
- **Match globals**: `$~`, `$1` through `$9`, `` $` ``, `$'`, `$&`, and `Regexp.last_match` all read the last match, which a pattern that finds nothing clears, and `grep` without a block leaves alone
- **Regexp methods**: `match`, `match?`, `=~`, `===`, `source`, `options`, `casefold?`, `names`, `named_captures`, `to_s`, `inspect`, `==`, and `hash`, with `Regexp.new`, `Regexp.compile`, `Regexp.union`, and `Regexp.last_match`; a pattern reports Regexp as its class
- **String#match**: answers the MatchData for a Regexp or a String pattern, from an optional offset, and hands it to a block when one is given
- **Case equality**: a Range answers `===` the way `cover?` does for a single value, and a Regexp reaches an operand's characters through `to_str`
- **Range inclusion**: `cover?` given a Range asks whether every value of it falls inside. `include?` compares against the ends for numbers and times, walks a String range the way `String#upto` does, and raises TypeError for a range missing an end that it would have to walk. `step` refuses to walk from a missing beginning
- **Set**: keeps its elements in the order they were added and holds any value with a stable rendering, so a Symbol, an Array, or nil is an element like a number is
- **Set algebra**: `union`, `difference`, `intersection`, and their `|`, `+`, `-`, `&`, `^` spellings, with `subset?`, `superset?`, `proper_subset?`, `proper_superset?`, `disjoint?`, and `intersect?`; an operand may be any Enumerable
- **Set in place**: `<<`, `add?`, `merge`, `subtract`, `replace`, `clear`, `delete_if`, `keep_if`, `select!`, `reject!`, and `map!`, plus `join`, `hash`, `dup`, and an `each` that answers an Enumerator without a block
- **Orphaned return**: `Proc.new { return }` called after its defining method has returned raises a LocalJumpError carrying `exit_value` and `reason`
- **Set membership**: `include?`, `member?`, and `===` ask the element for its `hash` and `eql?`, so two objects that agree on both are the same element; an Integer and a Float of the same value are not
- **Set during iteration**: adding to, merging into, replacing, or clearing a Set while a walk over it is open raises a RuntimeError rather than changing what the walk is reading
- **allocate**: `Array.allocate` and `Hash.allocate` answer an empty one, `Proc.allocate` raises TypeError, and `MatchData.allocate` does not exist
- **Proc method table**: `Proc` reports the methods it answers, so `public_instance_methods` lists `call`, `eql?`, `==`, `arity`, `curry`, and the rest
- **Array#join**: a Symbol element contributes the name it is spelled with, and a nil separator joins with nothing between
- **Hash#replace and #transform_values!**: `replace` answers the receiver even when the new contents are written as keyword arguments, and a `break` out of `transform_values!` keeps what it changed
- **Range#size**: reports a count only when the range steps from an Integer, answers nil for one that steps from a String or Symbol, and raises TypeError for one it cannot step from at all
- **start_with? with a Regexp**: records the match, so `$~`, `$1`, and `Regexp.last_match` read it afterwards
- **Repeated group names**: a pattern may write the same name on more than one group, and the match reports the farthest one under it that matched; a name spelled with multi-byte characters counts in characters
- **Regexp introspection**: `names` lists each written name once and `named_captures` reports the group numbers each was written on, both on the pattern and on the match
- **Regexp equality**: two patterns differing only in the `/n` encoding option are the same pattern, and hash the same
- **sub and gsub record the match**: `$~`, `$1`, and `Regexp.last_match` read it afterwards, whether the pattern was a Regexp or a String
- **A method called through a Method object keeps `$~` to itself**: it neither reads the caller's match nor leaves one behind
- **Set comparison and identity**: `<=>` reports containment, `delete` answers the set while `delete?` answers nil for an element that was not there, and two names for one native method answer the same Method object
- **Set subclasses**: `class Bag < Set` instances carry their elements and answer Set's methods, which is what lets `to_set(Bag)` build one
- **Set rendering**: a Set inspects as `Set[1, 2]`, and one that reaches itself as `Set[...]`
- **String#to_str, #codepoints, and #each_codepoint**: the implicit conversion and the code point of each character
- **String#to_r**: an underscore between digits is a separator rather than part of the number
- **Enumerable#sum**: an infinity stays one rather than turning into a NaN, since there is no rounding to compensate for
- **Lambda parameters**: a lambda written without parentheses takes defaults and keyword parameters the same as one written with them, so `-> name = "world" { }` and `-> *rest, tag: :none { }` both read as declared
- **Locals known while reading**: the parser declares a local where it reads the assignment, parameter, block-local, rescue or loop variable, named capture or pattern variable that binds it, so `name [1]` indexes a local bound before it and calls a method otherwise. Code inside `#{}` and code given to `eval`, `instance_eval`, `class_eval` and `Binding#eval` is read with the locals of the scope it runs in
- **A range that ends its line**: `x = 1..` followed by a line that starts a value is `1..` that value, as MRI reads it, and an endless range closed by `)` or `]` on the next line stays endless
- **Syntax error messages**: a program the Ripper grammar refuses, read by `Ripper` or `RubyVM::AbstractSyntaxTree`, is replayed through the tables of MRI's parser, so the error names the token MRI's parser stops at, as `unexpected backtick literal` or `unexpected instance variable`, and lists the tokens it would take when there are at most four. A program the grammar reads but MRI's parser refuses, such as `foo(a: 1, 2)`, is refused the same way. Each error goes through the Ripper event MRI uses for it: `parse_error` for a misplaced `return`, `yield` or jump, a definition in a method body, a repeated parameter or pattern variable, an alternative pattern that captures, and a jump used as a value, `assign_error` for a dynamic constant or an assignment to `self`, `nil` or `$1`, `alias_error` and `class_name_error`, and `compile_error` for the scanner's errors and for a numbered parameter or `it` written where it cannot be, with a second line naming the earlier use. `RubyVM::AbstractSyntaxTree.parse` raises SyntaxError listing every error with its line quoted. The scanner reads on past an escape or a number MRI's lexer refuses, such as `"\xg"`, `"\u{zz}"`, `0x` and `@1`, reporting it at the place it names, so a later syntax error is reported as well
- **Inspecting text in other encodings**: a String, Symbol or Regexp in an encoding other than UTF-8 is inspected as MRI does, writing a character past ASCII by its byte, as `"caf\xE9"` in ISO-8859-1, or by its bytes together, as `:"\x{82A0}"` in Shift_JIS. A Regexp keeps the bytes of the text it was made from as its source, and UTF-8 that spells nothing is refused with `invalid multibyte character`
- **Encoding names**: the encodings, their constants and their aliases are MRI's, generated by `scripts/generate_encoding_table.rb`, so `Encoding.list`, `Encoding#names`, `Encoding.aliases` and `Encoding.name_list` answer as MRI's do, and `Encoding.find` takes any alias, such as `646`, `eucJP` or `Big5-HKSCS:2008`
- **Hash keys that read as values**: a String key that reads as a number, a boolean or nil, such as `"646"`, is a key of its own beside `646`, since the two are not `eql?`
- **Conversion errors**: the error `String#encode` raises names the step of the conversion it stopped at, as `U+3042 to ISO-8859-1 in conversion from Shift_JIS to UTF-8 to ISO-8859-1`, and carries that step's `source_encoding` and `destination_encoding` with their names, the `error_char` an UndefinedConversionError could not spell, and the `error_bytes`, `readagain_bytes` and `incomplete_input?` of an InvalidByteSequenceError, which Shift_JIS and EUC-JP input raises as UTF-8 input does
- **Literal conditions**: a regexp or string literal written as a condition, and an integer at either end of a flip-flop, earn MRI's warning while the code is read, whatever the verbosity: `warn.rb:2: warning: regex literal in condition` for a file, under the eval's name for code given to `eval`, and under `(none)` for `RubyVM::AbstractSyntaxTree.parse`, which a Ripper subclass sees through `warn`. `-W0` turns them off
- **Lambda bodies**: a `->` lambda takes its parameters before its body opens, so `-> { |x| x }` and `-> do |x| x end` are refused with `unexpected '|'` as MRI refuses them, while `lambda { |x| x }` takes them in the block
- **Calls written with parentheses**: `name()` calls the method `name` even where a local of that name exists, and raises NoMethodError when there is no such method. A method written at the top level runs with `main` as `self`, called bare, with parentheses or through `method(:name)`
- **Lambda arity**: a lambda takes its arguments the way a method does, refusing a call that gives it the wrong number
- **`**nil`**: a method declares that it takes no keyword arguments at all
- **A method as a block**: `&some_method` hands the method over as the block, keeping its own arity, so `hash.each(&recorder.method(:record))` gets one `[key, value]` pair per entry
- **match with a block**: `String#match`, `Symbol#match`, and `Regexp#match` hand the MatchData to a block and answer what the block answers; `match?` takes the offset to start at
- **Symbol has no constructor**: `Symbol.new` and `Symbol.allocate` are undefined, the way Ruby leaves them
- **Nested Sets**: a Set holding Sets compares by what those hold, so the order they were added in does not matter
- **Kernel#trap**: Kernel's name for `Signal.trap`, listed among its private instance methods
- **String bytes**: `bytesize`, `bytes`, `each_byte`, and `getbyte` read a String as the bytes it is made of, alongside `chr`, `ascii_only?`, and `valid_encoding?`
- **String#hex and #oct**: read a number off the front of the text, `hex` honoring only the `0x` prefix and `oct` honoring `0x`, `0b`, `0o`, and `0d`, with an underscore between digits treated as a separator
- **Kernel Conversion Functions**: `Hash()`, `Integer()`, `Rational()`, and `String()`, with coercion through `to_hash` / `to_int` / `to_i` / `to_r` / `to_s` and `exception: false`
- **Rational Numbers**: the `5r` literal suffix, arithmetic and ordering against Integer, Float, and Rational, `to_r` on String, Integer, and Float, and results always in lowest terms and frozen
- **Numeric Literals**: decimal, `0x`/`0b`/`0o`/`0d` radix prefixes, bare-leading-zero octal, scientific notation, `_` digit separators, and the `r` rational suffix
- **String Subclasses**: `class Name < String` instances carry their characters and answer String's methods, comparing equal to a String with the same content
- **Runtime Class System**: Classes support inheritance, runtime method definition, instance variables, and class-level state
- **File Loading**: `require_relative` with extension auto-detection, deduplication, circular dependency handling, and shared scope

### Meta-Programming (Core Innovation)
- **Code-as-Object**: AST nodes are first-class objects manipulable at runtime
- **Runtime Method Definition**: `define_method` for dynamic behavior
- **Method Missing Hook**: `method_missing` intercepts calls to undefined methods with method name and arguments
- **Runtime Class Modification**: `remove_method`, `undef_method`, `alias_method`, `module_function`, `class_variable_set`, `class_variable_get`, `class_variable_defined?`, `class_variables` for dynamic class/module manipulation
- **Reflection and Introspection**: `class`, `instance_of?`, `is_a?`, `itself`, `respond_to?`, `methods`, `send`, `instance_variables`, `instance_variable_get`, `instance_variable_set`, `local_variables`, `__method__`, `__callee__`
- **AST Manipulation**: `eval` for runtime code execution, `parse` for AST inspection, runtime code generation via string evaluation
- **Block Execution**: Blocks are objects with `.call()` method; trailing `do...end` and `{...}` blocks captured implicitly via `&block` parameter with `block_given?` support
- **DSL Construction**: Build domain-specific languages naturally

### Standard Library
- **Networking**: HTTP client/server, WebSocket, TCP/UDP, TLS/SSL
- **Serialization**: JSON, XML, YAML, CSV, MessagePack
- **Cryptography**: Hashing, encryption, secure random, certificates
- **Concurrency**: OS threads, fibers, async/await, channels, atomics
- **Advanced Math**: Complex numbers, arbitrary precision, statistics

### Developer Experience
- **Documentation System**: Doc comments with automatic HTML generation
- **Debugger**: Full debugging with breakpoints and inspection
- **LSP Support**: Language Server Protocol for IDE integration
- **Build System**: Incremental compilation, profiles, and optimization
- **Linter & Formatter**: Code quality and style enforcement

## Core Philosophy and Identity

| Element                 | Description                                                                                                                       |
| :---------------------- | :-------------------------------------------------------------------------------------------------------------------------------- |
| **Foundation Language** | **Rust** (for VM safety and speed)                                                                                                |
| **Syntax Heritage**     | **Ruby** (block structure, optional parentheses)                                                                                  |
| **Primary Paradigms**   | **Full Object-Oriented**, **Imperative**, **Functional** (with ADTs and immutable structures)                                     |
| **Key Differentiator**  | **Code-as-Object (The Meta Core)**: The Abstract Syntax Tree (AST) is directly exposed as native, manipulable objects at runtime. |
| **Typing**              | **Dynamic by default**, with **optional static typing** and gradual type inference                                                |
| **Performance**         | Compiles to bytecode that an interpreter runs. [ROADMAP.md](ROADMAP.md) works on speed next                                       |

## Syntax Overview

METOREX syntax prioritizes readability while minimizing keystrokes, combining elements from Ruby.

### Basic Syntax

See [examples/basic_syntax.rb](examples/oop/basic_syntax.rb)

### Exception Handling

See [examples/exception_handling.rb](examples/advanced/exception_handling.rb)

### Pattern Matching

See [examples/pattern_matching.rb](examples/advanced/pattern_matching.rb)

### Traits (Interfaces)

See [examples/traits.rb](examples/advanced/traits.rb)

### Optional Type Annotations

See [examples/type_annotations.rb](examples/advanced/type_annotations.rb)

## Meta-Programming: The Core Innovation

METOREX exposes the program's structure as native objects, eliminating the need for external `eval` functions.

### Code-as-Object Hierarchy

The parser converts source code into an in-memory graph of objects, defined in the Rust core and exposed in Metorex.

| Metorex Class Name   | Role                                                                                                              | Example of Manipulation                    |
| :------------------- | :---------------------------------------------------------------------------------------------------------------- | :----------------------------------------- |
| **`BlockStatement`** | **The Core Meta-Object.** Represents a sequence of code lines (a method body, loop body, or implicit code block). | `block.call` to execute the code.          |
| **`Assignment`**     | Represents `x = 10`.                                                                                              | `.target` to see the variable name.        |
| **`MethodCall`**     | Represents a function/method invocation.                                                                          | `.receiver` and `.args` for code analysis. |

### Implicit Block Capture and Execution

Methods can accept code blocks as objects.

See [examples/metaprogramming/blocks_as_objects.rb](examples/metaprogramming/blocks_as_objects.rb)

### Dynamic Method Definition

See [examples/dynamic_method_definition.rb](examples/advanced/dynamic_method_definition.rb)

### Building DSLs

See [examples/dsl_example.rb](examples/advanced/dsl_example.rb)

## Architecture

### Execution

Source is read by the lexer in `src/lexer` and the parser in `src/parser` into the syntax tree in `src/ast`. `src/bytecode/compiler.rs` compiles each body of the tree to instructions the first time it runs, and `VirtualMachine` (`src/vm/core.rs`, with the loop in `src/vm/bytecode_runner.rs`) runs them.

## Design Principles

| Principle             | Implementation                                                                         |
| :-------------------- | :------------------------------------------------------------------------------------- |
| **Syntax Simplicity** | Non-whitespace sensitive with mandatory `end` blocks. No colons, optional parentheses. |
| **OO Purity**         | Everything is an object rooted in `Object` class. No standalone functions.             |
| **Meta-First**        | AST is always accessible as first-class objects. Code can inspect and modify itself.   |
| **Gradual Typing**    | Dynamic by default, optional static types for performance. Best of both worlds.        |
| **Performance**       | Compiles to bytecode that an interpreter runs, with speed work next per ROADMAP.md.    |
| **Safety**            | Exception handling, memory safety from Rust, optional sandboxing.                      |
| **Concurrency**       | Multiple models: fibers, async/await, OS threads, channels. Choose the right tool.     |
| **Productivity**      | Built-in testing, documentation, linting, formatting. Everything you need included.    |

## Standard Library Highlights

### Networking

See [examples/networking.rb](examples/advanced/networking.rb)

### Concurrency

See [examples/concurrency.rb](examples/advanced/concurrency.rb)

### Serialization

See [examples/serialization.rb](examples/advanced/serialization.rb)

## Bundler

`metorex bundle` runs Bundler 4.0.20, as Ruby 4.0.7 ships it, which `scripts/vendor_bundler.rb` copies in from the release's source tree. `metorex bundle install` installs a Gemfile's gems from a gem server, and `metorex bundle exec` runs a script with them on the load path. `tests/_examples/bundler` holds a gem, the compact index a server reads it from, and a server for it, which the test of the two commands starts.

## Built-in Spec Runner

`metorex spec` runs a suite written for RSpec 3.13 with Rex, the spec framework metorex carries. The module is named `RSpec`, as RSpec's own is, and `Rex` is its other name, so what a suite prints names its classes as RSpec does. Its matchers, expectations, and doubles are RSpec's own rspec-expectations, rspec-mocks, and rspec-support, with diff-lcs, which `scripts/vendor_rspec.rb` copies in from the `gems/rspec` submodule, and its runner is Rex's own. `require "rspec"` loads Rex as well, and `RSpec` names it, so a suite runs with no gems installed. A script run with `metorex` that names `RSpec` loads Rex the first time it does, and runs its examples as it ends, as `require "rspec/autorun"` does. It reads `.rspec`, `.rspec-local`, and `SPEC_OPTS`, loads `spec_helper`, and prints what RSpec prints: the expected output of each fixture under `tests/_examples/rex` is RSpec's own output for the same suite.

It takes the options `metorex spec --help` lists, which are RSpec's, and these of its own:

- `--jobs N` runs the top-level groups in N forked worker processes, and `--jobs auto` runs one per processor. The parent hands each group's events to the formatters in the order a serial run would, so the output reads as it does with one job
- `--verify-parallel` runs the suite again in one process with the same seed after a parallel run, and warns about each example whose outcome changed
- `--no-fast-paths`, or `config.fast_paths = false`, runs the Ruby versions of the parts of Rex that also have native versions, so a difference between the two can be checked
- `--coverage` reports the lines and branches of the loaded files that ran, counted by the interpreter rather than a coverage gem. `--coverage-format` picks `summary`, `lcov`, or `json`, `--coverage-dir` says where the files go, `--coverage-threshold PERCENT` fails a run below it, and `--coverage-exclude PATTERN` leaves files out. Parallel workers merge their counts
- `config.source_expressions = true` quotes the expression handed to `expect` in a failure, `config.failure_context_lines = N` prints N lines around the failing line, and `config.diff_line_limit = N` leaves out a diff longer than N lines

### Compatibility

`scripts/run_rspec_suite.sh` runs the RSpec libraries' own suites from the `gems/rspec` submodule, at the RSpec 3.13.6 release Rex follows, a spec file to a process. It runs them under Rex, and with `ENGINE=mri` under MRI's own `rspec`, with the gems RSpec's Gemfile names, which `scripts/install_rspec_suite_gems.sh` installs. `RECORD=1` stores each file's counts in `scripts/rspec_suite_results.tsv`, `ruby scripts/suite_results.rb differences` lists the files whose Metorex counts differ from MRI's, `CHECK=1` makes a Metorex run fail when there are any, which CI runs, and `ruby scripts/suite_results.rb table` prints this table:

| Suite | Passing under MRI | Passing under Metorex | Examples | Pending |
| :-- | --: | --: | --: | --: |
| rspec-core | 2239 | 2239 | 2258 | 19 |
| rspec-expectations | 2438 | 2438 | 2455 | 17 |
| rspec-mocks | 1794 | 1794 | 1806 | 11 |

Under MRI 4.0.7 one example fails, in rspec-mocks' `receive_spec.rb`, which stubs `respond_to?` and recurses through the `Kernel#inspect` of Ruby 4.0. The script starts Metorex through a link named `ruby`, as rspec-core's `bisect/shell_command_spec.rb` expects the path of the interpreter a bisect run starts to contain `ruby`. diff-lcs's suite passes all 322 of its examples, with documentation output identical to RSpec's byte for byte.

`scripts/run_gem_suites.sh` runs the suites of the bundled gems Ruby 4.0.7 ships, from submodules under `gems/` at the releases Ruby 4.0.7 ships, a test file to a process, with the arguments each gem's own Rakefile gives a file. It runs them under Metorex, and with `ENGINE=mri` under MRI, against each gem's own library, with the assertions Ruby's repository keeps in `tool/lib`, which `scripts/vendor_gems.rb` copies into `scripts/gem_suite/lib`. `RECORD=1` stores each file's counts in `scripts/gem_suite_results.tsv`. `CHECK=1` compares a Metorex run with the Metorex counts stored there, fails when a file passes fewer examples than recorded, and prints the row to store for a file that passes more, which is what Jenkins's `gems` stage runs. `SUITE_RESULTS=scripts/gem_suite_results.tsv ruby scripts/suite_results.rb table` prints this table, where an example is a test and an omission counts as pending:

| Suite | Passing under MRI | Passing under Metorex | Examples | Pending |
| :-- | --: | --: | --: | --: |
| minitest | 418 | 418 | 422 | 4 |
| net-imap | 1691 | 1691 | 1691 | 0 |
| net-pop | 7 | 7 | 7 | 0 |
| net-smtp | 95 | 95 | 95 | 0 |
| power_assert | 102 | 102 | 102 | 0 |
| rake | 598 | 598 | 598 | 0 |
| rexml | 730 | 730 | 730 | 0 |
| rss | 362 | 362 | 362 | 0 |
| test-unit | 443 | 443 | 452 | 6 |

Every file's counts match MRI's. net-imap's `test_stringprep_tables.rb` runs under neither, since it reads `rfcs/rfc3454-stringprep_tables.json`, which net-imap's rake tasks build from RFC 3454 downloaded from the network.

### Speed

`ruby benchmarks/compare.rb`, run with a release build, times Metorex, with its JIT and with `METOREX_JIT=0`, beside CRuby with and without `--yjit` and prints this table. `--without-no-jit` leaves out the `METOREX_JIT=0` column, for a build from before the JIT. It covers startup, the scripts in `benchmarks/workloads` and `benchmarks/memory`, every benchmark in the [ruby-bench](https://github.com/ruby/ruby-bench) suite (the `benchmarks/ruby-bench` submodule) in its headline, other, and micro categories that needs no gems, and the Rex suite. Startup, workload, and Rex suite times are the fastest of three whole runs. A ruby-bench time is the mean of one iteration after the warmup ones, under ruby-bench's own harness: CRuby runs the harness's defaults, as ruby-bench's runner does, and Metorex runs one warmup iteration and two timed ones. Peak memory is the largest resident set size of the run.

On arm64, Metorex 0.1.0 built with rustc 1.96.0, and ruby 4.0.7:

| Benchmark | CRuby | CRuby --yjit | Metorex | Metorex, no JIT | Metorex ÷ CRuby | Metorex ÷ YJIT | Metorex ÷ no JIT | CRuby peak | YJIT peak | Metorex peak | Metorex, no JIT peak |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: |
| empty | 26.0 ms | 26.0 ms | 20.3 ms | 18.9 ms | 0.8× | 0.8× | 1.07 | 19 MB | 19 MB | 17 MB | 17 MB |
| block_calls | 34.8 ms | 33.5 ms | 192.5 ms | 205.9 ms | 5.5× | 5.7× | 0.93 | 19 MB | 19 MB | 17 MB | 17 MB |
| hash_access | 48.2 ms | 46.0 ms | 177.9 ms | 182.4 ms | 3.7× | 3.9× | 0.98 | 27 MB | 29 MB | 30 MB | 30 MB |
| method_calls | 32.0 ms | 29.5 ms | 137.3 ms | 144.5 ms | 4.3× | 4.7× | 0.95 | 19 MB | 19 MB | 17 MB | 18 MB |
| object_allocation | 37.7 ms | 32.4 ms | 362.4 ms | 447.9 ms | 9.6× | 11.2× | 0.81 | 19 MB | 20 MB | 17 MB | 17 MB |
| string_building | 39.5 ms | 37.9 ms | 116.4 ms | 147.4 ms | 2.9× | 3.1× | 0.79 | 25 MB | 26 MB | 28 MB | 28 MB |
| cycles | 49.2 ms | 45.7 ms | 392.0 ms | 387.7 ms | 8.0× | 8.6× | 1.01 | 19 MB | 20 MB | 19 MB | 18 MB |
| no_cycles | 33.2 ms | 31.8 ms | 276.8 ms | 285.3 ms | 8.3× | 8.7× | 0.97 | 19 MB | 20 MB | 17 MB | 17 MB |
| 30k_ifelse | 298.7 ms | 75.7 ms | 4.08 s | 5.12 s | 13.6× | 53.9× | 0.80 | 115 MB | 152 MB | 155 MB | 140 MB |
| 30k_methods | 253.9 ms | 47.5 ms | 2.14 s | 2.29 s | 8.4× | 45.0× | 0.93 | 66 MB | 75 MB | 97 MB | 92 MB |
| 30k_variables | 416.3 ms | 1.14 s | 14.15 s | 14.68 s | 34.0× | 12.4× | 0.96 | 41 MB | 171 MB | 56 MB | 48 MB |
| attr_accessor | 80.2 ms | 11.7 ms | 504.4 ms | 3.80 s | 6.3× | 43.3× | 0.13 | 20 MB | 21 MB | 21 MB | 21 MB |
| binarytrees | 152.3 ms | 70.7 ms | 4.63 s | 5.51 s | 30.4× | 65.5× | 0.84 | 27 MB | 28 MB | 32 MB | 31 MB |
| block_methods | 1.19 s | 274.9 ms | 23.29 s | 24.68 s | 19.5× | 84.7× | 0.94 | 87 MB | 99 MB | 23 MB | 22 MB |
| blurhash | 164.6 ms | 61.9 ms | 3.04 s | 3.47 s | 18.5× | 49.1× | 0.88 | 21 MB | 22 MB | 24 MB | 24 MB |
| cfunc_itself | 39.5 ms | 9.7 ms | 733.8 ms | 734.8 ms | 18.6× | 75.9× | 1.00 | 20 MB | 21 MB | 21 MB | 20 MB |
| etanni | 141.0 ms | 121.7 ms | 1.66 s | 1.89 s | 11.8× | 13.7× | 0.88 | 24 MB | 25 MB | 28 MB | 29 MB |
| fannkuchredux | 230.6 ms | 83.6 ms | 2.39 s | 2.95 s | 10.4× | 28.6× | 0.81 | 20 MB | 21 MB | 21 MB | 21 MB |
| fib | 98.3 ms | 16.3 ms | 3.21 s | 4.37 s | 32.6× | 196.3× | 0.74 | 20 MB | 21 MB | 21 MB | 21 MB |
| getivar | 55.4 ms | 8.5 ms | 251.3 ms | 525.6 ms | 4.5× | 29.6× | 0.48 | 20 MB | 21 MB | 21 MB | 21 MB |
| getivar-module | 110.7 ms | 35.6 ms | 1.45 s | 1.92 s | 13.1× | 40.8× | 0.76 | 20 MB | 21 MB | 21 MB | 21 MB |
| keyword_args | 84.8 ms | 15.2 ms | 7.76 s | 8.47 s | 91.5× | 511.4× | 0.92 | 20 MB | 21 MB | 21 MB | 21 MB |
| knucleotide | 15.7 ms | 15.7 ms | 114.1 ms | 111.3 ms | 7.3× | 7.3× | 1.03 | 22 MB | 23 MB | 21 MB | 21 MB |
| loops-times | 461.9 ms | 137.0 ms | 7.06 s | 7.59 s | 15.3× | 51.5× | 0.93 | 22 MB | 27 MB | 21 MB | 21 MB |
| matmul | 239.8 ms | 97.6 ms | 2.46 s | 3.08 s | 10.3× | 25.2× | 0.80 | 26 MB | 27 MB | 23 MB | 23 MB |
| nbody | 42.6 ms | 17.8 ms | 882.9 ms | 1.17 s | 20.7× | 49.5× | 0.75 | 20 MB | 21 MB | 21 MB | 21 MB |
| nqueens | 107.3 ms | 24.8 ms | 1.16 s | 1.70 s | 10.9× | 46.9× | 0.68 | 20 MB | 22 MB | 21 MB | 21 MB |
| object-new | 34.5 ms | 15.2 ms | 911.1 ms | 965.0 ms | 26.4× | 59.9× | 0.94 | 20 MB | 21 MB | 21 MB | 21 MB |
| object-new-initialize | 53.8 ms | 17.9 ms | 1.70 s | 2.04 s | 31.6× | 95.3× | 0.83 | 20 MB | 21 MB | 21 MB | 21 MB |
| object-new-no-escape | 98.6 ms | 34.4 ms | 3.32 s | 4.31 s | 33.6× | 96.4× | 0.77 | 20 MB | 21 MB | 21 MB | 21 MB |
| protoboeuf | 70.7 ms | 14.2 ms | 1.82 s | 2.31 s | 25.8× | 128.2× | 0.79 | 43 MB | 48 MB | 60 MB | 55 MB |
| protoboeuf-encode | 61.3 ms | 15.1 ms | 20.14 s | 20.77 s | 328.5× | 1335.8× | 0.97 | 880 MB | 3361 MB | 117 MB | 113 MB |
| respond_to | 94.2 ms | 6.8 ms | 3.27 s | 3.41 s | 34.7× | 479.0× | 0.96 | 20 MB | 21 MB | 21 MB | 20 MB |
| ruby-json | 102.5 ms | 82.6 ms | 13.97 s | 14.14 s | 136.3× | 169.1× | 0.99 | 21 MB | 21 MB | 22 MB | 22 MB |
| ruby-xor | 55.9 ms | 11.3 ms | 982.7 ms | 1.64 s | 17.6× | 86.8× | 0.60 | 21 MB | 22 MB | 21 MB | 21 MB |
| send_bmethod | 93.5 ms | 4.3 ms | 4.25 s | 4.92 s | 45.4× | 983.1× | 0.86 | 20 MB | 21 MB | 21 MB | 21 MB |
| send_cfunc_block | 117.0 ms | 39.8 ms | 2.95 s | 3.52 s | 25.2× | 74.0× | 0.84 | 20 MB | 21 MB | 21 MB | 21 MB |
| send_rubyfunc_block | 59.7 ms | 11.5 ms | 2.17 s | 2.17 s | 36.4× | 189.5× | 1.00 | 20 MB | 21 MB | 21 MB | 21 MB |
| send_rubyfunc_inline | 371.2 ms | 49.8 ms | 6.03 s | 8.65 s | 16.2× | 121.1× | 0.70 | 20 MB | 21 MB | 21 MB | 21 MB |
| setivar | 31.4 ms | 4.6 ms | 188.0 ms | 301.6 ms | 6.0× | 41.0× | 0.62 | 20 MB | 21 MB | 21 MB | 21 MB |
| setivar_object | 48.9 ms | 16.8 ms | 425.1 ms | 474.5 ms | 8.7× | 25.3× | 0.90 | 20 MB | 21 MB | 21 MB | 20 MB |
| setivar_young | 49.1 ms | 19.1 ms | 411.2 ms | 448.2 ms | 8.4× | 21.5× | 0.92 | 20 MB | 21 MB | 21 MB | 21 MB |
| splay | 45.9 ms | 30.5 ms | 977.5 ms | 1.55 s | 21.3× | 32.0× | 0.63 | 921 MB | 1020 MB | 280 MB | 280 MB |
| str_concat | 39.6 ms | 12.1 ms | 496.0 ms | 817.4 ms | 12.5× | 40.9× | 0.61 | 79 MB | 91 MB | 21 MB | 21 MB |
| struct-new-no-escape | 58.6 ms | 34.2 ms | 1.62 s | 2.37 s | 27.6× | 47.4× | 0.68 | 20 MB | 20 MB | 21 MB | 21 MB |
| structaref | 86.6 ms | 11.0 ms | 547.7 ms | 4.23 s | 6.3× | 49.7× | 0.13 | 20 MB | 21 MB | 21 MB | 21 MB |
| structaset | 67.2 ms | 28.7 ms | 462.3 ms | 4.26 s | 6.9× | 16.1× | 0.11 | 20 MB | 21 MB | 21 MB | 21 MB |
| sudoku | 272.6 ms | 64.3 ms | 2.95 s | 3.75 s | 10.8× | 46.0× | 0.79 | 20 MB | 21 MB | 21 MB | 21 MB |
| throw | 8.9 ms | 6.8 ms | 274.5 ms | 300.5 ms | 30.9× | 40.1× | 0.91 | 20 MB | 21 MB | 21 MB | 21 MB |
| rex_suite | 278.8 ms | 342.1 ms | 3.94 s | 3.95 s | 14.1× | 11.5× | 1.00 | 53 MB | 61 MB | 77 MB | 75 MB |

#### The bytecode baseline

The same table for commit `afdab34a`, where every body first ran as compiled instructions and the tree walker was removed, before the inline caches, shapes, specialized instructions, and the rest of the interpreter's speed work. Recorded on 2026-10-18 on the machine above, with `ruby benchmarks/compare.rb --metorex=PATH --without-no-jit` and that commit's release build:

| Benchmark | CRuby | CRuby --yjit | Metorex | Metorex ÷ CRuby | Metorex ÷ YJIT | CRuby peak | YJIT peak | Metorex peak |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: |
| empty | 25.5 ms | 24.7 ms | 55.6 ms | 2.2× | 2.3× | 19 MB | 19 MB | 53 MB |
| block_calls | 32.7 ms | 33.1 ms | 654.7 ms | 20.0× | 19.8× | 19 MB | 19 MB | 59 MB |
| hash_access | 46.6 ms | 44.1 ms | 425.7 ms | 9.1× | 9.7× | 28 MB | 28 MB | 95 MB |
| method_calls | 32.9 ms | 31.3 ms | 527.2 ms | 16.0× | 16.9× | 19 MB | 19 MB | 53 MB |
| object_allocation | 35.8 ms | 31.9 ms | 939.4 ms | 26.3× | 29.4× | 19 MB | 19 MB | 53 MB |
| string_building | 39.6 ms | 38.6 ms | 276.6 ms | 7.0× | 7.2× | 25 MB | 26 MB | 89 MB |
| cycles | 49.8 ms | 48.7 ms | 677.4 ms | 13.6× | 13.9× | 19 MB | 20 MB | 61 MB |
| no_cycles | 34.0 ms | 31.8 ms | 454.9 ms | 13.4× | 14.3× | 19 MB | 19 MB | 53 MB |
| 30k_ifelse | 309.8 ms | 78.5 ms | 7.77 s | 25.1× | 99.0× | 114 MB | 159 MB | 693 MB |
| 30k_methods | 249.8 ms | 47.0 ms | 4.88 s | 19.5× | 103.7× | 66 MB | 77 MB | 272 MB |
| 30k_variables | 398.7 ms | 1.11 s | 18.72 s | 46.9× | 16.9× | 41 MB | 171 MB | 275 MB |
| attr_accessor | 82.9 ms | 11.6 ms | 8.92 s | 107.6× | 770.3× | 20 MB | 21 MB | 54 MB |
| binarytrees | 151.5 ms | 71.4 ms | 9.37 s | 61.9× | 131.2× | 27 MB | 28 MB | 143 MB |
| block_methods | 1.20 s | 269.0 ms | 174.65 s | 145.0× | 649.2× | 89 MB | 88 MB | 56 MB |
| blurhash | 169.0 ms | 63.4 ms | 7.08 s | 41.9× | 111.7× | 21 MB | 22 MB | 58 MB |
| cfunc_itself | 40.1 ms | 10.0 ms | 1.22 s | 30.4× | 122.2× | 20 MB | 21 MB | 55 MB |
| etanni | 145.3 ms | 131.8 ms | 3.51 s | 24.2× | 26.7× | 28 MB | 29 MB | 78 MB |
| fannkuchredux | 230.9 ms | 86.6 ms | 4.61 s | 20.0× | 53.3× | 20 MB | 21 MB | 54 MB |
| fib | 101.1 ms | 16.8 ms | 8.51 s | 84.2× | 506.7× | 20 MB | 21 MB | 62 MB |
| getivar | 57.8 ms | 9.0 ms | 1.35 s | 23.3× | 148.7× | 20 MB | 21 MB | 54 MB |
| getivar-module | 114.7 ms | 36.5 ms | 3.25 s | 28.4× | 89.2× | 20 MB | 21 MB | 62 MB |
| keyword_args | 85.6 ms | 15.6 ms | 13.07 s | 152.7× | 838.2× | 20 MB | 21 MB | 55 MB |
| knucleotide | 16.1 ms | 16.0 ms | 318.8 ms | 19.8× | 19.9× | 22 MB | 23 MB | 55 MB |
| loops-times | 486.7 ms | 141.8 ms | 30.38 s | 62.4× | 214.3× | 22 MB | 26 MB | 54 MB |
| matmul | 250.0 ms | 106.8 ms | 7.27 s | 29.1× | 68.0× | 26 MB | 27 MB | 63 MB |
| nbody | 45.4 ms | 22.1 ms | 3.23 s | 71.1× | 146.3× | 20 MB | 21 MB | 55 MB |
| nqueens | 116.6 ms | 26.9 ms | 2.79 s | 24.0× | 103.6× | 20 MB | 22 MB | 55 MB |
| object-new | 31.0 ms | 15.3 ms | 1.34 s | 43.3× | 87.8× | 20 MB | 21 MB | 55 MB |
| object-new-initialize | 62.1 ms | 20.5 ms | 3.65 s | 58.8× | 178.0× | 20 MB | 21 MB | 56 MB |
| object-new-no-escape | 106.2 ms | 37.8 ms | 9.30 s | 87.5× | 246.2× | 20 MB | 21 MB | 62 MB |
| protoboeuf | 79.5 ms | 15.6 ms | 4.17 s | 52.5× | 267.5× | 43 MB | 47 MB | 197 MB |
| protoboeuf-encode | 63.7 ms | 15.6 ms | 24.89 s | 390.9× | 1592.7× | 849 MB | 3240 MB | 702 MB |
| respond_to | 96.3 ms | 7.1 ms | 5.28 s | 54.8× | 742.9× | 20 MB | 21 MB | 55 MB |
| ruby-json | 101.3 ms | 83.6 ms | 32.69 s | 322.7× | 391.2× | 21 MB | 22 MB | 326 MB |
| ruby-xor | 64.0 ms | 13.3 ms | 2.58 s | 40.4× | 194.4× | 21 MB | 22 MB | 55 MB |
| send_bmethod | 101.9 ms | 5.2 ms | 9.83 s | 96.5× | 1900.1× | 20 MB | 22 MB | 55 MB |
| send_cfunc_block | 134.0 ms | 44.9 ms | 9.38 s | 70.0× | 209.0× | 20 MB | 21 MB | 55 MB |
| send_rubyfunc_block | 64.9 ms | 45.7 ms | 9.06 s | 139.6× | 198.4× | 20 MB | 21 MB | 64 MB |
| send_rubyfunc_inline | 402.1 ms | 60.4 ms | 23.06 s | 57.3× | 381.7× | 20 MB | 21 MB | 55 MB |
| setivar | 31.2 ms | 4.9 ms | 821.6 ms | 26.3× | 168.2× | 20 MB | 21 MB | 62 MB |
| setivar_object | 181.1 ms | 82.3 ms | 1.57 s | 8.7× | 19.1× | 20 MB | 21 MB | 56 MB |
| setivar_young | 59.6 ms | 22.3 ms | 1.10 s | 18.5× | 49.5× | 20 MB | 21 MB | 56 MB |
| splay | 52.2 ms | 36.4 ms | 3.53 s | 67.6× | 96.8× | 821 MB | 927 MB | 1041 MB |
| str_concat | 41.4 ms | 12.8 ms | 1.44 s | 34.7× | 112.0× | 90 MB | 109 MB | 62 MB |
| struct-new-no-escape | 62.7 ms | 35.5 ms | 3.37 s | 53.8× | 94.9× | 20 MB | 21 MB | 55 MB |
| structaref | 90.3 ms | 11.0 ms | 6.53 s | 72.4× | 592.3× | 20 MB | 21 MB | 62 MB |
| structaset | 73.7 ms | 29.2 ms | 11.59 s | 157.2× | 396.4× | 20 MB | 21 MB | 54 MB |
| sudoku | 275.0 ms | 65.3 ms | 6.71 s | 24.4× | 102.7× | 20 MB | 21 MB | 55 MB |
| throw | 9.2 ms | 6.8 ms | 689.5 ms | 75.1× | 101.8× | 20 MB | 21 MB | 54 MB |
| rex_suite | 304.2 ms | 374.4 ms | 7.21 s | 23.7× | 19.2× | 55 MB | 61 MB | 502 MB |

#### Speed in the interpreter

Metorex's time and peak memory after each commit of the interpreter's speed work, timed the same way on the same day and beside the baseline. `57fb3e94` added the inline caches for method lookup and constants, instance variable shapes, the specialized instructions, block calls without a Proc, and the allocations removed from hot paths. `d55a49e3` compiles the core library and the libraries loaded at startup when Metorex is built. A ratio below 1 is faster than the baseline. Each column comes from a run of its own, so a row that changed little between two commits differs by the noise between runs.

| Benchmark | Baseline | `57fb3e94` | `d55a49e3` | `57fb3e94` ÷ baseline | `d55a49e3` ÷ baseline | Baseline peak | `57fb3e94` peak | `d55a49e3` peak |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: |
| empty | 55.6 ms | 55.2 ms | 24.7 ms | 0.99 | 0.44 | 53 MB | 53 MB | 33 MB |
| block_calls | 654.7 ms | 265.0 ms | 248.8 ms | 0.40 | 0.38 | 59 MB | 53 MB | 34 MB |
| hash_access | 425.7 ms | 294.2 ms | 295.3 ms | 0.69 | 0.69 | 95 MB | 92 MB | 89 MB |
| method_calls | 527.2 ms | 244.5 ms | 223.8 ms | 0.46 | 0.42 | 53 MB | 53 MB | 33 MB |
| object_allocation | 939.4 ms | 593.8 ms | 596.8 ms | 0.63 | 0.64 | 53 MB | 54 MB | 34 MB |
| string_building | 276.6 ms | 220.4 ms | 200.6 ms | 0.80 | 0.73 | 89 MB | 86 MB | 69 MB |
| cycles | 677.4 ms | 599.6 ms | 593.7 ms | 0.89 | 0.88 | 61 MB | 55 MB | 37 MB |
| no_cycles | 454.9 ms | 365.6 ms | 346.1 ms | 0.80 | 0.76 | 53 MB | 53 MB | 34 MB |
| 30k_ifelse | 7.77 s | 5.75 s | 6.44 s | 0.74 | 0.83 | 693 MB | 700 MB | 587 MB |
| 30k_methods | 4.88 s | 3.27 s | 3.55 s | 0.67 | 0.73 | 272 MB | 276 MB | 193 MB |
| 30k_variables | 18.72 s | 17.08 s | 17.39 s | 0.91 | 0.93 | 275 MB | 275 MB | 226 MB |
| attr_accessor | 8.92 s | 5.44 s | 5.39 s | 0.61 | 0.60 | 54 MB | 54 MB | 39 MB |
| binarytrees | 9.37 s | 6.78 s | 6.69 s | 0.72 | 0.71 | 143 MB | 165 MB | 148 MB |
| block_methods | 174.65 s | 39.67 s | 29.50 s | 0.23 | 0.17 | 56 MB | 58 MB | 43 MB |
| blurhash | 7.08 s | 4.20 s | 4.18 s | 0.59 | 0.59 | 58 MB | 61 MB | 45 MB |
| cfunc_itself | 1.22 s | 780.7 ms | 755.3 ms | 0.64 | 0.62 | 55 MB | 54 MB | 39 MB |
| etanni | 3.51 s | 2.44 s | 2.47 s | 0.70 | 0.70 | 78 MB | 79 MB | 67 MB |
| fannkuchredux | 4.61 s | 3.74 s | 3.84 s | 0.81 | 0.83 | 54 MB | 55 MB | 39 MB |
| fib | 8.51 s | 4.98 s | 4.87 s | 0.59 | 0.57 | 62 MB | 54 MB | 39 MB |
| getivar | 1.35 s | 480.3 ms | 474.6 ms | 0.36 | 0.35 | 54 MB | 54 MB | 39 MB |
| getivar-module | 3.25 s | 1.89 s | 1.87 s | 0.58 | 0.58 | 62 MB | 54 MB | 39 MB |
| keyword_args | 13.07 s | 9.92 s | 9.95 s | 0.76 | 0.76 | 55 MB | 55 MB | 39 MB |
| knucleotide | 318.8 ms | 280.0 ms | 301.4 ms | 0.88 | 0.95 | 55 MB | 55 MB | 40 MB |
| loops-times | 30.38 s | 12.65 s | 12.73 s | 0.42 | 0.42 | 54 MB | 54 MB | 40 MB |
| matmul | 7.27 s | 5.45 s | 5.45 s | 0.75 | 0.75 | 63 MB | 64 MB | 42 MB |
| nbody | 3.23 s | 1.54 s | 1.52 s | 0.48 | 0.47 | 55 MB | 55 MB | 39 MB |
| nqueens | 2.79 s | 2.19 s | 2.22 s | 0.78 | 0.80 | 55 MB | 54 MB | 39 MB |
| object-new | 1.34 s | 976.1 ms | 970.4 ms | 0.73 | 0.72 | 55 MB | 54 MB | 39 MB |
| object-new-initialize | 3.65 s | 2.47 s | 2.43 s | 0.68 | 0.67 | 56 MB | 62 MB | 39 MB |
| object-new-no-escape | 9.30 s | 5.59 s | 5.48 s | 0.60 | 0.59 | 62 MB | 54 MB | 39 MB |
| protoboeuf | 4.17 s | 3.05 s | 3.08 s | 0.73 | 0.74 | 197 MB | 183 MB | 157 MB |
| protoboeuf-encode | 24.89 s | 21.28 s | 22.10 s | 0.85 | 0.89 | 702 MB | 393 MB | 362 MB |
| respond_to | 5.28 s | 4.54 s | 4.72 s | 0.86 | 0.89 | 55 MB | 54 MB | 39 MB |
| ruby-json | 32.69 s | 19.07 s | 18.61 s | 0.58 | 0.57 | 326 MB | 334 MB | 324 MB |
| ruby-xor | 2.58 s | 2.11 s | 2.07 s | 0.82 | 0.80 | 55 MB | 62 MB | 39 MB |
| send_bmethod | 9.83 s | 6.47 s | 6.50 s | 0.66 | 0.66 | 55 MB | 54 MB | 39 MB |
| send_cfunc_block | 9.38 s | 8.91 s | 8.82 s | 0.95 | 0.94 | 55 MB | 55 MB | 40 MB |
| send_rubyfunc_block | 9.06 s | 5.52 s | 5.35 s | 0.61 | 0.59 | 64 MB | 55 MB | 40 MB |
| send_rubyfunc_inline | 23.06 s | 10.49 s | 10.21 s | 0.45 | 0.44 | 55 MB | 54 MB | 39 MB |
| setivar | 821.6 ms | 321.6 ms | 286.8 ms | 0.39 | 0.35 | 62 MB | 55 MB | 39 MB |
| setivar_object | 1.57 s | 621.9 ms | 458.1 ms | 0.40 | 0.29 | 56 MB | 55 MB | 39 MB |
| setivar_young | 1.10 s | 458.0 ms | 456.5 ms | 0.42 | 0.42 | 56 MB | 54 MB | 39 MB |
| splay | 3.53 s | 2.68 s | 2.15 s | 0.76 | 0.61 | 1041 MB | 995 MB | 980 MB |
| str_concat | 1.44 s | 1.23 s | 1.07 s | 0.85 | 0.74 | 62 MB | 57 MB | 39 MB |
| struct-new-no-escape | 3.37 s | 2.43 s | 2.37 s | 0.72 | 0.70 | 55 MB | 55 MB | 39 MB |
| structaref | 6.53 s | 3.65 s | 3.62 s | 0.56 | 0.55 | 62 MB | 54 MB | 39 MB |
| structaset | 11.59 s | 8.61 s | 8.45 s | 0.74 | 0.73 | 54 MB | 54 MB | 39 MB |
| sudoku | 6.71 s | 5.82 s | 5.80 s | 0.87 | 0.86 | 55 MB | 56 MB | 40 MB |
| throw | 689.5 ms | 408.7 ms | 406.5 ms | 0.59 | 0.59 | 54 MB | 54 MB | 40 MB |
| rex_suite | 7.21 s | 4.98 s | 5.02 s | 0.69 | 0.70 | 502 MB | 347 MB | 305 MB |

`scripts/bench_rex.sh` runs `benchmarks/rex_suite`, 3000 trivial examples, with a release build and compares boot time, load time, examples per second, and peak memory with `benchmarks/rex_suite/baseline.json`. `scripts/compare_fast_paths.sh` runs the RSpec libraries' suites with the fast paths on and off and checks that every spec file prints the same.

Times on an Apple M-series machine, release build:

| Suite | Before | After | MRI 4.0 with RSpec 3.13 |
| :-- | --: | --: | --: |
| `benchmarks/rex_suite`, running the examples | 5.24 s | 3.81 s | 0.12 s |
| `benchmarks/rex_suite`, loading the spec files | 3.21 s | 1.88 s | 0.13 s |
| diff-lcs, all 322 examples | 2.32 s | 2.22 s | 1.44 s |
| diff-lcs, the 272 examples outside `ldiff_spec.rb` | 0.63 s | 0.59 s | 0.03 s |

For `rex_suite`, "before" is the baseline recorded before any fast path. For diff-lcs it is the build with the fast paths in place, before the interpreter changes its profile led to: skipping refinement lookups when no `using` is in force, a faster hash for method and variable tables, keeping the method caches when an example's singleton class is made, and reading the clock less often. diff-lcs runs with MRI's `ruby` first on `PATH`, since `ldiff_spec.rb` starts it for each case, and most of that file's time is spent waiting on those processes.

## Roadmap

[ROADMAP.md](ROADMAP.md) settles the value representation, memory management, frame layout, and instruction format, moves execution from the tree walker onto bytecode that `VirtualMachine` runs, removes the tree walker, and then works on speed in the interpreter and with a JIT.

## Contributing

METOREX is in active development. We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository (including Ruby spec submodules)
git clone https://github.com/metorex-lang/metorex.git
cd metorex
git submodule update --init

# Build the project
cargo build

# Run tests
cargo test

# Run the REPL
cargo run

# Run a Metorex file
cargo run -- my_script.rb

# Discover and run test files in a directory
cargo run -- test tests/

# Install the coverage tool (needs the llvm-tools component)
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov

# Run code coverage (the vendored Ruby suite under ruby/ is left out)
cargo llvm-cov --ignore-filename-regex 'ruby/'

# Run Ruby spec suite (requires submodules)
scripts/run_ruby_spec.sh

# Compare speed with CRuby and YJIT (by hand, needs a release build and the
# benchmarks/ruby-bench submodule)
cargo build --release
ruby benchmarks/compare.rb

# Profile a run with sample (macOS) or perf (Linux), and count what it
# evaluates, calls, and allocates (docs/engine/profiling.md)
scripts/profile.sh report.txt path/to/program.rb
METOREX_COUNTERS=1 target/release/metorex path/to/program.rb

# Write how many bodies were compiled and how many were refused for being
# nested too deep, and the node types that ran out of registers
METOREX_ENGINE_REPORT=report.txt target/release/metorex path/to/program.rb

# Write what was resident at the peak resident set size, by what allocated
# it, and the objects alive by class (docs/engine/memory-management.md)
METOREX_MEMORY_REPORT=1 target/release/metorex path/to/program.rb

# Write each body's instructions to standard error as it is compiled
METOREX_DISASM=1 target/release/metorex path/to/program.rb

# Run with the JIT off, or compile each body to machine code on its first
# run rather than after a threshold of its own (docs/engine/jit.md)
METOREX_JIT=0 target/release/metorex path/to/program.rb
METOREX_JIT_CALL_THRESHOLD=1 target/release/metorex path/to/program.rb

```

### Running everything

`./test.sh` runs what CI runs, on this machine and on Linux, on both
architectures: formatting, clippy, the Rust tests, the Ruby spec suite,
both again with the JIT compiling every body on its first run, coverage,
and the check that no test sits in `src/`. On this machine the
engine report of those runs is added up in `target/engine-report.txt`. It
stops where a step fails, so what went wrong is the last thing on the screen.

```bash
./test.sh              # all of it
./test.sh mac          # this machine alone
./test.sh linux        # both Linux architectures
./test.sh linux-amd64  # the one CI runs on
./test.sh linux-arm64
```

The architecture that is not this machine's runs under emulation. It is slower
and it catches what only shows there: a spec guarded to x86, the signedness of
a C character, the width of a pointer.

The image carries the Ruby the vendored suite is written for, which is the
release metorex reports itself as. It is the same build the CI runner
installs, unpacked where the runner keeps it. That Ruby runs the mspec driver; each spec
file is executed by metorex, so the driver's version settles nothing about
which examples run, but it is what a comparison against real Ruby inside the
container answers with.

### Building on Linux

Some of what metorex reaches for differs between platforms: the clocks a
system keeps, where an interface holds its point-to-point address, and which
library carries `crypt`. CI runs on Linux, so a build that passes on macOS can
still fail there. `scripts/linux.sh` runs any command against a Linux build in
the image CI uses, with the working tree mounted and a build directory of its
own so it leaves the host's build alone. `LINUX_ARCH` picks the architecture,
`amd64` or `arm64`, and defaults to this machine's. `LINUX_USER=root` runs the
container as root, which the spec suite's `as_superuser` blocks need.

```bash
scripts/linux.sh                          # cargo test
scripts/linux.sh cargo build
scripts/linux.sh cargo clippy --all-targets
scripts/linux.sh scripts/run_ruby_spec.sh
LINUX_ARCH=amd64 scripts/linux.sh cargo test
LINUX_USER=root scripts/linux.sh bash -c 'CARGO_TARGET_DIR=target/root cargo build'

# Name a test that hangs: the name is printed before the test runs, so the
# last line with no `ok` after it is the one that stopped.
scripts/linux.sh cargo test -- --test-threads=1
```

## License

See [LICENSE](LICENSE) for details. `vendor/prism` is the Prism parser, under its own MIT license in `vendor/prism/LICENSE.md`, `vendor/nkf` is nkf, under the BSD 2-clause license in `vendor/nkf/LICENSE.txt`, `vendor/rbs` is rbs's extension and parser, under the BSD 2-clause license in `vendor/rbs/BSDL`, and `vendor/debug` is debug's extension, under the BSD 2-clause license in `vendor/debug/LICENSE.txt`. The bundled gems under `src/vm/stdlib/gems` keep their own license files. `src/vm/stdlib/parser_tables.rb` holds the parser tables of Ruby's `parse.c`, generated by `scripts/generate_parser_tables.rb`, under Ruby's license.

## Why METOREX?

**For DSL Creators**: Build domain-specific languages naturally with first-class AST access.

**For Scripters**: Ruby-like syntax with powerful built-in libraries.

**For Systems Programmers**: Rust-based VM with performance and safety guarantees.

**For Functional Enthusiasts**: Optional algebraic data types, immutable structures, and functional patterns.

**For Pragmatists**: One language that adapts to your needs - from quick scripts to production systems.

**METOREX: Where meta-programming meets production-ready performance.**
