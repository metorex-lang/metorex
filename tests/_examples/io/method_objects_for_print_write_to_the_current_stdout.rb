# A Method object for print, puts, or p writes to whatever $stdout holds when
# it is called, as calling the function by name does.
require "stringio"
printer = method(:print)
putter = method(:puts)
shower = method(:p)
p [printer.owner, printer.arity, printer.name]
printer.call("to the terminal\n")
captured = StringIO.new
$stdout = captured
printer.call("printed ")
putter.call("put")
shower.call(:shown)
[1].each(&printer)
$stdout = STDOUT
p captured.string
