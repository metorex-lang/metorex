#include "ruby.h"

void Init_fiddle(void) {
  VALUE fiddle = rb_define_module("Fiddle");
  rb_define_const(fiddle, "BUILD_RUBY_PLATFORM", rb_str_new_cstr(RUBY_PLATFORM));
}
