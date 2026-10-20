# An instance of a Module subclass compares with the `==` a module included
# in that subclass defines.
module Comparing
  def ==(other)
    other == @stand_in
  end

  def stand_in=(value)
    @stand_in = value
  end
end

class ModuleDouble < Module
  include Comparing
end

double = ModuleDouble.new
double.stand_in = :proxy
p double == :proxy
p double == double
p double != :proxy
double.stand_in = 7
p double == 7.0
