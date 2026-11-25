#include "ruby.h"

static VALUE set_default(VALUE self, VALUE hash, VALUE value, VALUE is_proc) {
  RHASH_SET_IFNONE(hash, value);
  if (RTEST(is_proc)) {
    FL_SET(hash, FL_USER2);
  } else {
    FL_UNSET(hash, FL_USER2);
  }
  return hash;
}
static VALUE proc_default_p(VALUE self, VALUE hash) { return FL_TEST(hash, FL_USER2) ? Qtrue : Qfalse; }
static VALUE lambda_p(VALUE self, VALUE procedure) { return rb_proc_lambda_p(procedure); }
static VALUE nest(VALUE object, VALUE pairs, int recursive) {
  if (recursive) return rb_str_new_cstr("recursive");
  if (RARRAY_LEN(pairs) == 0) return rb_str_new_cstr("done");
  VALUE rest = rb_ary_subseq(pairs, 1, RARRAY_LEN(pairs) - 1);
  return rb_exec_recursive_paired(nest, object, rb_ary_entry(pairs, 0), rest);
}
static VALUE recurse_paired(VALUE self, VALUE object, VALUE pairs) { return nest(object, pairs, 0); }
static VALUE user_flags(VALUE self) {
  return rb_ary_new_from_args(3, ULONG2NUM(FL_USER1), ULONG2NUM(FL_USER2), ULONG2NUM(FL_USER19));
}

void Init_c_proc_defaults(void) {
  VALUE cls = rb_define_class("CProcDefaults", rb_cObject);
  rb_define_method(cls, "set_default", set_default, 3);
  rb_define_method(cls, "proc_default?", proc_default_p, 1);
  rb_define_method(cls, "lambda?", lambda_p, 1);
  rb_define_method(cls, "recurse_paired", recurse_paired, 2);
  rb_define_method(cls, "user_flags", user_flags, 0);
}
