# `||=` on constants a module body has not set yet and on ones it has.

module Limits
  LOWER = 3
  LOWER ||= 30
  UPPER ||= LOWER * 2
  Limits::NESTED = 4
  Limits::NESTED ||= 40
  Limits::LATER ||= 5
  ::LIMITS_TOP = 1
  ::LIMITS_TOP ||= 10
  @@ceiling ||= 9
  @@ceiling ||= 90
  $limits_global ||= 7
  $limits_global ||= 70
  $limits_global &&= 8
end

p [Limits::LOWER, Limits::UPPER, Limits::NESTED, Limits::LATER, LIMITS_TOP]
p [Limits.class_variable_get(:@@ceiling), $limits_global]
