//! The core library's Ruby source, in the order it is read. The build
//! script reads this file too, to parse the source once at build time.

#[path = "collections.rs"]
mod collections;
#[path = "core_types.rs"]
mod core_types;
#[path = "encoding.rs"]
mod encoding;
#[path = "enumerable.rs"]
mod enumerable;
#[path = "enumerator.rs"]
mod enumerator;
#[path = "enumerator_lazy.rs"]
mod enumerator_lazy;
#[path = "enumerator_parts.rs"]
mod enumerator_parts;
#[path = "file.rs"]
mod file;
#[path = "io.rs"]
mod io;
#[path = "io_reading.rs"]
mod io_reading;
#[path = "io_streams.rs"]
mod io_streams;
#[path = "io_writing.rs"]
mod io_writing;
#[path = "kernel_argf.rs"]
mod kernel_argf;
#[path = "match_data_thread.rs"]
mod match_data_thread;
#[path = "object_space_gc.rs"]
mod object_space_gc;
#[path = "proc_random.rs"]
mod proc_random;
#[path = "process.rs"]
mod process;
#[path = "ractor.rs"]
mod ractor;
#[path = "ruby_vm.rs"]
mod ruby_vm;
#[path = "string_io.rs"]
mod string_io;
#[path = "threading.rs"]
mod threading;
#[path = "time.rs"]
mod time;
#[path = "time_reading.rs"]
mod time_reading;
#[path = "tracepoint_marshal.rs"]
mod tracepoint_marshal;
#[path = "warning_math_numeric.rs"]
mod warning_math_numeric;

/// The Ruby source evaluated into every fresh VM, in the order it is read.
pub(crate) const PRELUDE_CHUNKS: &[&str] = &[
    warning_math_numeric::SOURCE,
    enumerable::SOURCE,
    match_data_thread::SOURCE,
    string_io::SOURCE,
    enumerator::SOURCE,
    enumerator_lazy::SOURCE,
    enumerator_parts::SOURCE,
    collections::SOURCE,
    time::SOURCE,
    time_reading::SOURCE,
    proc_random::SOURCE,
    process::SOURCE,
    io::SOURCE,
    io_reading::SOURCE,
    io_writing::SOURCE,
    file::SOURCE,
    tracepoint_marshal::SOURCE,
    core_types::SOURCE,
    kernel_argf::SOURCE,
    threading::SOURCE,
    ractor::SOURCE,
    ruby_vm::SOURCE,
    object_space_gc::SOURCE,
    encoding::SOURCE,
    io_streams::SOURCE,
];
