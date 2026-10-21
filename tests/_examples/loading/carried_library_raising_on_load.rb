# A library that raised while loading is not loaded, so requiring it again
# runs it again.
OpenStruct = 1
begin
  require "ostruct"
rescue TypeError => error
  p error.class
end
Object.send(:remove_const, :OpenStruct)
p require("ostruct")
p OpenStruct.new(total: 3).total
