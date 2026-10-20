# Plan: an API for the syntax tree

A Metorex program can read its own code today. `Method#body`, `Proc#statements`, and `parse` return syntax tree nodes as hashes built by `src/vm/native_methods/ast_methods.rs`. It cannot write code back, and the hashes leave out much of the tree:

- The serializer handles 22 of the 41 expression types and 10 of the 30 statement types. The rest come out as `{"type" => "Unknown"}`.
- `InterpolatedString` leaves out its parts, `MethodDef` leaves out its body, `MethodCall` leaves out its trailing block, and lambda and method parameters keep only their names.
- No node carries its source position.

This plan gives programs node objects in place of hashes. Each object wraps the parser's own `Expression`, `Statement`, or nested structure, so nothing is copied into a second format and nothing can be left out. Nodes are immutable. A program reads them, builds new ones with typed constructors or `copy`, walks and rewrites trees, runs them, and installs them as methods. Code built from nodes behaves the same as the code would if it had been written in a file.

Every item keeps `cargo test` and `scripts/run_ruby_spec.sh` passing. Every Ruby method keeps its Ruby behavior. The new operations get Metorex-only names, as `get_source` and `parse` have.

## Design decisions

These come first because every class, reader, and constructor is written against them.

- [ ] Class layout: the namespace (`Metorex::AST` or another), one class per node type and per nested structure (`Parameter`, each `MatchPattern` variant, `HashPatternRest`, `MatchCase`, `ExprMatchCase`, `RescueClause`, `ElsifBranch`, `InterpolationPart`), the parent classes (`Node`, `Expression`, `Statement`, `Pattern`), and reader names taken from the Rust field names. Where an expression and a statement share a name (`If`, `Unless`), the decision records how the two classes are told apart
  - Acceptance: `docs/meta/classes.md` lists every class with its parent, readers, and the Rust type it wraps
- [ ] Field values: what a reader returns for each Rust field type. Operators are Symbols (`:+`, `:<=>`), names are Symbols or Strings, lists are frozen Arrays, an absent optional field is `nil`, and a position is an `AST::Position` with `line`, `column`, and `offset`
  - Acceptance: the mapping for every Rust field type is in `docs/meta/classes.md`
- [ ] Equality and identity: whether `==` compares positions, what `hash` and `eql?` do so nodes work as Hash keys, and whether reading the same child twice returns the same object
  - Acceptance: recorded in `docs/meta/classes.md` with the reason for each choice
- [ ] Positions of built nodes: what a node built by a constructor reports in a backtrace, `__LINE__`, `__FILE__`, `Method#source_location`, TracePoint events, and coverage
  - Acceptance: `docs/meta/positions.md` records the file name and line each of those shows for a built node, with the reason for the choice

## Groundwork

- [ ] Node values: a heap value kind that holds an `Rc` of a parser node or nested structure, with its class looked up from the node's variant. Child readers wrap the child when it is read, with no copy of the Rust node
  - Acceptance: a test parses a method, reads its body through the wrapper, and checks the wrapped `Rc` points at the method's own body. `ObjectSpace.each_object` and `GC.start` handle the new kind
- [ ] One table for every node type: the classes, readers, and constructors come from a single list of variants and fields, generated or derived from the syntax tree types, so a node type or field added to the parser does not compile until it has a class and a reader
  - Acceptance: a compile-fail test shows the build failing when the parser has a variant or field the table lacks
- [ ] Constructors check their arguments: `AST::BinaryOp.new(op: :+, left: a, right: b)` raises `TypeError` naming the class, the keyword, and the expected type when an argument is wrong, and `ArgumentError` for a missing or unknown keyword
  - Acceptance: tests pass a wrong type, a missing keyword, and an unknown keyword to one class of each parent, and check each message
- [ ] `copy`: `node.copy(**fields)` returns a new node with those fields replaced, checked as a constructor checks them
  - Acceptance: tests check `node.copy` with no arguments equals `node`, and a replaced field reads back as the new value

## Node classes

Each item adds the classes, readers, and constructors for its node types. The acceptance for each is a test that parses `tests/_examples/meta/nodes_<group>.rb`, a file that uses every node type in the group, rebuilds every node through its constructor from the values its readers return, and compares the rebuilt node with the parser's node, positions included. The example prints one node of each type.

- [ ] Literals, variables, and names: `IntLiteral`, `FloatLiteral`, `BigIntLiteral`, `StringLiteral`, `InterpolatedString` with its parts, `RegexLiteral`, `BoolLiteral`, `NilLiteral`, `Symbol`, `Identifier`, `InstanceVariable`, `ClassVariable`, `GlobalVariable`, `MagicFile`, `MagicLine`, `MagicDir`, `SelfExpr`, `TopLevelConstant`, `ScopeResolution`
- [ ] Calls, operators, and collections: `Call`, `MethodCall` with its trailing block, `Super`, `Yield`, `Splat`, `KeywordSplat`, `BlockArg`, `Lambda` with its parameters, `BinaryOp`, `UnaryOp`, `Index`, `Array`, `Dictionary`, `Range`, `Grouped`, `Defined`, and `Parameter`
- [ ] Control flow: the `If`, `Unless`, `Case`, `BeginRescue`, and `PatternTest` expressions, and the `If`, `Unless`, `While`, `DoWhile`, `For`, `Return`, `Break`, `Continue`, `Redo`, `Retry`, `Raise`, `Begin`, `BeginBlock`, and `Block` statements, with `ElsifBranch` and `RescueClause`
- [ ] Pattern matching: `Match` and `CaseIn`, with `MatchCase`, `ExprMatchCase`, every `MatchPattern` variant, and `HashPatternRest`
- [ ] Definitions and assignment: `FunctionDef` and `MethodDef` with body and parameters, `ClassDef`, `ModuleDef`, `SingletonClass`, `Include`, `Extend`, `Alias`, `AttrReader`, `AttrWriter`, `AttrAccessor`, `Assignment`, `MultipleAssignment`, `DeclareLocals`, and the `Expression` statement
- [ ] Every file: the rebuild test passes on every `.rb` file under `tests/_examples`

## Getting nodes

- [ ] `AST.parse(source)` returns an array of statement nodes, and raises `SyntaxError` with the parser's message and position for invalid source
  - Acceptance: `tests/_examples/meta/parse.rb` parses a method definition and prints its name and parameters, and parses invalid source and prints the error
- [ ] `Method#ast` and `Proc#ast` return the `MethodDef` node of a method and the `Lambda` node of a Proc, wrapping the bodies they run
  - Acceptance: `tests/_examples/meta/method_ast.rb` prints `Invoice#total`'s operator, parameters, and position, and does the same for a block
- [ ] `quote { ... }` returns the statements of the block as nodes without running it
  - Acceptance: `tests/_examples/meta/quote.rb` quotes `a + b` with neither `a` nor `b` defined and prints the node
- [ ] `node.source` returns the source text the node was parsed from, or `nil` for a built node
  - Acceptance: `tests/_examples/meta/source.rb` prints the source of a method body and of one argument of a call in it

## Walking and rewriting

- [ ] `children` returns a node's child nodes in source order, and `each_node` yields the node and every node below it, depth first
  - Acceptance: `tests/_examples/meta/walk.rb` counts the `MethodCall` nodes in a parsed file and prints their method names
- [ ] `AST::Visitor`: a subclass defines `visit_binary_op` and the rest, one method per node class, and the default for each visits the children
  - Acceptance: `tests/_examples/meta/visitor.rb` collects every local variable a method reads
- [ ] `transform { |node| ... }` rebuilds the tree bottom up, using the block's return value in place of each node, and returns the new tree
  - Acceptance: `tests/_examples/meta/transform.rb` replaces every `IntLiteral` 5 with 10 in a method body and prints both trees' source
- [ ] Pattern matching: `deconstruct_keys` and `deconstruct` on every node class, so `case node in AST::BinaryOp[op: :+, right: AST::IntLiteral[value: 5]]` matches
  - Acceptance: `tests/_examples/meta/match.rb` finds every `x + 5` in a method body with a `case`/`in`

## Running and installing

- [ ] `node.evaluate(binding = caller's binding)` runs a statement node or an array of them and returns the last value, compiled through `compile_body` as `eval` is. Locals the nodes assign are visible to later code in the same binding
  - Acceptance: `tests/_examples/meta/evaluate.rb` builds a `BinaryOp` tree with constructors and prints its value, evaluates a `Lambda` node and calls the Proc, evaluates a `ClassDef` node and calls a method on the class, and shows the backtrace of a built `Raise` node with the position `docs/meta/positions.md` records
- [ ] `Module#define_method_from(method_def_node)` installs the method through the path `def` uses, so the method caches, compiled units, and JIT code for the old body are dropped with it
  - Acceptance: `tests/_examples/meta/install.rb` reads `Invoice#total`'s tree, changes `+ 5` to `+ 10`, installs it, and prints the new total. A test calls a method in a loop until its body is compiled to machine code, installs a new body, and checks the next call returns the new body's value
- [ ] `node.to_source` prints a tree as Ruby source, for display
  - Acceptance: for every `.rb` file under `tests/_examples`, `AST.parse(tree.to_source)` equals the tree with positions left out of the comparison

## Compatibility

- [ ] `node.to_h` returns the node as a hash with every field, and `Method#body`, `Proc#statements`, and `parse` return `to_h` hashes, so the keys programs read today (`"type"`, `"op"`, `"left"`, `"right"`, `"name"`, `"value"`, and the rest) keep their meaning. The hand-written serializer in `ast_methods.rs` is removed
  - Acceptance: `tests/_examples/site/meta.rb`, `meta_block.rb`, and `meta_parse.rb` print the same output as before, and `ast_methods.rs` no longer exists

## Documentation

- [ ] metorex.org: the "Code as objects" page documents the node classes, reading, building, walking, rewriting, pattern matching, running, and installing, with a reference page listing every class and its readers. The home page says a program can read and write its syntax tree at runtime. Each sample is a `tests/_examples/site/meta_*.rb` file with its output recorded by the examples runner
  - Acceptance: `blogin build` and `bin/check-links.raku` pass, and each sample on the page matches its recorded output
