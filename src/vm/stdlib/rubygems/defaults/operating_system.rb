# What metorex sets for RubyGems as the packager of its own Ruby, which
# RubyGems reads from here as it would an operating system's defaults. Gems go
# under ~/.gem/metorex when GEM_HOME names no place. A gem's C extensions are
# built against metorex's headers the first time the gem is needed, into a
# directory named for a digest of those headers, since an installed gem's
# extensions were built for MRI. A gem whose library metorex carries, such as
# prism, is left to the copy metorex carries when another gem depends on it,
# and an installed copy of one whose extensions metorex provides is not put on
# the load path.

module Gem
  def self.default_dir
    File.join(Dir.home, ".gem", "metorex")
  end

  def self.default_path
    [default_dir]
  end

  # A digest of the C headers metorex compiles extensions against.
  def self.extension_api_version
    @extension_api_version ||= begin
      header = File.join(RbConfig::CONFIG["rubyhdrdir"], "ruby", "internal", "abi.h")
      File.exist?(header) ? File.read(header)[/RUBY_ABI_VERSION 0x(\h+)/, 1].to_s : ""
    end
  end

  def self.default_ext_dir_for(_base_dir)
    File.join(default_dir, "extensions", RbConfig::CONFIG["arch"], extension_api_version)
  end

  # Whether metorex carries the library a gem of this name provides. The
  # table of names is built the first time this is asked rather than at
  # startup.
  def self.carried?(name)
    @carried_libraries ||= __embedded_library_names__().to_h { |library| [library, true] }
    @carried_libraries.key?(name.tr("-", "/"))
  end

  # The gems Ruby ships whose C extensions metorex provides itself, compiled
  # in or written in Rust.
  EXTENSIONS_METOREX_PROVIDES = %w[
    bigdecimal date debug digest erb etc fcntl fiddle io-console io-nonblock
    io-wait json nkf openssl prism psych racc rbs resolv stringio strscan
    syslog zlib
  ].freeze

  # The RSpec gems, which Rex answers for when `require "rspec"` and its
  # siblings load it.
  REX_ANSWERS_FOR = %w[rspec rspec-core rspec-expectations rspec-mocks rspec-support].freeze

  # Whether metorex provides a gem of this name itself, so an installed copy
  # is left off the load path in favor of metorex's own: one whose C
  # extensions an installed copy built for MRI, or one of RSpec's.
  def self.provided_by_metorex?(name)
    (EXTENSIONS_METOREX_PROVIDES.include?(name) && carried?(name)) || REX_ANSWERS_FOR.include?(name)
  end

  module MetorexStubSpecification
    def extensions
      Gem.provided_by_metorex?(name) ? [] : super
    end
  end

  StubSpecification.prepend(MetorexStubSpecification)

  # Such a gem puts nothing on the load path, and holds no file a `require`
  # could activate it for, so a library it would have provided and metorex
  # does not is a LoadError rather than an activation that finds nothing.
  # The libraries built for metorex come ahead of the gem's own directories,
  # where installing it copied the ones built for MRI.
  module MetorexRequirePaths
    def full_require_paths
      return [] if Gem.provided_by_metorex?(name)

      paths = super
      return paths unless have_extensions?

      [extension_dir] | paths
    end

    def contains_requirable_file?(file)
      Gem.provided_by_metorex?(name) ? false : super
    end
  end

  BasicSpecification.prepend(MetorexRequirePaths)

  module MetorexSpecification
    # A gem's extensions count as missing only once building them for
    # metorex has been tried. A process an extension's build started builds
    # none of its own, since the gem it is building would count as missing
    # its extensions there too.
    def missing_extensions?
      return false if Gem.provided_by_metorex?(name)
      return false unless super
      return true if @extensions_tried_for_metorex || ENV[BUILDING_EXTENSIONS]

      @extensions_tried_for_metorex = true
      build_extensions_for_metorex
      super
    end

    def activate
      missing_extensions?
      super
    end

    def activate_dependencies
      @activating_for_metorex = true
      super
    ensure
      @activating_for_metorex = false
    end

    def runtime_dependencies
      dependencies = super
      return dependencies unless @activating_for_metorex

      dependencies.reject { |dependency| Gem.carried?(dependency.name) }
    end

    private

    BUILDING_EXTENSIONS = "METOREX_BUILDING_EXTENSIONS"

    # The environment an extension's build runs in: the one the program
    # started with, before Bundler named its bundle and put bundler/setup in
    # RUBYOPT, and marked as a build's.
    def environment_for_extension_builds
      changes = { BUILDING_EXTENSIONS => "1" }
      return changes unless defined?(Bundler) && Bundler.respond_to?(:unbundled_env)

      unbundled = Bundler.unbundled_env
      ENV.each_key { |key| changes[key] = nil unless unbundled.key?(key) }
      unbundled.each { |key, value| changes[key] = value unless ENV[key] == value }
      changes
    end

    # Run each extconf.rb under metorex in a directory of metorex's own, so
    # the installed gem is left as it is, then make, keeping the output in
    # gem_make.out beside the libraries built.
    def build_extensions_for_metorex
      require "fileutils"
      target = extension_dir
      FileUtils.mkdir_p(target)
      # Processes started together each find the extensions missing. The
      # first to take the lock builds them, and the others find them built.
      File.open(File.join(target, "gem_build.lock"), File::RDWR | File::CREAT) do |lock|
        lock.flock(File::LOCK_EX)
        build_extensions_into(target) unless File.exist?(gem_build_complete_path)
      end
    end

    def build_extensions_into(target)
      log = File.join(target, "gem_make.out")
      built = File.open(log, "w") do |output|
        extensions.all? do |extconf|
          build = File.join(target, "build", File.dirname(extconf))
          FileUtils.mkdir_p(build)
          environment = environment_for_extension_builds
          made = Dir.chdir(build) do
            system(environment, Gem.ruby, File.join(full_gem_path, extconf), out: output, err: output) &&
              system(environment, "make", out: output, err: output)
          end
          made && install_libraries_built_for_metorex(build, target)
        end
      end
      if built
        File.write(gem_build_complete_path, "")
      else
        warn "Ignoring #{full_name} because its extensions could not be built for metorex. See #{log}"
      end
    end

    # Copy the libraries a build made to where the extension's name puts
    # them under the gem's extension directory.
    def install_libraries_built_for_metorex(build, target)
      makefile = File.read(File.join(build, "Makefile"))
      prefix = makefile[/^target_prefix = (.*)$/, 1].to_s.delete_prefix("/")
      destination = File.join(target, prefix)
      FileUtils.mkdir_p(destination)
      libraries = Dir[File.join(build, "*.#{RbConfig::CONFIG["DLEXT"]}")]
      libraries.each { |library| FileUtils.cp(library, destination) }
      !libraries.empty?
    end
  end

  Specification.prepend(MetorexSpecification)
end
