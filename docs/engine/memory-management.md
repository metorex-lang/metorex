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

## Cycles leak now

`benchmarks/memory/cycles.rb` makes 200,000 objects that each hold a lambda
referring back to the object, which is how an object that keeps a callback
to itself looks, and keeps none of them. `benchmarks/memory/no_cycles.rb`
makes the same objects with an Integer in place of the lambda. From
`ruby benchmarks/compare.rb --only=cycles,no_cycles` on 2026-10-15:

| Script | CRuby peak memory | Metorex peak memory |
| :-- | --: | --: |
| cycles | 19 MB | 551 MB |
| no_cycles | 19 MB | 49 MB |

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
object rather than only instances, classes, and modules as now, and
`count_objects` counts each `T_` kind from the same list. `_id2ref`, which
Ruby 4.0 deprecated, reads a weak table from object ids to objects, filled
when an id is first asked for.

### WeakMap, WeakKeyMap, and WeakRef

A header keeps a weak count beside the strong one, as `Rc` does now, and a
weak reference reads the object only while the strong count is above zero.
`ObjectSpace::WeakMap` and `WeakKeyMap` hold their entries through weak
references and drop an entry when its object is freed, in place of the array
of strong references `WeakMap` keeps now. `WeakRef` already holds a Rust
`Weak`. `library/weakref`'s `make_dead_weakref` finds a dead reference on its
first try, since counting frees the object as soon as the last reference
goes.

### Finalizers

A finalizer runs after its object is freed, whether by counting or by a
collection. The free queues the finalizer, and the interpreter runs the queue
at its next safe point, since a finalizer is Ruby code and a free can happen
inside a native method. The finalizers of objects still alive when the
program ends run at exit, as they all do now, which is what
`core/objectspace/define_finalizer_spec.rb` checks.

### GC

`GC.start` runs a collection. `GC.count` counts collections, and `GC.stat`
reports the objects allocated, freed, and alive from the list. `GC.disable`
stops collections, and counting still frees objects whose count reaches
zero. The specs under `core/gc` check what these methods answer and that
`GC.start` returns nil, which they keep doing.

## Threads

Counts are not atomic, as `Rc`'s are not now. Metorex runs Ruby threads and
fibers on stacks it switches between, with `corosensei`, on one operating
system thread. The operating system threads it starts run a C function
handed to `rb_thread_call_without_gvl` and the blocking work in
`src/vm/fibers/waiting.rs`. The first touches no Ruby object, as the GVL's
rule for C extensions requires, and the second is a closure Rust only lets
cross to another thread when it holds nothing reference counted. So no two
threads change a count at once.
