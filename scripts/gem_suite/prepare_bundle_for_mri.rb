# Readies the gems installed under one gem directory for MRI, run under MRI:
#
#   ruby scripts/gem_suite/prepare_bundle_for_mri.rb GEM_DIR
#
# Metorex installs gems into the gem suites' directories as simplecov's Rails
# sandboxes run, with their extensions built for Metorex alone, so MRI builds
# its own for each gem that has none, and its Bundler does not warn on every
# run that it is ignoring them. Every gem's executables are written again to
# start with `#!/usr/bin/env ruby`, as Bundler writes them, so `bundle exec`
# under either engine runs one in that engine rather than in the one whose
# path a shebang would name. Only the gems installed under GEM_DIR are
# touched, never Ruby's own default gems.
require "rubygems/installer"

gem_dir = File.expand_path(ARGV.fetch(0))
specs = Dir[File.join(gem_dir, "specifications", "*.gemspec")].filter_map do |path|
  Gem::Specification.load(path)
end

specs.each do |spec|
  next unless spec.missing_extensions?

  puts "Building #{spec.full_name}'s extensions for MRI"
  spec.build_extensions
end

specs.each do |spec|
  next if spec.executables.empty?

  installer = Gem::Installer.for_spec(
    spec,
    install_dir: gem_dir,
    bin_dir: File.join(gem_dir, "bin"),
    env_shebang: true,
    wrappers: true,
    force: true
  )
  installer.generate_bin
end
puts "Wrote the executables of #{specs.count { |spec| !spec.executables.empty? }} gems in #{gem_dir}"
