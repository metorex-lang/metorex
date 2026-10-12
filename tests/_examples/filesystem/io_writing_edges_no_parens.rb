require "tmpdir"

Dir.mktmpdir do |directory|
  path = File.join directory, "log.txt"

  # `newline: :crlf` converts every newline written, by `write` and `puts`.
  File.open path, "w", newline: :crlf do |file|
    file.write "start\n"
    file.puts "a", "b"
  end
  p File.binread path

  # `pread` past the end clears the buffer it was handed.
  File.write path, "abc"
  buffer = +"kept"
  File.open path do |file|
    begin
      file.pread 1, 10, buffer
    rescue EOFError
      p buffer
    end
  end

  # A negative offset is reported before a negative length.
  begin
    IO.read path, -1, -1
  rescue ArgumentError => problem
    p problem.message
  end

  # `copy_stream` reads its length through `to_int`, and copies nothing for
  # a length of zero.
  length = Object.new
  def length.to_int = 2
  target = File.join directory, "copy.txt"
  p IO.copy_stream path, target, length
  p File.read target
  p IO.copy_stream path, target, 0
  bad = Object.new
  def bad.to_int = "2"
  begin
    IO.copy_stream path, target, bad
  rescue TypeError => problem
    p problem.message
  end

  # An IO::Buffer writes from an offset to the end when the length is zero.
  buffer = IO::Buffer.new 5
  buffer.set_string "Hello"
  File.open path, "wb+" do |file|
    p buffer.write file, 0, 1
    file.rewind
    p file.read
  end
  buffer.free
end
