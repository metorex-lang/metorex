# Instruction format and dispatch

## Decision

- Metorex runs a register machine. A unit's registers are its locals
  followed by its temporaries, and they are the frame's part of the value
  stack segment that `docs/engine/frames-and-calls.md` describes. A call's
  arguments are written to the caller's highest registers, which become the
  callee's first registers, so arguments are still passed in place.
- Each instruction is 4 bytes: an opcode byte and either three 8-bit
  operands or one 8-bit and one 16-bit operand. In Rust each instruction is
  a variant of an enum with `u8` and `u16` fields, which is 4 bytes with no
  packing done by hand.
- An operand of an arithmetic or comparison instruction below 128 names a
  register, and one at 128 or above names the unit's constant at the
  operand less 128. Other operands name registers or, as the 16-bit operand,
  a constant, a jump target, or an entry in a side table.
- A method call names its call site with the 16-bit operand: an entry in
  the unit's side table holding the method name, the argument count, the
  keyword names, the block's unit, and the inline cache. A source position
  for each instruction is kept in a side table as well.
- An operand that does not fit, a register past 255, a constant past 127 in
  arithmetic or past 65,535 elsewhere, or a jump target past 65,535, is
  given by an `Extend` instruction before it, whose operands supply the
  high bits, as CPython's `EXTENDED_ARG` does.
- Dispatch is a `match` on the instruction in one loop.

## Rust has no guaranteed tail call

A threaded interpreter in C or with LLVM's `musttail` ends each
instruction's handler with a jump to the next one. In Rust that is the
`become` expression, the `explicit_tail_calls` feature, which is unstable
in rustc 1.96.0 (tracking issue #112788). Without it, a handler per
instruction is a function called from a loop, the function pointers column
below, and it is 1.11 to 1.41 times the time of `match`, which rustc
compiles to one jump table.

## The prototype

`prototypes/values/src/dispatch.rs` runs the six kernels of
`docs/engine/value-representation.md` over the 8-byte tagged value in five
forms:

- **Stack, byte stream**: an opcode byte followed by its operand bytes, a
  wide operand big-endian, as `src/bytecode` lays out instructions now.
- **Stack**: the same instructions, 4 bytes each.
- **Register**: register instructions, 4 bytes each, with each literal
  loaded into a register where it is used.
- **Register, constant operands**: the decision above.
- **Register, function pointers**: the register instructions decoded once
  into a function pointer and operands each, 16 bytes, and run by calling
  the pointer.

Each loop is compiled as a compiler without an optimizer would compile it:
`i += 1` is a load of `i`, a load of `1`, an add, and a store on the stack
machine, and one add on the register machine with constant operands.

```sh
cargo run --release --manifest-path prototypes/values/Cargo.toml
```

Recorded on 2026-10-15 on an Apple M5 Max running macOS 27.0.1, with rustc
1.96.0. Each time is the fastest of five runs.

| Kernel | Stack, byte stream | Stack | Register | Register, constant operands | Register, function pointers | Stack ÷ register, constant operands |
| :-- | --: | --: | --: | --: | --: | --: |
| fib | 52.9 ms | 53.5 ms | 55.0 ms | 48.9 ms | 77.4 ms | 1.09× |
| while_loop | 1014.2 ms | 954.6 ms | 830.8 ms | 709.1 ms | 1037.8 ms | 1.35× |
| getivar | 452.3 ms | 433.5 ms | 359.8 ms | 317.1 ms | 472.1 ms | 1.37× |
| object_new | 515.9 ms | 505.9 ms | 428.9 ms | 397.2 ms | 475.7 ms | 1.27× |
| float_arithmetic | 574.2 ms | 543.0 ms | 521.9 ms | 426.0 ms | 679.7 ms | 1.27× |
| array_sum | 229.6 ms | 221.1 ms | 169.3 ms | 145.8 ms | 234.6 ms | 1.52× |

| Kernel | Stack, instructions run | Register | Register, constant operands |
| :-- | --: | --: | --: |
| fib | 26925369 | 20194027 | 14808954 |
| while_loop | 650000010 | 350000006 | 250000005 |
| getivar | 280000024 | 160000015 | 120000014 |
| object_new | 170000008 | 90000005 | 70000004 |
| float_arithmetic | 300000010 | 200000006 | 120000005 |
| array_sum | 135000021 | 75000014 | 55000012 |

The register machine with constant operands runs 38 to 55% as many
instructions as the stack machine. Each register instruction does more,
reading two operands and writing a third, so the time falls by less: it is
1.09 to 1.52 times faster. Without constant operands, fib is 3% slower on
registers than on the stack, since its literals each take an instruction.

## Alternatives

- **A stack machine**, as YARV and `src/bytecode` are: 1.09 to 1.52 times
  the time of the decision on these kernels.
- **A byte stream** with operands of their own sizes: 0.99 to 1.06 times
  the time of 4-byte instructions on the stack machine, and an instruction
  cannot be read without decoding the ones before it.
- **Register operands only**, with literals loaded by an instruction of
  their own: 1.08 to 1.23 times the time of constant operands.
- **A function pointer per instruction**: 1.11 to 1.41 times the time of
  `match`, as above.

## In Metorex

- `src/bytecode/instruction.rs` defines `Instruction`, the enum of the
  decision, 4 bytes each. `src/bytecode/unit.rs` defines `Unit`: the
  instructions, a source position for each, the statement each starts, the
  spot `error_highlight` points at for an error each raises, and the side
  tables of constants, names, locals, call sites, and rescue handlers.
- `src/bytecode/compiler.rs` compiles a body to a unit: literals of every
  kind, including Arrays, Hashes, Ranges, Regexps, and interpolated
  Strings, `__FILE__`, `__LINE__`, and `__dir__`, locals, instance, class,
  and global variables, constants, `defined?`, every binary and unary
  operator, `&&` and `||`, subscripts, assignment to every kind of target,
  the operator assignments, multiple assignment, block and lambda
  literals, pattern tests, calls with or without a receiver, with any
  arguments, a block, and `&.`, `if`, `unless`, `while`, `until`, `for`,
  `case` written as an expression, flip-flops, `break`, `next`, `redo`,
  `return`, and `yield`. A body holding any other node is refused
  with that node's type name and runs on the tree walker. Class and module
  bodies run on the tree walker under either engine for now, since their
  statements follow rules of their own.
- A call without a receiver, `name(arguments)`, decides how it is made
  before its arguments are evaluated, from what the scope binds `name` to
  and what `self` answers: `ResolveCall` and `eval_call` both call
  `resolve_bare_call`, and `CallResolved` and `eval_call` both call
  `run_call_route`.
- Arguments with `*`, `**`, or `&` among them are gathered in an argument
  list, `StartArguments` and one instruction per argument, which call the
  same `ArgumentList` the tree walker's `evaluate_arguments` uses.
- `defined?` and pattern tests call the tree walker's `eval_defined` and
  `match_pattern`, which evaluate the expressions inside them, such as a
  pinned expression or a receiver `defined?` asks about.
- Branches and loops are jumps inside the unit. A `break`, `next`, or
  `redo` inside a loop of the unit jumps, after dropping any argument list,
  call route, or `case` scope it leaves open. One outside every loop ends
  the unit with that control flow, which the method, block, or top level
  the unit is the body of takes as the tree walker takes the same
  statement: a block's `next` answers its value, its `break` returns from
  the call the block was attached to, its `redo` runs it again, and its
  `return` returns from the method the block was written in, or from the
  block when it is a lambda.
- `for` compiles to the call of `each` the tree walker turns it into.
- No temporary register lives from one statement to the next, so the runner
  clears them as each statement starts, and a value only a temporary held
  can be freed.
- An assignment through a setter, `[]=`, or `Ns::Name =` evaluates the
  receiver, subscripts, or namespace into registers, and names them in the
  unit's table of targets. `StoreTarget` and `ReadTarget` turn the entry
  back into the tree walker's prepared target and write or read through
  it, so both engines call the same setter and `[]=` code.
- A Hash literal's keys and values are all evaluated before any key's
  `hash` is called, as in CRuby, on both engines. Each hole of an
  interpolated String is converted to text before the next is evaluated.
- An Array or Hash literal longer than 64 elements or pairs is built 64 at
  a time, `NewArray` or `NewHash` for the first and `AppendArray` or
  `AddToHash` for each after, so its registers stay under 256.
- `VirtualMachine::run_unit`, in `src/vm/bytecode_runner.rs`, dispatches
  each instruction by `match` in one loop over a register file the unit's
  size. Registers are a `Vec` per run for now, not the frame's part of a
  shared value stack.
- An instruction that needs the interpreter calls the function the tree
  walker calls: `operate_on_values` for an operator, `call_evaluated_method`
  for a call, `read_identifier` for a constant or a name the unit does not
  assign, `eval_instance_var_read` for an instance variable,
  `read_global_variable` for a global, and `assign_value` for a write.
- A unit numbers the locals its body assigns and keeps their names. Each
  local is held in the cell the scope holds it in, the same cell a block
  closing over it, `binding`, `local_variables`, `eval`, and
  `Binding#local_variable_get` and `local_variable_set` read and write. The
  first `SetLocal` of a run writes through the scope and keeps the cell in
  the local's slot, and later reads and writes of the run go to the cell.
  Until then `GetLocal` reads the scope by name. The locals are not
  registers for now.
- An instruction that starts a statement runs what the tree walker runs
  before each statement: the line event, signals, a due collection, the
  thread's turn, and coverage.
- An error raised inside a handler's range that a `rescue` can catch puts
  the exception in the handler's register and jumps to its target.
- `METOREX_COUNTERS=1` counts each instruction run, by name.
- `METOREX_DISASM=1` writes each unit to standard error as it is compiled,
  one instruction a line under the source line it came from, with what a
  side table operand names. Units of the libraries Metorex carries are left
  out.
