# GzipWriter#flush hands what it has compressed so far to the IO it writes
# to, so a reader can inflate each part as it arrives, and closing the
# writer does not close an IO that has no close of its own.
require "zlib"

class Collector
  attr_reader :parts

  def initialize
    @parts = []
  end

  def write(data)
    @parts << data.dup
    data.bytesize
  end
end

collector = Collector.new
gzip = Zlib::GzipWriter.new(collector)
gzip.write("foo")
gzip.flush
inflater = Zlib::Inflate.new(Zlib::MAX_WBITS + 32)
p collector.parts.map { |part| inflater.inflate(part) }.join
gzip.write("bar")
gzip.flush
p collector.parts.drop(1).map { |part| inflater.inflate(part) }.join
gzip.close
p collector.parts.size > 2
p Zlib.gunzip(collector.parts.join)
p gzip.closed?

require "stringio"
buffer = StringIO.new("".b)
writer = Zlib::GzipWriter.new(buffer)
writer.write("a" * 5000)
writer.flush
writer.write("tail")
p writer.crc == Zlib.crc32("a" * 5000 + "tail")
writer.finish
p buffer.closed?
p Zlib::GzipReader.new(StringIO.new(buffer.string)).read.bytesize
p buffer.string.bytesize < 200

class Recorder
  def write(data)
    p [data.bytesize, data.unpack1("H*")]
    data.bytesize
  end
end
recorded = Zlib::GzipWriter.new(Recorder.new)
recorded.mtime = 1
recorded.write("foo")
recorded.flush
recorded.write("bar")
recorded.flush
recorded.finish
