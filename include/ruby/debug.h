#ifndef RUBY_DEBUG_H
#define RUBY_DEBUG_H 1

#include "ruby/ruby.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t rb_event_flag_t;

#define RUBY_EVENT_NONE 0x0000
#define RUBY_EVENT_LINE 0x0001
#define RUBY_EVENT_CLASS 0x0002
#define RUBY_EVENT_END 0x0004
#define RUBY_EVENT_CALL 0x0008
#define RUBY_EVENT_RETURN 0x0010
#define RUBY_EVENT_C_CALL 0x0020
#define RUBY_EVENT_C_RETURN 0x0040
#define RUBY_EVENT_RAISE 0x0080
#define RUBY_EVENT_ALL 0x00ff
#define RUBY_EVENT_B_CALL 0x0100
#define RUBY_EVENT_B_RETURN 0x0200
#define RUBY_EVENT_THREAD_BEGIN 0x0400
#define RUBY_EVENT_THREAD_END 0x0800
#define RUBY_EVENT_FIBER_SWITCH 0x1000
#define RUBY_EVENT_SCRIPT_COMPILED 0x2000
#define RUBY_EVENT_RESCUE 0x4000
#define RUBY_EVENT_TRACEPOINT_ALL 0xffff

VALUE rb_tracepoint_new(VALUE target_thread, rb_event_flag_t events, void (*func)(VALUE, void *),
                        void *data);
VALUE rb_tracepoint_enable(VALUE trace);
VALUE rb_tracepoint_disable(VALUE trace);
VALUE rb_tracepoint_enabled_p(VALUE trace);

typedef struct rb_trace_arg_struct rb_trace_arg_t;
rb_trace_arg_t *rb_tracearg_from_tracepoint(VALUE trace);
rb_event_flag_t rb_tracearg_event_flag(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_event(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_lineno(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_path(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_parameters(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_method_id(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_callee_id(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_defined_class(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_binding(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_self(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_return_value(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_raised_exception(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_eval_script(rb_trace_arg_t *trace_arg);
VALUE rb_tracearg_instruction_sequence(rb_trace_arg_t *trace_arg);

typedef void (*rb_event_hook_func_t)(rb_event_flag_t event, VALUE data, VALUE self, ID method_id, VALUE klass);
typedef enum {
  RUBY_EVENT_HOOK_FLAG_SAFE = 0x01,
  RUBY_EVENT_HOOK_FLAG_DELETED = 0x02,
  RUBY_EVENT_HOOK_FLAG_RAW_ARG = 0x04
} rb_event_hook_flag_t;
void rb_add_event_hook(rb_event_hook_func_t function, rb_event_flag_t events, VALUE data);
void rb_add_event_hook2(rb_event_hook_func_t function, rb_event_flag_t events, VALUE data,
                        rb_event_hook_flag_t hook_flag);
int rb_remove_event_hook(rb_event_hook_func_t function);
int rb_remove_event_hook_with_data(rb_event_hook_func_t function, VALUE data);

typedef struct rb_debug_inspector_struct rb_debug_inspector_t;
typedef VALUE (*rb_debug_inspector_func_t)(const rb_debug_inspector_t *inspector, void *data);
VALUE rb_debug_inspector_open(rb_debug_inspector_func_t function, void *data);
VALUE rb_debug_inspector_frame_self_get(const rb_debug_inspector_t *inspector, long index);
VALUE rb_debug_inspector_frame_class_get(const rb_debug_inspector_t *inspector, long index);
VALUE rb_debug_inspector_frame_binding_get(const rb_debug_inspector_t *inspector, long index);
VALUE rb_debug_inspector_frame_iseq_get(const rb_debug_inspector_t *inspector, long index);
VALUE rb_debug_inspector_backtrace_locations(const rb_debug_inspector_t *inspector);
#define RB_DEBUG_INSPECTOR_FRAME_DEPTH(dc, index) rb_debug_inspector_frame_depth(dc, index)
VALUE rb_debug_inspector_frame_depth(const rb_debug_inspector_t *inspector, long index);
VALUE rb_debug_inspector_current_depth(void);

#ifdef __cplusplus
}
#endif

#endif
