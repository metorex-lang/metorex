require "rex/core/formatters/helpers"
require 'stringio'

module Rex
  module Core
    module Formatters
      class BaseFormatter
        Formatters.register self, :start, :example_group_started, :close
        attr_accessor :example_group
        attr_reader :output

        def initialize(output)
          @output = output || StringIO.new
          @example_group = nil
        end

        def start(notification)
          start_sync_output
          @example_count = notification.count
        end

        def example_group_started(notification)
          @example_group = notification.group
        end

        def close(_notification)
          restore_sync_output
        end

      private

        def start_sync_output
          @old_sync, output.sync = output.sync, true if output_supports_sync
        end

        def restore_sync_output
          output.sync = @old_sync if output_supports_sync && !output.closed?
        end

        def output_supports_sync
          output.respond_to?(:sync=)
        end
      end
    end
  end
end
