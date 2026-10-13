puts_method = String.method(:puts)
p [puts_method.name, puts_method.owner]
puts_method.call("called through String")

class Report
  p method(:format).owner
end

module Formatting
  p method(:print).owner
end

Class.new.class_exec { p method(:puts).name }
p String.public_method(:puts) rescue p $!.class
