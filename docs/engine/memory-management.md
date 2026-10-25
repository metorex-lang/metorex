# Memory management

## Decision

Objects stay reference counted, and a cycle collector frees the cycles
counting cannot.

- Each heap object counts its references in its header, as each `Rc` does
  now, and is freed when the count reaches zero.
- Every heap object is listed. The cycle collector runs CPython's trial
  deletion over the list: it takes each object's count, subtracts the
  references other listed objects hold to it, and keeps every object left
  above zero and everything those reach. The rest are cycles nothing outside
  them refers to, and they are freed.
- A collection runs when the list has doubled since the last one left it, and
  when the program calls `GC.start`.

## Why not a tracing collector

A tracing collector frees what it cannot reach from the roots, so it has to
find every root. Metorex's native methods are Rust, and they hold values in
Rust collections a collector cannot see: `Vec<Object>` is written 321 times
in 146 files under `src/`, and 98 more places hold values in a `HashMap` or
an `IndexMap`. Scanning the native stack finds the vector but not what it
points to on the heap, so a collection during a native method would free
values the method still holds. Making every one of those holders visible to
a collector means rewriting the native methods around handles.

Trial deletion needs no roots. A value a Rust `Vec` holds adds one to the
count of the object it refers to, and no listed object accounts for that
one, so the object stays.

## Cycles before the collector

`benchmarks/memory/cycles.rb` makes 200,000 objects that each hold a lambda
referring back to the object, which is how an object that keeps a callback
to itself looks, and keeps none of them. `benchmarks/memory/no_cycles.rb`
makes the same objects with an Integer in place of the lambda. From
`ruby benchmarks/compare.rb --only=cycles,no_cycles` on 2026-10-15:

| Script | CRuby peak memory | Metorex peak memory |
| :-- | --: | --: |
| cycles | 19 MB | 551 MB |
| no_cycles | 19 MB | 49 MB |

## Cycles with the collector

From the same command on 2026-10-17:

| Script | CRuby peak memory | Metorex peak memory |
| :-- | --: | --: |
| cycles | 19 MB | 35 MB |
| no_cycles | 19 MB | 33 MB |

Each `cycles` object and its lambda take about 1.6 KB, since a lambda keeps
its own copy of the file name, the scopes open where it was written, and the
locals it closes over. About 1,500 objects are alive after startup, so the
first collection waits for 2,048 listed objects, which keeps the garbage
between collections under 2 MB. With 4,096 the peak was 40 MB.

## The prototype

`prototypes/values` runs its interpreter over the 8-byte tagged value with
its heap cells managed three ways:

- **Counted**: freed when the count reaches zero, and never when the cells
  form a cycle.
- **Cycle collected**: counted, every cell listed, and trial deletion run
  when the list has doubled.
- **Traced**: nothing counted, every cell listed, and a mark and sweep from
  the interpreter's stack run when the list has doubled. The prototype knows
  every root, which Metorex does not, so this column is what tracing would
  run at if it could.

The `cycles` kernel makes 500,000 objects that each refer to themselves.
Each time is the fastest of five runs, and the cells are the most alive at
once in the last run.

| Kernel | Counted | Counted, cells | Cycle collected | Cycle collected, cells | Collections | Traced | Traced, cells | Collections |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: |
| fib | 52.7 ms | 0 | 50.5 ms | 0 | 0 | 48.8 ms | 0 | 0 |
| getivar | 424.7 ms | 1 | 401.4 ms | 1 | 0 | 256.3 ms | 1 | 0 |
| object_new | 498.9 ms | 2 | 528.2 ms | 2 | 0 | 426.5 ms | 283 | 14327 |
| array_sum | 226.1 ms | 1 | 217.2 ms | 1 | 0 | 134.8 ms | 1 | 0 |
| cycles | 21.2 ms | 500000 | 34.0 ms | 470 | 716 | 20.3 ms | 294 | 716 |

- The cycle collector keeps the `cycles` kernel at a few hundred cells, where
  counting alone keeps every one, at 13 ms more for 500,000 objects.
- Listing every cell costs 6% on `object_new`, which makes and frees 10
  million objects.
- The cycle collected runs take 1.57 times as long as the traced ones on
  `getivar` and 1.61 times on `array_sum`, since tracing copies a value
  without touching its cell. That is the price of keeping what Rust holds
  safe. An instruction that reads a value it does not keep can skip taking a
  reference, which the instruction set decides.

## How each requirement keeps ruby/spec passing

### The C API

CRuby extensions hold `VALUE`s in C locals and C structs without counting
them, and expect what they hold to stay alive. A `VALUE` handed to C is the
value's bits, and the object is pinned: a bit in its header is set and one
reference is added, the first time it is handed over, and the object goes on
a list of pinned objects. A collection unpins every pinned object it cannot
reach from what C holds:

- the native stack, scanned conservatively from where Metorex called into C,
  on each thread and fiber stack that has C frames, which is what keeps
  `RB_GC_GUARD` working
- what each `T_DATA` object's `dmark` function marks
- the addresses given to `rb_gc_register_address` and `rb_global_variable`,
  and the objects given to `rb_gc_register_mark_object`

Unpinning drops the added reference, so an object only C held is freed then.
Handing over an object already pinned tests one bit. `optional/capi/gc_spec.rb`
covers `RB_GC_GUARD`, `rb_gc_register_address`, `rb_global_variable`,
`rb_gc_enable`, and `rb_gc_disable`.

### ObjectSpace

`each_object` walks the list of heap objects, so it reaches every kind of
object, and `count_objects` counts each `T_` kind from the same list. `_id2ref`, which
Ruby 4.0 deprecated, reads a weak table from object ids to objects, filled
when an id is first asked for.

### WeakMap, WeakKeyMap, and WeakRef

A header keeps a weak count beside the strong one, as `Rc` does, and a weak
reference reads the object only while the strong count is above zero.
`ObjectSpace::WeakMap` holds its keys and values through weak references and
drops an entry when either is freed. `WeakKeyMap` holds its keys that way and
drops an entry when its key is freed. `WeakRef` holds a Rust `Weak`. `library/weakref`'s `make_dead_weakref` finds a dead reference on its
first try, since counting frees the object as soon as the last reference
goes.

### Finalizers

A finalizer runs after its object is freed, whether by counting or by a
collection. Defining one adds a weak reference to the object to a list, and
each collection runs the finalizers of the objects on the list that are no
longer alive. A collection runs at the start of a statement or in `GC.start`,
never inside a native method, since a finalizer is Ruby code. The finalizers
of objects still alive when the program ends run at exit, which is what
`core/objectspace/define_finalizer_spec.rb` checks.

### GC

`GC.start` runs a collection. `GC.count` counts collections, and `GC.stat`
reports the objects allocated, freed, and alive from the list. `GC.disable`
stops collections, and counting still frees objects whose count reaches
zero. The specs under `core/gc` check what these methods answer and that
`GC.start` returns nil, which they keep doing.

## In Metorex

- `src/object/live.rs` keeps the list: a weak reference to each String,
  Array, Hash, Set, instance, exception, block, Method, Binding, class, and
  module. An object goes on the list the first time a value is made for it,
  which is when its `Rc` has no weak reference yet.
- `src/object/collector.rs` runs trial deletion. The nodes it counts are the
  listed objects and the parts they share: the cell a pointer value points
  at, which the value's copies share, the locals a block closes over, and
  each local's cell. Each node reports the references it holds, one per
  reference.
- A node borrowed for writing when the collector reads it is in use, so it
  is kept with everything it reaches. A reference the collector does not
  read only keeps its target alive. It does not read a class's constants,
  methods, or class variables, an instance's singleton methods, or a
  binding's frame, so a cycle through one of those stays.
- When the list has doubled, and holds at least 2,048 objects, the next
  statement to start runs a collection.
  `GC.start` runs one at once.
- An object handed to C is pinned in `src/vm/capi/values.rs`. Metorex
  does not scan the native stack: while a call into C has not returned on
  any thread or fiber stack, a collection unpins nothing, which keeps what
  C locals hold, `RB_GC_GUARD`'s among them. Otherwise it unpins every
  object that no registered address, `rb_gc_register_mark_object` object,
  `dmark` function of a reached Data object, or table the C API keeps for
  an object names. It runs again until nothing more is unpinned, then frees
  the Data objects nothing reaches.

## Where the memory goes

With `METOREX_MEMORY_REPORT` set to anything but `0` or nothing, Metorex
writes a report to standard error at exit, from `src/vm/memory_report.rs`.

- Each heap block carries a 16-byte header naming what allocated it: the
  syntax tree (lexing, parsing, and reading a library's tree from the
  binary), compiled units, the JIT compiling, or everything else. The header
  also says whether startup allocated it, which is loading the core library
  and the libraries a default start loads, or the program did.
- Every 256 KB allocated, the allocator reads the resident set size. When
  it is the largest yet, it keeps the bytes of the heap blocks in use on
  each row, and measures what else is resident: malloc's resident pages that
  hold no block in use, the JIT's machine code, the pages that map files,
  and the stack. On macOS malloc's pages are the resident pages of the
  regions it tagged, and on Linux they are glibc's arenas and the blocks it
  mapped on their own.
- When that resident set is a tenth more than at the last walk, the next
  object listed starts a walk of the list. It adds up each object's size
  and the capacities of what it holds, by class.

The headers take memory of their own, so a run with the report peaks
higher: `ruby-json` peaked at 333 MB without it and 414 MB with it.

The core library and the libraries a default start loads are held in the
binary as they were parsed when it was built, with each method, block,
class, and module body encoded after the statements around it. A body is
decoded the first time something reads it, so a method that never runs
keeps no syntax tree and leaves its bytes in the binary unread. The
libraries' sources sit in a section of the binary of their own, apart from
the table of their names a `require` reads.

The reports of 2026-10-24 on macOS, with the ruby-bench rows run for one
warmup iteration and two more, and `rex_suite` run with `metorex spec`, in
MB. Where a row has two numbers, the first is startup's and the second the
program's.

| Row | Peak resident | Syntax tree | Compiled units | JIT compiling | Other heap blocks | Headers | malloc's other pages | Machine code | Files | Stack | Accounted for |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: | --: |
| empty | 22.5 | 0.6, 0.0 | 1.0, 0.0 | 0.0 | 5.0, 0.0 | 0.9 | 3.1 | 0.0 | 10.0 | 0.2 | 93% |
| rex_suite | 279.1 | 0.6, 13.1 | 1.0, 14.5 | 0.7 | 7.9, 117.5 | 23.4 | 85.9 | 0.4 | 12.2 | 0.4 | 99% |
| 30k_ifelse | 631.6 | 0.6, 297.7 | 1.0, 67.6 | 9.5 | 7.7, 130.7 | 38.6 | 59.8 | 5.8 | 10.9 | 0.4 | 100% |
| ruby-json | 413.9 | 0.6, 2.6 | 1.0, 2.5 | 0.0 | 5.1, 216.7 | 51.1 | 121.8 | 0.0 | 11.0 | 0.2 | 100% |

The objects the last walk found:

| Row | Objects' bytes | Largest classes |
| :-- | --: | :-- |
| empty | 0.8 | 445 classes and modules, 0.5 MB, and 728 Strings, 0.2 MB |
| rex_suite | 44.7 | 37,244 Strings, 12.2 MB, 12,847 Hashes, 9.8 MB, and 12,907 Procs, 5.3 MB |
| 30k_ifelse | 19.3 | 30,200 method definitions, 18.4 MB |
| ruby-json | 1.8 | 2,343 Strings, 0.8 MB |

- In `empty` the binary's pages and the system libraries' are 10.0 MB, the
  largest part. Most of them are the code a start runs.
- In `30k_ifelse` the program's syntax tree is 297.7 MB, and it stays after
  its units are compiled.
- In `ruby-json` 216.7 MB of the program's heap blocks are in neither the
  syntax tree nor the compiled units, and the objects alive hold 1.8 MB of
  them.
- In the three rows that run a program, malloc keeps 60 to 122 MB resident
  in pages that hold no block in use.

## Threads

Counts are not atomic, as `Rc`'s are not now. Metorex runs Ruby threads and
fibers on stacks it switches between, with `corosensei`, on one operating
system thread. The operating system threads it starts run a C function
handed to `rb_thread_call_without_gvl` and the blocking work in
`src/vm/fibers/waiting.rs`. The first touches no Ruby object, as the GVL's
rule for C extensions requires, and the second is a closure Rust only lets
cross to another thread when it holds nothing reference counted. So no two
threads change a count at once.
