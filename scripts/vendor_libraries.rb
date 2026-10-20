# Copies the default gems MRI writes in Ruby that metorex carries unchanged,
# erb, net-http, net-protocol, optparse, and uri, into src/vm/stdlib, where
# `require` finds each file by feature name through the table in
# src/vm/stdlib/mod.rs.
#
#   ruby scripts/vendor_libraries.rb path/to/ruby-X.Y.Z
#
# The libraries are the ones the source tree of that Ruby release carries.

require_relative "ruby_release"

release = RubyRelease.from_arguments("vendor_libraries.rb")
stdlib = File.expand_path("../src/vm/stdlib", __dir__)
lib = release.path("lib")

GEMS = {
  "erb" => %w[erb.rb erb/*.rb],
  "net-http" => %w[net/http.rb net/https.rb net/http/*.rb],
  "net-protocol" => %w[net/protocol.rb],
  "optparse" => %w[optparse.rb optparse/*.rb],
  "uri" => %w[uri.rb uri/*.rb]
}.freeze

# uri.rb is carried as uri_vendored.rb.
RENAMED = { "uri.rb" => "uri_vendored.rb" }.freeze

versions = GEMS.map do |name, patterns|
  Dir.glob(patterns, base: lib).sort.each do |path|
    text = require_by_feature(File.read(File.join(lib, path)), File.dirname(path))
    destination = File.join(stdlib, RENAMED.fetch(path, path))
    FileUtils.mkdir_p(File.dirname(destination))
    File.write(destination, text)
  end
  "#{name} #{release.default_gem_version(name)}"
end

puts "#{versions.join(", ")} as Ruby #{release.version} ships them"
