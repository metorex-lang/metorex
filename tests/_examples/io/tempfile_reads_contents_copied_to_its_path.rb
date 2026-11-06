# A Tempfile reads what another write put at its path, and keywords reach
# it through a method_missing marked with ruby2_keywords.
require "tempfile"
require "fileutils"

source = Tempfile.new("source")
source.write("contents\n")
source.flush

file = Tempfile.new(["copy", ".txt"], encoding: Encoding::BINARY)
FileUtils.copy_file(source.path, file.path)
p file.readlines(chomp: true)
file.rewind

class Wrapper
  def initialize(held)
    @held = held
  end

  def method_missing(name, *arguments, &block)
    @held.__send__(name, *arguments, &block)
  end
  ruby2_keywords(:method_missing)
end

wrapped = Wrapper.new(file)
p wrapped.gets(chomp: true)
wrapped.rewind
p wrapped.readlines(chomp: true)
