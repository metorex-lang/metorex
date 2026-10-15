//! Times the value representation prototype and prints Markdown tables for
//! docs/engine/value-representation.md. Each time is the fastest of five
//! runs.

mod enum_value;
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

fn main() {
    println!(
        "Value sizes: `Object` {} bytes, tagged value {} bytes, `Option<Object>` {} bytes.\n",
        std::mem::size_of::<metorex::object::Object>(),
        std::mem::size_of::<tagged_value::Tagged>(),
        std::mem::size_of::<Option<metorex::object::Object>>(),
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
}
