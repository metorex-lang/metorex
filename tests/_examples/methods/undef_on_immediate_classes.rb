class Object
  def with_marker
    :marked
  end
end
NilClass.undef_method(:with_marker)
p nil.respond_to?(:with_marker)
begin
  nil.with_marker
rescue NoMethodError => error
  p error.message
end
Integer.undef_method(:with_marker)
p 1.respond_to?(:with_marker)
p 1.0.respond_to?(:with_marker)
