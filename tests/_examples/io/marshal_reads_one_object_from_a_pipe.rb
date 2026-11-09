reader, writer = IO.pipe
p writer.sync, reader.sync
Marshal.dump(21, writer)
Marshal.dump([1, "two"], writer)
p Marshal.load(reader)
p Marshal.load(reader)
writer.close
begin
  Marshal.load(reader)
rescue EOFError => error
  p error.message
end

child_read, parent_write = IO.pipe
parent_read, child_write = IO.pipe
pid = fork do
  parent_write.close
  parent_read.close
  until child_read.eof?
    Marshal.dump(Marshal.load(child_read) * 2, child_write)
  end
  child_write.close
end
child_read.close
child_write.close
Marshal.dump(21, parent_write)
p Marshal.load(parent_read)
parent_write.close
Process.wait(pid)
