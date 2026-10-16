#include "ruby.h"

struct holder {
  VALUE held;
};

static void holder_mark(void *pointer) {
  struct holder *holder = pointer;
  rb_gc_mark(holder->held);
}

static const rb_data_type_t holder_type = {
  "holder",
  {holder_mark, RUBY_TYPED_DEFAULT_FREE, NULL},
  NULL,
  NULL,
  0
};

static VALUE hold(VALUE self, VALUE held) {
  struct holder *holder = malloc(sizeof(struct holder));
  holder->held = held;
  return TypedData_Wrap_Struct(rb_cObject, &holder_type, holder);
}

static VALUE held(VALUE self, VALUE wrapped) {
  struct holder *holder;
  TypedData_Get_Struct(wrapped, struct holder, &holder_type, holder);
  return holder->held;
}

static VALUE let_go(VALUE self, VALUE wrapped) {
  struct holder *holder;
  TypedData_Get_Struct(wrapped, struct holder, &holder_type, holder);
  holder->held = Qnil;
  return Qnil;
}

void Init_c_holding(void) {
  VALUE module = rb_define_module("CHolding");
  rb_define_singleton_method(module, "hold", hold, 1);
  rb_define_singleton_method(module, "held", held, 1);
  rb_define_singleton_method(module, "let_go", let_go, 1);
}
