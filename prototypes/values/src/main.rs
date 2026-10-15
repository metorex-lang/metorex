//! Times the prototypes and prints Markdown tables for
//! docs/engine/value-representation.md, docs/engine/memory-management.md,
//! docs/engine/frames-and-calls.md, and docs/engine/instruction-format.md.
//! Each time is the fastest of five runs.

mod calls;
mod dispatch;
mod enum_value;
mod fibers;
mod handles;
mod kernels;
mod machine;
mod strings;
mod tagged_value;

use enum_value::EnumValue;
use machine::Representation;
use std::time::{Duration, Instant};
use tagged_value::TaggedValue;

const RUNS: usize = 5;

/// The fastest of `RUNS` runs of `work`, and what the last run answered.
fn fastest<T>(mut work: impl FnMut() -> T) -> (Duration, T) {
    let mut best = Duration::MAX;
    let mut answer = None;
    for _ in 0..RUNS {
        let started = Instant::now();
        let made = work();
        best = best.min(started.elapsed());
        answer = Some(made);
    }
    (best, answer.unwrap())
}

fn milliseconds(duration: Duration) -> String {
    format!("{:.1} ms", duration.as_secs_f64() * 1000.0)
}

fn ratio(slower: Duration, faster: Duration) -> String {
    format!("{:.2}×", slower.as_secs_f64() / faster.as_secs_f64())
}

/// One kernel under one way of managing cells: its answer, its fastest
/// time, the most cells alive at once in the last run, and the collections
/// that run made.
fn memory_run<const MODE: u8>(kernel: &kernels::Kernel) -> (String, Duration, usize, usize) {
    let mut best = Duration::MAX;
    let mut answer = String::new();
    let mut peak_and_collections = (0, 0);
    for _ in 0..RUNS {
        tagged_value::reset_counts(MODE);
        let started = Instant::now();
        answer = tagged_value::Tagging::<MODE>::describe(&machine::run::<
            tagged_value::Tagging<MODE>,
        >(&kernel.program));
        best = best.min(started.elapsed());
        peak_and_collections = tagged_value::peak_cells_and_collections(MODE);
    }
    (answer, best, peak_and_collections.0, peak_and_collections.1)
}

fn main() {
    println!(
        "Value sizes: `Object` {} bytes, tagged value {} bytes, `Option<Object>` {} bytes.\n",
        std::mem::size_of::<enum_value::Object>(),
        std::mem::size_of::<tagged_value::Tagged>(),
        std::mem::size_of::<Option<enum_value::Object>>(),
    );

    println!(
        "| Kernel | From | `Object` enum | Tagged value | Enum ÷ tagged | Floats on the heap |"
    );
    println!("| :-- | :-- | --: | --: | --: | --: |");
    for kernel in kernels::all() {
        let (enum_time, enum_answer) =
            fastest(|| EnumValue::describe(&machine::run::<EnumValue>(&kernel.program)));
        tagged_value::HEAP_FLOATS.set(0);
        let (tagged_time, tagged_answer) =
            fastest(|| TaggedValue::describe(&machine::run::<TaggedValue>(&kernel.program)));
        let heap_floats = tagged_value::HEAP_FLOATS.get();
        assert_eq!(
            enum_answer, tagged_answer,
            "{} answered differently",
            kernel.name
        );
        println!(
            "| {} | {} | {} | {} | {} | {} |",
            kernel.name,
            kernel.from,
            milliseconds(enum_time),
            milliseconds(tagged_time),
            ratio(enum_time, tagged_time),
            heap_floats
        );
    }

    println!(
        "\n| Kernel | Counted | Counted, cells | Cycle collected | Cycle collected, cells | Collections | Traced | Traced, cells | Collections |"
    );
    println!("| :-- | --: | --: | --: | --: | --: | --: | --: | --: |");
    for kernel in kernels::memory() {
        let counted = memory_run::<{ tagged_value::COUNTED }>(&kernel);
        let cycles = memory_run::<{ tagged_value::CYCLES }>(&kernel);
        let traced = memory_run::<{ tagged_value::TRACED }>(&kernel);
        assert!(
            counted.0 == cycles.0 && cycles.0 == traced.0,
            "{} answered differently",
            kernel.name
        );
        println!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            kernel.name,
            milliseconds(counted.1),
            counted.2,
            milliseconds(cycles.1),
            cycles.2,
            cycles.3,
            milliseconds(traced.1),
            traced.2,
            traced.3
        );
    }

    let lots = 2_000;
    let (char_time, char_sum) = fastest(|| strings::encode::<strings::CharString>(lots));
    let (byte_time, byte_sum) = fastest(|| strings::encode::<strings::ByteString>(lots));
    assert_eq!(
        char_sum, byte_sum,
        "the String layouts wrote different bytes"
    );
    println!("\n| String layout, protoboeuf-encode's buffer operations | Time |");
    println!("| :-- | --: |");
    println!(
        "| One character per byte (now) | {} |",
        milliseconds(char_time)
    );
    println!(
        "| Bytes with an encoding tag | {} |",
        milliseconds(byte_time)
    );
    println!("| Ratio | {} |", ratio(char_time, byte_time));

    let (objects, rounds) = (1_000, 10_000);
    let (handle_time, handle_count) = fastest(|| handles::through_handles(objects, rounds));
    let (tagged_time, tagged_count) = fastest(|| handles::through_tagged(objects, rounds));
    assert_eq!(handle_count, tagged_count);
    let per_round_trip = |time: Duration| {
        format!(
            "{:.2} ns",
            time.as_secs_f64() * 1e9 / (objects * rounds) as f64
        )
    };
    println!("\n| C API, an object to a VALUE and back | Per round trip |");
    println!("| :-- | --: |");
    println!("| Handle table (now) | {} |", per_round_trip(handle_time));
    println!(
        "| The tagged value's bits | {} |",
        per_round_trip(tagged_time)
    );

    // block_calls' total passes the largest Integer held inline past 2.4
    // million steps, and every addition after that makes a Bignum.
    let call_kernels = [
        calls::method_calls(10_000_000),
        calls::block_calls(2_000_000),
    ];
    println!(
        "\n| Workload | Calls | Heap frames | Contiguous stack | Heap ÷ stacked | Heap frames, per call | Contiguous stack, per call |"
    );
    println!("| :-- | --: | --: | --: | --: | --: | --: |");
    for kernel in &call_kernels {
        let (heap_time, heap_answer) =
            fastest(|| TaggedValue::describe(&calls::HeapFrames::run(&kernel.units)));
        let (stacked_time, stacked_answer) =
            fastest(|| TaggedValue::describe(&calls::Stacked::run(&kernel.units)));
        assert_eq!(
            heap_answer, stacked_answer,
            "{} answered differently",
            kernel.name
        );
        let per_call =
            |time: Duration| format!("{:.1} ns", time.as_secs_f64() * 1e9 / kernel.calls as f64);
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            kernel.name,
            kernel.calls,
            milliseconds(heap_time),
            milliseconds(stacked_time),
            ratio(heap_time, stacked_time),
            per_call(heap_time),
            per_call(stacked_time)
        );
    }

    let count = 100_000;
    let (metorex_time, metorex_sum) =
        fastest(|| fibers::make_and_finish(count, fibers::METOREX_STACK_BYTES));
    let (cruby_time, cruby_sum) =
        fastest(|| fibers::make_and_finish(count, fibers::CRUBY_STACK_BYTES));
    assert_eq!(metorex_sum, cruby_sum);
    let (segment_time, _) = fastest(|| fibers::make_segments(count));
    let switch_count = 10_000_000;
    let (switch_time, _) = fastest(|| fibers::switches(switch_count));
    let per_fiber = |time: Duration| format!("{:.0} ns", time.as_secs_f64() * 1e9 / count as f64);
    println!("\n| A fiber made, suspended once, and finished | Per fiber |");
    println!("| :-- | --: |");
    println!(
        "| Native stack of 32 MB (now) | {} |",
        per_fiber(metorex_time)
    );
    println!("| Native stack of 512 KB | {} |", per_fiber(cruby_time));
    println!(
        "| A value stack segment of 128 KB, made and dropped | {} |",
        per_fiber(segment_time)
    );
    println!(
        "\nA resume and the suspension it runs to: {:.1} ns.",
        switch_time.as_secs_f64() * 1e9 / switch_count as f64
    );

    println!(
        "\nInstruction sizes: stack {} bytes, register {} bytes, decoded register {} bytes.",
        std::mem::size_of::<dispatch::StackOp>(),
        std::mem::size_of::<dispatch::RegisterOp>(),
        std::mem::size_of::<dispatch::Decoded>(),
    );
    println!(
        "\n| Kernel | Stack, byte stream | Stack | Register | Register, constant operands | Register, function pointers | Stack ÷ register, constant operands |"
    );
    println!("| :-- | --: | --: | --: | --: | --: | --: |");
    let mut instructions_run = Vec::new();
    for kernel in kernels::all() {
        let expected = TaggedValue::describe(&machine::run::<TaggedValue>(&kernel.program));
        let stack = dispatch::stack_program(&kernel);
        let registers = dispatch::register_program(kernel.name, false);
        let operands = dispatch::register_program(kernel.name, true);
        let decoded = dispatch::decode(&registers);
        let (bytes_time, bytes_answer) =
            fastest(|| TaggedValue::describe(&dispatch::run_bytes(&stack)));
        let (stack_time, stack_answer) =
            fastest(|| TaggedValue::describe(&dispatch::run_stack::<false>(&stack).0));
        let (register_time, register_answer) = fastest(|| {
            TaggedValue::describe(&dispatch::run_registers::<false, false>(&registers).0)
        });
        let (operand_time, operand_answer) =
            fastest(|| TaggedValue::describe(&dispatch::run_registers::<false, true>(&operands).0));
        let (handler_time, handler_answer) =
            fastest(|| TaggedValue::describe(&dispatch::run_handlers(&registers, &decoded)));
        for answer in [
            bytes_answer,
            stack_answer,
            register_answer,
            operand_answer,
            handler_answer,
        ] {
            assert_eq!(answer, expected, "{} answered differently", kernel.name);
        }
        println!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            kernel.name,
            milliseconds(bytes_time),
            milliseconds(stack_time),
            milliseconds(register_time),
            milliseconds(operand_time),
            milliseconds(handler_time),
            ratio(stack_time, operand_time),
        );
        instructions_run.push((
            kernel.name,
            dispatch::run_stack::<true>(&stack).1,
            dispatch::run_registers::<true, false>(&registers).1,
            dispatch::run_registers::<true, true>(&operands).1,
        ));
    }
    println!("\n| Kernel | Stack, instructions run | Register | Register, constant operands |");
    println!("| :-- | --: | --: | --: |");
    for (name, stack, registers, operands) in instructions_run {
        println!("| {name} | {stack} | {registers} | {operands} |");
    }

    // Counting every allocation from here on slows each one, so the
    // timings above run before it starts.
    unsafe { std::env::set_var("METOREX_COUNTERS", "1") };
    metorex::vm::counters::enable_from_environment();
    let allocations_per_call = |kernel: &calls::CallKernel, run: &dyn Fn()| {
        let before = metorex::vm::counters::allocations_so_far();
        run();
        let made = metorex::vm::counters::allocations_so_far() - before;
        format!("{:.2}", made as f64 / kernel.calls as f64)
    };
    println!(
        "\n| Workload | Heap frames, allocations per call | Contiguous stack, allocations per call |"
    );
    println!("| :-- | --: | --: |");
    for kernel in &call_kernels {
        let heap = allocations_per_call(kernel, &|| {
            calls::HeapFrames::run(&kernel.units);
        });
        let stacked = allocations_per_call(kernel, &|| {
            calls::Stacked::run(&kernel.units);
        });
        println!("| {} | {} | {} |", kernel.name, heap, stacked);
    }
}
