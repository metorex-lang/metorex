# NoMemoryError and SystemStackError raised in a thread end that thread and
# leave the program running.
Thread.report_on_exception = false
t = Thread.new do
  begin
    raise NoMemoryError, "worker died"
  rescue Exception => error
    raise
  end
end
sleep 0.2
p t.status
p :alive
t2 = Thread.new { raise SystemStackError, "deep" }
sleep 0.1
p t2.status
p :still
