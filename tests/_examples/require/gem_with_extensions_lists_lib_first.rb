require "rubygems"
require "tmpdir"

Dir.mktmpdir do |base|
  Dir.mkdir File.join(base, "lib")
  spec = Gem::Specification.new do |gem|
    gem.name = "demo"
    gem.version = "1.0"
    gem.extensions = ["ext/demo/extconf.rb"]
    gem.require_paths = ["lib"]
  end
  spec.full_gem_path = base
  paths = spec.full_require_paths
  p paths.length
  p paths.first == File.join(base, "lib")
  p paths.last.end_with?("demo-1.0")
end
