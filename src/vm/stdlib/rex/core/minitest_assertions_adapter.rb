begin
  require 'minitest'
  require 'minitest/assertions'
rescue LoadError
  require 'minitest/unit'
  Minitest = MiniTest
end

module Rex
  module Core
    module MinitestAssertionsAdapter
      include ::Minitest::Assertions
      include ::Rex::Core::Pending

      attr_writer :assertions
      def assertions
        @assertions ||= 0
      end
    end
  end
end
