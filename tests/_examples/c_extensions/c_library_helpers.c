#include <ruby/defines.h>
#include "ruby.h"
#include "ruby/st.h"
#include "ruby/util.h"

static VALUE copy(VALUE self, VALUE text) {
  char *held = ruby_strdup(StringValueCStr(text));
  VALUE copied = rb_str_new_cstr(held);
  xfree(held);
  return copied;
}
static VALUE compare(VALUE self, VALUE left, VALUE right) {
  return INT2FIX(st_strcasecmp(StringValueCStr(left), StringValueCStr(right)));
}
static VALUE compare_prefix(VALUE self, VALUE left, VALUE right, VALUE count) {
  return INT2FIX(st_strncasecmp(StringValueCStr(left), StringValueCStr(right), (size_t)RB_NUM2LONG(count)));
}
static VALUE named(VALUE self, VALUE text) {
  ID id;
  CONST_ID(id, "ledger");
  return rb_funcall(self, id, 1, text);
}
static VALUE ledger(VALUE self, VALUE text) { return rb_str_plus(rb_str_new_cstr("ledger: "), text); }
static VALUE total(VALUE self, VALUE numbers) {
  VALUE buffer;
  long count = RARRAY_LEN(numbers);
  long *values = ALLOCV(buffer, sizeof(long) * (size_t)count);
  long sum = 0;
  for (long at = 0; at < count; at++) values[at] = NUM2LONG(rb_ary_entry(numbers, at));
  for (long at = 0; at < count; at++) sum += values[at];
  ALLOCV_END(buffer);
  return LONG2NUM(sum);
}
static VALUE missing(VALUE self, VALUE value) { return RB_NIL_P(value) ? Qtrue : Qfalse; }
static VALUE process(VALUE self) {
  rb_pid_t pid = getpid();
  return pid > 0 ? Qtrue : Qfalse;
}
static VALUE out_of_memory(VALUE self) {
  rb_memerror();
  return Qnil;
}

void Init_c_library_helpers(void) {
  VALUE cls = rb_define_class("CLibraryHelpers", rb_cObject);
  rb_define_method(cls, "copy", copy, 1);
  rb_define_method(cls, "compare", compare, 2);
  rb_define_method(cls, "compare_prefix", compare_prefix, 3);
  rb_define_method(cls, "named", named, 1);
  rb_define_method(cls, "ledger", ledger, 1);
  rb_define_method(cls, "total", total, 1);
  rb_define_method(cls, "missing", missing, 1);
  rb_define_method(cls, "process", process, 0);
  rb_define_method(cls, "out_of_memory", out_of_memory, 0);
}
