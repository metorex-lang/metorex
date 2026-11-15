#include "ruby.h"

static VALUE eParseError;

static VALUE parse(VALUE self, VALUE text) {
  rb_raise(eParseError, "cannot parse %s", RSTRING_PTR(text));
  return Qnil;
}

void Init_c_error_classes(void) {
  VALUE outer = rb_define_module("CErrorClasses");
  VALUE inner = rb_define_module_under(outer, "Secure");
  eParseError = rb_define_class_under(outer, "ParseError", rb_eStandardError);
  rb_define_class_under(inner, "SecureError", rb_eStandardError);
  rb_define_module_function(outer, "parse", parse, 1);
}
