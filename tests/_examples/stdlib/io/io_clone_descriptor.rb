copy = $stdout.clone
puts copy.fileno == $stdout.fileno
copy.close
puts "stdout still open"

require "tempfile"
captured = Tempfile.new("capture")
saved = $stdout.clone
$stdout.reopen(captured)
system("echo from a child")
$stdout.flush
$stdout.reopen(saved)
saved.close
captured.rewind
puts "captured #{captured.read.inspect}"
captured.close
captured.unlink
