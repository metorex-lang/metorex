receiver = Object.new

def receiver.record(entry)
  entry
end

def receiver.assign=(first, second)
  @assigned = [first, second]
end

begin
  receiver.record
rescue ArgumentError => error
  puts error.backtrace.first(2).map { |line| line.sub(__FILE__, "FILE") }
end

begin
  receiver.assign = 1
rescue ArgumentError => error
  puts error.backtrace.first(2).map { |line| line.sub(__FILE__, "FILE") }
end
