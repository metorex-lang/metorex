handled = []
Signal.trap("USR1") { handled << :usr1 }
reader, writer = IO.pipe
pid = fork do
  reader.close
  sleep 0.05
  Process.kill "USR1", Process.ppid
  sleep 0.05
  writer.write "done"
  exit! 0
end
writer.close
ready = IO.select([reader], nil, nil, 5)
p handled
p ready[0].size
p reader.read
Process.wait pid
