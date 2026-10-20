module Rex
  module Expectations
    class << self
      # @private
      class Differ
        # @private
        OBJECT_PREPARER = lambda do |object|
          Rex::Matchers::Composable.surface_descriptions_in(object)
        end
      end

      # @private
      def differ
        Rex::Support::Differ.new(
          :object_preparer => Differ::OBJECT_PREPARER,
          :color => Rex::Matchers.configuration.color?
        )
      end

      # Raises an RSpec::Expectations::ExpectationNotMetError with message.
      # @param [String] message
      # @param [Object] expected
      # @param [Object] actual
      #
      # Adds a diff to the failure message when `expected` and `actual` are
      # both present.
      def fail_with(message, expected=nil, actual=nil)
        unless message
          raise ArgumentError, "Failure message is nil. Does your matcher define the " \
                               "appropriate failure_message[_when_negated] method to return a string?"
        end

        message = ::Rex::Matchers::MultiMatcherDiff.from(expected, actual).message_with_diff(message, differ)

        Rex::Support.notify_failure(Rex::Expectations::ExpectationNotMetError.new message)
      end
    end
  end
end
