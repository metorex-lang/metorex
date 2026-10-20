# A module made by a subclass of Module calls the methods that subclass and
# its modules define by their bare names inside its own methods, as an
# object of any other class does.
module Counting
  def null_object? = false
  def label = "#{kind}, null #{null_object?}"
end

class ModuleDouble < Module
  include Counting

  def kind = "module double"

  def describe_self
    label
  end
end

double = ModuleDouble.new
puts double.describe_self
puts double.label
