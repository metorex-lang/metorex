# In a forked child `$$` and `Process.pid` name the child, which is how
# concurrent-ruby's executors notice a fork and start their pools over.
parent = $$
reader, writer = IO.pipe
child = fork do
  reader.close
  writer.puts [$$ != parent, $$ == Process.pid].inspect
  writer.close
  exit!(0)
end
writer.close
puts reader.read
p Process.waitpid2(child)[1].exitstatus
p $$ == parent, $$ == Process.pid
