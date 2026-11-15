# wait_readable answers at once for lines an earlier gets read ahead from a
# pipe and for a byte put back with ungetc, so a child's lines already
# waiting in the pipe are all read with wait_readable and gets.
child = 'STDOUT.sync = true; 6.times { |turn| STDOUT.write "line #{turn}\n" }; sleep 3'
server = IO.popen([RbConfig.ruby, "-e", child])
sleep 1
seen = []
deadline = Process.clock_gettime(Process::CLOCK_MONOTONIC) + 2
until Process.clock_gettime(Process::CLOCK_MONOTONIC) > deadline || seen.size == 6
  if server.wait_readable(0.2) and line = server.gets
    seen << line
  end
end
p seen
p server.wait_readable(0.2).inspect[0, 8]
server.ungetc("x")
p server.wait_readable(0.2).inspect[0, 8]
p server.getc
Process.kill("KILL", server.pid)
Process.wait(server.pid)
