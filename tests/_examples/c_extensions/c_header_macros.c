// The macros ruby.h gives an extension: the HAVE_RUBY_*_H flags for the
// headers under ruby/, RUBY_METHOD_FUNC, and RETURN_ENUMERATOR.
#include <ruby.h>
#if defined(HAVE_RUBY_ENCODING_H)
#include <ruby/encoding.h>
#endif

static VALUE encoded_text(VALUE self) {
#if defined(HAVE_RUBY_ENCODING_H)
  return rb_enc_str_new("caf\xc3\xa9", 5, rb_utf8_encoding());
#else
  return rb_str_new_cstr("no encoding header");
#endif
}

static VALUE each_pair_of(VALUE self, VALUE first, VALUE second) {
  VALUE arguments[] = {first, second};
  RETURN_ENUMERATOR(self, 2, arguments);
  rb_yield(first);
  return rb_yield(second);
}

void Init_c_header_macros(void) {
  VALUE module = rb_define_module("HeaderMacros");
  rb_define_singleton_method(module, "encoded_text", RUBY_METHOD_FUNC(encoded_text), 0);
  rb_define_singleton_method(module, "each_pair_of", RUBY_METHOD_FUNC(each_pair_of), 2);
}
