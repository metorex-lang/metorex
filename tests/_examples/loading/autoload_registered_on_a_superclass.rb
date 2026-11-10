# An autoload a superclass registered is loaded for that superclass, so a
# subclass reaching it finds the constant the file defines there, and no
# "Expected ... to define" warning is printed under $VERBOSE, as
# Bundler::Source::Path reaches Bundler::Source's Gemspec.
require "tmpdir"

DIRECTORY = Dir.mktmpdir
File.write(File.join(DIRECTORY, "gemspec_source.rb"), <<~RUBY)
  class Source
    class Gemspec < Path
    end
  end
RUBY

class Source
  autoload :Gemspec, File.join(DIRECTORY, "gemspec_source")

  class Path < Source
    def self.gemspec_kind = Gemspec
  end
end

$VERBOSE = true
warnings = []
$stderr.define_singleton_method(:write) { |*text| warnings << text.join; text.join.size }
found = Source::Path.gemspec_kind
$stderr.singleton_class.remove_method(:write)
p found, found.superclass, warnings
