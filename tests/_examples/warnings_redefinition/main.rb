require "stringio"
captured = StringIO.new
$stderr = captured
$VERBOSE = true
load File.join(__dir__, "definitions.rb")
require "socket"
require "pp"
$stderr = STDERR
puts captured.string.gsub(File.join(__dir__, ""), "")
