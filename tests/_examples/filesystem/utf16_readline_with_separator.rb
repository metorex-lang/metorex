require "tmpdir"

Dir.mktmpdir do |dir|
  path = File.join(dir, "notes.xml")
  File.binwrite(path, "\xFF\xFE".b + "<a>b</a>".encode("UTF-16LE").b)
  separator = ">".encode("UTF-16LE")

  File.open(path) do |file|
    p file.read(2).bytes
    file.binmode
    file.set_encoding("UTF-16LE", "UTF-16LE")
    p file.external_encoding
    line = file.readline(separator)
    p [line.encoding, line.encode("UTF-8")]
    line = file.readline(separator)
    p [line.encoding, line.encode("UTF-8")]
    p file.eof?
  end

  # Characters are counted from where the read starts, so a separator
  # whose bytes straddle two of them is not found.
  File.open(path) do |file|
    p file.read(3).bytes
    file.binmode
    file.set_encoding("UTF-16LE", "UTF-16LE")
    line = file.readline(separator)
    p [line.encoding, line.bytesize]
    p file.eof?
  end
end
