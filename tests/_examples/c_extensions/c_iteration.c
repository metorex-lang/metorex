#include "ruby.h"
#include "ruby/encoding.h"
#include "ruby/io.h"

static VALUE collect(RB_BLOCK_CALL_FUNC_ARGLIST(element, gathered)) {
  return rb_ary_push(gathered, element);
}

static VALUE pairs_of(VALUE object) { return rb_funcall(object, rb_intern("each_pair"), 0); }

static VALUE each_of(VALUE self, VALUE object) {
  VALUE gathered = rb_ary_new();
  rb_iterate(rb_each, object, collect, gathered);
  return gathered;
}

static VALUE pairs(VALUE self, VALUE object) {
  VALUE gathered = rb_ary_new();
  rb_iterate(pairs_of, object, collect, gathered);
  return gathered;
}

static VALUE super_with_keywords(int count, VALUE *arguments, VALUE self) {
  return rb_call_super_kw(count, arguments, RB_PASS_KEYWORDS);
}

static VALUE define_super(VALUE self, VALUE klass, VALUE name) {
  rb_define_method(klass, StringValueCStr(name), super_with_keywords, -1);
  return Qnil;
}

static VALUE prepend(VALUE self, VALUE klass, VALUE module) {
  rb_prepend_module(klass, module);
  return Qnil;
}

static VALUE deprecate(VALUE self, VALUE module, VALUE name) {
  rb_deprecate_constant(module, StringValueCStr(name));
  return Qnil;
}

static VALUE dummy_p(VALUE self, VALUE encoding) {
  return rb_enc_dummy_p(rb_to_encoding(encoding)) ? Qtrue : Qfalse;
}

static VALUE get_io(VALUE self, VALUE object) { return rb_io_get_io(object); }

static VALUE positive_pow(VALUE self, VALUE base, VALUE exponent) {
  return rb_int_positive_pow(NUM2LONG(base), NUM2ULONG(exponent));
}

static VALUE timeout_error(VALUE self) { return rb_eIOTimeoutError; }

static VALUE opening(VALUE self, VALUE io) {
  int mode = rb_io_mode(io);
  VALUE flags = rb_ary_new();
  if (mode & FMODE_APPEND) rb_ary_push(flags, ID2SYM(rb_intern("append")));
  if (mode & FMODE_CREATE) rb_ary_push(flags, ID2SYM(rb_intern("create")));
  if (mode & FMODE_TRUNC) rb_ary_push(flags, ID2SYM(rb_intern("trunc")));
  return flags;
}

void Init_c_iteration(void) {
  VALUE module = rb_define_module("CIteration");
  rb_define_module_function(module, "each_of", each_of, 1);
  rb_define_module_function(module, "pairs", pairs, 1);
  rb_define_module_function(module, "define_super", define_super, 2);
  rb_define_module_function(module, "prepend", prepend, 2);
  rb_define_module_function(module, "deprecate", deprecate, 2);
  rb_define_module_function(module, "dummy?", dummy_p, 1);
  rb_define_module_function(module, "get_io", get_io, 1);
  rb_define_module_function(module, "positive_pow", positive_pow, 2);
  rb_define_module_function(module, "timeout_error", timeout_error, 0);
  rb_define_module_function(module, "opening", opening, 1);
}
