previous = File.umask
[Float::NAN, Float::INFINITY, 1e30, 65536, -65535, 2**64 - 1, 2**64, -2**63 - 1, nil].each do |mask|
  Process.exec("no_such_command_for_the_umask_check", umask: mask)
rescue RangeError, TypeError => error
  p [error.class, error.message]
end
p File.umask == previous
