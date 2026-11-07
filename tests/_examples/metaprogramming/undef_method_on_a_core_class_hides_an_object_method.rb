class Object
  def with_held
    :object
  end
end
[NilClass, Integer, Symbol].each { |klass| klass.undef_method(:with_held) }
p "text".with_held
begin
  1.with_held
rescue NoMethodError => problem
  p problem.message
end
p Integer.method_defined?(:with_held)
