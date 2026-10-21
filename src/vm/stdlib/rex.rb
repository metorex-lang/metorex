# Rex, the spec framework Metorex carries. It follows RSpec 3.13, and RSpec
# names the same module, so a suite written for RSpec runs unchanged. The
# module is named RSpec, so what a suite prints names its classes as RSpec
# does. A program that defined an RSpec of its own before loading Rex keeps
# it, and Rex goes by its own name alone.

$_rspec_core_load_started_at = Time.now

if Object.autoload?(:RSpec) || !defined?(RSpec)
  Object.const_set(:RSpec, Module.new)
  Rex = RSpec
end

module Rex
  module Core
  end

  module Expectations
  end

  module Matchers
  end

  module Mocks
  end
end

require "rex/rspec/support"
%w[
  warnings
  recursive_const_methods
  reentrant_mutex
  encoded_string
  directory_maker
  object_formatter
  fuzzy_matcher
  differ
  with_keywords_when_needed
  source
].each { |name| Rex::Support.require_rspec_support(name) }
# `require_rspec_core` and `require_rspec_mocks`, which rspec-core's and
# rspec-mocks' own entry files define in RSpec, and which reach Rex's files.
%i[core mocks].each { |library| Rex::Support.define_optimized_require_for_rspec(library) {} }
require "rex/core/framework"
require "rex/source"

%w[
  version
  warnings
  set
  flat_map
  filter_manager
  dsl
  notifications
  reporter
  hooks
  memoized_helpers
  metadata
  metadata_filter
  pending
  formatters
  ordering
  world
  configuration
  option_parser
  configuration_options
  runner
  invocations
  example
  shared_example_group
  example_group
  shared_context
  rex_settings
].each { |name| require "rex/core/#{name}" }

module Rex
  extend Rex::Support::Warnings
  extend Rex::Core::Warnings

  class << self
    attr_writer :configuration, :world
  end

  def self.reset
    Rex::ExampleGroups.remove_all_constants
    @world = nil
    @configuration = nil
  end

  def self.clear_examples
    world.reset
    configuration.reset_reporter
    configuration.start_time = ::Rex::Core::Time.now
    configuration.reset_filters
  end

  def self.configuration
    @configuration ||= Rex::Core::Configuration.new
  end

  def self.configure
    yield configuration if block_given?
  end

  def self.current_example
    Rex::Support.thread_local_data[:current_example]
  end

  def self.current_example=(example)
    Rex::Support.thread_local_data[:current_example] = example
  end

  def self.current_scope=(scope)
    Rex::Support.thread_local_data[:current_scope] = scope
  end
  Rex.current_scope = :suite

  def self.current_scope
    Rex::Support.thread_local_data[:current_scope]
  end

  def self.world
    @world ||= Rex::Core::World.new
  end

  module Core
    autoload :ExampleStatusPersister, "rex/core/example_status_persister"
    autoload :Profiler, "rex/core/profiler"
    autoload :DidYouMean, "rex/core/did_you_mean"

    class Time
      class << self
        define_method(:now, &::Time.method(:now))
      end
    end

    EXECUTABLE_SOURCE = <<~'RUBY'
      #!/usr/bin/env ruby

      require 'rspec/core'
      RSpec::Core::Runner.invoke
    RUBY

    # RSpec's `exe/rspec`, which bisect runs in each child process. Rex's
    # files are carried in the binary, so it is written once to a directory
    # of Rex's own.
    def self.path_to_executable
      @path_to_executable ||= begin
        require "tmpdir"
        require "fileutils"
        directory = File.join(Dir.tmpdir, "metorex-rex-#{Process.uid}")
        path = File.join(directory, "rspec")
        unless File.file?(path) && File.read(path) == EXECUTABLE_SOURCE
          FileUtils.mkdir_p(directory)
          written = "#{path}.#{Process.pid}"
          File.write(written, EXECUTABLE_SOURCE)
          File.chmod(0o755, written)
          File.rename(written, path)
        end
        path
      end
    end
  end

  MODULES_TO_AUTOLOAD = {
    :Matchers     => "rspec/expectations",
    :Expectations => "rspec/expectations",
    :Mocks        => "rspec/mocks"
  }

  def self.const_missing(name)
    require MODULES_TO_AUTOLOAD.fetch(name) { return super }
    ::Rex.const_get(name)
  end

  Core::DSL.expose_globally!
  Core::SharedExampleGroup::TopLevelDSL.expose_globally!
end

require "rex/rspec/expectations"
require "rex/expectation_additions"
require "rex/rspec/mocks"
require "rex/core/parallel"
require "rex/core/coverage"
require "rex/core/fast_paths"

Rex::Core::ExampleGroup.extend(Rex::Matchers::DSL)
Rex::Expectations.configuration.reset_syntaxes_to_default
