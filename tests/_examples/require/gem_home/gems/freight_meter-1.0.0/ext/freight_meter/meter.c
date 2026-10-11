#include "ruby.h"
#include "ruby/internal/symbol.h"
#include "charges.h"
#include "meter_version.h"

typedef struct {
  VALUE label;
} meter_t;

/* The total charge for each weight, in cents, at one rate. */
static VALUE meter_charges(VALUE self, VALUE weights, VALUE rate) {
  long count = rb_array_len(weights);
  long *cents = ALLOC_N(long, count);
  long *copied = ZALLOC_N(long, count);
  for (long index = 0; index < count; index++) {
    cents[index] = freight_charge_cents(NUM2LONG(rb_ary_entry(weights, index)), NUM2LONG(rate));
  }
  MEMCPY(copied, cents, long, count);
  REALLOC_N(cents, long, count + 1);
  VALUE answer = rb_ary_new();
  for (long index = 0; index < count; index++) {
    rb_ary_push(answer, LONG2NUM(copied[index]));
  }
  xfree(cents);
  ruby_xfree(copied);
  RB_GC_GUARD(weights);
  return answer;
}

static VALUE meter_any_heavy(VALUE self, VALUE weights) {
  for (long index = 0; index < rb_array_len(weights); index++) {
    if (RB_TEST(rb_funcall(rb_ary_entry(weights, index), rb_intern(">"), 1, INT2FIX(100)))) {
      return Qtrue;
    }
  }
  return Qfalse;
}

static VALUE meter_relabel(VALUE self, VALUE label) {
  meter_t *meter = ALLOC(meter_t);
  RB_OBJ_WRITE(self, &meter->label, label);
  VALUE kept = meter->label;
  xfree(meter);
  return kept;
}

void Init_meter(void) {
  VALUE module = rb_define_module("FreightMeter");
  rb_define_const(module, "NATIVE_VERSION", rb_str_new_cstr(FREIGHT_METER_VERSION));
  rb_define_module_function(module, "charges", meter_charges, 2);
  rb_define_module_function(module, "any_heavy?", meter_any_heavy, 1);
  rb_define_module_function(module, "relabel", meter_relabel, 1);
  rb_define_const(module, "TAG", ID2SYM(rb_intern("freight")));
}
