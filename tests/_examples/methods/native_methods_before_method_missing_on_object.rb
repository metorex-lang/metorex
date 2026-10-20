# A `method_missing` mixed into Object answers only names nothing else
# answers: Kernel's own methods, such as `===`, `frozen?`, and `hash`, still
# run for every object.
module Fallback
  def method_missing(name, *args)
    return "fallback #{name}" if name.to_s.start_with?("be_")

    super
  end

  def respond_to_missing?(name, include_private = false)
    name.to_s.start_with?("be_") || super
  end
end

class Object
  include Fallback
end

marker = Object.new.freeze
p marker === marker
p marker.frozen?
p marker.hash == marker.hash
p Object.new.be_shipped
begin
  Object.new.unknown_label
rescue NoMethodError => error
  puts error.message
end
