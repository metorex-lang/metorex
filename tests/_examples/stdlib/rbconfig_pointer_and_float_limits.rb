require "rbconfig/sizeof"
%w[INTPTR_MAX INTPTR_MIN UINTPTR_MAX SIZE_MAX PTRDIFF_MIN INTMAX_MAX UINT_LEAST8_MAX
   DBL_MANT_DIG DBL_EPSILON DBL_TRUE_MIN FLT_DIG FLT_RADIX].each do |name|
  puts "#{name} #{RbConfig::LIMITS[name].inspect}"
end
