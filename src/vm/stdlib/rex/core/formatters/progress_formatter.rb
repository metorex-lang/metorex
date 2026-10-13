require "rex/core/formatters/base_text_formatter"
require "rex/core/formatters/console_codes"

module Rex
  module Core
    module Formatters
      class ProgressFormatter < BaseTextFormatter
        Formatters.register self, :example_passed, :example_pending, :example_failed, :start_dump

        def example_passed(_notification)
          output.print ConsoleCodes.wrap('.', :success)
        end

        def example_pending(_notification)
          output.print ConsoleCodes.wrap('*', :pending)
        end

        def example_failed(_notification)
          output.print ConsoleCodes.wrap('F', :failure)
        end

        def start_dump(_notification)
          output.puts
        end
      end
    end
  end
end
