# Copies the bundled gems MRI ships that metorex carries into this repository.
# Each gem's Ruby library goes into src/vm/stdlib/gems, which `require` finds
# by feature name, and the C extensions of nkf, rbs, and debug into
# vendor/nkf, vendor/rbs, and vendor/debug, which build.rs compiles into the
# binary. MRI's resolv.rb, which resolv-replace and rinda's
# ring build on, and its pp.rb and prettyprint.rb, which irb prints with,
# replace metorex's own.
#
#   ruby scripts/vendor_gems.rb path/to/ruby-X.Y.Z
#
# Everything is copied from the source tree of a Ruby release, at the
# versions that release ships.

require_relative "ruby_release"

release = RubyRelease.from_arguments("vendor_gems.rb")

GEMS = %w[
  debug irb minitest mutex_m net-imap net-pop net-smtp nkf power_assert racc rake rbs rdoc readline
  reline repl_type_completor resolv-replace rexml rinda rss test-unit typeprof
].freeze

root = File.expand_path("..", __dir__)
library = File.join(root, "src", "vm", "stdlib", "gems")
FileUtils.rm_rf(library)

entries = []
versions = []
GEMS.each do |name|
  gem_dir = release.bundled_gem(name)
  versions << "#{name} #{release.bundled_gem_version(name)}"
  lib = File.join(gem_dir, "lib")
  license = %w[LICENSE.txt BSDL COPYING].map { |file| File.join(gem_dir, file) }.select { |path| File.exist?(path) }
  license.each do |path|
    FileUtils.mkdir_p(File.join(library, name))
    FileUtils.cp(path, File.join(library, name, File.basename(path)))
  end
  Dir.glob("**/*.rb", base: lib).sort.each do |path|
    text = require_by_feature(File.read(File.join(lib, path)), File.dirname(path))
    destination = File.join(library, name, path)
    FileUtils.mkdir_p(File.dirname(destination))
    File.write(destination, text)
    entries << %(    ("#{path.delete_suffix(".rb")}", include_str!("gems/#{name}/#{path}")),\n)
  end
end

# irb finds its message files along the load path, where metorex has none of
# its own: they are carried with the rest of irb, so irb is told to look there.
File.open(File.join(library, "irb", "irb", "locale.rb"), "a") do |locale|
  locale.write(<<~'RUBY')

    # Added by metorex's scripts/vendor_gems.rb: irb's message files are carried
    # inside the interpreter with the rest of irb, and found there by name.
    module IRB
      class Locale
        alias_method :__search_file_on_disk__, :search_file
        def search_file(lib_paths, dir, file)
          each_localized_path(dir, file) do |lc_path|
            if __embedded_library_names__().include?(lc_path.delete_suffix(".rb"))
              return "<metorex>/#{lc_path}"
            end
          end
          __search_file_on_disk__(lib_paths, dir, file)
        end

        alias_method :__load_from_disk__, :load
        def load(file)
          found = find(file)
          return __load_from_disk__(file) unless found&.start_with?("<metorex>/")
          require found.delete_prefix("<metorex>/").delete_suffix(".rb")
        end
      end
    end
  RUBY
end

# rbs reads the signatures of Ruby's core and standard library from the
# directories beside its library. They are carried with it, and written out
# once to a directory of the user's own, which rbs is told to read instead.
rbs_version = release.bundled_gem_version("rbs")
data = []
%w[core stdlib].each do |part|
  base = File.join(release.bundled_gem("rbs"), part)
  Dir.glob("**/*", base: base).sort.each do |path|
    source = File.join(base, path)
    next unless File.file?(source)
    destination = File.join(library, "rbs", "data", part, path)
    FileUtils.mkdir_p(File.dirname(destination))
    FileUtils.cp(source, destination)
    data << %(    ("rbs/#{part}/#{path}", include_str!("gems/rbs/data/#{part}/#{path}")),\n)
  end
end
File.write(File.join(library, "rbs", "rbs", "carried_data.rb"), <<~RUBY)
  # Added by metorex's scripts/vendor_gems.rb: rbs reads the signatures of
  # Ruby's core and standard library from directories beside its own, and
  # metorex carries them inside the interpreter, so they are written out once
  # to a directory of the user's own and read from there.
  require "fileutils"
  require "tmpdir"

  module RBS
    module CarriedData
      def self.root
        @root ||= begin
          root = File.join(Dir.tmpdir, "metorex-\#{Process.uid}", "rbs-#{rbs_version}")
          unless File.directory?(root)
            writing = "\#{root}.\#{Process.pid}"
            __carried_gem_data__("rbs").each do |path, text|
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
entries << %(    ("rbs/carried_data", include_str!("gems/rbs/rbs/carried_data.rb")),\n)
{
  File.join("rbs", "repository.rb") => ["DEFAULT_STDLIB_ROOT", "stdlib"],
  File.join("rbs", "environment_loader.rb") => ["DEFAULT_CORE_ROOT", "core"]
}.each do |path, (constant, part)|
  file = File.join(library, "rbs", path)
  text = File.read(file)
  written = %(#{constant} = Pathname(_ = __dir__) + "../../#{part}")
  abort "#{path} no longer sets #{constant} as #{written}" unless text.include?(written)
  File.write(file, text.sub(written, %(require "rbs/carried_data"\n    #{constant} = Pathname(RBS::CarriedData.root) + "#{part}")))
end

File.write(File.join(root, "src", "vm", "stdlib", "gem_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_gems.rb from #{versions.join(", ")} as Ruby #{release.version} ships them.

  /// The files of the bundled gems metorex carries, by the feature name each
  /// is required as.
  pub(crate) const GEM_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];

  /// The files other than its library a bundled gem reads, by the gem's name
  /// and the path under its directory.
  pub(crate) const GEM_DATA: &[(&str, &str)] = &[
  #{data.join.chomp}
  ];
RUST

# Libraries MRI writes in Ruby that metorex carries as MRI has them: its own
# resolv.rb, io/console/size.rb, pp.rb, prettyprint.rb, and pathname.rb, and
# pathname_builtin.rb, which MRI compiles in rather than installing.
{
  release.path("lib", "resolv.rb") => "resolv.rb",
  release.path("lib", "pp.rb") => "pp.rb",
  release.path("lib", "prettyprint.rb") => "prettyprint.rb",
  release.path("ext", "io", "console", "lib", "console", "size.rb") => "io_console_size.rb",
  release.path("lib", "pathname.rb") => "pathname.rb",
  release.path("pathname_builtin.rb") => "pathname_builtin.rb",
}.each do |source, name|
  FileUtils.cp(source, File.join(root, "src", "vm", "stdlib", name))
end

# nkf's extension: its Ruby binding and the nkf library it includes.
nkf_ext = File.join(release.bundled_gem("nkf"), "ext", "nkf")
vendor = File.join(root, "vendor", "nkf")
FileUtils.rm_rf(vendor)
FileUtils.mkdir_p(File.join(vendor, "nkf-utf8"))
FileUtils.cp(File.join(nkf_ext, "nkf.c"), vendor)
Dir.glob(File.join(nkf_ext, "nkf-utf8", "*.{c,h}")).each { |path| FileUtils.cp(path, File.join(vendor, "nkf-utf8")) }
FileUtils.cp(File.join(release.bundled_gem("nkf"), "LICENSE.txt"), vendor)

# rbs's extension and the signature parser it wraps.
rbs = release.bundled_gem("rbs")
vendor = File.join(root, "vendor", "rbs")
FileUtils.rm_rf(vendor)
Dir.glob("{src,include,ext/rbs_extension}/**/*.{c,h}", base: rbs).each do |path|
  FileUtils.mkdir_p(File.join(vendor, File.dirname(path)))
  FileUtils.cp(File.join(rbs, path), File.join(vendor, path))
end
%w[BSDL COPYING].each { |file| FileUtils.cp(File.join(rbs, file), vendor) }

# debug's extension, with the version header its extconf.rb writes.
debug = release.bundled_gem("debug")
vendor = File.join(root, "vendor", "debug")
FileUtils.rm_rf(vendor)
FileUtils.mkdir_p(vendor)
Dir.glob(File.join(debug, "ext", "debug", "*.{c,h}")).each { |path| FileUtils.cp(path, vendor) }
File.write(File.join(vendor, "debug_version.h"),
           %(#define RUBY_DEBUG_VERSION "#{release.bundled_gem_version("debug")}"\n))
FileUtils.cp(File.join(debug, "LICENSE.txt"), vendor)

puts "#{versions.join(", ")}: #{entries.size} Ruby files"

# The assertions Ruby's own repository keeps in tool/lib, which the bundled
# gems' suites load as MRI's own runs of them do, for scripts/run_gem_suites.sh.
helpers = File.join(root, "scripts", "gem_suite", "lib")
FileUtils.rm_rf(helpers)
FileUtils.mkdir_p(helpers)
%w[core_assertions.rb envutil.rb find_executable.rb memory_status.rb].each do |name|
  FileUtils.cp(release.path("tool", "lib", name), helpers)
end
%w[BSDL COPYING].each { |name| FileUtils.cp(release.path(name), File.dirname(helpers)) }
