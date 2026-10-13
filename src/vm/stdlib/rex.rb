# Rex, the spec framework Metorex carries. It follows RSpec 3.13, and RSpec
# names the same module, so a suite written for RSpec runs unchanged.

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
require "rex/core/configuration"
require "rex/core/hooks"
require "rex/core/memoized_helpers"
require "rex/core/metadata"
require "rex/core/example"
require "rex/core/example_group"
require "rex/core/world"
require "rex/core/shared_example_group"
require "rex/core/dsl"
require "rex/expectations"
require "rex/matchers"
require "rex/mocks"
require "rex/core/formatters"
require "rex/core/deprecations"
require "rex/core/runner"
require "rex/core/parallel"
require "rex/core/coverage"

Rex::Core::ExampleGroup.__send__(:include, Rex::Matchers)
Rex::Core::ExampleGroup.__send__(:include, Rex::Mocks::ExampleMethods)
Rex::Core::ExampleGroup.extend(Rex::Matchers::DSL)
Rex::Expectations.configuration.reset_syntaxes_to_default
