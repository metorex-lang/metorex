#include "ruby.h"
#include "ruby/debug.h"

typedef struct rb_iseq_struct rb_iseq_t;
const rb_iseq_t *rb_iseqw_to_iseq(VALUE iseqw);
VALUE rb_iseqw_new(VALUE iseq);
VALUE rb_iseq_realpath(const rb_iseq_t *iseq);
VALUE rb_iseq_type(const rb_iseq_t *iseq);
VALUE rb_iseq_parameters(const rb_iseq_t *iseq, int is_proc);
void rb_iseq_code_location(const rb_iseq_t *iseq, int *first_lineno, int *first_column,
                           int *last_lineno, int *last_column);
void rb_objspace_each_objects(int (*callback)(void *start, void *end, size_t stride, void *data),
                              void *data);
size_t rb_obj_memsize_of(VALUE object);

static VALUE describe(VALUE self, VALUE iseqw) {
  const rb_iseq_t *iseq = rb_iseqw_to_iseq(iseqw);
  int first_line, last_line;
  VALUE described = rb_ary_new();
  rb_iseq_code_location(iseq, &first_line, NULL, &last_line, NULL);
  rb_ary_push(described, rb_iseq_type(iseq));
  rb_ary_push(described, rb_iseq_realpath(iseq));
  rb_ary_push(described, rb_iseq_parameters(iseq, 0));
  rb_ary_push(described, rb_iseq_parameters(iseq, 1));
  rb_ary_push(described, INT2NUM(first_line));
  rb_ary_push(described, INT2NUM(last_line));
  return described;
}

static int is_iseq(VALUE object) {
  return RB_TYPE_P(object, T_IMEMO) && ((RBASIC(object)->flags >> FL_USHIFT) & 0xf) == 7;
}

static int yield_iseqs(void *start, void *end, size_t stride, void *data) {
  VALUE object;
  for (object = (VALUE)start; object != (VALUE)end; object += stride) {
    if (RBASIC(object)->flags && is_iseq(object)) {
      rb_yield(rb_iseqw_new(object));
    }
  }
  return 0;
}

static VALUE each_iseq(VALUE self) {
  rb_objspace_each_objects(yield_iseqs, NULL);
  return Qnil;
}

static int keep_first(void *start, void *end, size_t stride, void *data) {
  VALUE object;
  for (object = (VALUE)start; object != (VALUE)end; object += stride) {
    if (RBASIC(object)->flags && is_iseq(object)) {
      *(VALUE *)data = rb_iseqw_new(object);
      return 1;
    }
  }
  return 0;
}

static VALUE first_iseq(VALUE self) {
  VALUE found = Qnil;
  rb_objspace_each_objects(keep_first, &found);
  return found;
}

static VALUE memsize(VALUE self, VALUE object) { return SIZET2NUM(rb_obj_memsize_of(object)); }

static VALUE frame_depths(const rb_debug_inspector_t *inspector, void *data) {
  VALUE depths = rb_ary_new();
  rb_ary_push(depths, rb_debug_inspector_current_depth());
  rb_ary_push(depths, rb_debug_inspector_frame_depth(inspector, 0));
  rb_ary_push(depths, rb_debug_inspector_frame_depth(inspector, 1));
  return depths;
}

static VALUE depths(VALUE self) { return rb_debug_inspector_open(frame_depths, NULL); }

void Init_c_sequences(void) {
  VALUE module = rb_define_module("CSequences");
  rb_define_module_function(module, "describe", describe, 1);
  rb_define_module_function(module, "each_iseq", each_iseq, 0);
  rb_define_module_function(module, "first_iseq", first_iseq, 0);
  rb_define_module_function(module, "memsize", memsize, 1);
  rb_define_module_function(module, "depths", depths, 0);
}
