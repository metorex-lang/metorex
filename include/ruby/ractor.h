#ifndef RUBY_RACTOR_H
#define RUBY_RACTOR_H 1

#include "ruby/ruby.h"

#ifdef __cplusplus
extern "C" {
#endif

/* What a Ractor-local pointer is marked and freed with. metorex keeps a
   Ractor's storage for the life of the program, so neither is called. */
struct rb_ractor_local_storage_type {
  void (*mark)(void *ptr);
  void (*free)(void *ptr);
};

typedef struct rb_ractor_local_key_struct *rb_ractor_local_key_t;

bool rb_ractor_main_p(void);
bool rb_ractor_shareable_p(VALUE object);
VALUE rb_ractor_make_shareable(VALUE object);
#define RB_OBJ_SHAREABLE_P(object) rb_ractor_shareable_p(object)

rb_ractor_local_key_t rb_ractor_local_storage_value_newkey(void);
VALUE rb_ractor_local_storage_value(rb_ractor_local_key_t key);
bool rb_ractor_local_storage_value_lookup(rb_ractor_local_key_t key, VALUE *found);
void rb_ractor_local_storage_value_set(rb_ractor_local_key_t key, VALUE value);
rb_ractor_local_key_t rb_ractor_local_storage_ptr_newkey(const struct rb_ractor_local_storage_type *type);
void *rb_ractor_local_storage_ptr(rb_ractor_local_key_t key);
void rb_ractor_local_storage_ptr_set(rb_ractor_local_key_t key, void *pointer);

#ifdef __cplusplus
}
#endif

#endif
