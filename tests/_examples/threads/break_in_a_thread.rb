# `break` in a thread's block has no call to return from, and raises
# LocalJumpError where it is written.
thread = Thread.new do
  begin
    break :break
  rescue LocalJumpError => error
    error.class
  end
end
p thread.value
