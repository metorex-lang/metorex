# What metorex sets for RubyGems as the packager of its own Ruby, which
# RubyGems reads from here as it would an operating system's defaults. Gems go
# under ~/.gem/metorex when GEM_HOME names no place. A gem's C extensions are
# built against metorex's headers the first time the gem is needed, into a
# directory named for a digest of those headers, since an installed gem's
# extensions were built for MRI. A gem whose library metorex carries, such as
# prism, is left to the copy metorex carries when another gem depends on it.

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

  # Whether metorex carries the library a gem of this name provides.
  def self.carried?(name)
    __embedded_library_names__().include?(name.tr("-", "/"))
  end

  module MetorexSpecification
    # A gem's extensions count as missing only once building them for
    # metorex has been tried.
    def missing_extensions?
      return false unless super
      return true if @extensions_tried_for_metorex

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

    # Run each extconf.rb under metorex in a directory of metorex's own, so
    # the installed gem is left as it is, then make, keeping the output in
    # gem_make.out beside the libraries built.
    def build_extensions_for_metorex
      require "fileutils"
      target = extension_dir
      FileUtils.mkdir_p(target)
      log = File.join(target, "gem_make.out")
      built = File.open(log, "w") do |output|
        extensions.all? do |extconf|
          build = File.join(target, "build", File.dirname(extconf))
          FileUtils.mkdir_p(build)
          made = Dir.chdir(build) do
            system(Gem.ruby, File.join(full_gem_path, extconf), out: output, err: output) &&
              system("make", out: output, err: output)
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
