#include "ruby.h"
#include "ruby/ractor.h"

static rb_ractor_local_key_t value_key;
static rb_ractor_local_key_t pointer_key;
static const struct rb_ractor_local_storage_type pointer_type = {NULL, NULL};
static long counts[2];

static VALUE store(VALUE self, VALUE value) {
  rb_ractor_local_storage_value_set(value_key, value);
  long *count = rb_ractor_main_p() ? &counts[0] : &counts[1];
  *count += 1;
  rb_ractor_local_storage_ptr_set(pointer_key, count);
  return Qnil;
}

static VALUE stored(VALUE self) {
  VALUE found = Qfalse;
  bool held = rb_ractor_local_storage_value_lookup(value_key, &found);
  long *count = rb_ractor_local_storage_ptr(pointer_key);
  return rb_ary_new_from_args(4, held ? Qtrue : Qfalse, found, rb_ractor_local_storage_value(value_key),
                              count ? LONG2NUM(*count) : Qnil);
}

static VALUE shareable(VALUE self, VALUE object) {
  bool before = rb_ractor_shareable_p(object);
  VALUE made = rb_ractor_make_shareable(object);
  return rb_ary_new_from_args(4, before ? Qtrue : Qfalse, RB_OBJ_SHAREABLE_P(made) ? Qtrue : Qfalse,
                              made == object ? Qtrue : Qfalse, rb_ractor_main_p() ? Qtrue : Qfalse);
}

void Init_c_ractors(void) {
  rb_ext_ractor_safe(true);
  value_key = rb_ractor_local_storage_value_newkey();
  pointer_key = rb_ractor_local_storage_ptr_newkey(&pointer_type);
  VALUE klass = rb_define_class("CRactors", rb_cObject);
  rb_define_method(klass, "store", store, 1);
  rb_define_method(klass, "stored", stored, 0);
  rb_define_method(klass, "shareable", shareable, 1);
}
