require "zlib"
require "stringio"

class Opts < Hash
  def inspect
    "#<#{self.class.name} #{super}>"
  end
end
options = Opts.new
p options.inspect
options[:a] = 1
p options.inspect

class Object
  def with_marker
    :marked
  end
end
NilClass.undef_method(:with_marker)
p nil.respond_to?(:with_marker)
p true.respond_to?(:with_marker)

begin
  load "nor/this/one.rb"
rescue LoadError => error
  p error.path
end

[Zlib::BEST_SPEED, Zlib::BEST_COMPRESSION, Zlib::DEFAULT_COMPRESSION, 5].each do |level|
  output = StringIO.new("".b)
  writer = Zlib::GzipWriter.new(output, level)
  writer.write("Hello World" * 100)
  writer.close
  p [level, output.string.getbyte(8), Zlib::GzipReader.new(StringIO.new(output.string)).level]
end
