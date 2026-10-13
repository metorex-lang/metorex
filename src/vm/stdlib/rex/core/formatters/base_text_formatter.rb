require "rex/core/formatters/base_formatter"

module Rex
  module Core
    module Formatters
      class BaseTextFormatter < BaseFormatter
        Formatters.register self,
                            :message, :dump_summary, :dump_failures, :dump_pending, :seed

        def message(notification)
          output.puts notification.message
        end

        def dump_failures(notification)
          return if notification.failure_notifications.empty?
          output.puts notification.fully_formatted_failed_examples
        end

        def dump_summary(summary)
          output.puts summary.fully_formatted
        end

        def dump_pending(notification)
          return if notification.pending_examples.empty?
          output.puts notification.fully_formatted_pending_examples
        end

        def seed(notification)
          return unless notification.seed_used?
          output.puts notification.fully_formatted
        end

        def close(_notification)
          return if output.closed?

          output.puts

          output.flush
        end
      end
    end
  end
end
