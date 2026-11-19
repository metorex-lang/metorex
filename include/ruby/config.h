#ifndef RUBY_CONFIG_H
#define RUBY_CONFIG_H 1
/* The sizes and features metorex's C API is built with, under the name
   MRI's configure writes them to, which an extconf.rb may read. */
#define SIZEOF_INT __SIZEOF_INT__
#define SIZEOF_SHORT __SIZEOF_SHORT__
#define SIZEOF_LONG __SIZEOF_LONG__
#define SIZEOF_LONG_LONG __SIZEOF_LONG_LONG__
#define SIZEOF_VOIDP __SIZEOF_POINTER__
#define SIZEOF_SIZE_T __SIZEOF_SIZE_T__
#define SIZEOF_DOUBLE __SIZEOF_DOUBLE__
#define SIZEOF_FLOAT __SIZEOF_FLOAT__
#define SIZEOF_PTRDIFF_T __SIZEOF_PTRDIFF_T__
#define SIZEOF_INTPTR_T __SIZEOF_POINTER__
#define SIZEOF_UINTPTR_T __SIZEOF_POINTER__
#define SIZEOF_SSIZE_T __SIZEOF_SIZE_T__
#define SIZEOF_OFF_T 8
#define SIZEOF_INT8_T 1
#define SIZEOF_UINT8_T 1
#define SIZEOF_INT16_T 2
#define SIZEOF_UINT16_T 2
#define SIZEOF_INT32_T 4
#define SIZEOF_UINT32_T 4
#define SIZEOF_INT64_T 8
#define SIZEOF_UINT64_T 8
#ifdef __SIZEOF_INT128__
#define int128_t __int128
#define uint128_t unsigned __int128
#define SIZEOF_INT128_T __SIZEOF_INT128__
#define SIZEOF_UINT128_T __SIZEOF_INT128__
#endif
#define SIZEOF_TIME_T __SIZEOF_LONG__
#define SIZEOF_VALUE SIZEOF_VOIDP
#define HAVE_LONG_LONG 1
#define HAVE_UNISTD_H 1
#define HAVE_STDBOOL_H 1
#define RUBY_INTEGER_UNIFICATION 1
#define LONG_LONG long long
/* mkmf names the machine metorex runs on. */
#ifndef RUBY_PLATFORM
#define RUBY_PLATFORM "unknown-unknown"
#endif
#endif
