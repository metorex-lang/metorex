module Rex
  module Core
    module Formatters
      class FallbackMessageFormatter
        Formatters.register self, :message

        def initialize(output)
          @output = output
        end

        attr_reader :output

        def message(notification)
          output.puts notification.message
        end
      end
    end
  end
end
