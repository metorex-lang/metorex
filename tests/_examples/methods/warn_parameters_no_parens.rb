# `Kernel#warn` names the keywords it takes.
taken = Kernel.instance_method :warn
p taken.parameters

# `exception:` takes true or false and nothing else.
reader, writer = IO.pipe
begin
  reader.read_nonblock 4, exception: nil
rescue ArgumentError => problem
  p problem.message
end
begin
  writer.write_nonblock "x", exception: 0
rescue ArgumentError => problem
  p problem.message
end
reader.close
writer.close

# `close_others:` and `daemon` arguments take true, false, or nil.
begin
  Process.spawn "true", close_others: 1
rescue ArgumentError => problem
  p problem.message
end
begin
  Process.daemon "true"
rescue ArgumentError => problem
  p problem.message
end
begin
  Process.daemon true, "x"
rescue ArgumentError => problem
  p problem.message
end
