shadow = Module.new do
  def inspect(value) = value
  module_function :inspect
end
begin
  p shadow.singleton_class.to_s
rescue ArgumentError => error
  p error.class
end
o = Object.new
def o.inspect = "custom"
p o.singleton_class.to_s.start_with?("#<Class:#<Object:0x")
named = Module.new { def self.inspect = "Named" }
p named.singleton_class.to_s
