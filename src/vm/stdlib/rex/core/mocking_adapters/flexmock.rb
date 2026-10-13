
require 'flexmock/rspec'

module Rex
  module Core
    module MockingAdapters
      module Flexmock
        include ::FlexMock::MockContainer

        def self.framework_name
          :flexmock
        end

        def setup_mocks_for_rspec
        end

        def verify_mocks_for_rspec
          flexmock_verify
        end

        def teardown_mocks_for_rspec
          flexmock_close
        end
      end
    end
  end
end
