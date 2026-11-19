# What metorex sets for RubyGems as the packager of its own Ruby, which
# RubyGems reads from here as it would an operating system's defaults. Gems go
# under ~/.gem/metorex when GEM_HOME names no place. A gem's C extensions are
# built against metorex's headers the first time the gem is needed, into a
# directory named for a digest of those headers beside the gem, since an installed gem's
# extensions were built for MRI. A gem whose library metorex carries, such as
# prism, is left to the copy metorex carries when another gem depends on it,
# and an installed copy of one whose extensions metorex provides is not put on
# the load path, except a json of another version than the one carried.

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

  # Beside the gem when its directory is writable, as MRI keeps them, so a
  # process that changes HOME finds the ones built before.
  def self.default_ext_dir_for(base_dir)
    holder = base_dir && File.writable?(base_dir) ? base_dir : default_dir
    File.join(holder, "extensions", RbConfig::CONFIG["arch"], extension_api_version)
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

  # The gems whose carried copy stands in only for an installed copy of the
  # same version, with the version carried. An installed copy of another
  # version has its extensions built for metorex and is loaded, as MRI
  # loads the version a bundle locks, unless the build fails.
  CARRIED_VERSIONS = { "json" => "2.18.0" }.freeze

  # Whether metorex provides a gem of this name, and of this version when
  # one is given, itself, so an installed copy is left off the load path in
  # favor of metorex's own: one whose C extensions an installed copy built
  # for MRI, or one of RSpec's.
  def self.provided_by_metorex?(name, version = nil)
    return true if REX_ANSWERS_FOR.include?(name)
    return false unless EXTENSIONS_METOREX_PROVIDES.include?(name) && carried?(name)

    version.nil? || !CARRIED_VERSIONS.key?(name) || CARRIED_VERSIONS[name] == version.to_s
  end

  # Whether the carried copy stands in for an installed copy of this gem
  # whose extensions did not build for metorex.
  def self.carried_when_unbuilt?(name)
    CARRIED_VERSIONS.key?(name) && carried?(name)
  end

  module MetorexStubSpecification
    def extensions
      Gem.provided_by_metorex?(name, version) ? [] : super
    end
  end

  StubSpecification.prepend(MetorexStubSpecification)

  # Such a gem puts nothing on the load path, and holds no file a `require`
  # could activate it for, so a library it would have provided and metorex
  # does not is a LoadError rather than an activation that finds nothing.
  # The libraries built for metorex come ahead of the gem's own directories
  # when installing it copied the ones built for MRI there.
  module MetorexRequirePaths
    def full_require_paths
      return [] if Gem.provided_by_metorex?(name, version) || unbuilt_with_a_carried_copy?

      paths = super
      return paths unless have_extensions? && holds_libraries_built_for_mri?(paths)

      [extension_dir] | paths
    end

    def holds_libraries_built_for_mri?(paths)
      return @holds_libraries_built_for_mri unless @holds_libraries_built_for_mri.nil?

      pattern = "**/*.#{RbConfig::CONFIG["DLEXT"]}"
      @holds_libraries_built_for_mri = (paths - [extension_dir]).any? do |dir|
        !Dir.glob(pattern, base: dir).empty?
      end
    end
    private :holds_libraries_built_for_mri?

    def contains_requirable_file?(file)
      Gem.provided_by_metorex?(name, version) ? false : super
    end

    # Whether the carried copy is loaded in place of this gem because its
    # extensions did not build for metorex, building them first if no
    # build has been tried.
    def unbuilt_with_a_carried_copy?
      return false unless Gem.carried_when_unbuilt?(name) && have_extensions?
      return false if File.exist?(gem_build_complete_path)

      spec = is_a?(Specification) ? self : to_spec
      spec.missing_extensions?
      !File.exist?(gem_build_complete_path)
    end
    private :unbuilt_with_a_carried_copy?
  end

  BasicSpecification.prepend(MetorexRequirePaths)

  module MetorexSpecification
    # A gem's extensions count as missing only once building them for
    # metorex has been tried. A process an extension's build started builds
    # none of its own, since the gem it is building would count as missing
    # its extensions there too.
    def missing_extensions?
      return false if Gem.provided_by_metorex?(name, version)
      return false unless super
      # The carried copy stands in for one whose extensions do not build.
      return !Gem.carried_when_unbuilt?(name) if @extensions_tried_for_metorex || ENV[BUILDING_EXTENSIONS]

      @extensions_tried_for_metorex = true
      build_extensions_for_metorex
      super && !Gem.carried_when_unbuilt?(name)
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
    # RUBYOPT, and marked as a build's. The gems installed beside this one
    # are on GEM_PATH, so an extconf.rb finds the ones it builds with, such
    # as mini_portile2, as it does when installing puts them there.
    def environment_for_extension_builds
      changes = { BUILDING_EXTENSIONS => "1" }
      if defined?(Bundler) && Bundler.respond_to?(:unbundled_env)
        unbundled = Bundler.unbundled_env
        ENV.each_key { |key| changes[key] = nil unless unbundled.key?(key) }
        unbundled.each { |key, value| changes[key] = value unless ENV[key] == value }
      end
      inherited = changes.key?("GEM_PATH") ? changes["GEM_PATH"] : ENV["GEM_PATH"]
      paths = [base_dir, *inherited.to_s.split(File::PATH_SEPARATOR), *Gem.default_path]
      changes["GEM_PATH"] = paths.reject(&:empty?).uniq.join(File::PATH_SEPARATOR)
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
        next if File.exist?(gem_build_complete_path)

        # A build that failed with this same metorex fails again, so it is
        # tried once per metorex rather than once per process.
        failed = File.join(target, "gem_build_failed")
        if File.exist?(failed) && File.read(failed) == metorex_identity
          warn "Ignoring #{full_name} because its extensions could not be built for metorex. See #{File.join(target, "gem_make.out")}"
          next
        end
        File.write(failed, metorex_identity) unless build_extensions_into(target)
      end
    end

    # What tells one metorex binary from another: its size and when it was
    # written.
    def metorex_identity
      held = File.stat(Gem.ruby)
      "#{held.size} #{held.mtime.to_i}"
    end

    def build_extensions_into(target)
      log = File.join(target, "gem_make.out")
      source = source_for_metorex_builds(target)
      built = File.open(log, "w") do |output|
        extensions.all? do |extconf|
          build = File.join(target, "build", File.dirname(extconf))
          FileUtils.mkdir_p(build)
          environment = environment_for_extension_builds
          made = Dir.chdir(build) do
            system(environment, Gem.ruby, File.join(source, extconf), out: output, err: output) &&
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
      built
    end

    # The gem's files as it was packaged, unpacked from the .gem file
    # installing it kept, since installing it for MRI may have removed what
    # the build reads, as nokogiri's removes the sources of the libraries it
    # builds. A gem installed without its .gem file builds from where it is
    # installed.
    def source_for_metorex_builds(target)
      return full_gem_path unless File.file?(cache_file)

      require "rubygems/package"
      source = File.join(target, "source")
      Gem::Package.new(cache_file).extract_files(source) unless File.directory?(source)
      source
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
