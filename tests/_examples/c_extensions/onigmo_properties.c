#include "ruby.h"
#include "ruby/encoding.h"
#include "ruby/oniguruma.h"

static VALUE property_number(VALUE self, VALUE name) {
  const char *text = StringValueCStr(name);
  UChar *start = (UChar *)text;
  return INT2FIX(ONIGENC_PROPERTY_NAME_TO_CTYPE(rb_utf8_encoding(), start, start + strlen(text)));
}
static VALUE property_ranges(VALUE self, VALUE number) {
  const OnigCodePoint *ranges;
  OnigCodePoint single_byte_out;
  int status = ONIGENC_GET_CTYPE_CODE_RANGE(rb_utf8_encoding(), NUM2INT(number), &single_byte_out, &ranges);
  if (status != 0) return INT2FIX(status);
  VALUE result = rb_ary_new();
  for (unsigned int at = 0; at < ranges[0]; at++) {
    rb_ary_push(result, rb_range_new(INT2FIX(ranges[at * 2 + 1]), INT2FIX(ranges[at * 2 + 2]), 0));
  }
  return rb_ary_new_from_args(2, INT2FIX(single_byte_out), result);
}
void Init_onigmo_properties(void) {
  VALUE module = rb_define_module("OnigmoProperties");
  rb_define_singleton_method(module, "number", property_number, 1);
  rb_define_singleton_method(module, "ranges", property_ranges, 1);
}
