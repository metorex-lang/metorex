#ifndef PRISM_XALLOCATOR_H
#define PRISM_XALLOCATOR_H

#include <stddef.h>

void *metorex_prism_malloc(size_t size);
void *metorex_prism_calloc(size_t count, size_t size);
void *metorex_prism_realloc(void *pointer, size_t size);
void metorex_prism_free(void *pointer);

#define xmalloc metorex_prism_malloc
#define xcalloc metorex_prism_calloc
#define xrealloc metorex_prism_realloc
#define xfree metorex_prism_free

#endif
