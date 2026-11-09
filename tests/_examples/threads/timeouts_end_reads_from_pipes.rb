require "timeout"
reader, writer = IO.pipe
begin
  Timeout.timeout(0.2) { reader.gets }
rescue Timeout::Error
  p :pipe_timed_out
end
IO.popen(["sleep", "5"], "r+") do |child|
  begin
    Timeout.timeout(0.2) { child.gets }
  rescue Timeout::Error
    p :popen_timed_out
  end
  Process.kill(:TERM, child.pid)
end
