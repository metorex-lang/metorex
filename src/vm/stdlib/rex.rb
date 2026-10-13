# Rex, the spec framework Metorex carries. It follows RSpec 3.13, and RSpec
# names the same module, so a suite written for RSpec runs unchanged.

$_rspec_core_load_started_at = Time.now

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

RSpec = Rex unless defined?(RSpec)

require "rex/support"
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

    def self.path_to_executable
      "metorex spec"
    end
  end

  Core::DSL.expose_globally!
  Core::SharedExampleGroup::TopLevelDSL.expose_globally!
end

require "rex/expectations"
require "rex/matchers"
require "rex/mocks"
require "rex/core/parallel"
require "rex/core/coverage"

Rex::Core::ExampleGroup.extend(Rex::Matchers::DSL)
Rex::Expectations.configuration.reset_syntaxes_to_default
