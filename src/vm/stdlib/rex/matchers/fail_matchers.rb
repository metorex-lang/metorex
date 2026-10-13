require "rex"

module Rex
  module Matchers
    # Matchers for specs of matchers, included with
    # `config.include RSpec::Matchers::FailMatchers`.
    module FailMatchers
      def fail(&block)
        raise_error(Rex::Expectations::ExpectationNotMetError, &block)
      end

      def fail_with(message)
        raise_error(Rex::Expectations::ExpectationNotMetError, message)
      end

      def fail_including(*snippets)
        raise_error(Rex::Expectations::ExpectationNotMetError, a_string_including(*snippets))
      end
    end
  end
end
