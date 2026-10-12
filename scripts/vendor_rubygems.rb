# Copies MRI's RubyGems library into src/vm/stdlib, where `require` finds each
# file by feature name, and writes src/vm/stdlib/rubygems_libraries.rs, which
# carries them in the binary. What metorex sets as the packager of its own
# Ruby is in src/vm/stdlib/rubygems/defaults/operating_system.rb, which MRI
# ships no file of that name over.
#
#   ruby scripts/vendor_rubygems.rb
#
# The library is the one the running Ruby loads.

require "fileutils"

root = File.expand_path("..", __dir__)
stdlib = File.join(root, "src", "vm", "stdlib")
source = RbConfig::CONFIG["rubylibdir"]
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
  # An embedded library is required by feature name, so a relative require
  # or an autoload of a path beside the file names the feature it resolves to.
  text = text.gsub(/require_relative[ (]["']([^"']+)["']\)?/) do
    %(require "#{File.expand_path($1, directory).delete_prefix("/")}")
  end
  text = text.gsub(/autoload(\s+):(\w+),(\s+)File\.expand_path\("([^"]+)", __dir__\)/) do
    %(autoload#{$1}:#{$2},#{$3}"#{File.expand_path($4, directory).delete_prefix("/")}")
  end
  destination = File.join(stdlib, path)
  FileUtils.mkdir_p(File.dirname(destination))
  File.write(destination, text)
  %(    ("#{path.delete_suffix(".rb")}", include_str!("#{path}")),\n)
end

File.write(File.join(stdlib, "rubygems_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_rubygems.rb from RubyGems #{Gem::VERSION} as Ruby #{RUBY_VERSION} ships it.

  /// The files of RubyGems, by the feature name each is required as.
  pub(crate) const RUBYGEMS_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];
RUST

puts "RubyGems #{Gem::VERSION}: #{entries.size} Ruby files"
