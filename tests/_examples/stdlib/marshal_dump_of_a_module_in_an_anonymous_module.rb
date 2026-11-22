anonymous = Module.new
module anonymous::Nested; end
begin
  Marshal.dump(anonymous::Nested)
rescue TypeError => error
  puts error.message.sub(/0x\h+/, "0x")
end
outer = Class.new
outer::Inner = Class.new
begin
  Marshal.dump(outer::Inner)
rescue TypeError => error
  puts error.message.sub(/0x\h+/, "0x")
end
module DumpedModules; end
DumpedModules::Kept = anonymous::Nested
p Marshal.dump(anonymous::Nested)
p Marshal.load(Marshal.dump(anonymous::Nested)).equal?(anonymous::Nested)
