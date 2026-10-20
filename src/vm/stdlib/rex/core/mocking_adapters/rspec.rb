require 'rspec/mocks'

module Rex
  module Core
    module MockingAdapters
      module RSpec
        include ::Rex::Mocks::ExampleMethods

        def self.framework_name
          :rspec
        end

        def self.configuration
          ::Rex::Mocks.configuration
        end

        def setup_mocks_for_rspec
          ::Rex::Mocks.setup
        end

        # An example that made no doubles has a mock space with nothing to
        # verify or take down, which `__mock_space_idle__` answers without
        # walking it.
        def verify_mocks_for_rspec
          space = ::Rex::Mocks.space
          ::Rex::Mocks.verify unless __mock_space_idle__(space, space.class)
        end

        def teardown_mocks_for_rspec
          space = ::Rex::Mocks.space
          return ::Rex::Mocks.teardown unless __mock_space_idle__(space, space.class)

          ::Rex::Mocks.instance_exec do
            @space_stack.pop
            @space = @space_stack.last || @root_space
          end
        end
      end
    end
  end
end
