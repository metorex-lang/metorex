old = Signal.trap("INT", "DEFAULT")
p old
rd, wr = IO.pipe
pid = fork do
  rd.close
  begin
    wr.puts "READY"
    wr.flush
    sleep 30
    wr.puts "woke"
  rescue Interrupt => e
    wr.puts "interrupted #{e.class}"
  end
  wr.close
end
wr.close
p rd.gets
sleep 0.5
Process.kill("INT", pid)
p rd.gets
Process.wait(pid)
p $?.exitstatus
begin
  Thread.new { sleep 0.3; Process.kill("INT", Process.pid) }
  sleep 5
  puts "woke"
rescue Interrupt => e
  puts "interrupted"
end
begin
  Process.kill("INT", Process.pid)
  sleep 1
rescue Interrupt => error
  p error.message, error.signo
end
