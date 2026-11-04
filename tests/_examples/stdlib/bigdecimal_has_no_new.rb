require "bigdecimal"
p BigDecimal.respond_to?(:new)
begin
  BigDecimal.new("1")
rescue NoMethodError => error
  puts error.message
end
p BigDecimal("1.25") * 2
