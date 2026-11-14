// A C extension written in C++, which mkmf compiles with the C++ compiler.
#include <ruby.h>
#include <cctype>
#include <string>

static VALUE shout(VALUE self, VALUE text) {
  std::string held(RSTRING_PTR(text), RSTRING_LEN(text));
  for (char &letter : held) {
    letter = static_cast<char>(std::toupper(static_cast<unsigned char>(letter)));
  }
  return rb_str_new(held.data(), static_cast<long>(held.size()));
}

extern "C" void Init_c_plus_plus(void) {
  rb_define_singleton_method(rb_define_module("Loud"), "shout", RUBY_METHOD_FUNC(shout), 1);
}
