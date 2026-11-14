# Copies the default gems MRI writes in Ruby that metorex carries unchanged,
# erb, ipaddr, json, net-http, net-protocol, open-uri, optparse, psych,
# tempfile, tmpdir, uri, yaml, and openssl's buffering.rb, into
# src/vm/stdlib, where
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
  "ipaddr" => %w[ipaddr.rb],
  "net-http" => %w[net/http.rb net/https.rb net/http/*.rb],
  "net-protocol" => %w[net/protocol.rb],
  "open-uri" => %w[open-uri.rb],
  "optparse" => %w[optparse.rb optparse/*.rb],
  "tempfile" => %w[tempfile.rb],
  "tmpdir" => %w[tmpdir.rb],
  "uri" => %w[uri.rb uri/*.rb],
  "yaml" => %w[yaml.rb yaml/*.rb]
}.freeze

# Psych, OpenSSL, and json keep their Ruby halves beside their C extensions.
# src/vm/stdlib/psych_native.rb and src/vm/stdlib/openssl.rb play the parts
# of the first two, and json's is compiled into the binary from vendor/json.
EXTENSION_GEMS = {
  "psych" => ["ext/psych/lib", %w[psych.rb psych/*.rb psych/**/*.rb]],
  "json" => ["ext/json/lib", %w[json.rb json/*.rb json/**/*.rb]],
  "openssl" => ["ext/openssl/lib", %w[openssl/buffering.rb]]
}.freeze

# uri.rb is carried as uri_vendored.rb, and open-uri.rb as open_uri.rb.
RENAMED = { "uri.rb" => "uri_vendored.rb", "open-uri.rb" => "open_uri.rb" }.freeze

sources = GEMS.map { |name, patterns| [name, lib, patterns] } +
          EXTENSION_GEMS.map { |name, (base, patterns)| [name, release.path(base), patterns] }
versions = sources.map do |name, base, patterns|
  Dir.glob(patterns, base: base).sort.uniq.each do |path|
    text = require_by_feature(File.read(File.join(base, path)), File.dirname(path))
    destination = File.join(stdlib, RENAMED.fetch(path, path))
    FileUtils.mkdir_p(File.dirname(destination))
    File.write(destination, text)
  end
  "#{name} #{release.default_gem_version(name)}"
end

puts "#{versions.join(", ")} as Ruby #{release.version} ships them"
