# system hands its environment Hash to the child when a stream is redirected
# to an IO.
ENV["SYSTEM_EXAMPLE_REMOVED"] = "present"
reader, writer = IO.pipe
system({ "SYSTEM_EXAMPLE_ADDED" => "added", "SYSTEM_EXAMPLE_REMOVED" => nil },
       "/bin/sh", "-c", 'echo "added=$SYSTEM_EXAMPLE_ADDED removed=$SYSTEM_EXAMPLE_REMOVED"',
       out: writer, err: writer)
writer.close
puts reader.read
system({ "SYSTEM_EXAMPLE_ADDED" => "to stdout" }, "/bin/sh", "-c", 'echo "added=$SYSTEM_EXAMPLE_ADDED"', out: $stdout)
