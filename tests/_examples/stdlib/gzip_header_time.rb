# A gzip header given no time of its own carries the time it was written,
# and one given a time carries that.
require "zlib"
require "stringio"

started = Time.now.to_i
written = Zlib::GzipReader.new(StringIO.new(Zlib.gzip("a"))).mtime.to_i
p written >= started && written <= Time.now.to_i

io = StringIO.new
writer = Zlib::GzipWriter.new(io)
writer.write("b")
writer.close
stamped = Zlib::GzipReader.new(StringIO.new(io.string)).mtime.to_i
p stamped >= started

io = StringIO.new
writer = Zlib::GzipWriter.new(io)
writer.mtime = Time.at(1_700_000_000)
writer.write("c")
writer.close
p Zlib::GzipReader.new(StringIO.new(io.string)).mtime.to_i
