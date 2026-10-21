require_relative "def_in_a_block/builder"

receiver = BUILD_RECEIVER.call
file, line = receiver.method(:record).source_location
puts File.basename(file), line

begin
  receiver.record
rescue ArgumentError => error
  puts File.basename(error.backtrace.first)
end
