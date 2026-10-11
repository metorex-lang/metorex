# The gems installed where GEM_HOME and GEM_PATH say, which `require`
# activates when the load path holds no file of the name it was given, and
# which `gem` activates by name and version.

require "rubygems/version"
require "rubygems/requirement"

module Gem
  class LoadError < ::LoadError
    attr_accessor :name, :requirement
  end

  class MissingSpecError < LoadError
    def initialize(name, requirement, extra_message = nil)
      @name = name
      @requirement = requirement
      @extra_message = extra_message
      super(message)
    end

    def message
      "Could not find '#{name}' (#{requirement}) among #{Gem::Specification.all.size} total gem(s)\n" \
        "Checked in 'GEM_PATH=#{Gem.path.join(File::PATH_SEPARATOR)}' #{@extra_message}, execute `gem env` for more information"
    end
  end

  class MissingSpecVersionError < MissingSpecError
    attr_reader :specs

    def initialize(name, requirement, specs)
      @specs = specs
      super(name, requirement)
    end

    def message
      names = specs.map(&:full_name).join(",")
      "Could not find '#{name}' (#{requirement}) - did find: [#{names}]\n" \
        "Checked in 'GEM_PATH=#{Gem.path.join(File::PATH_SEPARATOR)}' , execute `gem env` for more information"
    end
  end

  # Where gems go when no GEM_HOME names a place.
  def self.default_dir
    File.join Dir.home, ".gem", "metorex"
  end

  def self.default_path
    [default_dir]
  end

  def self.default_spec_cache_dir
    File.join Dir.home, ".gem", "specs"
  end

  def self.path_separator
    File::PATH_SEPARATOR
  end

  # Where gems are looked for, read from GEM_HOME and GEM_PATH the first
  # time it is asked for.
  def self.paths
    @paths ||= Gem::PathSupport.new(ENV)
  end

  def self.paths=(env)
    clear_paths
    @paths = Gem::PathSupport.new(env)
  end

  def self.clear_paths
    @paths = nil
    Gem::Specification.reset
  end

  def self.dir
    paths.home
  end

  def self.path
    paths.path
  end

  def self.default_specifications_dir
    File.join default_dir, "specifications", "default"
  end

  def self.bindir
    File.join dir, "bin"
  end

  # Where the executable a gem installs is. Without one named, the gem's
  # first executable is meant.
  def self.bin_path name, executable = nil, *requirements
    spec = Gem::Specification.find_by_name(name, *requirements)
    executable ||= spec.executables.first
    raise Gem::Exception, "no default executable for #{spec.full_name}" if executable.nil?
    File.join spec.gem_dir, spec.bindir, executable
  end

  class Exception < RuntimeError
  end

  # A gem's files go in front of the ones the interpreter carries, which is
  # what the index names.
  def self.load_path_insert_index
    $LOAD_PATH.length
  end

  # The gems activated so far, by name.
  def self.loaded_specs
    @loaded_specs ||= {}
  end

  # Activate the newest installed gem holding `path` among the files it
  # can be required by, answering whether one was found.
  def self.try_activate path
    spec = Gem::Specification.find_by_path(path)
    return false if spec.nil?
    return false if loaded_specs[spec.name]
    spec.activate
    true
  end

  def self.ruby
    RbConfig.ruby
  end

  # The settings a gem command reads. Metorex keeps no configuration file,
  # so each setting starts at the default RubyGems gives it.
  class ConfigFile
    DEFAULT_BACKTRACE = true
    DEFAULT_VERBOSITY = true

    attr_accessor :verbose, :backtrace

    def initialize(_arguments = [])
      @verbose = DEFAULT_VERBOSITY
      @backtrace = DEFAULT_BACKTRACE
    end

    # Whether every detail is wanted, which a verbosity other than true,
    # false, or nil asks for.
    def really_verbose
      case verbose
      when true, false, nil then
        false
      else
        true
      end
    end
  end

  def self.configuration
    @configuration ||= Gem::ConfigFile.new
  end

  def self.configuration=(config)
    @configuration = config
  end

  # A gem one installed gemspec names. Its name, version, platform and the
  # directories it is required from come from the gemspec's stub line, and
  # the rest from evaluating the gemspec when it is activated.
  class Specification
    attr_accessor :name, :require_paths, :bindir, :executables, :files,
                  :loaded_from, :base_dir, :platform, :summary, :authors
    attr_reader :version, :dependencies

    def self.default_specifications_dir
      Gem.default_specifications_dir
    end

    def initialize(name = nil, version = nil)
      @name = name
      @version = version && Gem::Version.create(version)
      @require_paths = ["lib"]
      @bindir = "bin"
      @executables = []
      @files = []
      @dependencies = []
      @platform = "ruby"
      @full_load = true
      yield self if block_given?
    end

    def version=(version)
      @version = Gem::Version.create(version)
    end

    def add_runtime_dependency(name, *requirements)
      @dependencies << Gem::Dependency.new(name, *requirements.flatten)
    end
    alias add_dependency add_runtime_dependency

    def add_development_dependency(_name, *_requirements)
    end

    def runtime_dependencies
      full_spec.dependencies
    end

    # Every other setting a gemspec writes is kept as it was given.
    def method_missing(name, *arguments)
      written = name.to_s
      return super unless written.end_with?("=") && arguments.size == 1
      instance_variable_set "@#{written.chomp("=")}", arguments.first
    end

    def respond_to_missing?(name, include_private = false)
      name.to_s.end_with?("=") || super
    end

    def full_name
      platform.to_s == "ruby" ? "#{name}-#{version}" : "#{name}-#{version}-#{platform}"
    end

    def gem_dir
      File.join base_dir, "gems", full_name
    end

    def full_gem_path
      gem_dir
    end

    def full_require_paths
      require_paths.map { |held| File.expand_path(held, gem_dir) }
    end

    # A gem that came with Ruby, whose files metorex carries itself.
    attr_accessor :default_gem

    def default_gem?
      @default_gem == true
    end

    # Whether `path` names a file the gem can be required by. A default
    # gem's files are the ones metorex carries, so none is.
    def contains_requirable_file?(path)
      return false if default_gem?
      full_require_paths.any? do |directory|
        ["", ".rb"].any? { |ending| File.file?(File.join(directory, "#{path}#{ending}")) }
      end
    end

    def activated?
      Gem.loaded_specs[name].equal?(self)
    end

    # Put the gem's directories on the load path, and those of the newest
    # installed versions of the gems it depends on, unless one of them is
    # active already.
    def activate
      active = Gem.loaded_specs[name]
      return false if active.equal?(self)
      if active
        raise Gem::LoadError, "can't activate #{full_name}, already activated #{active.full_name}"
      end
      Gem.loaded_specs[name] = self
      runtime_dependencies.each do |dependency|
        next if Gem.loaded_specs[dependency.name]
        dependency.to_spec.activate
      end
      at = Gem.load_path_insert_index
      full_require_paths.reverse_each { |directory| $LOAD_PATH.insert(at, directory) unless $LOAD_PATH.include?(directory) }
      true
    end

    # This spec with everything its gemspec says, read the first time
    # something past the stub line is asked for.
    def full_spec
      return self if @full_load
      @full_load = true
      loaded = Gem::Specification.load(loaded_from)
      @dependencies = loaded.dependencies
      @executables = loaded.executables
      @bindir = loaded.bindir
      @files = loaded.files
      self
    end

    %i[executables bindir files].each do |reader|
      define_method(reader) do
        full_spec
        instance_variable_get "@#{reader}"
      end
    end

    # The spec a gemspec file builds.
    def self.load(path)
      spec = eval(File.read(path), TOPLEVEL_BINDING, path)
      spec.loaded_from = path
      spec.base_dir = File.dirname(File.dirname(path))
      spec
    end

    # A spec read from a gemspec's stub line alone, or by evaluating the
    # file when it has none.
    def self.stub_for(path)
      stub = File.open(path) { |file| 2.times.map { file.gets.to_s } }.last
      matched = /\A# stub: (\S+) (\S+) (\S+) (.+)\n?\z/.match(stub)
      return load(path) if matched.nil?
      spec = new(matched[1], matched[2])
      spec.platform = matched[3]
      spec.require_paths = matched[4].split("\0")
      spec.loaded_from = path
      spec.base_dir = File.dirname(File.dirname(path))
      spec.instance_variable_set :@full_load, false
      spec
    end

    # Every installed gem under the gem path, read once.
    def self.all
      @all ||= Gem.path.flat_map do |directory|
        installed = Dir[File.join(directory, "specifications", "*.gemspec")].sort.map { |path| stub_for(path) }
        defaults = Dir[File.join(directory, "specifications", "default", "*.gemspec")].sort.map do |path|
          spec = stub_for(path)
          spec.default_gem = true
          spec
        end
        installed + defaults
      end
    end

    def self.reset
      @all = nil
    end

    def self.each_spec _directories
      all.each { |spec| yield spec } if block_given?
      nil
    end

    def self.find_all_by_name(name, *requirements)
      requirement = Gem::Requirement.create(requirements.empty? ? Gem::Requirement.default : requirements)
      all.select { |spec| spec.name == name && requirement.satisfied_by?(spec.version) }
         .sort_by(&:version).reverse
    end

    # The active spec of that name when it satisfies the requirements, and
    # the newest installed one that does otherwise.
    def self.find_by_name(name, *requirements)
      Gem::Dependency.new(name, *requirements).to_spec
    end

    # The newest installed gem, of the ones not already active under
    # another version, that holds a file `path` names.
    def self.find_by_path(path)
      candidates = all.select { |spec| spec.contains_requirable_file?(path) }
      candidates.max_by(&:version)
    end

    # The newest version of each installed gem.
    def self.latest_specs _prerelease = false
      all.group_by(&:name).map { |_, specs| specs.max_by(&:version) }
    end
  end

  # A gem named with the versions of it that will do.
  class Dependency
    attr_reader :name, :requirement

    def initialize(name, *requirements)
      @name = name
      @requirement = Gem::Requirement.create(requirements.empty? ? Gem::Requirement.default : requirements.flatten)
    end

    # The spec that satisfies this, preferring the active one.
    def to_spec
      active = Gem.loaded_specs[name]
      return active if active && requirement.satisfied_by?(active.version)
      found = Gem::Specification.find_all_by_name(name, requirement)
      return found.first unless found.empty?
      named = Gem::Specification.all.select { |spec| spec.name == name }.sort_by(&:version).reverse
      raise Gem::MissingSpecError.new(name, requirement) if named.empty?
      raise Gem::MissingSpecVersionError.new(name, requirement, named)
    end
  end
end

require 'rbconfig'

require "rubygems/path_support"

module Kernel
  # Activate the gem `name` at a version the requirements allow, answering
  # whether it was activated now rather than already.
  def gem(name, *requirements)
    spec = Gem::Dependency.new(name, *requirements).to_spec
    active = Gem.loaded_specs[name]
    return false if active.equal?(spec)
    spec.activate
  end
  private :gem
end
