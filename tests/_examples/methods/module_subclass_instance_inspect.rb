# An anonymous module made by `new` on a Module subclass names that subclass
# when inspected, and a NoMethodError raised on it says the method is
# undefined.
class ModuleDouble < Module
end

double = ModuleDouble.new
puts double.inspect.sub(/0x\h+/, "0x")
begin
  double.crate_count
rescue NoMethodError => error
  puts error.message.sub(/0x\h+/, "0x")
end
