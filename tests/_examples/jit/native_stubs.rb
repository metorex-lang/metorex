# Calls of String#<< and Integer#to_s, which the core classes list as
# stubs standing for their native methods.
def build(count)
  line = +""
  count.times { |step| line << step.to_s }
  line
end

p build(40).size

events = []
trace = TracePoint.new(:c_call) { |point| events << point.method_id if point.method_id == :<< }
trace.enable { build(3) }
p events

class String
  private :<<
end
begin
  build(40)
rescue NoMethodError => error
  p error.message
end

class String
  public :<<
end
p build(40).size

module Loud
  def <<(other)
    super("#{other}!")
  end
end
String.prepend(Loud)
p build(3)

class Integer
  def to_s
    "n"
  end
end
p build(40).size
