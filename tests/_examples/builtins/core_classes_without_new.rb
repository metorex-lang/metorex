[Float, Integer, Symbol, NilClass, TrueClass, FalseClass, Rational, Complex,
 Method, UnboundMethod, Binding, Encoding].each do |klass|
  puts "#{klass} #{klass.respond_to?(:new)}"
end
p Rational.respond_to?(:allocate), Complex.respond_to?(:allocate), Float.respond_to?(:allocate)
begin
  Encoding.new
rescue NoMethodError => error
  puts error.message
end
begin
  Complex.allocate
rescue NoMethodError => error
  puts error.message
end
p Class.new.respond_to?(:new)
