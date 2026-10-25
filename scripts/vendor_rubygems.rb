# Copies MRI's RubyGems library into src/vm/stdlib, where `require` finds each
# file by feature name, and writes src/vm/stdlib/rubygems_libraries.rs, which
# carries them in the binary. What metorex sets as the packager of its own
# Ruby is in src/vm/stdlib/rubygems/defaults/operating_system.rb, which MRI
# ships no file of that name over.
#
#   ruby scripts/vendor_rubygems.rb path/to/ruby-X.Y.Z
#
# The library is the one the source tree of that Ruby release carries.

require_relative "ruby_release"

release = RubyRelease.from_arguments("vendor_rubygems.rb")
root = File.expand_path("..", __dir__)
stdlib = File.join(root, "src", "vm", "stdlib")
source = release.path("lib")
rubygems_version = File.read(File.join(source, "rubygems.rb"))[/VERSION = "([^"]+)"/, 1]
files = ["rubygems.rb"] + Dir.glob("rubygems/**/*.rb", base: source).sort

# MRI evaluates its own Kernel#require from the file on disk. Metorex's
# require activates gems through Gem.try_activate itself, so the file is not
# read.
KERNEL_REQUIRE = <<~'RUBY'
  path = File.join(__dir__, "rubygems/core_ext/kernel_require.rb")
  # When https://bugs.ruby-lang.org/issues/17259 is available, there is no need to override Kernel#warn
  if RUBY_ENGINE == "truffleruby" ||
     RUBY_ENGINE == "ruby"
    file = "<internal:#{path}>"
  else
    require_relative "rubygems/core_ext/kernel_warn"
    file = path
  end
  eval File.read(path), nil, file
RUBY

entries = files.map do |path|
  text = File.read(File.join(source, path))
  directory = File.join("/", File.dirname(path))
  if path == "rubygems.rb"
    raise "rubygems.rb no longer evaluates kernel_require.rb as expected" unless text.include?(KERNEL_REQUIRE)
    text = text.sub(KERNEL_REQUIRE, "# Metorex's require activates gems through Gem.try_activate.\n")
  end
  text = require_by_feature(text, File.dirname(path))
  # An autoload of a path beside the file names the feature it resolves to.
  text = text.gsub(/autoload(\s+):(\w+),(\s+)File\.expand_path\("([^"]+)", __dir__\)/) do
    %(autoload#{$1}:#{$2},#{$3}"#{File.expand_path($4, directory).delete_prefix("/")}")
  end
  destination = File.join(stdlib, path)
  FileUtils.mkdir_p(File.dirname(destination))
  File.write(destination, text)
  %(    ("#{path.delete_suffix(".rb")}", carried_source!("#{path}")),\n)
end

File.write(File.join(stdlib, "rubygems_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_rubygems.rb from RubyGems #{rubygems_version} as Ruby #{release.version} ships it.

  /// The files of RubyGems, by the feature name each is required as.
  pub(crate) const RUBYGEMS_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];
RUST

puts "RubyGems #{rubygems_version}: #{entries.size} Ruby files"
