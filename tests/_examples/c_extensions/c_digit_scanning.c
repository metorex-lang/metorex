#include "ruby.h"
#include "ruby/encoding.h"

static const char *const greeting = "abc";

static VALUE scan(VALUE self, VALUE text, VALUE length, VALUE base) {
  size_t read = 0;
  int overflow = 0;
  unsigned long value = ruby_scan_digits(RSTRING_PTR(text), NUM2LONG(length), NUM2INT(base), &read, &overflow);
  return rb_ary_new_from_args(3, ULONG2NUM(value), SIZET2NUM(read), INT2FIX(overflow));
}
static VALUE hexdigits(VALUE self) { return rb_str_new_cstr(ruby_hexdigits); }
static VALUE digit_value(VALUE self, VALUE character) {
  return INT2FIX(ruby_digit36_to_number_table[(unsigned char)RSTRING_PTR(character)[0]]);
}
static VALUE hex_digit(VALUE self, VALUE character) { return ISXDIGIT(RSTRING_PTR(character)[0]) ? Qtrue : Qfalse; }
static VALUE literal_length(VALUE self) { return INT2FIX(rb_strlen_lit("abc") + (greeting[0] == 'a')); }
static VALUE ascii_compatible(VALUE self, VALUE text) { return rb_enc_str_asciicompat_p(text) ? Qtrue : Qfalse; }
static VALUE clean(VALUE self, VALUE text) {
  int range = ENC_CODERANGE(text);
  ENC_CODERANGE_SET(text, range);
  return ENC_CODERANGE_CLEAN_P(range) ? Qtrue : Qfalse;
}
static VALUE character(VALUE self, VALUE code) {
  char buffer[8];
  OnigCodePoint point = (OnigCodePoint)NUM2UINT(code);
  return rb_enc_str_new(buffer, rb_enc_mbcput(point, buffer, rb_utf8_encoding()), rb_utf8_encoding());
}
static VALUE size_overflow(VALUE self) {
  ruby_malloc_size_overflow(4, 8);
  return Qnil;
}

void InitVM_c_digit_scanning(void) {
  VALUE cls = rb_define_class("CDigitScanning", rb_cObject);
  rb_define_method(cls, "scan", scan, 3);
  rb_define_method(cls, "hexdigits", hexdigits, 0);
  rb_define_method(cls, "digit_value", digit_value, 1);
  rb_define_method(cls, "hex_digit", hex_digit, 1);
  rb_define_method(cls, "literal_length", literal_length, 0);
  rb_define_method(cls, "ascii_compatible", ascii_compatible, 1);
  rb_define_method(cls, "clean", clean, 1);
  rb_define_method(cls, "character", character, 1);
  rb_define_method(cls, "size_overflow", size_overflow, 0);
}

void Init_c_digit_scanning(void) { InitVM(c_digit_scanning); }
