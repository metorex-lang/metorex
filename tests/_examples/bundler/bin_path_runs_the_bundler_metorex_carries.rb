require "bundler/version"
require "rbconfig"

# Bundler's own index holds a spec like this one for the Bundler that is
# running when no installed gem provides it.
if Gem::Specification.find_all_by_name("bundler", Bundler::VERSION).empty?
  Gem::Specification.add_spec(Gem::Specification.new do |spec|
    spec.name = "bundler"
    spec.version = Bundler::VERSION
    spec.bindir = "exe"
    spec.executables = %w[bundle bundler]
    spec.loaded_from = File.join(Gem.dir, "specifications", "bundler-#{Bundler::VERSION}.gemspec")
  end)
end

executable = Gem.bin_path("bundler", "bundle", Bundler::VERSION)
puts IO.popen([RbConfig.ruby, executable, "--version"], &:read)
