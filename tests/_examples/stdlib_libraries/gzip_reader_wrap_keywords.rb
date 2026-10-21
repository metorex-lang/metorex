# GzipReader.wrap hands the keywords it was given on to the reader, as
# RubyGems reads a gem's checksums.
require "zlib"
require "stringio"

packed = StringIO.new
Zlib::GzipWriter.wrap(packed) { |gzip| gzip.write("invoice 1042: 75.00") }
read = Zlib::GzipReader.wrap(StringIO.new(packed.string), external_encoding: Encoding::UTF_8) do |gzip|
  gzip.read
end
p read
p read.encoding
