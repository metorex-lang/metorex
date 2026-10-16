# Value representation

## Decision

- A value is 8 bytes, laid out as a C extension's `VALUE` is:
  - an Integer that fits in 63 bits, shifted left with its low bit set
  - a Float in the common exponent range, rotated into a flonum tagged `0b10`
  - `false`, `nil`, and `true` as the constants `include/ruby/ruby.h` already
    gives C extensions: `0x00`, `0x08`, and `0x14`. CRuby 4.0's headers make
    `nil` `0x04`. Extensions loaded by Metorex are compiled against Metorex's
    headers, so its own constant stands.
  - a static Symbol as its id shifted left with the tag `0x0c`, CRuby's
    `RUBY_SYMBOL_FLAG`
  - anything else as a pointer to an object on the heap, aligned to 8 bytes
- A String holds its bytes and an encoding tag, not a Rust `String`.
- A C extension's `VALUE` is the value's own bits. The handle table in
  `src/vm/capi/handles.rs` goes.

How a heap object is freed, and how one a C extension holds stays alive, is
the memory management decision's.

## Alternatives

- **The `Object` enum as it is.** Every value is 32 bytes, so a stack slot,
  a local, an Array element, and an instance variable each take 32 bytes, and
  copying or dropping a value matches on its variant.
- **NaN-boxing**, which keeps every double unboxed. A C extension's macros
  test the low bits of a `VALUE` for fixnums and special constants, so a
  NaN-boxed value would need converting at every C API call. CRuby's flonum
  keeps the doubles programs use unboxed: no kernel of the prototype put a
  Float on the heap, which the last column of the table below counts.
- **A String as Rust text.** It holds each byte of a binary string as a
  character, so bytes 0x80 to 0xFF take two bytes of UTF-8 each, and finding
  the byte at an offset means reading the text up to it.

## The prototype

`prototypes/values` runs one stack interpreter over both values: the same
instructions, the same frames, and the same object body, a reference-counted
vector of slots, so the value is the only difference. The `Object` side uses
a copy of the enum `Object` was before it became the tagged value: its
numeric and Array variants, and its Range variant, which gave it its 32
bytes. Its kernels are the inner loops of
ruby-bench benchmarks, written in the prototype's instructions.

```sh
cargo run --release --manifest-path prototypes/values/Cargo.toml
```

Recorded on 2026-10-15 on an Apple M5 Max running macOS 27.0.1, with rustc
1.96.0. Each time is the fastest of five runs.

`Object` is 32 bytes, and the tagged value is 8.

| Kernel | From | `Object` enum | Tagged value | Enum ÷ tagged | Floats on the heap |
| :-- | :-- | --: | --: | --: | --: |
| fib | fib | 80.4 ms | 50.2 ms | 1.60× | 0 |
| while_loop | loops-times | 1639.2 ms | 959.8 ms | 1.71× | 0 |
| getivar | getivar | 746.2 ms | 440.6 ms | 1.69× | 0 |
| object_new | object-new-initialize | 643.8 ms | 496.7 ms | 1.30× | 0 |
| float_arithmetic | nbody | 809.7 ms | 545.9 ms | 1.48× | 0 |
| array_sum | matmul | 381.2 ms | 217.8 ms | 1.75× | 0 |

The array in `array_sum` holds 5 million Integers, 160 MB as `Object`s and
40 MB as tagged values.

### Strings

`prototypes/values/src/strings.rs` replays protoboeuf-encode's buffer
operations on 2,000 messages: appending tag and varint bytes, reading
`bytesize` before and after each of 40 submessages, writing each one's length
into a placeholder with `setbyte`, and reading the result back with `getbyte`.
The one-character-per-byte layout does what Metorex does now: `bytesize`
copies the kept byte spelling, and `setbyte` spells out every byte and writes
the text back.

| String layout | Time |
| :-- | --: |
| One character per byte (now) | 170.9 ms |
| Bytes with an encoding tag | 1.6 ms |
| Ratio | 105.21× |

Under the current layout each `bytesize` and `setbyte` takes time in
proportion to the buffer's length, so a message costs time in proportion to
the square of its length. protoboeuf-encode takes 1,656 times as long under
Metorex as under CRuby in the README's table, and a `sample` profile of it
taken on 2026-10-15 found 1,875 of the program thread's samples on top of the
stack in `string_to_bytes`, which `bytesize`, `setbyte`, and `pack(buffer:)` call, more than
in every other function together.

### The C API

`prototypes/values/src/handles.rs` hands 1,000 Arrays to C and back 10,000
times each. The handle table side copies `to_value` and `to_object` from
`src/vm/capi/handles.rs`: a hash map lookup by address, and an index into
the table that keeps the object alive.

| An object to a VALUE and back | Per round trip |
| :-- | --: |
| Handle table (now) | 12.74 ns |
| The tagged value's bits | 0.34 ns |

The handle table also keeps every object a C extension was ever handed until
the extension releases it. Under the tagged value, what keeps such an object
alive is the memory management decision's.

## In Metorex

`Object`, in `src/object/types.rs`, is the 8-byte value described above.

- Code reads a value by matching on `object.view()`, a `View` of the value's
  parts borrowed from it. `View`'s variants have the names `Object`'s
  variants had, and the functions that make a value have them too, so
  `Object::Int(number)` makes an Integer as it did before.
- A heap value points at a `Body` held by an `Rc`. The `Body` holds the
  part's own `Rc`, such as the `Rc<Class>` or `Rc<RefCell<Vec<Object>>>`
  that code across `src/` keeps and shares, so a heap value takes two
  allocations. The memory management decision gives each object a header of
  its own, which removes the second.
- Copies of a value share one `Body`, so the count of a part's `Rc` does not
  count them. `Object::references` counts the copies and every other holder
  of the part. The C API's collection reads it to tell whether anything
  besides its pinned copy reaches a Data object, and a Hash reads it before
  writing in place to the record of its key objects that its copies share.
- A C extension's `VALUE` is the value's bits, made in
  `src/vm/capi/values.rs`. Copies of one object can point at different
  `Body` cells, so the first `Body` an object is handed to C through is the
  one C is handed for it from then on, which keeps two `VALUE`s for one
  object equal.
- A Symbol is its id over the tag `0x0c`. An ASCII name in an
  ASCII-compatible encoding gets one id whatever the encoding, so a Symbol C
  makes from ASCII bytes has the bits Ruby's has. The id indexes a table of names
  kept for each thread, and a name stays in the table for the life of the
  thread, where CRuby frees a Symbol made at runtime once nothing refers to
  it.
- An Integer that fits in an i64 and not in 63 bits is held on the heap.
- A String, in `src/object/string_value.rs`, holds its bytes and an encoding
  tag. `bytesize`, `getbyte`, `setbyte`, `bytesplice`, `unpack`, and
  `pack(buffer:)` read and write the bytes in place. `force_encoding` keeps
  the bytes and changes the tag. Methods that work on characters read the
  bytes as text through the encoding: as UTF-8, or one character per byte
  for a binary string, a UTF-16 or UTF-32 string, and a string whose bytes
  are not valid in its text encoding. UTF-8 is read from the bytes without a
  copy. Text read one character per byte is built on first use when a byte
  is above 0x7F, and dropped when the bytes change.
