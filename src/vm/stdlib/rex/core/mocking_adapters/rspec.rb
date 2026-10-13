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

        def verify_mocks_for_rspec
          ::Rex::Mocks.verify
        end

        def teardown_mocks_for_rspec
          ::Rex::Mocks.teardown
        end
      end
    end
  end
end
