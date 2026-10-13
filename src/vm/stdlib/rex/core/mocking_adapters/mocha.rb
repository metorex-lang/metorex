begin
  require 'mocha/api'

  begin
    require 'mocha/object'
  rescue LoadError
  end
rescue LoadError
  require 'mocha/standalone'
  require 'mocha/object'
end

module Rex
  module Core
    module MockingAdapters
      module Mocha
        def self.framework_name
          :mocha
        end

        begin
          include ::Mocha::API
        rescue NameError
          include ::Mocha::Standalone
        end

        def setup_mocks_for_rspec
          mocha_setup
        end

        def verify_mocks_for_rspec
          mocha_verify
        end

        def teardown_mocks_for_rspec
          mocha_teardown
        end
      end
    end
  end
end
