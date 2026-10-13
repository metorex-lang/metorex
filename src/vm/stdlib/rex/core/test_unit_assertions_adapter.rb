require 'test/unit/assertions'

module Rex
  module Core
    module TestUnitAssertionsAdapter
      include ::Test::Unit::Assertions

      begin
        if ancestors.include?(::Minitest::Assertions)
          require 'rspec/core/minitest_assertions_adapter'
          include ::Rex::Core::MinitestAssertionsAdapter
        end
      rescue NameError
      end
    end
  end
end
