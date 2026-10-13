def boom = raise("closed")
at_exit do
  boom
rescue => error
  puts(error.backtrace.map { |line| File.basename(line) })
end
