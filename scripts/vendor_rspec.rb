# Copies RSpec's own rspec-support and rspec-expectations libraries from the
# gems/rspec submodule, and diff-lcs, which rspec-support diffs with, into
# src/vm/stdlib/rspec_vendor, and writes src/vm/stdlib/rspec_libraries.rs,
# which carries them in the binary by the feature name each is required as.
# Their code names Rex where RSpec's names RSpec, so they define their
# classes on Rex, and src/vm/stdlib/rex/expectation_additions.rb adds what
# Rex does beyond them.
#
#   ruby scripts/vendor_rspec.rb
#
# diff-lcs is the one scripts/rspec_suite/Gemfile.lock pins, installed by
# scripts/install_rspec_suite_gems.sh.

require "fileutils"
require "ripper"

root = File.expand_path("..", __dir__)
rspec = File.join(root, "gems", "rspec")
abort "gems/rspec is missing, run: git submodule update --init gems/rspec" unless File.exist?(File.join(rspec, "rspec-core"))

lockfile = File.read(File.join(root, "scripts", "rspec_suite", "Gemfile.lock"))
diff_lcs_version = lockfile[/^    diff-lcs \(([^)]+)\)$/, 1]
abort "scripts/rspec_suite/Gemfile.lock pins no diff-lcs" unless diff_lcs_version
bundle = ENV.fetch("RSPEC_BUNDLE", File.join(root, "target", "rspec_bundle"))
diff_lcs = Dir.glob(File.join(bundle, "ruby", "*", "gems", "diff-lcs-#{diff_lcs_version}")).first
abort "diff-lcs #{diff_lcs_version} is not installed, run: scripts/install_rspec_suite_gems.sh" unless diff_lcs

# Each library's lib directory and the files of it carried. rspec-support's
# spec helpers are left to the load path, which the suites put them on, and
# its CallerFilter to Rex's, which also passes over Rex's own frames and the
# frames of these files inside the binary.
LIBRARIES = {
  "rspec-support" => [File.join(rspec, "rspec-support", "lib"), %w[rspec/support.rb rspec/support/**/*.rb],
                      %w[rspec/support/spec.rb rspec/support/spec/**/*.rb rspec/support/caller_filter.rb]],
  "rspec-expectations" => [File.join(rspec, "rspec-expectations", "lib"),
                           %w[rspec/expectations.rb rspec/expectations/**/*.rb rspec/matchers.rb rspec/matchers/**/*.rb], []],
  "diff-lcs" => [File.join(diff_lcs, "lib"), %w[diff/lcs.rb diff/lcs/**/*.rb], []]
}.freeze

LICENSES = {
  "rspec-support" => File.join(rspec, "rspec-support", "LICENSE.md"),
  "rspec-expectations" => File.join(rspec, "rspec-expectations", "LICENSE.md"),
  "diff-lcs" => File.join(diff_lcs, "LICENCE.md")
}.freeze

# rspec-support requires the files of each RSpec library with a
# `require_relative` handed in as a block, which has no file to be relative
# to inside the binary. The branch that requires each by feature name is
# taken instead.
REQUIRE_RELATIVE_BRANCH = "      elsif Kernel.respond_to?(:require_relative)\n"
BY_FEATURE_NAME = "      elsif false # Metorex carries these files by feature name.\n"

# The three files a program requires to load RSpec, which load Rex under
# Metorex: `RSpec` names Rex, and naming it before Rex has loaded loads Rex.
# RSpec's own files go under these names, which Rex requires in its order.
ENTRY_FILES = {
  "rspec/support" => "rex/rspec/support",
  "rspec/expectations" => "rex/rspec/expectations",
  "rspec/matchers" => "rex/rspec/matchers"
}.freeze
ENTRY_REQUIRE = /require ['"](#{ENTRY_FILES.keys.map { |name| Regexp.escape(name) }.join("|")})['"]/

# The code names its own module as `RSpec`, which is Rex's other name. A
# suite may define an `RSpec` of its own, so the code names Rex. Strings and
# comments keep RSpec's name, which is the one their messages report.
def name_rex(text, path)
  tokens = Ripper.lex(text)
  abort "#{path} does not lex back to itself" unless tokens.map { |token| token[2] }.join == text
  tokens.map { |(_, type, written)| type == :on_const && written == "RSpec" ? "Rex" : written }.join
end

stdlib = File.join(root, "src", "vm", "stdlib")
vendor = File.join(stdlib, "rspec_vendor")
FileUtils.rm_rf(vendor)

entries = []
LIBRARIES.each do |library, (lib, patterns, skipped)|
  left_out = Dir.glob(skipped, base: lib)
  files = Dir.glob(patterns, base: lib).sort - left_out
  files.each do |path|
    text = File.read(File.join(lib, path))
    if path == "rspec/support.rb"
      abort "rspec/support.rb no longer has the require_relative branch it is patched at" unless text.include?(REQUIRE_RELATIVE_BRANCH)
      text = text.sub(REQUIRE_RELATIVE_BRANCH, BY_FEATURE_NAME)
    end
    text = text.gsub(ENTRY_REQUIRE) { %(require "#{ENTRY_FILES.fetch($1)}") }
    text = name_rex(text, path) if library.start_with?("rspec-")
    destination = File.join(vendor, library, path)
    FileUtils.mkdir_p(File.dirname(destination))
    File.write(destination, text)
    feature = path.delete_suffix(".rb")
    entries << %(    ("#{ENTRY_FILES.fetch(feature, feature)}", include_str!("rspec_vendor/#{library}/#{path}")),\n)
  end
  FileUtils.cp(LICENSES.fetch(library), File.join(vendor, library))
end

versions = {
  "rspec-support" => File.read(File.join(rspec, "rspec-support", "lib", "rspec", "support", "version.rb"))[/STRING = '([^']+)'/, 1],
  "rspec-expectations" => File.read(File.join(rspec, "rspec-expectations", "lib", "rspec", "expectations", "version.rb"))[/STRING = '([^']+)'/, 1],
  "diff-lcs" => diff_lcs_version
}

File.write(File.join(stdlib, "rspec_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_rspec.rb from #{versions.map { |name, version| "#{name} #{version}" }.join(", ")}.

  /// RSpec's own libraries Rex runs, and diff-lcs, by the feature name each
  /// is required as.
  pub(crate) const RSPEC_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];
RUST

puts "#{versions.map { |name, version| "#{name} #{version}" }.join(", ")}: #{entries.size} Ruby files"
