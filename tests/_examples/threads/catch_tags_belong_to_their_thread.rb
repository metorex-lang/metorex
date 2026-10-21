queue = Thread::Queue.new
reader = nil
outcome = catch(:main_tag) do
  reader = Thread.new do
    catch(:eof) do
      queue.pop
      throw :eof
    end
    :reader_finished
  end
  Thread.pass until reader.stop?
  :main_finished
end
queue << :go
p outcome
p reader.value

begin
  catch(:outside) { Fiber.new { throw :outside }.resume }
rescue UncaughtThrowError => error
  p error.message
end
