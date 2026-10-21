# Copies MRI's Bundler library into src/vm/stdlib, where `require` finds each
# file by feature name, and writes src/vm/stdlib/bundler_libraries.rs, which
# carries them in the binary. The `bundle` executable MRI installs is carried
# as `bundler/metorex/bundle`, which `metorex bundle` runs.
#
#   ruby scripts/vendor_bundler.rb path/to/ruby-X.Y.Z
#
# The library is the one the source tree of that Ruby release carries.

require_relative "ruby_release"

release = RubyRelease.from_arguments("vendor_bundler.rb")
root = File.expand_path("..", __dir__)
stdlib = File.join(root, "src", "vm", "stdlib")
source = release.path("lib")
version = release.default_gem_version("bundler")
files = ["bundler.rb"] + Dir.glob("bundler/**/*.rb", base: source).sort

# A relative require names the feature it resolves to. Bundler defines a
# `require` of its own, which a bare `require` inside `module Bundler` would
# reach, so the rewritten call names Kernel's.
entries = files.map do |path|
  text = require_by_feature(File.read(File.join(source, path)), File.dirname(path), "Kernel.require")
  destination = File.join(stdlib, path)
  FileUtils.mkdir_p(File.dirname(destination))
  File.write(destination, text)
  %(    ("#{path.delete_suffix(".rb")}", include_str!("#{path}")),\n)
end

# Bundler names its own executable for the processes it starts. The one
# metorex carries is the file `metorex bundle` runs, which no gem directory
# holds.
helpers = File.join(stdlib, "bundler", "shared_helpers.rb")
looked_up = %(exe_file = Gem.bin_path("bundler", "bundle", VERSION) unless File.exist?(exe_file))
text = File.read(helpers)
abort "bundler/shared_helpers.rb no longer finds its executable as #{looked_up}" unless text.include?(looked_up)
File.write(helpers, text.sub(looked_up, %(exe_file = "<metorex>/bundler/metorex/bundle.rb" unless File.exist?(exe_file))))

executable = File.join(stdlib, "bundler", "metorex", "bundle.rb")
FileUtils.mkdir_p(File.dirname(executable))
File.write(executable, File.read(release.path("libexec", "bundle")))
entries << %(    ("bundler/metorex/bundle", include_str!("bundler/metorex/bundle.rb")),\n)

File.write(File.join(stdlib, "bundler_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_bundler.rb from Bundler #{version} as Ruby #{release.version} ships it.

  /// The files of Bundler, by the feature name each is required as.
  pub(crate) const BUNDLER_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];
RUST

puts "Bundler #{version}: #{entries.size} Ruby files"
