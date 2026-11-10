#ifndef RUBY_UTIL_H
#define RUBY_UTIL_H 1

#include "ruby/ruby.h"

#ifdef __cplusplus
extern "C" {
#endif

double ruby_strtod(const char *text, char **end);
#undef strtod
#define strtod(text, end) ruby_strtod((text), (end))

static inline char *ruby_strdup(const char *text) {
  size_t length = strlen(text) + 1;
  return (char *)memcpy(xmalloc(length), text, length);
}

#ifdef __cplusplus
}
#endif

#endif
