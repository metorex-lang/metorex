# Metorex loads no library compiled for MRI, so Bundler takes the ruby
# platform variant of a gem with prebuilt variants, and leaves the choice
# alone for a gem whose library Metorex provides itself.
require "bundler"

p Gem::Dependency.new("nokogiri").default_force_ruby_platform
p Gem::Dependency.new("sqlite3").default_force_ruby_platform
p Gem::Dependency.new("prism").default_force_ruby_platform
p Bundler::LazySpecification.new("nokogiri", "1.19.7", "arm64-darwin").force_ruby_platform
