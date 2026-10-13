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
      end

      def aggregate
        outer = Thread.current[:rex_failure_aggregator]
        Thread.current[:rex_failure_aggregator] = method(:call)
        begin
          yield
        rescue Rex::ExpectationNotMet => error
          @failures << error
        rescue *Rex::Core::DANGEROUS_EXCEPTIONS
          raise
        rescue Exception => error
          @other_errors << error
        ensure
          Thread.current[:rex_failure_aggregator] = outer
        end
        notify_aggregated_failures
      end

      def call(failure)
        failure.set_backtrace(caller) unless failure.backtrace
        @failures << failure
        nil
      end

      private

      def notify_aggregated_failures
        all_errors = failures + other_errors
        case all_errors.size
        when 0 then true
        when 1 then raise all_errors.first
        else raise MultipleExpectationsNotMetError.new(self)
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
