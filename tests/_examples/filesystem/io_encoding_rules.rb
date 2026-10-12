require "tmpdir"

Dir.mktmpdir do |directory|
  path = File.join(directory, "readings.txt")
  File.write(path, "1\n")

  # A stream reading BINARY carries its bytes into no other encoding.
  File.open(path, "r:binary:ibm437") do |file|
    p file.external_encoding
    p file.internal_encoding
  end

  File.open(path, "r") do |file|
    # Naming only an internal encoding is refused.
    begin
      file.set_encoding(nil, Encoding::UTF_8)
    rescue TypeError => problem
      p problem.message
    end
    # So are names that are not ASCII or that hold a NUL byte.
    begin
      file.set_encoding("utf-8".encode(Encoding::UTF_16BE))
    rescue ArgumentError => problem
      p problem.message
    end
    begin
      file.set_encoding(Encoding::UTF_8, "null\0byte")
    rescue ArgumentError => problem
      p problem.message
    end
    # A stream that reads needs binmode for an encoding that is not a
    # superset of ASCII.
    begin
      file.set_encoding(Encoding::UTF_16BE)
    rescue ArgumentError => problem
      p problem.message
    end
    begin
      file.set_encoding("utf-8", newline: :sideways)
    rescue ArgumentError => problem
      p problem.message
    end
    file.binmode
    begin
      file.set_encoding("utf-8", newline: :lf)
    rescue ArgumentError => problem
      p problem.message
    end
  end

  # A stream naming only its internal encoding reads as the external one
  # the program had when it was opened.
  File.open(path, mode: "r", internal_encoding: "utf-8") do |file|
    held = Encoding.default_external
    Encoding.default_external = Encoding::IBM437
    p file.external_encoding == held
    Encoding.default_external = held
  end

  # A closed File names its path, and a closed pipe shows only that it is
  # closed.
  file = File.open(path)
  file.close
  p file.inspect.end_with?("readings.txt (closed)>")
  reader, writer = IO.pipe
  reader.close
  p reader.inspect
  writer.close
end
