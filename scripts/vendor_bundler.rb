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
  %(    ("#{path.delete_suffix(".rb")}", carried_source!("#{path}")),\n)
end

# Bundler reads the templates `bundle init`, `bundle gem` and its binstubs
# write from the directory beside its own files. Metorex carries them as
# data, written out once to a directory of the user's own, which those reads
# name in place of the one beside Bundler's files.
templates = Dir.glob("bundler/templates/**/*", File::FNM_DOTMATCH, base: source)
  .select { |path| File.file?(File.join(source, path)) }.sort
data = templates.map do |path|
  destination = File.join(stdlib, path)
  FileUtils.mkdir_p(File.dirname(destination))
  FileUtils.cp(File.join(source, path), destination)
  %(    ("#{path}", carried_source!("#{path}")),\n)
end
template_read = /File\.expand_path\("(?:\.\.\/)?(templates[^"]*)", __dir__\)/
rewritten = 0
(["bundler.rb"] + Dir.glob("bundler/**/*.rb", base: source)).each do |path|
  file = File.join(stdlib, path)
  text = File.read(file)
  next unless text.match?(template_read)
  rewritten += text.scan(template_read).size
  File.write(file, text.gsub(template_read) { %(Bundler::CarriedTemplates.path("#{$1}")) })
end
abort "Bundler no longer reads its templates beside its own files" if rewritten.zero?
loader = File.join(stdlib, "bundler.rb")
autoloaded = %(  autoload :CIDetector,             File.expand_path("bundler/ci_detector", __dir__)\n)
text = File.read(loader)
abort "bundler.rb no longer autoloads CIDetector as #{autoloaded.inspect}" unless text.include?(autoloaded)
File.write(loader, text.sub(autoloaded, autoloaded + %(  autoload :CarriedTemplates,       File.expand_path("bundler/metorex/carried_templates", __dir__)\n)))
carried_templates = File.join(stdlib, "bundler", "metorex", "carried_templates.rb")
FileUtils.mkdir_p(File.dirname(carried_templates))
File.write(carried_templates, <<~RUBY)
  # Added by metorex's scripts/vendor_bundler.rb: Bundler reads its templates
  # from a directory beside its own files, and metorex carries them inside the
  # interpreter, so they are written out once to a directory of the user's own
  # and read from there.
  require "fileutils"
  require "tmpdir"

  module Bundler
    module CarriedTemplates
      def self.path(relative)
        File.join(root, relative)
      end

      def self.root
        @root ||= begin
          root = File.join(Dir.tmpdir, "metorex-\#{Process.uid}", "bundler-#{version}")
          unless File.directory?(root)
            writing = "\#{root}.\#{Process.pid}"
            __carried_gem_data__("bundler").each do |path, text|
              file = File.join(writing, path)
              FileUtils.mkdir_p(File.dirname(file))
              File.binwrite(file, text)
            end
            begin
              File.rename(writing, root)
            rescue SystemCallError
              # Another process wrote the same files first.
              FileUtils.rm_rf(writing)
            end
          end
          root
        end
      end
    end
  end
RUBY
entries << %(    ("bundler/metorex/carried_templates", carried_source!("bundler/metorex/carried_templates.rb")),\n)

# Bundler names its own executable for the processes it starts. The one
# metorex carries is the file `metorex bundle` runs, which no gem directory
# holds.
helpers = File.join(stdlib, "bundler", "shared_helpers.rb")
looked_up = %(exe_file = Gem.bin_path("bundler", "bundle", VERSION) unless File.exist?(exe_file))
text = File.read(helpers)
abort "bundler/shared_helpers.rb no longer finds its executable as #{looked_up}" unless text.include?(looked_up)
File.write(helpers, text.sub(looked_up, %(exe_file = "<metorex>/bundler/metorex/bundle.rb" unless File.exist?(exe_file))))

# An installed gem's extensions were built for MRI, and RubyGems' answer to
# whether a gem is missing them builds them for metorex, so Bundler's stub
# asks it in place of answering that they are missing.
stub = File.join(stdlib, "bundler", "stub_specification.rb")
answered = "      return false if manually_installed?\n\n      true\n"
text = File.read(stub)
abort "bundler/stub_specification.rb no longer answers missing_extensions? as #{answered.inspect}" unless text.include?(answered)
File.write(stub, text.sub(answered, answered.sub("      true\n", "      stub.missing_extensions?\n")))

# Metorex loads no library compiled for MRI, so Bundler takes the RUBY
# platform variant of every gem and builds its extensions, except a gem whose
# library metorex provides itself.
force_platform = File.join(stdlib, "bundler", "force_platform.rb")
defaulted = <<-RUBY
    # variant unless the name is explicitly allowlisted.

    def default_force_ruby_platform
      return false unless RUBY_ENGINE == "truffleruby"
RUBY
text = File.read(force_platform)
abort "bundler/force_platform.rb no longer defaults as #{defaulted.inspect}" unless text.include?(defaulted)
File.write(force_platform, text.sub(defaulted, <<-RUBY))
    # variant unless the name is explicitly allowlisted.
    #
    # Metorex loads no library compiled for MRI, so it takes the RUBY platform
    # variant of every gem and builds its extensions, except a gem whose
    # library it provides itself, where the variant installed is never loaded.

    def default_force_ruby_platform
      return !Gem.provided_by_metorex?(name) if Gem.respond_to?(:provided_by_metorex?)
      return false unless RUBY_ENGINE == "truffleruby"
RUBY

executable = File.join(stdlib, "bundler", "metorex", "bundle.rb")
FileUtils.mkdir_p(File.dirname(executable))
File.write(executable, File.read(release.path("libexec", "bundle")))
entries << %(    ("bundler/metorex/bundle", carried_source!("bundler/metorex/bundle.rb")),\n)

File.write(File.join(stdlib, "bundler_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_bundler.rb from Bundler #{version} as Ruby #{release.version} ships it.

  /// The files of Bundler, by the feature name each is required as.
  pub(crate) const BUNDLER_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];

  /// The templates Bundler writes files from, each as its path under
  /// Bundler's directory and its text.
  pub(crate) const BUNDLER_DATA: &[(&str, &str)] = &[
  #{data.join.chomp}
  ];
RUST

puts "Bundler #{version}: #{entries.size} Ruby files, #{data.size} templates"
