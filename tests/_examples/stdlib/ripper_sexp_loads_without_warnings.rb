require "stringio"

captured = StringIO.new
$stderr = captured
$VERBOSE = true
require "ripper"
p Ripper::SexpBuilder.new("1 +").tap(&:parse).error?
$stderr = STDERR
p captured.string
