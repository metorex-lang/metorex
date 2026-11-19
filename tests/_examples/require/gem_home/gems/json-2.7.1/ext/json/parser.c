#include "ruby.h"

void Init_parser(void) {
  VALUE json = rb_define_module("JSON");
  VALUE ext = rb_define_module_under(json, "Ext");
  rb_define_const(ext, "PARSER_BUILT", Qtrue);
}
