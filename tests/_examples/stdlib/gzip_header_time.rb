# A gzip header given no time of its own carries the time it was written,
# one given a time carries that, and one given 0 carries 0, as sprockets
# sets it for output that is the same on every run.
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

io = StringIO.new
writer = Zlib::GzipWriter.new(io, Zlib::BEST_COMPRESSION)
writer.mtime = 0
writer << "foobar"
writer.finish
p io.string.bytes.take(8), io.string.length
