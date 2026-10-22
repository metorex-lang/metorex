require "zlib"
require "tmpdir"

Dir.mktmpdir do |dir|
  path = File.join(dir, "notes.txt.gz")
  Zlib::GzipWriter.open(path) { |gz| gz.write("maintenance window moves to 02:00\n") }

  Zlib::GzipReader.open(path, encoding: "utf-8") do |gz|
    p gz.external_encoding
    p gz.read
  end

  Zlib::GzipReader.open(path, external_encoding: "ASCII-8BIT") do |gz|
    p gz.read.encoding
  end

  p Zlib::GzipReader.open(path, &:read)
end
