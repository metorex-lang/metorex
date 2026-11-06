# Text handed to GzipWriter#write is kept even when writing the header to
# the IO raises, so finish still writes it.
require "zlib"
class Stream
  def initialize(writer) = @writer = writer
  def write(data) = @writer.call(data)
end
Abort = Class.new(RuntimeError)
buf = []
inflater = Zlib::Inflate.new(Zlib::MAX_WBITS + 32)
writer = lambda do |part|
  tmp = inflater.inflate(part)
  p [:part, part.bytesize, tmp]
  buf << tmp if tmp.bytesize > 0
  raise Abort
end
begin
  begin
    gzip = Zlib::GzipWriter.new(Stream.new(writer))
    gzip.write("foo")
  ensure
    gzip.finish
  end
rescue Abort
  p :aborted
end
p((inflater.finish rescue $!))
p buf
