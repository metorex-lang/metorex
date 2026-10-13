module Rex
  module Expectations
    # Collects the failures inside `aggregate_failures` and raises them
    # together when the block ends.
    class FailureAggregator
      attr_reader :block_label, :metadata, :failures, :other_errors

      def initialize(block_label, metadata)
        @block_label = block_label
        @metadata = metadata
        @failures = []
        @other_errors = []
        @seen_source_ids = {}
      end

      # What `call` answers in place of raising, so the caller goes on.
      class AggregatedFailure
        MESSAGE =
          "AggregatedFailure: This method caused a failure which has been " \
          "suppressed to be aggregated into our failure report by returning " \
          "this value, further errors can be ignored."

        def inspect
          MESSAGE
        end
      end

      AGGREGATED_FAILURE = AggregatedFailure.new

      def aggregate
        Rex::Support.with_failure_notifier(self) do
          begin
            yield
          rescue Rex::ExpectationNotMet => error
            @failures << error
          rescue Rex::Support::AllExceptionsExceptOnesWeMustNotRescue => error
            @other_errors << error
          end
        end
        notify_aggregated_failures
      end

      def call(failure, options = {})
        source_id = options[:source_id]
        return if source_id && @seen_source_ids.key?(source_id)

        @seen_source_ids[source_id] = true
        failure.set_backtrace(caller) unless failure.backtrace
        @failures << failure
        AGGREGATED_FAILURE
      end

      private

      def notify_aggregated_failures
        all_errors = failures + other_errors
        case all_errors.size
        when 0 then true
        when 1 then Rex::Support.notify_failure(all_errors.first)
        else Rex::Support.notify_failure(MultipleExpectationsNotMetError.new(self))
        end
      end
    end

    class MultipleExpectationsNotMetError < Rex::ExpectationNotMet
      attr_reader :aggregator

      def initialize(aggregator)
        @aggregator = aggregator
        super(summary)
      end

      def failures
        @aggregator.failures
      end

      def other_errors
        @aggregator.other_errors
      end

      def all_exceptions
        failures + other_errors
      end

      def aggregation_block_label
        @aggregator.block_label
      end

      def aggregation_metadata
        @aggregator.metadata
      end

      def summary
        "Got #{exception_count_description} from failure aggregation block#{block_description}"
      end

      def block_description
        aggregation_block_label ? " #{aggregation_block_label.inspect}" : ""
      end

      def exception_count_description
        failure_count = pluralize("failure", failures.size)
        return failure_count if other_errors.empty?

        error_count = pluralize("other error", other_errors.size)
        "#{failure_count} and #{error_count}"
      end

      private

      def pluralize(noun, count)
        "#{count} #{noun}#{'s' unless count == 1}"
      end
    end
  end
end
