# A binary string written to the standard output goes out as its bytes,
# whichever way it is written, and a UTF-8 one as its own.
require "rbconfig"

written = IO.popen([RbConfig.ruby, "-e", <<~'RUBY'], &:read)
  $stdout.write("\xEA\x01".b)
  STDOUT.print("\xFF".b)
  $stdout << "\xFE".b
  $stdout.write("é")
RUBY
p written.b.bytes
