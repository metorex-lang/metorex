# The source tree of a Ruby release, as its tarball from cache.ruby-lang.org
# unpacks, which the vendor scripts copy from. The tarball carries the default
# gems' libraries under lib and ext, prism's C parser under prism, and each
# bundled gem as the .gem file listed in gems/bundled_gems.

require "fileutils"
require "rubygems/package"

class RubyRelease
  attr_reader :root

  def self.from_arguments(script)
    abort "usage: ruby scripts/#{script} path/to/ruby-X.Y.Z" unless ARGV[0]
    new(ARGV[0])
  end

  def initialize(root)
    @root = File.expand_path(root)
    abort "no Ruby release source at #{@root}" unless File.exist?(path("gems", "bundled_gems"))
  end

  def path(*parts)
    File.join(root, *parts)
  end

  def version
    teeny = File.read(path("version.h"))[/#define RUBY_VERSION_TEENY (\d+)/, 1]
    api = File.read(path("include", "ruby", "version.h"))
    major = api[/#define RUBY_API_VERSION_MAJOR (\d+)/, 1]
    minor = api[/#define RUBY_API_VERSION_MINOR (\d+)/, 1]
    "#{major}.#{minor}.#{teeny}"
  end

  def full_revision
    File.read(path("revision.h"))[/#define RUBY_FULL_REVISION "(\h+)"/, 1]
  end

  def bundled_gem_version(name)
    line = File.readlines(path("gems", "bundled_gems")).find { |entry| entry.split.first == name }
    abort "#{name} is not a bundled gem of Ruby #{version}" unless line
    line.split[1]
  end

  # The bundled gem unpacked under target/ruby-release, answering its directory.
  def bundled_gem(name)
    full_name = "#{name}-#{bundled_gem_version(name)}"
    directory = File.expand_path("../target/ruby-release/gems/#{full_name}", __dir__)
    unless File.exist?(directory)
      FileUtils.mkdir_p(directory)
      Gem::Package.new(path("gems", "#{full_name}.gem")).extract_files(directory)
    end
    directory
  end

  def default_gem_version(name)
    gemspec = Dir.glob(path("{lib,ext}", "**", "#{name}.gemspec")).first
    abort "#{name} is not a default gem of Ruby #{version}" unless gemspec
    Dir.chdir(File.dirname(gemspec)) { Gem::Specification.load(gemspec).version.to_s }
  end
end

# A library carried in the binary is required by feature name, so a relative
# require names the feature it resolves to. `directory` is the file's own
# feature directory, such as "net" for net/http.rb, and `requiring` is the
# call written in its place.
def require_by_feature(text, directory, requiring = "require")
  # A path written with `#{}` inside a string is text the file writes, not a
  # require of its own.
  text.gsub(/require_relative[ (]["']([^"'#]+)["']\)?/) do
    %(#{requiring} "#{File.expand_path($1, File.join("/", directory)).delete_prefix("/")}")
  end
end
