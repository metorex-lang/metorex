#include "ruby.h"
#include "ruby/debug.h"
#include "ruby/io.h"

static void record_raw(VALUE seen, const rb_trace_arg_t *trace_arg) {
  rb_trace_arg_t *arg = (rb_trace_arg_t *)trace_arg;
  VALUE entry = rb_ary_new();
  rb_ary_push(entry, rb_tracearg_event(arg));
  rb_ary_push(entry, rb_tracearg_method_id(arg));
  rb_ary_push(entry, UINT2NUM(rb_tracearg_event_flag(arg)));
  rb_ary_push(entry, rb_funcall(rb_tracearg_path(arg), rb_intern("class"), 0));
  rb_ary_push(entry, rb_obj_is_kind_of(rb_tracearg_lineno(arg), rb_cInteger));
  rb_ary_push(seen, entry);
}

static void record_details(rb_event_flag_t event, VALUE seen, VALUE self, ID method_id, VALUE klass) {
  VALUE entry = rb_ary_new();
  rb_ary_push(entry, UINT2NUM(event));
  rb_ary_push(entry, self);
  rb_ary_push(entry, method_id ? ID2SYM(method_id) : Qnil);
  rb_ary_push(entry, klass);
  rb_ary_push(seen, entry);
}

static VALUE add_raw(VALUE self, VALUE seen) {
  rb_add_event_hook2((rb_event_hook_func_t)record_raw, RUBY_EVENT_CALL, seen,
                     RUBY_EVENT_HOOK_FLAG_SAFE | RUBY_EVENT_HOOK_FLAG_RAW_ARG);
  return Qnil;
}

static VALUE add_details(VALUE self, VALUE seen) {
  rb_add_event_hook(record_details, RUBY_EVENT_CALL | RUBY_EVENT_RETURN | RUBY_EVENT_C_CALL, seen);
  return Qnil;
}

static VALUE remove_raw(VALUE self, VALUE seen) {
  return INT2NUM(rb_remove_event_hook_with_data((rb_event_hook_func_t)record_raw, seen));
}

static VALUE remove_details(VALUE self) {
  return INT2NUM(rb_remove_event_hook(record_details));
}

static VALUE resize(VALUE self, VALUE array, VALUE length) {
  return rb_ary_resize(array, NUM2LONG(length));
}

static VALUE replace(VALUE self, VALUE destination, VALUE source) {
  return rb_str_replace(destination, source);
}

static VALUE unsigned_long_long(VALUE self, VALUE number) {
  return rb_funcall(ULL2NUM(rb_big2ull(number)), rb_intern("to_s"), 0);
}

static VALUE fixnum_as_unsigned(VALUE self, VALUE number) {
  return ULONG2NUM(FIX2ULONG(number));
}

static VALUE frozen_raw(VALUE self, VALUE object) {
  return RB_OBJ_FROZEN_RAW(object) ? Qtrue : Qfalse;
}

static VALUE largest_fixnum(VALUE self) {
  return LONG2NUM(RUBY_FIXNUM_MAX);
}

static VALUE end_of_file(VALUE self) {
  if (RB_UNLIKELY(1)) rb_eof_error();
  return Qnil;
}

RUBY_FUNC_EXPORTED void Init_c_event_hooks(void) {
  VALUE cls = rb_define_class("CEventHooks", rb_cObject);
  rb_define_method(cls, "add_raw", add_raw, 1);
  rb_define_method(cls, "add_details", add_details, 1);
  rb_define_method(cls, "remove_raw", remove_raw, 1);
  rb_define_method(cls, "remove_details", remove_details, 0);
  rb_define_method(cls, "resize", resize, 2);
  rb_define_method(cls, "replace", replace, 2);
  rb_define_method(cls, "unsigned_long_long", unsigned_long_long, 1);
  rb_define_method(cls, "fixnum_as_unsigned", fixnum_as_unsigned, 1);
  rb_define_method(cls, "frozen_raw", frozen_raw, 1);
  rb_define_method(cls, "largest_fixnum", largest_fixnum, 0);
  rb_define_method(cls, "end_of_file", end_of_file, 0);
}
