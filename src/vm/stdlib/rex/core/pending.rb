module Rex
  module Core
    module Pending
      class SkipDeclaredInExample < StandardError
        attr_reader :argument

        def initialize(argument)
          @argument = argument
        end
      end

      begin
        class PendingExampleFixedError < Test::Unit::AssertionFailedError; end
      rescue
        class PendingExampleFixedError < StandardError; end
      end

      NO_REASON_GIVEN = 'No reason given'

      NOT_YET_IMPLEMENTED = 'Not yet implemented'

      def pending(message=nil, &_block)
        current_example = Rex.current_example

        if block_given?
          raise ArgumentError, <<-EOS.gsub(/^\s+\|/, '')
            |The semantics of `RSpec::Core::Pending#pending` have changed in
            |RSpec 3. In RSpec 2.x, it caused the example to be skipped. In
            |RSpec 3, the rest of the example is still run but is expected to
            |fail, and will be marked as a failure (rather than as pending) if
            |the example passes.
            |
            |Passing a block within an example is now deprecated. Marking the
            |example as pending provides the same behavior in RSpec 3 which was
            |provided only by the block in RSpec 2.x.
            |
            |Move the code in the block provided to `pending` into the rest of
            |the example body.
            |
            |Called from #{CallerFilter.first_non_rspec_line}.
            |
          EOS
        elsif current_example
          Pending.mark_pending! current_example, message
        else
          raise "`pending` may not be used outside of examples, such as in " \
                "before(:context). Maybe you want `skip`?"
        end
      end

      def skip(message=nil)
        current_example = Rex.current_example

        Pending.mark_skipped!(current_example, message) if current_example

        raise SkipDeclaredInExample.new(message)
      end

      def self.mark_skipped!(example, message_or_bool)
        Pending.mark_pending! example, message_or_bool
        example.metadata[:skip] = true
      end

      def self.mark_pending!(example, message_or_bool)
        message = if !message_or_bool || !(String === message_or_bool)
                    NO_REASON_GIVEN
                  else
                    message_or_bool
                  end

        example.metadata[:pending] = true
        example.execution_result.pending_message = message
        example.execution_result.pending_fixed = false
      end

      def self.mark_fixed!(example)
        example.execution_result.pending_fixed = true
      end
    end
  end
end
