# Copies prism into this repository: its C parser into vendor/prism,
# which build.rs compiles into the binary, and its Ruby library into
# src/vm/stdlib/prism, which `require "prism"` loads.
#
#   ruby scripts/vendor_prism.rb path/to/ruby-X.Y.Z
#
# The parser is the one the source tree of that Ruby release carries, which
# lays out the gem's src and include directories as one prism directory with
# the C extension's files beside them.

require_relative "ruby_release"

release = RubyRelease.from_arguments("vendor_prism.rb")
root = File.expand_path("..", __dir__)
parser = release.path("prism")
EXTENSION_FILES = %w[extension.c extension.h api_node.c api_pack.c].freeze

# The license is the gem's, which the release does not carry, so it stays.
vendor = File.join(root, "vendor", "prism")
FileUtils.rm_rf(File.join(vendor, "src"))
FileUtils.rm_rf(File.join(vendor, "include"))
Dir.glob("{*,util/*}.{c,h}", base: parser).sort.each do |path|
  next if EXTENSION_FILES.include?(path)
  destination = if path.end_with?(".c")
    File.join(vendor, "src", path)
  elsif path == "prism.h"
    File.join(vendor, "include", path)
  else
    File.join(vendor, "include", "prism", path)
  end
  FileUtils.mkdir_p(File.dirname(destination))
  FileUtils.cp(File.join(parser, path), destination)
end
version = File.read(File.join(vendor, "include", "prism", "version.h"))[/PRISM_VERSION "([^"]+)"/, 1]
File.write(File.join(vendor, "VERSION"), "#{version}\n")

# The C extension is replaced by src/vm/stdlib/prism_backend.rb, and the FFI
# backend is not used.
lib = release.path("lib")
skipped = %w[prism/ffi.rb]
sources = Dir.glob("{prism.rb,prism/**/*.rb}", base: lib).sort.reject { |path| skipped.include?(path) }

library = File.join(root, "src", "vm", "stdlib", "prism")
FileUtils.rm_rf(library)
sources.each do |path|
  text = require_by_feature(File.read(File.join(lib, path)), File.dirname(path))
  destination = File.join(library, path)
  FileUtils.mkdir_p(File.dirname(destination))
  File.write(destination, text)
end

entries = sources.map do |path|
  feature = path.delete_suffix(".rb")
  %(    ("#{feature}", carried_source!("prism/#{path}")),\n)
end
File.write(File.join(root, "src", "vm", "stdlib", "prism_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_prism.rb from prism #{version} as Ruby #{release.version} ships it.

  /// The files of prism's Ruby library, by the feature name each is required as.
  pub(crate) const PRISM_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];
RUST

puts "prism #{version}: #{Dir.glob("**/*.c", base: vendor).size} C files, #{sources.size} Ruby files"
