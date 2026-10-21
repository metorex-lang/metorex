# A failure report quotes the whole statement its line falls in, which
# Rex::Source.__statement_lines__ answers as the first and last line, and
# names the value handed to `expect`, which __expect_argument__ reads.
require "rspec/core"

cart = File.expand_path("cart_spec.txt", __dir__)
one_line = File.expand_path("one_line_expects.txt", __dir__)
heredocs = File.expand_path("heredoc_and_semicolon.txt", __dir__)
unparsable = File.expand_path("unparsable.txt", __dir__)

# A one-line statement, a line inside a loop, a call whose arguments run on,
# the line closing a block, a line inside a block, a chain continued by
# leading dots, a loop header's `do`, and a blank line.
[18, 23, 3, 11, 10, 17, 26, 7].each do |line|
  p [line, Rex::Source.__statement_lines__(cart, line)]
end

# A heredoc argument, a line opening with the close of the statement before,
# two heredocs on one line, and a file that does not parse.
[3, 7, 9, 17].each do |line|
  p [line, Rex::Source.__statement_lines__(heredocs, line)]
end
p Rex::Source.__statement_lines__(unparsable, 1)

# A value handed to expect, a block handed to expect, and two expects on one
# line.
p Rex::Source.__expect_argument__(cart, 3)
p Rex::Source.__expect_argument__(cart, 9)
p Rex::Source.__expect_argument__(one_line, 1)
