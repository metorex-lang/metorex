//! TracePoints C makes with `rb_tracepoint_new`, whose hook is a C function,
//! and enabling, disabling and asking about any TracePoint.

use super::calls::{call, top_level_module};
use super::methods::{CFunction, EVENT_HOOK_ARITY, RAW_EVENT_HOOK_ARITY, TRACEPOINT_HOOK_ARITY};
use super::values::{QFALSE, QNIL, Value, to_object, to_value};
use crate::object::{Method, Object};
use std::cell::RefCell;
use std::rc::Rc;

/// The `RUBY_EVENT_*` bit for each event a TracePoint can be made for.
const EVENTS: [(u32, &str); 15] = [
    (0x0001, "line"),
    (0x0002, "class"),
    (0x0004, "end"),
    (0x0008, "call"),
    (0x0010, "return"),
    (0x0020, "c_call"),
    (0x0040, "c_return"),
    (0x0080, "raise"),
    (0x0100, "b_call"),
    (0x0200, "b_return"),
    (0x0400, "thread_begin"),
    (0x0800, "thread_end"),
    (0x1000, "fiber_switch"),
    (0x2000, "script_compiled"),
    (0x4000, "rescue"),
];

/// Makes a TracePoint for the events `flags` names whose hook calls
/// `function` with the TracePoint and `data`. MRI ignores the target
/// thread, and so does this.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracepoint_new(
    _target_thread: Value,
    flags: u32,
    function: *const (),
    data: usize,
) -> Value {
    to_value(&tracepoint_calling(
        flags,
        function,
        TRACEPOINT_HOOK_ARITY,
        data,
    ))
}

/// A TracePoint for the events `flags` names whose hook is the C function
/// at `function`, called as `arity` lays its arguments out, with `data`.
fn tracepoint_calling(flags: u32, function: *const (), arity: i32, data: usize) -> Object {
    let events: Vec<Object> = EVENTS
        .iter()
        .filter(|(bit, _)| flags & bit != 0)
        .map(|(_, name)| Object::symbol(*name))
        .collect();
    let trace = call(top_level_module("TracePoint"), "allocate", Vec::new());
    let mut hook = Method::new("call".to_string(), Vec::new(), Vec::new());
    hook.provenance.bound_self = Some(Box::new(trace.clone()));
    hook.provenance.c_function = Some(CFunction {
        address: function as usize,
        arity,
        data,
    });
    let hook = Object::Method(Rc::new(hook));
    let events = Object::array(events);
    call(
        trace.clone(),
        "__send__",
        vec![Object::symbol("__set_up__"), events.clone(), hook],
    );
    // A TracePoint made for no events traces none, where `TracePoint.new`
    // with none traces every one.
    call(
        trace.clone(),
        "instance_variable_set",
        vec![Object::symbol("@events"), events],
    );
    trace
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracepoint_enable(trace: Value) -> Value {
    call(to_object(trace), "enable", Vec::new());
    QNIL
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracepoint_disable(trace: Value) -> Value {
    call(to_object(trace), "disable", Vec::new());
    QNIL
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracepoint_enabled_p(trace: Value) -> Value {
    to_value(&call(to_object(trace), "enabled?", Vec::new()))
}

/// What a C hook is handed for the event being traced, which is the
/// TracePoint carrying it.
#[repr(C)]
pub struct TraceArgument {
    _opaque: [u8; 0],
}

fn traced(trace_arg: *mut TraceArgument) -> Object {
    to_object(trace_arg as Value)
}

fn read(trace_arg: *mut TraceArgument, name: &str) -> Value {
    to_value(&call(traced(trace_arg), name, Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracearg_from_tracepoint(trace: Value) -> *mut TraceArgument {
    trace as *mut TraceArgument
}

/// The `RUBY_EVENT_*` bit of the event being traced.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracearg_event_flag(trace_arg: *mut TraceArgument) -> u32 {
    let event = call(traced(trace_arg), "event", Vec::new());
    EVENTS
        .iter()
        .find(|(_, name)| event == Object::symbol(*name))
        .map_or(0, |(bit, _)| *bit)
}

macro_rules! trace_readers {
    ($($function:ident => $name:literal),* $(,)?) => {$(
        #[unsafe(no_mangle)]
        pub extern "C-unwind" fn $function(trace_arg: *mut TraceArgument) -> Value {
            read(trace_arg, $name)
        }
    )*};
}

trace_readers! {
    rb_tracearg_event => "event",
    rb_tracearg_lineno => "lineno",
    rb_tracearg_path => "path",
    rb_tracearg_parameters => "parameters",
    rb_tracearg_method_id => "method_id",
    rb_tracearg_callee_id => "callee_id",
    rb_tracearg_defined_class => "defined_class",
    rb_tracearg_binding => "binding",
    rb_tracearg_self => "self",
    rb_tracearg_return_value => "return_value",
    rb_tracearg_raised_exception => "raised_exception",
    rb_tracearg_eval_script => "eval_script",
    rb_tracearg_instruction_sequence => "instruction_sequence",
}

/// Calls a hook `rb_add_event_hook` was handed, which takes the event's
/// bit, the data VALUE, the receiver, the method's ID, and the class the
/// method was defined in. MRI hands over the ID and the class only for a C
/// method's events, and 0 and false for any other event.
pub(super) fn call_event_hook(address: *const (), data: Value, trace: Value) {
    let trace_arg = rb_tracearg_from_tracepoint(trace);
    let flag = rb_tracearg_event_flag(trace_arg);
    let (method_id, class) = if flag & C_METHOD_EVENTS != 0 {
        let method = rb_tracearg_method_id(trace_arg);
        (
            super::symbols::rb_sym2id(method),
            rb_tracearg_defined_class(trace_arg),
        )
    } else {
        (0, QFALSE)
    };
    // SAFETY: `rb_add_event_hook` declares the function as an
    // `rb_event_hook_func_t`.
    let function: extern "C-unwind" fn(u32, Value, Value, Value, Value) =
        unsafe { std::mem::transmute(address) };
    function(flag, data, rb_tracearg_self(trace_arg), method_id, class);
}

/// The `RUBY_EVENT_C_CALL` and `RUBY_EVENT_C_RETURN` bits.
const C_METHOD_EVENTS: u32 = 0x0020 | 0x0040;

/// The `rb_event_hook_flag_t` bit asking for the hook to be handed the
/// data VALUE and the trace argument in place of the event's details.
const HOOK_FLAG_RAW_ARG: u32 = 0x04;

/// A hook C added, by its function and data VALUE, and the TracePoint that
/// calls it.
struct EventHook {
    function: usize,
    data: Value,
    trace: Object,
}

thread_local! {
    static EVENT_HOOKS: RefCell<Vec<EventHook>> = const { RefCell::new(Vec::new()) };
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_add_event_hook(function: *const (), events: u32, data: Value) {
    rb_add_event_hook2(function, events, data, 0x01);
}

/// Calls `function` for each of the events `events` names from now until
/// the hook is removed.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_add_event_hook2(
    function: *const (),
    events: u32,
    data: Value,
    hook_flag: u32,
) {
    let arity = if hook_flag & HOOK_FLAG_RAW_ARG != 0 {
        RAW_EVENT_HOOK_ARITY
    } else {
        EVENT_HOOK_ARITY
    };
    let trace = tracepoint_calling(events, function, arity, data);
    call(trace.clone(), "enable", Vec::new());
    EVENT_HOOKS.with(|hooks| {
        hooks.borrow_mut().push(EventHook {
            function: function as usize,
            data,
            trace,
        })
    });
}

/// The `data` that stands for any data VALUE a hook was added with.
const ANY_DATA: Value = super::values::QUNDEF;

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_remove_event_hook(function: *const ()) -> i32 {
    rb_remove_event_hook_with_data(function, ANY_DATA)
}

/// Removes each hook added with `function` and `data`, answering how many
/// there were.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_remove_event_hook_with_data(function: *const (), data: Value) -> i32 {
    let removed: Vec<EventHook> = EVENT_HOOKS.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        let (removed, kept) = std::mem::take(&mut *hooks).into_iter().partition(|hook| {
            hook.function == function as usize && (data == ANY_DATA || hook.data == data)
        });
        *hooks = kept;
        removed
    });
    for hook in &removed {
        call(hook.trace.clone(), "disable", Vec::new());
    }
    removed.len() as i32
}
